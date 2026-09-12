//! Durable grouped edits and undo/redo. All fields here are ledger-local.
use crate::{digest, Document, Error, Result, State, Store, MAX_DOCUMENT_BYTES};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_HISTORY_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_HISTORY_ENTRIES: usize = 256;
pub const MAX_HISTORY_COMMAND_IDS: usize = 4096;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceEdit {
    pub start_byte: usize,
    pub end_byte: usize,
    pub removed_text: String,
    pub replacement: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedEdit {
    pub command_id: String,
    pub expected_revision: u64,
    pub expected_sha256: String,
    pub label: String,
    /// Nonoverlapping byte ranges in the same original source snapshot.
    pub edits: Vec<SourceEdit>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMove {
    pub command_id: String,
    pub expected_revision: u64,
    pub expected_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResult {
    pub document: Document,
    pub command_revision: u64,
    pub replayed_command: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatus {
    /// Oldest to newest, with the next action at the end of each stack.
    pub undo_labels: Vec<String>,
    pub redo_labels: Vec<String>,
    pub permanent_command_ids: usize,
    pub history_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Entry {
    label: String,
    before_text: String,
    after_text: String,
    before_sha256: String,
    after_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CommandReceipt {
    request_sha256: String,
    revision: u64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoryState {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    command_ids: BTreeMap<String, CommandReceipt>,
}
impl HistoryState {
    pub(crate) fn preserves(&self, older: &Self) -> bool {
        older
            .command_ids
            .iter()
            .all(|(id, receipt)| self.command_ids.get(id) == Some(receipt))
            && self.undo.ends_with(&older.undo)
            && self.redo.ends_with(&older.redo)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.undo.is_empty() && self.redo.is_empty() && self.command_ids.is_empty()
    }
    fn bytes(&self) -> usize {
        self.undo
            .iter()
            .chain(&self.redo)
            .map(|e| e.before_text.len() + e.after_text.len())
            .sum()
    }
    pub(crate) fn validate(&self, document: &Document) -> Result<()> {
        if self.undo.len() + self.redo.len() > MAX_HISTORY_ENTRIES
            || self.bytes() > MAX_HISTORY_BYTES
            || self.command_ids.len() > MAX_HISTORY_COMMAND_IDS
        {
            return Err(Error::new(
                "history_full",
                "history limit reached; explicit payload retention required",
            ));
        }
        for entry in self.undo.iter().chain(&self.redo) {
            if entry.label.len() > 256
                || entry.before_text.len() > MAX_DOCUMENT_BYTES
                || entry.after_text.len() > MAX_DOCUMENT_BYTES
                || digest(&entry.before_text) != entry.before_sha256
                || digest(&entry.after_text) != entry.after_sha256
            {
                return Err(Error::new(
                    "invalid_history",
                    "history payload hash/size mismatch",
                ));
            }
        }
        if self
            .undo
            .windows(2)
            .any(|p| p[0].after_sha256 != p[1].before_sha256)
            || self
                .redo
                .windows(2)
                .any(|p| p[0].before_sha256 != p[1].after_sha256)
            || self
                .undo
                .last()
                .is_some_and(|e| e.after_sha256 != document.source_sha256)
            || self
                .redo
                .last()
                .is_some_and(|e| e.before_sha256 != document.source_sha256)
        {
            return Err(Error::new(
                "invalid_history",
                "history chain differs from current source",
            ));
        }
        for (id, receipt) in &self.command_ids {
            crate::identifier(id)?;
            if receipt.revision > document.revision
                || receipt.request_sha256.len() != 64
                || !receipt
                    .request_sha256
                    .bytes()
                    .all(|c| c.is_ascii_hexdigit())
            {
                return Err(Error::new(
                    "invalid_history",
                    "history command receipt invalid",
                ));
            }
        }
        Ok(())
    }
}

pub(crate) fn record(next: &mut State, before: &Document, label: String) -> Result<()> {
    if before.text == next.document.text {
        return Ok(());
    }
    next.history.redo.clear();
    next.history.undo.push(Entry {
        label,
        before_text: before.text.clone(),
        after_text: next.document.text.clone(),
        before_sha256: before.source_sha256.clone(),
        after_sha256: next.document.source_sha256.clone(),
    });
    next.schema_version = next.schema_version.max(3);
    next.history.validate(&next.document)
}
fn fingerprint<T: Serialize>(kind: &str, value: &T) -> Result<String> {
    let text = serde_json::to_string(&(kind, value))
        .map_err(|e| Error::new("invalid_command", e.to_string()))?;
    Ok(digest(&text))
}
fn guard(document: &Document, revision: u64, sha256: &str) -> Result<()> {
    if document.revision != revision || document.source_sha256 != sha256 {
        return Err(Error::new(
            "document_conflict",
            "history command source revision/hash is stale",
        ));
    }
    Ok(())
}
fn result(state: &State, command_revision: u64, replayed: bool) -> HistoryResult {
    HistoryResult {
        document: state.document.clone(),
        command_revision,
        replayed_command: replayed,
        can_undo: !state.history.undo.is_empty(),
        can_redo: !state.history.redo.is_empty(),
    }
}
fn check_command(state: &State, id: &str, fingerprint: &str) -> Result<Option<HistoryResult>> {
    crate::identifier(id)?;
    if let Some(receipt) = state.history.command_ids.get(id) {
        return if receipt.request_sha256 == fingerprint {
            Ok(Some(result(state, receipt.revision, true)))
        } else {
            Err(Error::new(
                "command_id_conflict",
                "command ID already binds another operation",
            ))
        };
    }
    if state.history.command_ids.len() >= MAX_HISTORY_COMMAND_IDS {
        return Err(Error::new(
            "history_ids_full",
            "permanent command-ID limit reached; IDs cannot be evicted",
        ));
    }
    Ok(None)
}
fn remember(next: &mut State, id: String, fingerprint: String) {
    next.history.command_ids.insert(
        id,
        CommandReceipt {
            request_sha256: fingerprint,
            revision: next.document.revision,
        },
    );
    next.schema_version = next.schema_version.max(3);
}

impl Store {
    pub fn history_status(&self) -> Result<HistoryStatus> {
        self.ready()?;
        let history = &self
            .state
            .as_ref()
            .ok_or_else(|| Error::new("document_missing", "initialize source first"))?
            .history;
        Ok(HistoryStatus {
            undo_labels: history.undo.iter().map(|e| e.label.clone()).collect(),
            redo_labels: history.redo.iter().map(|e| e.label.clone()).collect(),
            permanent_command_ids: history.command_ids.len(),
            history_bytes: history.bytes(),
        })
    }
    pub fn apply_group(&mut self, group: GroupedEdit) -> Result<HistoryResult> {
        self.ready()?;
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| Error::new("document_missing", "initialize source first"))?;
        let request_hash = fingerprint("group", &group)?;
        if let Some(existing) = check_command(state, &group.command_id, &request_hash)? {
            return Ok(existing);
        }
        guard(
            &state.document,
            group.expected_revision,
            &group.expected_sha256,
        )?;
        if group.edits.is_empty() || group.edits.len() > 64 || group.label.len() > 256 {
            return Err(Error::new(
                "invalid_group",
                "require 1–64 edits and label at most 256 bytes",
            ));
        }
        let mut edits = group.edits;
        edits.sort_by_key(|e| (e.start_byte, e.end_byte));
        if edits.windows(2).any(|p| {
            p[0].end_byte > p[1].start_byte
                || (p[0].end_byte == p[1].start_byte
                    && (p[0].start_byte == p[0].end_byte || p[1].start_byte == p[1].end_byte))
        }) {
            return Err(Error::new(
                "overlapping_edits",
                "group ranges overlap or have ambiguous insertion affinity",
            ));
        }
        for edit in &edits {
            let removed = state
                .document
                .text
                .get(edit.start_byte..edit.end_byte)
                .ok_or_else(|| {
                    Error::new(
                        "invalid_source_range",
                        "range must be scalar-aligned bytes of original source",
                    )
                })?;
            if removed != edit.removed_text {
                return Err(Error::new(
                    "removed_text_conflict",
                    "group removed_text differs from original source",
                ));
            }
            if edit.replacement.len() > crate::MAX_REPLACEMENT_BYTES {
                return Err(Error::new(
                    "replacement_too_large",
                    "group replacement exceeds 64 KiB",
                ));
            }
        }
        let mut text = state.document.text.clone();
        for edit in edits.into_iter().rev() {
            text.replace_range(edit.start_byte..edit.end_byte, &edit.replacement);
        }
        let revision = state
            .document
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::new("revision_overflow", "revision exhausted"))?;
        let mut next = state.clone();
        next.document = Document::new(
            state.document.project_id.clone(),
            state.document.path.clone(),
            revision,
            text,
        )?;
        record(&mut next, &state.document, group.label)?;
        remember(&mut next, group.command_id, request_hash);
        let response = result(&next, revision, false);
        self.commit(next)?;
        Ok(response)
    }
    pub fn undo(&mut self, command: HistoryMove) -> Result<HistoryResult> {
        self.move_history(command, false)
    }
    pub fn redo(&mut self, command: HistoryMove) -> Result<HistoryResult> {
        self.move_history(command, true)
    }
    fn move_history(&mut self, command: HistoryMove, redo: bool) -> Result<HistoryResult> {
        self.ready()?;
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| Error::new("document_missing", "initialize source first"))?;
        let request_hash = fingerprint(if redo { "redo" } else { "undo" }, &command)?;
        if let Some(existing) = check_command(state, &command.command_id, &request_hash)? {
            return Ok(existing);
        }
        guard(
            &state.document,
            command.expected_revision,
            &command.expected_sha256,
        )?;
        let mut next = state.clone();
        let entry = if redo {
            next.history.redo.pop()
        } else {
            next.history.undo.pop()
        }
        .ok_or_else(|| Error::new("history_empty", "no retained history in that direction"))?;
        let text = if redo {
            entry.after_text.clone()
        } else {
            entry.before_text.clone()
        };
        let revision = state
            .document
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::new("revision_overflow", "revision exhausted"))?;
        next.document = Document::new(
            state.document.project_id.clone(),
            state.document.path.clone(),
            revision,
            text,
        )?;
        if redo {
            next.history.undo.push(entry);
        } else {
            next.history.redo.push(entry);
        }
        remember(&mut next, command.command_id, request_hash);
        let response = result(&next, revision, false);
        self.commit(next)?;
        Ok(response)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRetentionPolicy {
    pub snapshot_token: String,
    pub acknowledge_undo_redo_loss: bool,
    pub keep_latest_undo: usize,
    pub keep_latest_redo: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRetentionReport {
    pub dropped_payloads: usize,
    pub retained_undo: usize,
    pub retained_redo: usize,
    pub permanent_command_ids: usize,
    pub history_bytes: usize,
}
impl Store {
    pub fn retain_history(
        &mut self,
        policy: HistoryRetentionPolicy,
    ) -> Result<HistoryRetentionReport> {
        let export = self.export_recovery()?;
        if !policy.acknowledge_undo_redo_loss {
            return Err(Error::new(
                "retention_not_acknowledged",
                "explicit acknowledgement of lost undo/redo payloads required",
            ));
        }
        if policy.snapshot_token != export.snapshot_token {
            return Err(Error::new(
                "stale_recovery_snapshot",
                "history changed since retention review",
            ));
        }
        let mut next = self
            .state
            .as_ref()
            .expect("export checked initialization")
            .clone();
        let before = next.history.undo.len() + next.history.redo.len();
        let drop_undo = next
            .history
            .undo
            .len()
            .saturating_sub(policy.keep_latest_undo);
        let drop_redo = next
            .history
            .redo
            .len()
            .saturating_sub(policy.keep_latest_redo);
        next.history.undo.drain(..drop_undo);
        next.history.redo.drain(..drop_redo);
        let report = HistoryRetentionReport {
            dropped_payloads: before - next.history.undo.len() - next.history.redo.len(),
            retained_undo: next.history.undo.len(),
            retained_redo: next.history.redo.len(),
            permanent_command_ids: next.history.command_ids.len(),
            history_bytes: next.history.bytes(),
        };
        if report.dropped_payloads > 0 {
            self.commit(next)?;
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::setup;
    fn movement(store: &Store, id: &str) -> HistoryMove {
        let d = store.document().unwrap().unwrap();
        HistoryMove {
            command_id: id.into(),
            expected_revision: d.revision,
            expected_sha256: d.source_sha256.clone(),
        }
    }
    #[test]
    fn capture_undo_redo_and_command_retry_survive_restart() {
        let (dir, mut store, edit) = setup();
        let receipt = store.apply(edit.clone()).unwrap();
        let undo = movement(&store, "undo-1");
        store.undo(undo.clone()).unwrap();
        assert_eq!(store.document().unwrap().unwrap().text, "aé😀z");
        drop(store);
        let mut store = Store::open(dir.path()).unwrap();
        assert!(store.undo(undo).unwrap().replayed_command);
        assert_eq!(store.apply(edit).unwrap(), receipt);
        assert_eq!(store.document().unwrap().unwrap().text, "aé😀z");
        let redo = movement(&store, "redo-1");
        store.redo(redo).unwrap();
        assert_eq!(store.document().unwrap().unwrap().text, "a$x$z");
        assert_eq!(store.recovery().unwrap()[0].receipt, receipt);
    }
    #[test]
    fn grouped_original_byte_ranges_undo_as_one_transaction() {
        let (_dir, mut store, _) = setup();
        let d = store.document().unwrap().unwrap().clone();
        let group = GroupedEdit {
            command_id: "group-1".into(),
            expected_revision: d.revision,
            expected_sha256: d.source_sha256,
            label: "two changes".into(),
            edits: vec![
                SourceEdit {
                    start_byte: 1,
                    end_byte: 3,
                    removed_text: "é".into(),
                    replacement: "E".into(),
                },
                SourceEdit {
                    start_byte: 7,
                    end_byte: 8,
                    removed_text: "z".into(),
                    replacement: "Z".into(),
                },
            ],
        };
        store.apply_group(group.clone()).unwrap();
        assert_eq!(store.document().unwrap().unwrap().text, "aE😀Z");
        let undo = movement(&store, "undo-group");
        store.undo(undo).unwrap();
        assert_eq!(store.document().unwrap().unwrap().text, "aé😀z");
        assert!(store.apply_group(group).unwrap().replayed_command);
    }
    #[test]
    fn stale_move_and_conflicting_id_do_not_change_history() {
        let (_dir, mut store, edit) = setup();
        let stale = movement(&store, "undo-stale");
        store.apply(edit).unwrap();
        assert_eq!(store.undo(stale).unwrap_err().code, "document_conflict");
        let undo = movement(&store, "same");
        store.undo(undo).unwrap();
        let redo = movement(&store, "same");
        assert_eq!(store.redo(redo).unwrap_err().code, "command_id_conflict");
    }
    #[test]
    fn retention_drops_payloads_only() {
        let (_dir, mut store, edit) = setup();
        let receipt = store.apply(edit.clone()).unwrap();
        let undo = movement(&store, "undo-retain");
        store.undo(undo.clone()).unwrap();
        let policy = HistoryRetentionPolicy {
            snapshot_token: store.export_recovery().unwrap().snapshot_token,
            acknowledge_undo_redo_loss: true,
            keep_latest_undo: 0,
            keep_latest_redo: 0,
        };
        assert_eq!(store.retain_history(policy).unwrap().dropped_payloads, 1);
        assert!(store.undo(undo).unwrap().replayed_command);
        assert_eq!(store.apply(edit).unwrap(), receipt);
        let redo = movement(&store, "redo-missing");
        assert_eq!(store.redo(redo).unwrap_err().code, "history_empty");
    }
    #[test]
    fn io_failure_after_undo_rename_recovers_source_and_redo() {
        let (dir, mut store, edit) = setup();
        store.apply(edit).unwrap();
        let undo = movement(&store, "undo-crash");
        store.failpoint = Some("after_rename");
        assert!(store.undo(undo.clone()).is_err());
        drop(store);
        let mut store = Store::open(dir.path()).unwrap();
        assert!(store.undo(undo).unwrap().replayed_command);
        assert_eq!(store.document().unwrap().unwrap().text, "aé😀z");
        let redo = movement(&store, "redo-crash");
        store.redo(redo).unwrap();
        assert_eq!(store.document().unwrap().unwrap().text, "a$x$z");
    }

    #[test]
    fn grouped_overlap_scalar_and_removed_text_guards_are_atomic() {
        let (_dir, mut store, _) = setup();
        let original = store.document().unwrap().unwrap().clone();
        let valid = SourceEdit {
            start_byte: 1,
            end_byte: 3,
            removed_text: "é".into(),
            replacement: "E".into(),
        };
        for edits in [
            vec![valid.clone(), valid.clone()],
            vec![SourceEdit {
                start_byte: 2,
                ..valid.clone()
            }],
            vec![SourceEdit {
                removed_text: "wrong".into(),
                ..valid.clone()
            }],
        ] {
            let group = GroupedEdit {
                command_id: "invalid-group".into(),
                expected_revision: 1,
                expected_sha256: original.source_sha256.clone(),
                label: "group".into(),
                edits,
            };
            assert!(store.apply_group(group).is_err());
            assert_eq!(store.document().unwrap().unwrap(), &original);
            assert!(store.history_status().unwrap().undo_labels.is_empty());
        }
    }

    #[test]
    fn new_source_edit_clears_redo_without_releasing_capture_identity() {
        let (_dir, mut store, edit) = setup();
        let receipt = store.apply(edit.clone()).unwrap();
        let undo = movement(&store, "undo-branch");
        store.undo(undo).unwrap();
        let current = store.document().unwrap().unwrap().clone();
        store
            .replace_document(
                current.revision,
                &current.source_sha256,
                "new branch".into(),
            )
            .unwrap();
        assert!(store.history_status().unwrap().redo_labels.is_empty());
        assert_eq!(store.apply(edit).unwrap(), receipt);
        assert_eq!(store.document().unwrap().unwrap().text, "new branch");
    }

    #[test]
    fn history_retention_requires_acknowledgement_and_current_snapshot() {
        let (_dir, mut store, edit) = setup();
        store.apply(edit).unwrap();
        let policy = HistoryRetentionPolicy {
            snapshot_token: store.export_recovery().unwrap().snapshot_token,
            acknowledge_undo_redo_loss: false,
            keep_latest_undo: 0,
            keep_latest_redo: 0,
        };
        assert_eq!(
            store.retain_history(policy.clone()).unwrap_err().code,
            "retention_not_acknowledged"
        );
        let policy = HistoryRetentionPolicy {
            snapshot_token: "stale".into(),
            acknowledge_undo_redo_loss: true,
            ..policy
        };
        assert_eq!(
            store.retain_history(policy).unwrap_err().code,
            "stale_recovery_snapshot"
        );
        assert_eq!(store.history_status().unwrap().undo_labels.len(), 1);
    }
    #[test]
    fn history_limit_refuses_mutation_until_explicit_retention() {
        let (_dir, mut store, _) = setup();
        for index in 0..MAX_HISTORY_ENTRIES {
            let current = store.document().unwrap().unwrap().clone();
            store
                .replace_document(
                    current.revision,
                    &current.source_sha256,
                    format!("entry-{index}"),
                )
                .unwrap();
        }
        let current = store.document().unwrap().unwrap().clone();
        assert_eq!(
            store
                .replace_document(
                    current.revision,
                    &current.source_sha256,
                    "over limit".into()
                )
                .unwrap_err()
                .code,
            "history_full"
        );
        assert_eq!(store.document().unwrap().unwrap(), &current);
        let policy = HistoryRetentionPolicy {
            snapshot_token: store.export_recovery().unwrap().snapshot_token,
            acknowledge_undo_redo_loss: true,
            keep_latest_undo: 1,
            keep_latest_redo: 0,
        };
        let report = store.retain_history(policy).unwrap();
        assert_eq!(report.dropped_payloads, MAX_HISTORY_ENTRIES - 1);
        store
            .replace_document(
                current.revision,
                &current.source_sha256,
                "after retention".into(),
            )
            .unwrap();
        assert_eq!(store.history_status().unwrap().undo_labels.len(), 2);
    }
}

//! Durable grouped edits and undo/redo. All fields here are ledger-local.
use crate::{digest, Document, Error, Result, State, Store, MAX_DOCUMENT_BYTES};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::value::RawValue;
use std::io::{self, Write};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};

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
/// Source-free outcome of a durable command. Borrow `Store::document` for the
/// current source; `command_revision` can be older on an exact retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryCommandStatus {
    pub command_revision: u64,
    pub replayed_command: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}
impl HistoryCommandStatus {
    fn with_document(self, document: &Document) -> HistoryResult {
        HistoryResult {
            document: document.clone(),
            command_revision: self.command_revision,
            replayed_command: self.replayed_command,
            can_undo: self.can_undo,
            can_redo: self.can_redo,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatus {
    /// Oldest to newest, with the next action at the end of each stack.
    pub undo_labels: Vec<String>,
    pub redo_labels: Vec<String>,
    pub permanent_command_ids: usize,
    pub history_bytes: usize,
}
#[derive(Debug, Clone, Deserialize)]
struct Entry {
    label: String,
    before_text: String,
    after_text: String,
    before_sha256: String,
    after_sha256: String,
    // Private entries never mutate after construction. Deserialization starts cold;
    // cache state is neither persisted nor part of semantic equality.
    #[serde(skip)]
    validated: OnceLock<bool>,
    #[serde(skip)]
    encoded: OnceLock<Option<Box<RawValue>>>,
}
#[derive(Serialize)]
struct EntryFields<'a> {
    label: &'a str,
    before_text: &'a str,
    after_text: &'a str,
    before_sha256: &'a str,
    after_sha256: &'a str,
}
struct EncodingBuffer {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for EncodingBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let required = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("history cache budget"))?;
        if required > self.bytes.capacity() {
            let target = required
                .max(self.bytes.capacity().saturating_mul(2))
                .min(self.limit);
            self.bytes
                .try_reserve_exact(target - self.bytes.len())
                .map_err(io::Error::other)?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Serialize for Entry {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let cached = self.encoded.get_or_init(|| {
            // At most one MiB per entry, and at most retained source bytes +
            // 512 bytes per entry in aggregate. Heavily escaped input falls back.
            let limit = self
                .before_text
                .len()
                .saturating_add(self.after_text.len())
                .saturating_add(512)
                .min(1024 * 1024);
            let mut buffer = EncodingBuffer {
                bytes: Vec::new(),
                limit,
            };
            serde_json::to_writer(&mut buffer, &self.fields()).ok()?;
            RawValue::from_string(String::from_utf8(buffer.bytes).ok()?).ok()
        });
        match cached {
            Some(raw) => raw.serialize(serializer),
            None => self.fields().serialize(serializer),
        }
    }
}
impl Entry {
    fn fields(&self) -> EntryFields<'_> {
        EntryFields {
            label: &self.label,
            before_text: &self.before_text,
            after_text: &self.after_text,
            before_sha256: &self.before_sha256,
            after_sha256: &self.after_sha256,
        }
    }
    fn valid_content(&self) -> bool {
        *self.validated.get_or_init(|| {
            self.label.len() <= 256
                && self.before_text.len() <= MAX_DOCUMENT_BYTES
                && self.after_text.len() <= MAX_DOCUMENT_BYTES
                && digest(&self.before_text) == self.before_sha256
                && digest(&self.after_text) == self.after_sha256
        })
    }
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        (
            &self.label,
            &self.before_text,
            &self.after_text,
            &self.before_sha256,
            &self.after_sha256,
        ) == (
            &other.label,
            &other.before_text,
            &other.after_text,
            &other.before_sha256,
            &other.after_sha256,
        )
    }
}
impl Eq for Entry {}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CommandReceipt {
    request_sha256: String,
    revision: u64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoryState {
    undo: Vec<Arc<Entry>>,
    redo: Vec<Arc<Entry>>,
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
            if !entry.valid_content() {
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
    next.history.undo.push(Arc::new(Entry {
        label,
        before_text: before.text.clone(),
        after_text: next.document.text.clone(),
        before_sha256: before.source_sha256.clone(),
        after_sha256: next.document.source_sha256.clone(),
        validated: OnceLock::new(),
        encoded: OnceLock::new(),
    }));
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
fn result(state: &State, command_revision: u64, replayed: bool) -> HistoryCommandStatus {
    HistoryCommandStatus {
        command_revision,
        replayed_command: replayed,
        can_undo: !state.history.undo.is_empty(),
        can_redo: !state.history.redo.is_empty(),
    }
}
fn check_command(
    state: &State,
    id: &str,
    fingerprint: &str,
) -> Result<Option<HistoryCommandStatus>> {
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
        let status = self.apply_group_status(group)?;
        Ok(status.with_document(
            &self
                .state
                .as_ref()
                .expect("successful command has source")
                .document,
        ))
    }
    /// Apply the same durable command without cloning a response document.
    pub fn apply_group_status(&mut self, group: GroupedEdit) -> Result<HistoryCommandStatus> {
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
        let status = self.undo_status(command)?;
        Ok(status.with_document(
            &self
                .state
                .as_ref()
                .expect("successful command has source")
                .document,
        ))
    }
    pub fn redo(&mut self, command: HistoryMove) -> Result<HistoryResult> {
        let status = self.redo_status(command)?;
        Ok(status.with_document(
            &self
                .state
                .as_ref()
                .expect("successful command has source")
                .document,
        ))
    }
    /// Undo using the same receipts and snapshots without a response source clone.
    pub fn undo_status(&mut self, command: HistoryMove) -> Result<HistoryCommandStatus> {
        self.move_history(command, false)
    }
    /// Redo using the same receipts and snapshots without a response source clone.
    pub fn redo_status(&mut self, command: HistoryMove) -> Result<HistoryCommandStatus> {
        self.move_history(command, true)
    }
    fn move_history(&mut self, command: HistoryMove, redo: bool) -> Result<HistoryCommandStatus> {
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
    #[test]
    fn shared_history_preserves_legacy_bytes_and_deserialization_revalidates() {
        #[derive(Serialize)]
        struct LegacyEntry<'a> {
            label: &'a str,
            before_text: &'a str,
            after_text: &'a str,
            before_sha256: &'a str,
            after_sha256: &'a str,
        }
        let entry = Arc::new(Entry {
            label: "Source edit".into(),
            before_text: "α before".into(),
            after_text: "β after".into(),
            before_sha256: digest("α before"),
            after_sha256: digest("β after"),
            validated: OnceLock::new(),
            encoded: OnceLock::new(),
        });
        assert!(entry.valid_content());
        let legacy = LegacyEntry {
            label: &entry.label,
            before_text: &entry.before_text,
            after_text: &entry.after_text,
            before_sha256: &entry.before_sha256,
            after_sha256: &entry.after_sha256,
        };
        let exact = serde_json::to_vec(&entry).unwrap();
        assert_eq!(exact, serde_json::to_vec(&legacy).unwrap());
        let history = HistoryState {
            undo: vec![entry.clone()],
            ..Default::default()
        };
        let shared = history.clone();
        assert!(Arc::ptr_eq(&history.undo[0], &shared.undo[0]));
        let decoded: Entry = serde_json::from_slice(&exact).unwrap();
        assert!(decoded.validated.get().is_none());
        assert_eq!(entry.as_ref(), &decoded);
        assert!(decoded.valid_content());
        let mut corrupted = serde_json::to_value(&entry).unwrap();
        corrupted["before_text"] = serde_json::json!("corrupted");
        let decoded: Entry = serde_json::from_value(corrupted).unwrap();
        assert!(!decoded.valid_content());
    }

    #[test]
    fn reopened_history_cannot_inherit_warm_validation_after_disk_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let initial = Document::new("p".into(), "main.tex".into(), 1, "before".into()).unwrap();
        store.initialize(initial.clone()).unwrap();
        store
            .replace_document(1, &initial.source_sha256, "after".into())
            .unwrap();
        let state = store.state.as_ref().unwrap();
        assert_eq!(state.history.undo[0].validated.get(), Some(&true));
        let mut corrupt = serde_json::to_value(state).unwrap();
        corrupt["history"]["undo"][0]["before_text"] = serde_json::json!("tampered");
        drop(store);
        std::fs::write(
            dir.path().join("document.json"),
            serde_json::to_vec(&corrupt).unwrap(),
        )
        .unwrap();
        assert!(Store::open(dir.path()).is_err());
    }
    #[test]
    fn encoded_cache_matches_original_json_and_bounds_escape_heavy_entries() {
        for text in [
            "naïve \"quoted\" \\ α\n".repeat(20),
            "\u{0001}".repeat(2000),
            "x".repeat(600_000),
        ] {
            let entry = Entry {
                label: "source".into(),
                before_sha256: digest(&text),
                after_sha256: digest("after"),
                before_text: text,
                after_text: "after".into(),
                validated: OnceLock::new(),
                encoded: OnceLock::new(),
            };
            let legacy = serde_json::to_vec(&entry.fields()).unwrap();
            assert_eq!(serde_json::to_vec(&entry).unwrap(), legacy);
            assert_eq!(serde_json::to_vec(&entry).unwrap(), legacy);
            assert_eq!(
                serde_json::to_value(&entry).unwrap(),
                serde_json::to_value(entry.fields()).unwrap()
            );
            if let Some(Some(raw)) = entry.encoded.get() {
                assert!(raw.get().len() <= 1024 * 1024);
                assert!(raw.get().len() <= entry.before_text.len() + entry.after_text.len() + 512);
            }
            if entry.before_text.starts_with('\u{0001}') {
                assert!(entry.encoded.get().unwrap().is_none());
            }
            let mut decoded: serde_json::Value = serde_json::from_slice(&legacy).unwrap();
            decoded["encoded"] = serde_json::json!("forged-cache");
            decoded["validated"] = serde_json::json!(true);
            decoded["before_text"] = serde_json::json!("corrupted");
            let decoded: Entry = serde_json::from_value(decoded).unwrap();
            assert!(decoded.encoded.get().is_none());
            assert!(decoded.validated.get().is_none());
            assert!(!decoded.valid_content());
        }
        let mut bounded = EncodingBuffer {
            bytes: Vec::new(),
            limit: 16,
        };
        bounded.write_all(b"1234567890123456").unwrap();
        assert!(bounded.write_all(b"x").is_err());
        assert_eq!(bounded.bytes.len(), 16);
        assert!(bounded.bytes.capacity() <= 16);
    }
    #[test]
    #[ignore = "explicit release-mode paired immutable history encoding benchmark"]
    fn paired_cached_history_encoding() {
        use std::time::Instant;
        #[derive(Serialize)]
        struct LegacyHistory<'a> {
            undo: Vec<EntryFields<'a>>,
            redo: Vec<EntryFields<'a>>,
            command_ids: &'a BTreeMap<String, CommandReceipt>,
        }
        let mut text = "x".repeat(521792);
        let mut history = HistoryState::default();
        for count in 1..=20 {
            let mut after = text.clone();
            after.replace_range(0..3, &format!("{count:03}"));
            history.undo.push(Arc::new(Entry {
                label: "Source edit".into(),
                before_text: text.clone(),
                before_sha256: digest(&text),
                after_sha256: digest(&after),
                after_text: after.clone(),
                validated: OnceLock::new(),
                encoded: OnceLock::new(),
            }));
            text = after;
            if ![5, 10, 20].contains(&count) {
                continue;
            }
            let legacy = LegacyHistory {
                undo: history.undo.iter().map(|entry| entry.fields()).collect(),
                redo: vec![],
                command_ids: &history.command_ids,
            };
            // Populate only the runtime-only serialization cache before warm pairs.
            let cold = Instant::now();
            let expected = serde_json::to_vec(&history).unwrap();
            let cold_us = cold.elapsed().as_micros();
            let mut cached_us = Vec::new();
            let mut legacy_us = Vec::new();
            for pair in 0..6 {
                let mut cached = Vec::new();
                let mut ordinary = Vec::new();
                for use_cache in if pair % 2 == 0 {
                    [true, false]
                } else {
                    [false, true]
                } {
                    let start = Instant::now();
                    if use_cache {
                        cached = serde_json::to_vec(&history).unwrap();
                        cached_us.push(start.elapsed().as_micros());
                    } else {
                        ordinary = serde_json::to_vec(&legacy).unwrap();
                        legacy_us.push(start.elapsed().as_micros());
                    }
                }
                assert_eq!(cached, ordinary);
                assert_eq!(cached, expected);
            }
            println!(
                "ENCODING {}",
                serde_json::json!({"entries":count,"encoded_bytes":expected.len(),
                "cold_us":cold_us,"cached_us":cached_us,"legacy_us":legacy_us,"exact_pairs":6})
            );
        }
    }
}

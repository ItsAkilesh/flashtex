//! Explicit bounded compaction: discard acknowledged payloads, never applied IDs.
use crate::{AppliedReceipt, Error, PreparedEdit, Result, Store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RetainedEditId {
    pub receipt: AppliedReceipt,
    pub prepared_sha256: String,
    pub document_after_sha256: String,
}
pub(crate) fn prepared_digest(edit: &PreparedEdit) -> Result<String> {
    let bytes = serde_json::to_vec(edit).map_err(|e| Error::new("invalid_edit", e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub snapshot_token: String,
    /// Compaction never authorizes forgetting edit/capture IDs or replay safety.
    pub acknowledge_permanent_id_retention: bool,
    pub acknowledged_through_revision: u64,
    pub keep_latest_confirmed: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionReport {
    pub compacted_payloads: usize,
    pub permanent_edit_ids: usize,
    pub pending_receipts: usize,
    pub bytes_before: usize,
    pub bytes_after: usize,
    pub snapshot_token: String,
}
impl Store {
    pub fn compact(&mut self, policy: RetentionPolicy) -> Result<CompactionReport> {
        let export = self.export_recovery()?;
        if !policy.acknowledge_permanent_id_retention {
            return Err(Error::new("retention_not_acknowledged", "compaction requires explicit acknowledgement that all applied IDs remain permanent"));
        }
        if policy.snapshot_token != export.snapshot_token {
            return Err(Error::new(
                "stale_recovery_snapshot",
                "ledger changed since retention review",
            ));
        }
        if policy.acknowledged_through_revision > export.current_document.revision
            || policy.keep_latest_confirmed > crate::MAX_EDIT_IDS
        {
            return Err(Error::new(
                "invalid_retention_policy",
                "revision/count exceeds current ledger bounds",
            ));
        }
        let mut next = self
            .state
            .as_ref()
            .expect("export checked initialization")
            .clone();
        let bytes_before = serde_json::to_vec(&next)
            .map_err(|e| Error::new("invalid_store", e.to_string()))?
            .len();
        let mut confirmed: Vec<_> = next
            .transactions
            .values()
            .filter(|t| t.confirmed)
            .map(|t| (t.receipt.new_revision, t.receipt.edit_id.clone()))
            .collect();
        confirmed.sort_by(|a, b| b.cmp(a));
        let mut compacted_payloads = 0;
        for (revision, id) in confirmed.into_iter().skip(policy.keep_latest_confirmed) {
            if revision > policy.acknowledged_through_revision {
                continue;
            }
            let tx = next.transactions.remove(&id).expect("ID collected above");
            let retained = RetainedEditId {
                prepared_sha256: prepared_digest(&tx.edit)?,
                receipt: tx.receipt,
                document_after_sha256: tx.document_after_sha256,
            };
            next.retained_ids.insert(id, retained);
            compacted_payloads += 1;
        }
        if compacted_payloads > 0 {
            // Older readers must reject a compacted store rather than ignoring
            // retained IDs and accidentally permitting duplicate application.
            next.schema_version = 2;
        }
        let permanent_edit_ids = next.transactions.len() + next.retained_ids.len();
        let bytes_after = serde_json::to_vec(&next)
            .map_err(|e| Error::new("invalid_store", e.to_string()))?
            .len();
        if compacted_payloads > 0 {
            self.commit(next)?;
        }
        Ok(CompactionReport {
            compacted_payloads,
            permanent_edit_ids,
            pending_receipts: self.recovery()?.len(),
            bytes_before,
            bytes_after,
            snapshot_token: self.export_recovery()?.snapshot_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::setup;
    fn policy(store: &Store) -> RetentionPolicy {
        RetentionPolicy {
            snapshot_token: store.export_recovery().unwrap().snapshot_token,
            acknowledge_permanent_id_retention: true,
            acknowledged_through_revision: store.document().unwrap().unwrap().revision,
            keep_latest_confirmed: 0,
        }
    }
    #[test]
    fn compaction_retains_replay_identity_after_restart_and_undo() {
        let (dir, mut store, mut edit) = setup();
        edit.replacement = "x".repeat(32000);
        let receipt = store.apply(edit.clone()).unwrap();
        store.confirm(&receipt).unwrap();
        let report = store.compact(policy(&store)).unwrap();
        assert_eq!(report.compacted_payloads, 1);
        assert_eq!(report.permanent_edit_ids, 1);
        assert!(report.bytes_after + 30000 < report.bytes_before);
        let current = store.document().unwrap().unwrap().clone();
        store
            .replace_document(current.revision, &current.source_sha256, "aé😀z".into())
            .unwrap();
        drop(store);
        let mut reopened = Store::open(dir.path()).unwrap();
        assert_eq!(reopened.apply(edit.clone()).unwrap(), receipt);
        assert_eq!(reopened.document().unwrap().unwrap().text, "aé😀z");
        reopened.confirm(&receipt).unwrap();
        let changed = PreparedEdit {
            replacement: "different".into(),
            ..edit.clone()
        };
        assert_eq!(
            reopened.apply(changed).unwrap_err().code,
            "edit_id_conflict"
        );
        let reused_capture = PreparedEdit {
            edit_id: "another-edit".into(),
            ..edit
        };
        assert_eq!(
            reopened.apply(reused_capture).unwrap_err().code,
            "capture_id_conflict"
        );
    }
    #[test]
    fn unconfirmed_payload_and_snapshot_are_never_compacted() {
        let (_dir, mut store, edit) = setup();
        store.apply(edit).unwrap();
        assert_eq!(store.compact(policy(&store)).unwrap().compacted_payloads, 0);
        assert!(store.recovery().unwrap()[0].document_before.is_some());
    }
    #[test]
    fn explicit_acknowledgement_and_latest_retention_are_enforced() {
        let (_dir, mut store, edit) = setup();
        let receipt = store.apply(edit).unwrap();
        store.confirm(&receipt).unwrap();
        let mut p = policy(&store);
        p.acknowledge_permanent_id_retention = false;
        assert_eq!(
            store.compact(p).unwrap_err().code,
            "retention_not_acknowledged"
        );
        let mut p = policy(&store);
        p.keep_latest_confirmed = 1;
        assert_eq!(store.compact(p).unwrap().compacted_payloads, 0);
    }
    #[test]
    fn failed_compaction_cannot_lose_source_or_ids() {
        let (dir, mut store, edit) = setup();
        let receipt = store.apply(edit.clone()).unwrap();
        store.confirm(&receipt).unwrap();
        store.failpoint = Some("after_rename");
        assert!(store.compact(policy(&store)).is_err());
        drop(store);
        let mut reopened = Store::open(dir.path()).unwrap();
        assert_eq!(reopened.apply(edit).unwrap(), receipt);
        assert_eq!(reopened.document().unwrap().unwrap().text, "a$x$z");
    }
}

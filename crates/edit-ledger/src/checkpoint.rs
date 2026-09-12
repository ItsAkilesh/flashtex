//! Internal ledger checkpoints, not native project files. Import is always
//! plan-then-approve and never overwrites divergent live source.
use crate::{AppliedReceipt, Document, Error, Result, State, Store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub const MAX_CHECKPOINT_BYTES: usize = crate::MAX_STORE_BYTES as usize + 4096;
pub mod archive;
pub(crate) fn new_store_id() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| Error::new("identity_unavailable", e.to_string()))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn checksum<T: Serialize>(value: &T) -> Result<String> {
    let bytes =
        serde_json::to_vec(value).map_err(|e| Error::new("invalid_checkpoint", e.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreIdentity {
    pub store_id: String,
    pub project_id: String,
    pub path: String,
}
impl StoreIdentity {
    fn from_state(state: &State) -> Self {
        Self {
            store_id: state.store_id.clone(),
            project_id: state.document.project_id.clone(),
            path: state.document.path.clone(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub format_version: u8,
    pub created_unix_ms: u64,
    pub identity: StoreIdentity,
    pub integrity_sha256: String,
    state: State,
}
impl Checkpoint {
    fn computed_digest(&self) -> Result<String> {
        checksum(&(
            "flashtex-ledger-checkpoint",
            self.format_version,
            self.created_unix_ms,
            &self.identity,
            &self.state,
        ))
    }
    pub fn validate(&self) -> Result<()> {
        self.validated_encoding().map(|_| ())
    }
    // Reuse the exact bytes checked against the size limit. Encoding used to
    // allocate/serialize once for validation and again for the return value.
    fn validated_encoding(&self) -> Result<Vec<u8>> {
        if self.format_version != 1 {
            return Err(Error::new(
                "checkpoint_version",
                "unsupported checkpoint format",
            ));
        }
        self.state.validate()?;
        if self.state.store_id.is_empty()
            || self.identity != StoreIdentity::from_state(&self.state)
            || self.computed_digest()? != self.integrity_sha256
        {
            return Err(Error::new(
                "checkpoint_integrity",
                "checkpoint identity or digest mismatch",
            ));
        }
        let bytes = serde_json::to_vec(self)
            .map_err(|e| Error::new("invalid_checkpoint", e.to_string()))?;
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Error::new(
                "checkpoint_too_large",
                "checkpoint size limit exceeded",
            ));
        }
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(Error::new(
                "checkpoint_too_large",
                "checkpoint size limit exceeded",
            ));
        }
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|e| Error::new("invalid_checkpoint", e.to_string()))?;
        value.validate()?;
        Ok(value)
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validated_encoding()
    }
    pub fn document(&self) -> &Document {
        &self.state.document
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAuthorization {
    pub acknowledge_private_source_export: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportAction {
    InitializeEmpty,
    SameSourceMetadata,
    NoOp,
    Blocked,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportPlan {
    pub plan_id: String,
    pub expected_identity: StoreIdentity,
    pub target_directory: String,
    pub checkpoint_digest: String,
    pub current_state_digest: Option<String>,
    pub current_document: Option<Document>,
    pub checkpoint_document: Document,
    pub action: ImportAction,
    pub conflicts: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAuthorization {
    pub approve_plan_id: String,
    pub allow_initialize_empty: bool,
    pub allow_same_source_metadata: bool,
}
#[derive(PartialEq, Eq)]
struct Binding {
    prepared_sha256: String,
    receipt: AppliedReceipt,
    confirmed: bool,
}
fn bindings(state: &State) -> Result<BTreeMap<String, Binding>> {
    let mut result = BTreeMap::new();
    for (id, tx) in &state.transactions {
        result.insert(
            id.clone(),
            Binding {
                prepared_sha256: crate::retention::prepared_digest(&tx.edit)?,
                receipt: tx.receipt.clone(),
                confirmed: tx.confirmed,
            },
        );
    }
    for (id, tx) in &state.retained_ids {
        result.insert(
            id.clone(),
            Binding {
                prepared_sha256: tx.prepared_sha256.clone(),
                receipt: tx.receipt.clone(),
                confirmed: true,
            },
        );
    }
    Ok(result)
}
impl Store {
    /// Authorization acknowledges that the result contains private source and
    /// retained history. It does not publish or write an external destination.
    pub fn export_checkpoint(&mut self, authorization: ExportAuthorization) -> Result<Checkpoint> {
        self.ready()?;
        if !authorization.acknowledge_private_source_export {
            return Err(Error::new(
                "export_not_authorized",
                "private source export requires explicit acknowledgement",
            ));
        }
        let mut state = self
            .state
            .as_ref()
            .ok_or_else(|| Error::new("document_missing", "initialize source first"))?
            .clone();
        if state.store_id.is_empty() {
            state.store_id = new_store_id()?;
            state.schema_version = 4;
            self.commit(state.clone())?;
        }
        let mut checkpoint = Checkpoint {
            format_version: 1,
            created_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| Error::new("clock_error", e.to_string()))?
                .as_millis() as u64,
            identity: StoreIdentity::from_state(&state),
            integrity_sha256: String::new(),
            state,
        };
        checkpoint.integrity_sha256 = checkpoint.computed_digest()?;
        checkpoint.validate()?;
        Ok(checkpoint)
    }
    /// Pure review step. A live source must match exactly; recovering a different
    /// source requires an empty isolated store, keeping the original untouched.
    pub fn plan_checkpoint_import(
        &self,
        checkpoint: &Checkpoint,
        expected_identity: StoreIdentity,
    ) -> Result<ImportPlan> {
        self.ready()?;
        checkpoint.validate()?;
        let mut conflicts = Vec::new();
        if checkpoint.identity != expected_identity {
            conflicts.push(
                "checkpoint does not match explicitly expected store/project/path identity".into(),
            );
        }
        let mut action = ImportAction::InitializeEmpty;
        if let Some(current) = &self.state {
            action = ImportAction::SameSourceMetadata;
            if StoreIdentity::from_state(current) != expected_identity {
                conflicts.push("target live store identity differs".into());
            }
            if current.document != checkpoint.state.document {
                conflicts.push("live source revision/hash differs; restore into an isolated empty store instead".into());
            }
            let candidate = bindings(&checkpoint.state)?;
            for (id, binding) in bindings(current)? {
                match candidate.get(&id) {
                    Some(other)
                        if other.prepared_sha256 == binding.prepared_sha256
                            && other.receipt == binding.receipt
                            && (!binding.confirmed || other.confirmed) => {}
                    _ => {
                        conflicts.push(format!(
                            "checkpoint would lose or regress permanent receipt/edit binding {id}"
                        ));
                        break;
                    }
                }
            }
            if !checkpoint.state.history.preserves(&current.history) {
                conflicts.push(
                    "checkpoint would lose history command IDs or retained undo/redo payloads"
                        .into(),
                );
            }
            if current == &checkpoint.state {
                action = ImportAction::NoOp;
            }
        }
        if !conflicts.is_empty() {
            action = ImportAction::Blocked;
        }
        let mut plan = ImportPlan {
            plan_id: String::new(),
            expected_identity,
            target_directory: std::fs::canonicalize(&self.root)?
                .to_string_lossy()
                .into_owned(),
            checkpoint_digest: checkpoint.integrity_sha256.clone(),
            current_state_digest: self.state.as_ref().map(checksum).transpose()?,
            current_document: self.state.as_ref().map(|s| s.document.clone()),
            checkpoint_document: checkpoint.state.document.clone(),
            action,
            conflicts,
        };
        plan.plan_id = checksum(&plan)?;
        Ok(plan)
    }
    pub fn apply_checkpoint_import(
        &mut self,
        checkpoint: &Checkpoint,
        plan: &ImportPlan,
        authorization: ImportAuthorization,
    ) -> Result<Document> {
        let fresh = self.plan_checkpoint_import(checkpoint, plan.expected_identity.clone())?;
        if fresh != *plan {
            return Err(Error::new(
                "stale_import_plan",
                "target state/checkpoint changed or recovery plan was modified",
            ));
        }
        if authorization.approve_plan_id != fresh.plan_id {
            return Err(Error::new(
                "import_not_authorized",
                "exact reviewed plan ID must be approved",
            ));
        }
        match fresh.action {
            ImportAction::Blocked => {
                return Err(Error::new("restore_conflict", fresh.conflicts.join("; ")))
            }
            ImportAction::InitializeEmpty if !authorization.allow_initialize_empty => {
                return Err(Error::new(
                    "import_not_authorized",
                    "empty-store restoration was not authorized",
                ))
            }
            ImportAction::SameSourceMetadata if !authorization.allow_same_source_metadata => {
                return Err(Error::new(
                    "import_not_authorized",
                    "same-source metadata recovery was not authorized",
                ))
            }
            ImportAction::NoOp => return Ok(checkpoint.state.document.clone()),
            _ => {}
        }
        self.commit(checkpoint.state.clone())?;
        Ok(checkpoint.state.document.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::setup;
    fn export(store: &mut Store) -> Checkpoint {
        store
            .export_checkpoint(ExportAuthorization {
                acknowledge_private_source_export: true,
            })
            .unwrap()
    }
    fn auth(plan: &ImportPlan) -> ImportAuthorization {
        ImportAuthorization {
            approve_plan_id: plan.plan_id.clone(),
            allow_initialize_empty: true,
            allow_same_source_metadata: true,
        }
    }
    #[test]
    fn export_identity_survives_restart_and_restores_all_receipts() {
        let (dir, mut store, edit) = setup();
        let receipt = store.apply(edit.clone()).unwrap();
        let checkpoint = export(&mut store);
        drop(store);
        let mut reopened = Store::open(dir.path()).unwrap();
        assert_eq!(export(&mut reopened).identity, checkpoint.identity);
        let target = tempfile::tempdir().unwrap();
        let mut target = Store::open(target.path()).unwrap();
        let plan = target
            .plan_checkpoint_import(&checkpoint, checkpoint.identity.clone())
            .unwrap();
        assert_eq!(plan.action, ImportAction::InitializeEmpty);
        target
            .apply_checkpoint_import(&checkpoint, &plan, auth(&plan))
            .unwrap();
        assert_eq!(target.apply(edit).unwrap(), receipt);
        assert_eq!(target.recovery().unwrap().len(), 1);
        assert_eq!(target.document().unwrap().unwrap().text, "a$x$z");
    }
    #[test]
    fn source_rollback_and_wrong_identity_are_blocked() {
        let (_dir, mut store, edit) = setup();
        let old = export(&mut store);
        store.apply(edit).unwrap();
        let plan = store
            .plan_checkpoint_import(&old, old.identity.clone())
            .unwrap();
        assert_eq!(plan.action, ImportAction::Blocked);
        assert_eq!(
            store
                .apply_checkpoint_import(&old, &plan, auth(&plan))
                .unwrap_err()
                .code,
            "restore_conflict"
        );
        let mut wrong = old.identity.clone();
        wrong.project_id = "other".into();
        assert_eq!(
            store.plan_checkpoint_import(&old, wrong).unwrap().action,
            ImportAction::Blocked
        );
    }
    #[test]
    fn confirmation_regression_is_blocked_even_with_same_source() {
        let (_dir, mut store, edit) = setup();
        let receipt = store.apply(edit).unwrap();
        let old = export(&mut store);
        store.confirm(&receipt).unwrap();
        assert_eq!(
            store
                .plan_checkpoint_import(&old, old.identity.clone())
                .unwrap()
                .action,
            ImportAction::Blocked
        );
    }
    #[test]
    fn integrity_and_format_are_checked_before_plan() {
        let (_dir, mut store, _) = setup();
        let checkpoint = export(&mut store);
        let bytes = checkpoint.encode().unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["created_unix_ms"] = 0.into();
        assert_eq!(
            Checkpoint::decode(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .code,
            "checkpoint_integrity"
        );
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["format_version"] = 99.into();
        assert_eq!(
            Checkpoint::decode(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .code,
            "checkpoint_version"
        );
    }
    #[test]
    fn export_import_require_explicit_authorization_and_unchanged_plan() {
        let (_dir, mut store, _) = setup();
        assert_eq!(
            store
                .export_checkpoint(ExportAuthorization {
                    acknowledge_private_source_export: false
                })
                .unwrap_err()
                .code,
            "export_not_authorized"
        );
        let checkpoint = export(&mut store);
        let plan = store
            .plan_checkpoint_import(&checkpoint, checkpoint.identity.clone())
            .unwrap();
        let mut denied = auth(&plan);
        denied.approve_plan_id = "not-reviewed".into();
        assert_eq!(
            store
                .apply_checkpoint_import(&checkpoint, &plan, denied)
                .unwrap_err()
                .code,
            "import_not_authorized"
        );
        let current = store.document().unwrap().unwrap().clone();
        store
            .replace_document(current.revision, &current.source_sha256, "changed".into())
            .unwrap();
        assert_eq!(
            store
                .apply_checkpoint_import(&checkpoint, &plan, auth(&plan))
                .unwrap_err()
                .code,
            "stale_import_plan"
        );
    }
    #[test]
    fn interrupted_import_recovers_complete_state() {
        let (_dir, mut source, edit) = setup();
        let receipt = source.apply(edit.clone()).unwrap();
        let checkpoint = export(&mut source);
        let dir = tempfile::tempdir().unwrap();
        let mut target = Store::open(dir.path()).unwrap();
        let plan = target
            .plan_checkpoint_import(&checkpoint, checkpoint.identity.clone())
            .unwrap();
        target.failpoint = Some("after_rename");
        assert!(target
            .apply_checkpoint_import(&checkpoint, &plan, auth(&plan))
            .is_err());
        drop(target);
        let mut target = Store::open(dir.path()).unwrap();
        assert_eq!(target.apply(edit).unwrap(), receipt);
        assert_eq!(target.document().unwrap().unwrap().text, "a$x$z");
    }
    #[test]
    fn same_source_newer_acknowledgement_can_be_restored_without_losing_ids() {
        let (_dir, mut source, edit) = setup();
        let receipt = source.apply(edit.clone()).unwrap();
        let pending = export(&mut source);
        let target_dir = tempfile::tempdir().unwrap();
        let mut target = Store::open(target_dir.path()).unwrap();
        let plan = target
            .plan_checkpoint_import(&pending, pending.identity.clone())
            .unwrap();
        target
            .apply_checkpoint_import(&pending, &plan, auth(&plan))
            .unwrap();
        source.confirm(&receipt).unwrap();
        let acknowledged = export(&mut source);
        let plan = target
            .plan_checkpoint_import(&acknowledged, acknowledged.identity.clone())
            .unwrap();
        assert_eq!(plan.action, ImportAction::SameSourceMetadata);
        target
            .apply_checkpoint_import(&acknowledged, &plan, auth(&plan))
            .unwrap();
        assert!(target.recovery().unwrap().is_empty());
        assert_eq!(target.apply(edit).unwrap(), receipt);
    }
}

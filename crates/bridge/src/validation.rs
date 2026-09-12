//! Compile a hypothetical reviewed edit without changing the editor or journal.
//! The caller configures the original FlashTeX compiler executable explicitly.
use crate::{
    digest, identifier, range, relative_path, BridgeError, Document, PreparedEdit, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const MAX_WIRE: usize = 8 * 1024 * 1024;
const MAX_STDERR: u64 = 64 * 1024;
const MAX_SAFE_INTEGER: u64 = (1u64 << 53) - 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotIdentity {
    pub path: String,
    pub revision: u64,
    pub source_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewEvidence {
    pub request_id: String,
    pub project_id: String,
    pub revision: u64,
    pub snapshots: Vec<SnapshotIdentity>,
    pub original_snapshots: Vec<SnapshotIdentity>,
    pub status: String,
    pub diagnostics: Vec<Value>,
    pub pages: Vec<Value>,
    pub source_is_hypothetical: bool,
    pub compiler_compatibility: String,
}

/// Contains immutable source copies; constructing or validating it does not
/// prepare, approve, persist, apply or confirm any native editor transaction.
pub struct ProposedCompilation {
    id: String,
    project: String,
    revision: u64,
    entry: String,
    documents: BTreeMap<String, Document>,
    originals: Vec<SnapshotIdentity>,
}
impl ProposedCompilation {
    pub fn new(id: &str, entry: &str, documents: &[Document], edit: &PreparedEdit) -> Result<Self> {
        identifier(id)?;
        relative_path(entry)?;
        if edit.expected_revision > MAX_SAFE_INTEGER {
            return Err(BridgeError::new(
                "validation_revision",
                "Compiler protocol requires exactly representable revisions",
            ));
        }
        let mut snapshots = BTreeMap::new();
        let mut source_bytes = 0usize;
        let mut originals = Vec::new();
        for doc in documents {
            source_bytes = source_bytes.saturating_add(doc.text.len());
            if source_bytes > MAX_WIRE || snapshots.len() >= 1024 {
                return Err(BridgeError::new(
                    "validation_too_large",
                    "Project snapshot exceeds validation limits",
                ));
            }
            relative_path(&doc.path)?;
            originals.push(SnapshotIdentity {
                path: doc.path.clone(),
                revision: doc.revision,
                source_sha256: digest(doc.text.as_bytes()),
            });
            if doc.project_id != edit.project_id
                || snapshots.insert(doc.path.clone(), doc.clone()).is_some()
            {
                return Err(BridgeError::new(
                    "validation_snapshot",
                    "Supply unique snapshots from exactly one project",
                ));
            }
        }
        if !snapshots.contains_key(entry) {
            return Err(BridgeError::new(
                "validation_entry_missing",
                "The project entry snapshot is required",
            ));
        }
        let doc = snapshots.get_mut(&edit.path).ok_or_else(|| {
            BridgeError::new("validation_target_missing", "Target snapshot is required")
        })?;
        range(&doc.text, edit.start_byte, edit.end_byte)?;
        if doc.revision != edit.expected_revision
            || digest(doc.text.as_bytes()) != edit.document_before_sha256
            || doc.text[edit.start_byte..edit.end_byte] != edit.removed_text
        {
            return Err(BridgeError::new(
                "validation_stale_edit",
                "Source no longer matches the reviewed edit",
            ));
        }
        doc.text
            .replace_range(edit.start_byte..edit.end_byte, &edit.replacement);
        let result = Self {
            originals,
            id: id.into(),
            project: edit.project_id.clone(),
            revision: edit.expected_revision,
            entry: entry.into(),
            documents: snapshots,
        };
        result.request_bytes()?;
        Ok(result)
    }
    pub fn request_bytes(&self) -> Result<Vec<u8>> {
        let docs: Vec<_> = self
            .documents
            .values()
            .map(|d| json!({"path":d.path,"text":d.text}))
            .collect();
        let mut bytes = serde_json::to_vec(
            &json!({"protocol_version":1,"id":self.id,"type":"compile",
            "payload":{"project_id":self.project,"revision":self.revision,"entry_path":self.entry,"documents":docs}}),
        )?;
        bytes.push(b'\n');
        if bytes.len() > MAX_WIRE {
            return Err(BridgeError::new(
                "validation_too_large",
                "Hypothetical compile request exceeds 8 MiB",
            ));
        }
        Ok(bytes)
    }
    fn source(&self, value: &Value) -> Result<()> {
        if value.is_null() {
            return Ok(());
        }
        let path = value
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("Missing source path"))?;
        let doc = self
            .documents
            .get(path)
            .ok_or_else(|| invalid("Source refers to an unknown snapshot"))?;
        let start = value
            .get("start_byte")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| invalid("Invalid source start"))?;
        let end = value
            .get("end_byte")
            .and_then(Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| invalid("Invalid source end"))?;
        range(&doc.text, start, end)
            .map_err(|_| invalid("Compiler source range is not a valid UTF-8 span"))
    }
    pub fn response(&self, bytes: &[u8]) -> Result<ReviewEvidence> {
        if bytes.len() > MAX_WIRE {
            return Err(invalid("Compiler response exceeds limit"));
        }
        let response: Value = serde_json::from_slice(bytes)
            .map_err(|_| invalid("Compiler response is not one JSON object"))?;
        if response["protocol_version"] != 1
            || response["id"] != self.id
            || response["type"] != "compile_result"
        {
            return Err(invalid(
                "Compiler response ID/version/type does not match the request",
            ));
        }
        let p = &response["payload"];
        if p["project_id"] != self.project || p["revision"].as_u64() != Some(self.revision) {
            return Err(invalid("Compiler returned another project or revision"));
        }
        let status = p["status"]
            .as_str()
            .filter(|s| matches!(*s, "ok" | "recovered" | "failed"))
            .ok_or_else(|| invalid("Invalid compiler status"))?;
        let diagnostics = p["diagnostics"]
            .as_array()
            .ok_or_else(|| invalid("Missing compiler diagnostics"))?;
        for d in diagnostics {
            if !matches!(d["severity"].as_str(), Some("error" | "warning"))
                || !d["message"].is_string()
                || d.get("source").is_none()
                || !(d["recovery"].is_null() || d["recovery"].is_string())
            {
                return Err(invalid("Invalid compiler diagnostic"));
            }
            self.source(&d["source"])?;
        }
        let pages = p["pages"]
            .as_array()
            .ok_or_else(|| invalid("Missing compiler pages"))?;
        for (index, page) in pages.iter().enumerate() {
            if page["number"].as_u64() != Some(index as u64 + 1) {
                return Err(invalid("Invalid page sequence"));
            }
            for key in ["width_pt", "height_pt"] {
                positive(&page[key])?;
            }
            for item in page["items"]
                .as_array()
                .ok_or_else(|| invalid("Missing page items"))?
            {
                if item["kind"] != "text" || !item["text"].is_string() {
                    return Err(invalid("Unsupported compiler display primitive"));
                }
                positive(&item["font_size_pt"])?;
                for key in ["x_pt", "baseline_y_pt"] {
                    finite(&item[key])?;
                }
                if item.get("source").is_none() {
                    return Err(invalid("Missing item source"));
                }
                self.source(&item["source"])?;
            }
        }
        Ok(ReviewEvidence { request_id:self.id.clone(),project_id:self.project.clone(),revision:self.revision,
            snapshots:self.documents.values().map(|d|SnapshotIdentity { path:d.path.clone(),revision:d.revision,source_sha256:digest(d.text.as_bytes()) }).collect(),
            original_snapshots:self.originals.clone(),
            status:status.into(),diagnostics:diagnostics.clone(),pages:pages.clone(),source_is_hypothetical:true,
            compiler_compatibility:"Evidence reflects this compiler's reported subset; success does not prove full LaTeX or multi-file compatibility".into() })
    }
}
fn invalid(message: &str) -> BridgeError {
    BridgeError::new("validation_invalid_response", message)
}
fn finite(v: &Value) -> Result<f64> {
    v.as_f64()
        .filter(|n| n.is_finite())
        .ok_or_else(|| invalid("Non-finite or missing geometry"))
}
fn positive(v: &Value) -> Result<()> {
    if finite(v)? > 0.0 {
        Ok(())
    } else {
        Err(invalid("Nonpositive geometry"))
    }
}

pub struct CompilerValidator {
    pub executable: PathBuf,
    pub timeout: Duration,
}
impl CompilerValidator {
    /// File-backed IO prevents pipe deadlocks. Output files are polled for limits;
    /// an overproducing child is killed and never parsed beyond the read cap.
    pub fn validate(&self, request: &ProposedCompilation) -> Result<ReviewEvidence> {
        if self.timeout.is_zero() || self.timeout > Duration::from_secs(60) {
            return Err(BridgeError::new(
                "validation_timeout_config",
                "Set a timeout in (0,60] seconds",
            ));
        }
        let mut input = tempfile::tempfile()?;
        input.write_all(&request.request_bytes()?)?;
        input.seek(SeekFrom::Start(0))?;
        let mut output = tempfile::tempfile()?;
        let error = tempfile::tempfile()?;
        let mut child = Command::new(&self.executable)
            .stdin(Stdio::from(input))
            .stdout(Stdio::from(output.try_clone()?))
            .stderr(Stdio::from(error.try_clone()?))
            .spawn()
            .map_err(|_| {
                BridgeError::new(
                    "validation_launch",
                    "Could not launch the configured FlashTeX compiler",
                )
            })?;
        let deadline = Instant::now() + self.timeout;
        let execution = (|| -> Result<()> {
            loop {
                if output.metadata()?.len() > MAX_WIRE as u64
                    || error.metadata()?.len() > MAX_STDERR
                {
                    return Err(BridgeError::new(
                        "validation_output_limit",
                        "Compiler exceeded output limits",
                    ));
                }
                if let Some(status) = child.try_wait()? {
                    return if status.success() {
                        Ok(())
                    } else {
                        Err(BridgeError::new(
                            "validation_compiler_failed",
                            "Compiler process failed; source is unchanged",
                        ))
                    };
                }
                if Instant::now() >= deadline {
                    return Err(BridgeError::new(
                        "validation_timeout",
                        "Compiler validation timed out; source is unchanged",
                    ));
                }
                thread::sleep(Duration::from_millis(5));
            }
        })();
        if execution.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        execution?;
        output.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        Read::take(&mut output, MAX_WIRE as u64 + 1).read_to_end(&mut bytes)?;
        request.response(&bytes)
    }
}

impl crate::Bridge {
    /// Validate an unapproved proposal for display in the review UI. This never
    /// calls prepare_insert, saves a journal record or advances source revisions.
    pub fn validate_capture(
        &self,
        request_id: &str,
        capture_id: &str,
        expected_revision: u64,
        entry_path: &str,
        compiler: &CompilerValidator,
    ) -> Result<ReviewEvidence> {
        let record = self.store.require(capture_id)?;
        if record.rejected || record.applied.is_some() {
            return Err(BridgeError::new(
                "validation_capture_closed",
                "Rejected or applied captures cannot be validated for insertion",
            ));
        }
        let proposal = record.proposal.as_ref().ok_or_else(|| {
            BridgeError::new(
                "proposal_missing",
                "Convert the capture before validating it",
            )
        })?;
        proposal.validate()?;
        self.verify_proposal_context(&record)?;
        let anchor = self.capture_anchor(&record.capture)?;
        let doc = self.document(&anchor.project_id, &anchor.path)?;
        if doc.revision != expected_revision {
            return Err(BridgeError::new(
                "revision_conflict",
                "Validate against the current source revision",
            ));
        }
        let edit = PreparedEdit {
            capture_id: capture_id.into(),
            edit_id: format!("validation-{capture_id}"),
            project_id: anchor.project_id.clone(),
            path: anchor.path.clone(),
            expected_revision,
            start_byte: anchor.start_byte,
            end_byte: anchor.end_byte,
            removed_text: doc.text[anchor.start_byte..anchor.end_byte].into(),
            replacement: proposal.latex.clone(),
            document_before_sha256: digest(doc.text.as_bytes()),
        };
        let documents: Vec<_> = self
            .documents
            .values()
            .filter(|d| d.project_id == anchor.project_id)
            .cloned()
            .collect();
        let correlation = format!("validation-{}", digest(request_id.as_bytes()));
        let request = ProposedCompilation::new(&correlation, entry_path, &documents, &edit)?;
        compiler.validate(&request)
    }
}

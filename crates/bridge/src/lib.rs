//! Durable capture receipt and reviewed source edits. No TeX engine is embedded.
pub mod context;
pub mod grok;
pub mod store;
pub mod validation;

use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt, io::Cursor};

pub const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_DOCUMENT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_CONTEXT_BYTES: usize = 16 * 1024;
pub const MAX_LATEX_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BridgeError {
    pub code: String,
    pub message: String,
}
impl BridgeError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for BridgeError {}
impl From<std::io::Error> for BridgeError {
    fn from(e: std::io::Error) -> Self {
        Self::new("storage_error", e.to_string())
    }
}
impl From<serde_json::Error> for BridgeError {
    fn from(e: serde_json::Error) -> Self {
        Self::new("invalid_json", e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, BridgeError>;

pub fn identifier(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(BridgeError::new(
            "invalid_id",
            "IDs require 1–128 ASCII letters, digits, hyphens or underscores",
        ));
    }
    Ok(())
}
pub fn relative_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path.contains('\0')
        || path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
    {
        return Err(BridgeError::new(
            "invalid_path",
            "Expected a normalized project-relative path",
        ));
    }
    Ok(())
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn range(text: &str, start: usize, end: usize) -> Result<()> {
    if start > end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return Err(BridgeError::new(
            "invalid_source_range",
            "Offsets must delimit complete UTF-8 scalars in the current source",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureImage {
    pub mime_type: String,
    pub data_base64: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureSubmit {
    pub capture_id: String,
    pub destination_id: String,
    pub base_revision: u64,
    pub image: CaptureImage,
    pub instructions: String,
}
impl CaptureSubmit {
    /// Validates the capture and, in place, normalizes the image: phone photos are
    /// near-universally stored pre-rotation with an EXIF orientation tag, and
    /// decoding raw pixels while ignoring it would send sideways or upside-down
    /// handwriting to Grok. When a non-default orientation is found, the pixels are
    /// rotated/flipped to match it and losslessly re-encoded as PNG so every later
    /// reader (including the Grok request body) sees an already-upright image.
    pub fn validate(&mut self) -> Result<()> {
        identifier(&self.capture_id)?;
        identifier(&self.destination_id)?;
        if self.instructions.len() > 4096 {
            return Err(BridgeError::new(
                "instructions_too_large",
                "Instructions exceed 4096 UTF-8 bytes",
            ));
        }
        if self.image.data_base64.len() > MAX_IMAGE_BYTES.div_ceil(3) * 4 {
            return Err(BridgeError::new(
                "image_too_large",
                "Encoded image exceeds 8 MiB decoded limit",
            ));
        }
        let bytes = STANDARD
            .decode(&self.image.data_base64)
            .map_err(|_| BridgeError::new("invalid_image", "Invalid base64 image"))?;
        if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
            return Err(BridgeError::new(
                "image_too_large",
                "Image must contain 1–8 MiB of encoded image bytes",
            ));
        }
        let format = match self.image.mime_type.as_str() {
            "image/png" => image::ImageFormat::Png,
            "image/jpeg" => image::ImageFormat::Jpeg,
            _ => {
                return Err(BridgeError::new(
                    "unsupported_image",
                    "Only PNG and JPEG captures are accepted; convert HEIC/HEIF or other phone formats to JPEG before capture",
                ))
            }
        };
        let invalid_image = || {
            BridgeError::new(
                "invalid_image",
                "Image is malformed, MIME-mismatched or exceeds decoded image limits",
            )
        };

        let mut limits = image::Limits::default();
        limits.max_image_width = Some(8192);
        limits.max_image_height = Some(8192);
        limits.max_alloc = Some(64 * 1024 * 1024);

        let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
        reader.limits(limits.clone());
        let mut decoder = reader.into_decoder().map_err(|_| invalid_image())?;
        // `into_decoder` checks width/height against `limits` at construction, but
        // — unlike `ImageReader::decode` — it does not also bound the decoded pixel
        // buffer's allocation. Reproduce that guard explicitly so a small file can't
        // declare huge-but-in-range dimensions and force a large allocation (the
        // classic decompression-bomb shape), before any pixel data is decoded.
        limits
            .reserve(decoder.total_bytes())
            .map_err(|_| invalid_image())?;
        let orientation = decoder.orientation().map_err(|_| invalid_image())?;
        let mut decoded =
            image::DynamicImage::from_decoder(decoder).map_err(|_| invalid_image())?;

        if orientation != image::metadata::Orientation::NoTransforms {
            decoded.apply_orientation(orientation);
            let mut normalized = Vec::new();
            decoded
                .write_to(Cursor::new(&mut normalized), image::ImageFormat::Png)
                .map_err(|_| invalid_image())?;
            if normalized.len() > MAX_IMAGE_BYTES {
                return Err(BridgeError::new(
                    "image_too_large",
                    "Orientation-corrected image exceeds 8 MiB decoded limit",
                ));
            }
            self.image.mime_type = "image/png".to_string();
            self.image.data_base64 = STANDARD.encode(&normalized);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub latex: String,
    pub ambiguities: Vec<String>,
    pub required_dependencies: Vec<String>,
}
impl Proposal {
    pub fn validate(&self) -> Result<()> {
        if self.latex.is_empty() || self.latex.len() > MAX_LATEX_BYTES || self.latex.contains('\0')
        {
            return Err(BridgeError::new(
                "invalid_proposal",
                "LaTeX must contain 1–65536 UTF-8 bytes without NUL",
            ));
        }
        // Explicit control-sequence denylist and structural checks (shell
        // escape, file I/O, catcode/macro redefinition, unbalanced grouping,
        // embedded `\end{document}`, bidi-override characters, ...). See
        // `validation::scan_latex` for the full threat model.
        validation::scan_latex(&self.latex)?;
        for list in [&self.ambiguities, &self.required_dependencies] {
            if list.len() > 32 || list.iter().any(|v| v.len() > 2048 || v.contains('\0')) {
                return Err(BridgeError::new(
                    "invalid_proposal",
                    "Proposal annotations exceed bounded limits",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextDependency {
    pub path: String,
    pub revision: u64,
    pub source_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Context {
    pub project_id: String,
    pub path: String,
    pub revision: u64,
    pub source_before: String,
    pub selected_source: String,
    pub source_after: String,
    pub definitions: Vec<String>,
    pub supported_features: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<ContextDependency>,
}
pub trait Converter {
    fn convert(&self, capture: &CaptureSubmit, context: &Context) -> Result<Proposal>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub project_id: String,
    pub path: String,
    pub revision: u64,
    pub text: String,
}
/// One revision-checked UTF-8 source edit; shares the document_edit wire schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EditRequest {
    pub project_id: String,
    pub path: String,
    pub base_revision: u64,
    pub revision: u64,
    pub start_byte: usize,
    pub end_byte: usize,
    pub replacement: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Anchor {
    pub destination_id: String,
    pub project_id: String,
    pub path: String,
    pub pinned_revision: u64,
    pub current_revision: u64,
    pub start_byte: usize,
    pub end_byte: usize,
    pub valid: bool,
    pub binding: AnchorBinding,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnchorBinding {
    pub project_id: String,
    pub path: String,
    pub revision: u64,
    pub start_byte: usize,
    pub end_byte: usize,
    pub source_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedEdit {
    pub capture_id: String,
    pub edit_id: String,
    pub project_id: String,
    pub path: String,
    pub expected_revision: u64,
    pub start_byte: usize,
    pub end_byte: usize,
    pub removed_text: String,
    pub replacement: String,
    pub document_before_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppliedEdit {
    pub edit_id: String,
    pub new_revision: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaptureRecord {
    pub schema_version: u8,
    pub capture: CaptureSubmit,
    pub request_sha256: String,
    pub proposal: Option<Proposal>,
    pub context: Option<Context>,
    pub prepared: Option<PreparedEdit>,
    pub applied: Option<AppliedEdit>,
    #[serde(default)]
    pub rejected: bool,
    #[serde(default)]
    pub destination_binding: Option<AnchorBinding>,
}

pub struct Bridge {
    pub store: store::Store,
    documents: BTreeMap<(String, String), Document>,
    anchors: BTreeMap<String, Anchor>,
}
impl Bridge {
    pub fn new(store: store::Store) -> Self {
        Self {
            store,
            documents: BTreeMap::new(),
            anchors: BTreeMap::new(),
        }
    }
    pub fn open_document(&mut self, doc: Document) -> Result<()> {
        identifier(&doc.project_id)?;
        relative_path(&doc.path)?;
        if doc.text.len() > MAX_DOCUMENT_BYTES {
            return Err(BridgeError::new(
                "document_too_large",
                "Document exceeds 8 MiB",
            ));
        }
        let key = (doc.project_id.clone(), doc.path.clone());
        if let Some(old) = self.documents.get(&key) {
            if old == &doc {
                return Ok(());
            }
            if doc.revision <= old.revision {
                return Err(BridgeError::new(
                    "revision_conflict",
                    "Snapshots must advance revision when source changes",
                ));
            }
            for anchor in self
                .anchors
                .values_mut()
                .filter(|a| a.project_id == doc.project_id && a.path == doc.path)
            {
                anchor.valid = false;
            }
        }
        self.documents.insert(key, doc);
        Ok(())
    }
    pub fn document(&self, project: &str, path: &str) -> Result<&Document> {
        self.documents
            .get(&(project.into(), path.into()))
            .ok_or_else(|| {
                BridgeError::new(
                    "document_missing",
                    "Open the source snapshot on the Mac first",
                )
            })
    }
    pub fn pin(
        &mut self,
        destination: &str,
        project: &str,
        path: &str,
        revision: u64,
        start: usize,
        end: usize,
    ) -> Result<Anchor> {
        identifier(destination)?;
        let doc = self.document(project, path)?;
        if doc.revision != revision {
            return Err(BridgeError::new(
                "revision_conflict",
                "Pin against the current source revision",
            ));
        }
        range(&doc.text, start, end)?;
        let anchor = Anchor {
            destination_id: destination.into(),
            project_id: project.into(),
            path: path.into(),
            pinned_revision: revision,
            current_revision: revision,
            start_byte: start,
            end_byte: end,
            valid: true,
            binding: AnchorBinding {
                project_id: project.into(),
                path: path.into(),
                revision,
                start_byte: start,
                end_byte: end,
                source_sha256: digest(doc.text.as_bytes()),
            },
        };
        if let Some(old) = self.anchors.get(destination) {
            if old != &anchor {
                return Err(BridgeError::new(
                    "destination_conflict",
                    "Use a new destination ID when repinning a different target",
                ));
            }
        }
        self.anchors.insert(destination.into(), anchor.clone());
        Ok(anchor)
    }
    pub fn edit(&mut self, request: &EditRequest) -> Result<()> {
        let project = request.project_id.as_str();
        let path = request.path.as_str();
        let base = request.base_revision;
        let revision = request.revision;
        let start = request.start_byte;
        let end = request.end_byte;
        let replacement = request.replacement.as_str();
        let doc = self.document(project, path)?;
        if doc.revision != base || revision <= base {
            return Err(BridgeError::new(
                "revision_conflict",
                "Edit must advance the exact current revision",
            ));
        }
        range(&doc.text, start, end)?;
        if doc.text.len() - (end - start) + replacement.len() > MAX_DOCUMENT_BYTES {
            return Err(BridgeError::new(
                "document_too_large",
                "Edit exceeds document size limit",
            ));
        }
        let mut next = doc.clone();
        next.text.replace_range(start..end, replacement);
        next.revision = revision;
        let shift = replacement.len() as i64 - (end - start) as i64;
        for a in self
            .anchors
            .values_mut()
            .filter(|a| a.project_id == project && a.path == path && a.valid)
        {
            // Insertion exactly at the target is ambiguous; invalidate instead of guessing affinity.
            if (start == end && start >= a.start_byte && start <= a.end_byte)
                || (start < a.end_byte && end > a.start_byte)
                || (a.start_byte == a.end_byte && start <= a.start_byte && end > a.start_byte)
            {
                a.valid = false;
            } else if end <= a.start_byte {
                a.start_byte = (a.start_byte as i64 + shift) as usize;
                a.end_byte = (a.end_byte as i64 + shift) as usize;
            }
            a.current_revision = revision;
        }
        self.documents.insert((project.into(), path.into()), next);
        Ok(())
    }
    fn capture_anchor(&self, capture: &CaptureSubmit) -> Result<&Anchor> {
        let a = self
            .anchors
            .get(&capture.destination_id)
            .filter(|a| a.valid)
            .ok_or_else(|| {
                BridgeError::new(
                    "destination_reselection_required",
                    "The pinned target is missing or changed ambiguously",
                )
            })?;
        if capture.base_revision != a.pinned_revision {
            return Err(BridgeError::new(
                "revision_conflict",
                "Capture does not refer to the pinned destination revision",
            ));
        }
        if let Some(record) = self.store.get(&capture.capture_id)? {
            if record.destination_binding.as_ref() != Some(&a.binding) {
                return Err(BridgeError::new("destination_reselection_required", "Restored destination does not match this capture's durably bound source and range"));
            }
        }
        Ok(a)
    }
    pub fn receive(&mut self, mut capture: CaptureSubmit) -> Result<CaptureRecord> {
        capture.validate()?;
        if let Some(old) = self.store.get(&capture.capture_id)? {
            if old.capture != capture {
                return Err(BridgeError::new(
                    "capture_id_conflict",
                    "Capture ID already belongs to different content",
                ));
            }
            return Ok(old);
        }
        let destination_binding = Some(self.capture_anchor(&capture)?.binding.clone());
        let record = CaptureRecord {
            schema_version: 1,
            request_sha256: digest(&serde_json::to_vec(&capture)?),
            capture,
            proposal: None,
            context: None,
            prepared: None,
            applied: None,
            rejected: false,
            destination_binding,
        };
        self.store.save(&record)?;
        Ok(record)
    }
    pub fn context(
        &self,
        capture: &CaptureSubmit,
        supported_features: Vec<String>,
    ) -> Result<Context> {
        let a = self.capture_anchor(capture)?;
        let doc = self.document(&a.project_id, &a.path)?;
        context::build(
            doc,
            a.start_byte,
            a.end_byte,
            self.documents.values(),
            supported_features,
        )
    }

    pub fn convert(
        &mut self,
        capture_id: &str,
        supported: Vec<String>,
        converter: &dyn Converter,
    ) -> Result<CaptureRecord> {
        let mut record = self.store.require(capture_id)?;
        if record.rejected {
            return Err(BridgeError::new(
                "capture_rejected",
                "This capture was rejected during review",
            ));
        }
        if record.proposal.is_some() && (record.prepared.is_some() || record.applied.is_some()) {
            // An issued edit cannot be replaced by another proposal; its receipt
            // must be reconciled even if source dependencies have since changed.
            return Ok(record);
        }
        let context = self.context(&record.capture, supported)?;
        if record.proposal.is_some()
            && record.context.as_ref().is_some_and(|old| {
                !old.dependencies.is_empty()
                    && old.dependencies == context.dependencies
                    && old.supported_features == context.supported_features
            })
        {
            return Ok(record);
        }
        let mut proposal = converter.convert(&record.capture, &context)?;
        proposal.validate()?;
        // Hard violations already failed above; surface non-fatal but
        // reviewer-worthy findings (e.g. deep nesting, `\loop`/`\repeat`)
        // through the same `ambiguities` channel the review UI already
        // renders alongside `latex`, so a human sees them before approving
        // `prepare_insert`. Re-validate afterward so an unreasonable number
        // of findings cannot silently exceed the proposal's own bounds.
        for advisory in validation::scan_latex(&proposal.latex)?.advisories {
            if !proposal.ambiguities.contains(&advisory) {
                proposal.ambiguities.push(advisory);
            }
        }
        proposal.validate()?;
        record.context = Some(context);
        record.proposal = Some(proposal);
        self.store.save(&record)?;
        Ok(record)
    }
    fn verify_proposal_context(&self, record: &CaptureRecord) -> Result<()> {
        let Some(saved) = record.context.as_ref() else {
            return Err(BridgeError::new("proposal_context_stale", "Conversion context is missing; explicitly convert again and review the new proposal"));
        };
        let current = self.context(&record.capture, saved.supported_features.clone())?;
        if saved.dependencies.is_empty() || saved.dependencies != current.dependencies {
            let action = if record.prepared.is_some() {
                "An edit was already issued: reconcile its Mac receipt before starting a new capture"
            } else {
                "Explicitly convert this capture again and review the new proposal before approval"
            };
            return Err(BridgeError::new(
                "proposal_context_stale",
                format!("A source dependency changed or is unavailable. {action}"),
            ));
        }
        Ok(())
    }
    pub fn prepare_insert(
        &mut self,
        capture_id: &str,
        expected_revision: u64,
        approved: bool,
    ) -> Result<PreparedEdit> {
        if !approved {
            return Err(BridgeError::new(
                "review_required",
                "Explicit review approval is required",
            ));
        }
        let mut record = self.store.require(capture_id)?;
        if record.rejected {
            return Err(BridgeError::new(
                "capture_rejected",
                "This capture was rejected during review",
            ));
        }
        if record.applied.is_some() {
            return Err(BridgeError::new(
                "already_applied",
                "This capture was already inserted",
            ));
        }
        if let Some(edit) = &record.prepared {
            let doc = self.document(&edit.project_id, &edit.path)?;
            if edit.expected_revision == expected_revision
                && doc.revision == expected_revision
                && digest(doc.text.as_bytes()) == edit.document_before_sha256
            {
                self.verify_proposal_context(&record)?;
                return Ok(edit.clone());
            }
            return Err(BridgeError::new(
                "revision_conflict",
                "Previously prepared edit requires receipt reconciliation",
            ));
        }
        let a = self.capture_anchor(&record.capture)?;
        let doc = self.document(&a.project_id, &a.path)?;
        if doc.revision != expected_revision {
            return Err(BridgeError::new(
                "revision_conflict",
                "Review and prepare against the current Mac source",
            ));
        }
        let proposal = record.proposal.as_ref().ok_or_else(|| {
            BridgeError::new(
                "proposal_missing",
                "Convert the capture before reviewing insertion",
            )
        })?;
        self.verify_proposal_context(&record)?;
        if doc.text.len() - (a.end_byte - a.start_byte) + proposal.latex.len() > MAX_DOCUMENT_BYTES
        {
            return Err(BridgeError::new(
                "document_too_large",
                "Proposed edit exceeds document limit",
            ));
        }
        let edit = PreparedEdit {
            capture_id: capture_id.into(),
            edit_id: format!("capture-{capture_id}"),
            project_id: a.project_id.clone(),
            path: a.path.clone(),
            expected_revision,
            start_byte: a.start_byte,
            end_byte: a.end_byte,
            removed_text: doc.text[a.start_byte..a.end_byte].into(),
            replacement: proposal.latex.clone(),
            document_before_sha256: digest(doc.text.as_bytes()),
        };
        record.prepared = Some(edit.clone());
        self.store.save(&record)?;
        Ok(edit)
    }
    pub fn reject(&mut self, capture_id: &str) -> Result<()> {
        let mut record = self.store.require(capture_id)?;
        if record.prepared.is_some() || record.applied.is_some() {
            return Err(BridgeError::new("receipt_conflict", "A prepared edit requires Mac ledger reconciliation; rejection cannot revoke an issued edit"));
        }
        record.rejected = true;
        self.store.save(&record)
    }
    pub fn confirm_insert(
        &mut self,
        capture_id: &str,
        edit_id: &str,
        new_revision: u64,
    ) -> Result<AppliedEdit> {
        let mut record = self.store.require(capture_id)?;
        if let Some(applied) = &record.applied {
            if applied.edit_id == edit_id && applied.new_revision == new_revision {
                return Ok(applied.clone());
            }
            return Err(BridgeError::new(
                "receipt_conflict",
                "Capture already has a different insertion receipt",
            ));
        }
        let edit = record.prepared.as_ref().ok_or_else(|| {
            BridgeError::new(
                "review_required",
                "Prepare a reviewed edit before confirming it",
            )
        })?;
        if edit.edit_id != edit_id || new_revision <= edit.expected_revision {
            return Err(BridgeError::new("receipt_conflict", "Invalid edit receipt"));
        }
        let doc = self.document(&edit.project_id, &edit.path)?;
        if doc.revision != edit.expected_revision
            || digest(doc.text.as_bytes()) != edit.document_before_sha256
        {
            return Err(BridgeError::new(
                "revision_conflict",
                "Source changed after insertion was prepared; reconcile Mac edit ledger",
            ));
        }
        // Persist the receipt before mutating the in-memory snapshot. On restart the Mac
        // resends its snapshot and its persisted edit ledger, never blindly replays edits.
        let applied = AppliedEdit {
            edit_id: edit_id.into(),
            new_revision,
        };
        record.applied = Some(applied.clone());
        self.store.save(&record)?;
        self.edit(&EditRequest {
            project_id: edit.project_id.clone(),
            path: edit.path.clone(),
            base_revision: edit.expected_revision,
            revision: new_revision,
            start_byte: edit.start_byte,
            end_byte: edit.end_byte,
            replacement: edit.replacement.clone(),
        })?;
        Ok(applied)
    }
}

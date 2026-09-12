//! Opt-in exact outline export through the original PDF owner's container API.
//! Original font/GID/source identity stays in evidence; no embedded text claim.
use crate::{digest, mixed::BoundedOutput, pdf_stream::*, MAX_MESSAGE_BYTES, TICKS_PER_BP};
use flashtex_pdf::exact::{self, Content, Decimal, ExactDocument, ExactPage};
use serde_json::Value;
#[derive(Debug)]
pub enum ExportError {
    Stream(PdfStreamError),
    Backend(exact::ExactError),
    Budget,
    Unsupported(&'static str),
    Verification(String),
}
impl From<PdfStreamError> for ExportError {
    fn from(e: PdfStreamError) -> Self {
        Self::Stream(e)
    }
}
impl From<exact::ExactError> for ExportError {
    fn from(e: exact::ExactError) -> Self {
        Self::Backend(e)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct ExportLimits {
    pub max_pages: usize,
    pub max_content_bytes: usize,
    pub max_pdf_bytes: usize,
    pub max_evidence_bytes: usize,
}
impl Default for ExportLimits {
    fn default() -> Self {
        Self {
            max_pages: 256,
            max_content_bytes: MAX_MESSAGE_BYTES,
            max_pdf_bytes: 2 * MAX_MESSAGE_BYTES,
            max_evidence_bytes: MAX_MESSAGE_BYTES,
        }
    }
}
pub struct ExportedPdf {
    pdf: Vec<u8>,
    evidence: Vec<u8>,
}
impl ExportedPdf {
    pub fn bytes(&self) -> &[u8] {
        &self.pdf
    }
    pub fn evidence_bytes(&self) -> &[u8] {
        &self.evidence
    }
}
fn dimension(ticks: i64) -> Result<Decimal, ExportError> {
    if ticks <= 0 {
        return Err(ExportError::Unsupported("nonpositive PDF page dimension"));
    }
    let decimal = Decimal::from_ratio(ticks as i128, TICKS_PER_BP as u128, 64)
        .ok_or(ExportError::Stream(PdfStreamError::NonTerminatingDecimal))?;
    // from_ratio is a constructor; enforce the backend's numeric-token length contract too.
    Ok(Decimal::new(decimal.as_str())?)
}
/// Produces a real PDF and bounded identity sidecar. All streams are outline-only;
/// their existing exact decimal refusal rules are preserved before container work.
pub fn export(
    streams: &[PdfCommandStream],
    limits: ExportLimits,
) -> Result<ExportedPdf, ExportError> {
    if !(1..=256).contains(&limits.max_pages)
        || !(1..=MAX_MESSAGE_BYTES).contains(&limits.max_content_bytes)
        || !(1..=2 * MAX_MESSAGE_BYTES).contains(&limits.max_pdf_bytes)
        || !(1..=MAX_MESSAGE_BYTES).contains(&limits.max_evidence_bytes)
        || streams.is_empty()
        || streams.len() > limits.max_pages
    {
        return Err(ExportError::Budget);
    }
    let mut doc = ExactDocument::default();
    let mut pages = Vec::new();
    let mut content_total = 0usize;
    let mut evidence_total = 0usize;
    for stream in streams {
        let content = stream.content_bytes()?;
        content_total = content_total
            .checked_add(content.len())
            .ok_or(ExportError::Budget)?;
        if content_total > limits.max_content_bytes {
            return Err(ExportError::Budget);
        }
        let evidence = stream.evidence_bytes()?;
        evidence_total = evidence_total
            .checked_add(evidence.len())
            .ok_or(ExportError::Budget)?;
        if evidence_total > limits.max_evidence_bytes {
            return Err(ExportError::Budget);
        }
        let operator_evidence: Value = serde_json::from_slice(&evidence)
            .map_err(|e| ExportError::Verification(e.to_string()))?;
        let width = dimension(stream.page().width.0)?;
        let height = dimension(stream.page().height.0)?;
        pages.push(serde_json::json!({"width_bp":width.as_str(),"height_bp":height.as_str(),"content_sha256":digest(&content),"content_bytes":content.len(),"operators":stream.operators().len(),"operator_evidence":operator_evidence}));
        doc.pages.push(ExactPage {
            width,
            height,
            content: Content::Verbatim(content),
            fonts: None,
        });
    }
    let output = exact::render_exact(&doc)?;
    if output.bytes.len() > limits.max_pdf_bytes {
        return Err(ExportError::Budget);
    }
    if !output.warnings.is_empty() {
        return Err(ExportError::Unsupported("PDF backend emitted warnings"));
    }
    // Verify actual object offsets and decoded streams, rather than only planned ops.
    flashtex_pdf::verify::check_structure(&output.bytes).map_err(ExportError::Verification)?;
    let parsed =
        flashtex_pdf::reader::PdfFile::parse(&output.bytes).map_err(ExportError::Verification)?;
    let actual = parsed.pages().map_err(ExportError::Verification)?;
    if actual.len() != doc.pages.len() {
        return Err(ExportError::Verification("PDF page count".into()));
    }
    for (page, expected) in actual.iter().zip(&doc.pages) {
        let Content::Verbatim(content) = &expected.content else {
            unreachable!()
        };
        if parsed
            .page_content(page)
            .map_err(ExportError::Verification)?
            != *content
        {
            return Err(ExportError::Verification(
                "PDF content bytes changed".into(),
            ));
        }
        let media = parsed
            .page_attr(page, "MediaBox")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExportError::Verification("PDF MediaBox missing".into()))?;
        let numbers: Vec<_> = media.iter().map(|n| n.as_number()).collect();
        if numbers
            != vec![
                Some("0"),
                Some("0"),
                Some(expected.width.as_str()),
                Some(expected.height.as_str()),
            ]
        {
            return Err(ExportError::Verification(
                "PDF exact page dimensions changed".into(),
            ));
        }
    }
    let evidence = serde_json::json!({"format":"flashtex-exact-pdf-export-v1","consumer_source_sha256":digest(include_bytes!("pdf_export.rs")),"backend_source_sha256":digest(include_bytes!("../../pdf/src/exact.rs")),"pdf_sha256":digest(&output.bytes),"pdf_bytes":output.bytes.len(),"pages":pages,"outline_only":true,"font_programs_embedded":false,"unicode_text_embedded":false,"paper":"white","preview_theme_applied":false,"resources_verified_by_exporter":false,"content_readback_verified":true,"visual_oracle_verified":false});
    let mut out = BoundedOutput {
        bytes: vec![],
        limit: limits.max_evidence_bytes,
    };
    serde_json::to_writer(&mut out, &evidence).map_err(|_| ExportError::Budget)?;
    Ok(ExportedPdf {
        pdf: output.bytes,
        evidence: out.bytes,
    })
}

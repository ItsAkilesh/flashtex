//! Actual PDF comparison via the owner's reader/classifier. Candidate provenance
//! is attached only after exact content regeneration; reference mapping is unknown.
use crate::{digest, mixed::BoundedOutput, pdf_stream::*, Paint, Tick, MAX_MESSAGE_BYTES};
use flashtex_pdf::{
    compare as backend,
    exact::{self, Op},
    reader::{Obj, PdfFile},
};
use serde_json::{json, Value};
#[derive(Debug)]
pub enum CompareError {
    Budget,
    Reader(String),
    Unsupported(String),
    Evidence(String),
}
#[derive(Clone, Copy)]
pub struct CompareLimits {
    pub max_pdf_bytes: usize,
    pub max_pages: usize,
    pub max_objects: usize,
    pub max_decoded_bytes: usize,
    pub max_operators: usize,
    pub max_differences: usize,
    pub max_report_bytes: usize,
}
impl Default for CompareLimits {
    fn default() -> Self {
        Self {
            max_pdf_bytes: MAX_MESSAGE_BYTES,
            max_pages: 64,
            max_objects: 4096,
            max_decoded_bytes: MAX_MESSAGE_BYTES,
            max_operators: 100000,
            max_differences: 256,
            max_report_bytes: 4 * 1024 * 1024,
        }
    }
}
pub struct PdfComparison {
    value: Value,
    bytes: Vec<u8>,
}
impl PdfComparison {
    pub fn value(&self) -> &Value {
        &self.value
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
// Budget traversal of already parsed page objects; PDF syntax remains owner-only.
fn page_budget(pdf: &PdfFile, l: CompareLimits) -> Result<(), CompareError> {
    let catalog = pdf.catalog().map_err(CompareError::Reader)?;
    let root = pdf
        .get(catalog, "Pages")
        .and_then(Obj::as_dict)
        .ok_or_else(|| CompareError::Reader("page tree root".into()))?;
    let mut stack = vec![(root, 0usize)];
    let mut seen = std::collections::BTreeSet::new();
    let mut pages = 0usize;
    while let Some((node, depth)) = stack.pop() {
        if depth > 32 || seen.len() >= l.max_objects {
            return Err(CompareError::Budget);
        }
        if !seen.insert(std::ptr::from_ref(node) as usize) {
            return Err(CompareError::Unsupported("repeated page-tree node".into()));
        }
        if node.get("Type").and_then(Obj::as_name) == Some("Page") {
            pages += 1;
            if pages > l.max_pages {
                return Err(CompareError::Budget);
            }
        } else {
            let kids = pdf
                .get(node, "Kids")
                .and_then(Obj::as_array)
                .ok_or_else(|| CompareError::Reader("page-tree children".into()))?;
            if kids.len() + stack.len() + seen.len() > l.max_objects {
                return Err(CompareError::Budget);
            }
            for kid in kids {
                stack.push((
                    pdf.resolve(kid)
                        .as_dict()
                        .ok_or_else(|| CompareError::Reader("page-tree child".into()))?,
                    depth + 1,
                ));
            }
        }
    }
    Ok(())
}
fn read(bytes: &[u8], l: CompareLimits) -> Result<PdfFile, CompareError> {
    if bytes.len() > l.max_pdf_bytes {
        return Err(CompareError::Budget);
    }
    let pdf = PdfFile::parse(bytes).map_err(CompareError::Reader)?;
    if pdf.objects.len() > l.max_objects {
        return Err(CompareError::Budget);
    }
    page_budget(&pdf, l)?;
    let mut decoded_lengths = std::collections::BTreeMap::new();
    let mut decoded = 0usize;
    for object in pdf.objects.values() {
        if matches!(object, Obj::Stream { .. }) {
            let length = pdf
                .decode_stream(object)
                .map_err(CompareError::Unsupported)?
                .len();
            decoded_lengths.insert(std::ptr::from_ref(object) as usize, length);
            decoded = decoded.checked_add(length).ok_or(CompareError::Budget)?;
            if decoded > l.max_decoded_bytes {
                return Err(CompareError::Budget);
            }
        }
    }
    let pages = pdf.pages().map_err(CompareError::Reader)?;
    if pages.is_empty() || pages.len() > l.max_pages {
        return Err(CompareError::Budget);
    }
    let mut expanded_content = 0usize;
    for page in pages {
        let parts = match pdf.get(page, "Contents") {
            Some(Obj::Array(parts)) => parts.iter().collect::<Vec<_>>(),
            Some(stream) => vec![stream],
            None => vec![],
        };
        if parts.len() > l.max_objects {
            return Err(CompareError::Budget);
        }
        for part in parts {
            let object = pdf.resolve(part);
            let size = decoded_lengths
                .get(&(std::ptr::from_ref(object) as usize))
                .copied()
                .ok_or_else(|| {
                    CompareError::Unsupported("content is not a known decoded stream".into())
                })?;
            expanded_content = expanded_content
                .checked_add(size + 1)
                .ok_or(CompareError::Budget)?;
            if expanded_content > l.max_decoded_bytes {
                return Err(CompareError::Budget);
            }
        }
        if pdf.page_fonts(page).len() > 128 {
            return Err(CompareError::Budget);
        }
    }
    Ok(pdf)
}
fn text(op: &Op) -> String {
    let mut v = Vec::new();
    op.write(&mut v);
    String::from_utf8_lossy(&v).trim_end().to_string()
}
fn kind(op: &Op) -> &'static str {
    match op {
        Op::FillGray(_) | Op::StrokeGray(_) | Op::FillRgb(_) | Op::StrokeRgb(_) => "paint",
        Op::Move(..)
        | Op::Line(..)
        | Op::Cubic(_)
        | Op::Concat(_)
        | Op::TextMove(..)
        | Op::TextMatrix(_) => "geometry",
        Op::Rect(_) => "rule_or_clip_geometry",
        Op::ClipNonZero | Op::ClipEvenOdd => "clip_policy",
        Op::Font(..) => "font_selection",
        Op::ShowText(_) | Op::ShowTextArray(_) => "encoded_text_or_advance",
        _ => "operator_or_graphics_state",
    }
}
struct Provenance {
    pages: Vec<Vec<(usize, usize, Value)>>,
}
fn provenance(
    pdf: &PdfFile,
    bytes: &[u8],
    evidence: &[u8],
    l: CompareLimits,
) -> Result<Provenance, CompareError> {
    if evidence.len() > MAX_MESSAGE_BYTES {
        return Err(CompareError::Budget);
    }
    let value = crate::mixed_replay::parse_unique(evidence)
        .map_err(|e| CompareError::Evidence(e.to_string()))?;
    if value["format"] != "flashtex-exact-pdf-export-v1" || value["pdf_sha256"] != digest(bytes) {
        return Err(CompareError::Evidence(
            "candidate PDF hash/format mismatch".into(),
        ));
    }
    let pages = pdf.pages().map_err(CompareError::Reader)?;
    let supplied = value["pages"]
        .as_array()
        .ok_or_else(|| CompareError::Evidence("pages missing".into()))?;
    if pages.len() != supplied.len() {
        return Err(CompareError::Evidence("page count mismatch".into()));
    }
    let mut mapped = Vec::new();
    for (page, entry) in pages.iter().zip(supplied) {
        let e = &entry["operator_evidence"];
        let source = &e["original_fixture"];
        let source_bytes =
            serde_json::to_vec(source).map_err(|e| CompareError::Evidence(e.to_string()))?;
        let limits = StreamLimits {
            max_operators: l.max_operators,
            ..StreamLimits::default()
        };
        let stream = match e["source_kind"].as_str() {
            Some("mixed") => PdfCommandStream::from_mixed_replay(&source_bytes, limits),
            Some("shaped") => {
                let ticks = e["page_ticks"]
                    .as_array()
                    .filter(|a| a.len() == 2)
                    .ok_or_else(|| CompareError::Evidence("page tick dimensions".into()))?;
                let width = ticks[0]
                    .as_i64()
                    .ok_or_else(|| CompareError::Evidence("page width".into()))?;
                let height = ticks[1]
                    .as_i64()
                    .ok_or_else(|| CompareError::Evidence("page height".into()))?;
                // This export profile currently supports only black shaped replay paint.
                PdfCommandStream::from_shaped_replay(
                    &source_bytes,
                    PdfPage {
                        width: Tick(width),
                        height: Tick(height),
                    },
                    Paint {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                    limits,
                )
            }
            _ => return Err(CompareError::Evidence("unsupported source kind".into())),
        }
        .map_err(|e| CompareError::Evidence(format!("source regeneration: {e:?}")))?;
        let content = pdf.page_content(page).map_err(CompareError::Reader)?;
        if stream
            .content_bytes()
            .map_err(|e| CompareError::Evidence(format!("{e:?}")))?
            != content
            || entry["content_sha256"] != digest(&content)
        {
            return Err(CompareError::Evidence(
                "candidate operator correspondence mismatch".into(),
            ));
        }
        let primitives = source["primitives"]
            .as_array()
            .ok_or_else(|| CompareError::Evidence("source primitives".into()))?;
        let mut ranges = Vec::new();
        for span in stream.spans() {
            let id = json!({"item_index":span.primitive.item_index,"glyph_index":span.primitive.glyph_index});
            let p = primitives
                .iter()
                .find(|p| p["identity"] == id)
                .ok_or_else(|| CompareError::Evidence("missing primitive identity".into()))?;
            let source_identity = if e["source_kind"] == "shaped" {
                json!({"document":source["source"],"range":p["source_range"]})
            } else {
                p["sources"].clone()
            };
            let font = if e["source_kind"] == "shaped" {
                json!({"font":source["identity"],"cff_resource":p["cff"]["resource"]})
            } else {
                json!({"font_sha256":p["font_sha256"],"cff_resource":p["geometry"]["full_font_identity"]})
            };
            let mapped = json!({"primitive":id,"original_gid":p["original_gid"],"font":font,"source":source_identity,"synthetic_reason":p["synthetic_reason"],"declared_source_sha256":e["source_sha256"],"mapping":"candidate_operator_span_verified","reference_correspondence":"unknown","font_and_source_claims_independently_verified":false});
            if serde_json::to_vec(&mapped)
                .map_err(|e| CompareError::Evidence(e.to_string()))?
                .len()
                > 8192
            {
                return Err(CompareError::Budget);
            }
            ranges.push((span.start, span.end, mapped));
        }
        mapped.push(ranges);
    }
    Ok(Provenance { pages: mapped })
}
/// No visual equality inference; raw byte equality remains a separate fact.
pub fn compare(
    candidate: &[u8],
    reference: &[u8],
    candidate_evidence: Option<&[u8]>,
    l: CompareLimits,
) -> Result<PdfComparison, CompareError> {
    if !(1..=MAX_MESSAGE_BYTES).contains(&l.max_pdf_bytes)
        || !(1..=256).contains(&l.max_pages)
        || !(1..=8192).contains(&l.max_objects)
        || !(1..=MAX_MESSAGE_BYTES).contains(&l.max_decoded_bytes)
        || !(1..=100000).contains(&l.max_operators)
        || !(1..=4096).contains(&l.max_differences)
        || !(1..=MAX_MESSAGE_BYTES).contains(&l.max_report_bytes)
    {
        return Err(CompareError::Budget);
    }
    let a = read(candidate, l)?;
    let b = read(reference, l)?;
    let pa = a.pages().map_err(CompareError::Reader)?;
    let pb = b.pages().map_err(CompareError::Reader)?;
    let mapped = candidate_evidence
        .map(|v| provenance(&a, candidate, v, l))
        .transpose()?;
    let mut differences = Vec::new();
    let mut unsupported = Vec::new();
    let mut all_equal = pa.len() == pb.len();
    let mut known = true;
    let mut truncated = false;
    let mut total_ops = 0usize;
    let mut push = |v: Value| {
        if differences.len() < l.max_differences {
            differences.push(v)
        } else {
            truncated = true
        }
    };
    if pa.len() != pb.len() {
        push(
            json!({"kind":"page_membership","candidate_pages":pa.len(),"reference_pages":pb.len(),"correspondence":"unknown"}),
        );
    }
    for (i, (ap, bp)) in pa.iter().zip(&pb).enumerate() {
        let ca = a.page_content(ap).map_err(CompareError::Reader)?;
        let cb = b.page_content(bp).map_err(CompareError::Reader)?;
        let (oa, ob) = match (exact::parse(&ca), exact::parse(&cb)) {
            (Ok(x), Ok(y)) => (x, y),
            (x, y) => {
                known = false;
                unsupported.push(json!({"page":i+1,"candidate":x.err().map(|e|e.to_string()),"reference":y.err().map(|e|e.to_string())}));
                continue;
            }
        };
        total_ops = total_ops
            .checked_add(oa.len() + ob.len())
            .ok_or(CompareError::Budget)?;
        if total_ops > l.max_operators {
            return Err(CompareError::Budget);
        }
        all_equal &= oa == ob;
        if oa.len() != ob.len() {
            push(
                json!({"kind":"operator_membership","page":i+1,"candidate_count":oa.len(),"reference_count":ob.len(),"correspondence":"unknown"}),
            );
            continue;
        }
        let aligned = oa
            .iter()
            .zip(&ob)
            .all(|(x, y)| std::mem::discriminant(x) == std::mem::discriminant(y));
        if !aligned {
            push(json!({"kind":"operator_sequence","page":i+1,"correspondence":"unknown"}));
            continue;
        }
        for (index, (x, y)) in oa.iter().zip(&ob).enumerate() {
            if x != y {
                let x = text(x);
                let y = text(y);
                if x.len() > 2048 || y.len() > 2048 {
                    return Err(CompareError::Budget);
                }
                let provenance = mapped
                    .as_ref()
                    .and_then(|m| {
                        m.pages[i]
                            .iter()
                            .find(|(start, end, _)| *start <= index && index < *end)
                    })
                    .map(|(_, _, v)| v);
                push(
                    json!({"kind":kind(&oa[index]),"page":i+1,"operator_index":index,"candidate":x,"reference":y,"correspondence":"same_opcode_at_ordinal_only","candidate_provenance":provenance}),
                );
            }
        }
    }
    // Owner classifier supplies font/object/compression categories; no duplicate parser.
    let coarse = backend::classify(&a, &b, "candidate", "reference");
    let mut lines = Vec::new();
    for line in &coarse.lines {
        if lines.len() >= l.max_differences {
            truncated = true;
            break;
        }
        if line.len() > 2048 {
            truncated = true;
            lines.push(line.chars().take(2048).collect::<String>())
        } else {
            lines.push(line.clone())
        }
    }
    let mut categories: Vec<_> = coarse.categories.iter().map(|c| format!("{c:?}")).collect();
    if !unsupported.is_empty() && !categories.iter().any(|v| v == "ContentUnsupported") {
        categories.push("ContentUnsupported".into())
    }
    let value = json!({"format":"flashtex-actual-pdf-comparison-v1","candidate_sha256":digest(candidate),"reference_sha256":digest(reference),"candidate_evidence_sha256":candidate_evidence.map(digest),"consumer_source_sha256":digest(include_bytes!("pdf_compare.rs")),"classifier_source_sha256":digest(include_bytes!("../../pdf/src/compare.rs")),"raw_bytes_equal":candidate==reference,"unclassified_raw_difference":candidate!=reference&&categories.is_empty(),"parsed_operators_equal":if known&&!truncated{Some(all_equal)}else{None},"visual_equal":null,"truncated":truncated,"categories":categories,"classifier_lines":lines,"font_program_comparison":coarse.font_program_identical,"differences":differences,"unsupported_pages":unsupported,"reference_source_correspondence":"unknown","reference_reemitted":false});
    let mut out = BoundedOutput {
        bytes: vec![],
        limit: l.max_report_bytes,
    };
    serde_json::to_writer(&mut out, &value).map_err(|_| CompareError::Budget)?;
    Ok(PdfComparison {
        value,
        bytes: out.bytes,
    })
}

//! Reads a runtime-v1 `compile_result` envelope into [`CompileResult`].
//!
//! Only the fields the writer needs are read. `source` spans, diagnostics, and
//! `pdf_path` are accepted and ignored. Unknown protocol versions and envelope
//! types are rejected, as the contract requires, never silently accepted.

use crate::json::{self, Value};
use crate::{CompileResult, Item, Page, PdfError};

pub const PROTOCOL_VERSION: f64 = 1.0;
pub const ENVELOPE_TYPE: &str = "compile_result";

#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub result: CompileResult,
    /// Skipped, non-text items. Each entry names the page and item kind.
    pub warnings: Vec<String>,
}

pub fn parse_compile_result(envelope_json: &str) -> Result<Parsed, PdfError> {
    let root = json::parse(envelope_json).map_err(PdfError::Json)?;
    if !matches!(root, Value::Object(_)) {
        return Err(PdfError::Protocol("envelope must be a JSON object".into()));
    }

    match root.get("protocol_version").and_then(Value::as_f64) {
        Some(v) if v == PROTOCOL_VERSION => {}
        Some(v) => {
            return Err(PdfError::Protocol(format!(
                "protocol_version {v} is not supported (expected {PROTOCOL_VERSION})"
            )));
        }
        None => {
            return Err(PdfError::Protocol(
                "missing numeric protocol_version".into(),
            ));
        }
    }

    match root.get("type").and_then(Value::as_str) {
        Some(ENVELOPE_TYPE) => {}
        Some(other) => {
            return Err(PdfError::Protocol(format!(
                "type {other:?} is not supported (expected {ENVELOPE_TYPE:?})"
            )));
        }
        None => return Err(PdfError::Protocol("missing string type".into())),
    }

    let payload = root
        .get("payload")
        .filter(|p| matches!(p, Value::Object(_)))
        .ok_or_else(|| PdfError::Protocol("missing payload object".into()))?;
    let pages_json = payload
        .get("pages")
        .and_then(Value::as_array)
        .ok_or_else(|| PdfError::Protocol("payload.pages must be an array".into()))?;

    let mut warnings = Vec::new();
    let mut pages = Vec::with_capacity(pages_json.len());
    for (index, page) in pages_json.iter().enumerate() {
        pages.push(parse_page(page, index, &mut warnings)?);
    }

    Ok(Parsed {
        result: CompileResult { pages },
        warnings,
    })
}

fn number(v: &Value, what: &str) -> Result<f64, PdfError> {
    v.get(what)
        .and_then(Value::as_f64)
        .ok_or_else(|| PdfError::Protocol(format!("{what} must be a number")))
}

fn parse_page(page: &Value, index: usize, warnings: &mut Vec<String>) -> Result<Page, PdfError> {
    let number = number(page, "number").map_err(|e| context(e, index))?;
    if number < 1.0 || number.fract() != 0.0 || number > u32::MAX as f64 {
        return Err(PdfError::Protocol(format!(
            "pages[{index}].number must be a positive integer, got {number}"
        )));
    }
    let width_pt = number_field(page, "width_pt", index)?;
    let height_pt = number_field(page, "height_pt", index)?;
    let items_json = page
        .get("items")
        .and_then(Value::as_array)
        .ok_or_else(|| PdfError::Protocol(format!("pages[{index}].items must be an array")))?;

    let mut items = Vec::with_capacity(items_json.len());
    for (item_index, item) in items_json.iter().enumerate() {
        match item.get("kind").and_then(Value::as_str) {
            Some("text") => items.push(Item {
                text: item
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        PdfError::Protocol(format!(
                            "pages[{index}].items[{item_index}].text must be a string"
                        ))
                    })?
                    .to_string(),
                x_pt: item_number(item, "x_pt", index, item_index)?,
                baseline_y_pt: item_number(item, "baseline_y_pt", index, item_index)?,
                font_size_pt: item_number(item, "font_size_pt", index, item_index)?,
            }),
            Some(other) => warnings.push(format!(
                "page {number}: item {item_index} of kind {other:?} is not supported by this writer and was skipped"
            )),
            None => {
                return Err(PdfError::Protocol(format!(
                    "pages[{index}].items[{item_index}].kind must be a string"
                )));
            }
        }
    }

    Ok(Page {
        number: number as u32,
        width_pt,
        height_pt,
        items,
    })
}

fn context(e: PdfError, index: usize) -> PdfError {
    match e {
        PdfError::Protocol(m) => PdfError::Protocol(format!("pages[{index}].{m}")),
        other => other,
    }
}

fn number_field(page: &Value, what: &str, index: usize) -> Result<f64, PdfError> {
    number(page, what).map_err(|e| context(e, index))
}

fn item_number(item: &Value, what: &str, page: usize, index: usize) -> Result<f64, PdfError> {
    number(item, what).map_err(|e| match e {
        PdfError::Protocol(m) => PdfError::Protocol(format!("pages[{page}].items[{index}].{m}")),
        other => other,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../protocol/fixtures/compile-result.json");

    #[test]
    fn reads_fixture() {
        let parsed = parse_compile_result(FIXTURE).unwrap();
        assert!(parsed.warnings.is_empty());
        let pages = &parsed.result.pages;
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
        assert_eq!((pages[0].width_pt, pages[0].height_pt), (612.0, 792.0));
        assert_eq!(
            pages[0].items,
            vec![Item {
                text: "Hello FlashTeX.".into(),
                x_pt: 72.0,
                baseline_y_pt: 84.0,
                font_size_pt: 12.0,
            }]
        );
    }

    #[test]
    fn rejects_wrong_version_and_type() {
        let v2 = FIXTURE.replace("\"protocol_version\":1", "\"protocol_version\":2");
        assert!(
            matches!(parse_compile_result(&v2), Err(PdfError::Protocol(m)) if m.contains("protocol_version 2"))
        );
        let compile = FIXTURE.replace("\"type\":\"compile_result\"", "\"type\":\"compile\"");
        assert!(
            matches!(parse_compile_result(&compile), Err(PdfError::Protocol(m)) if m.contains("\"compile\""))
        );
        assert!(matches!(
            parse_compile_result("{nope"),
            Err(PdfError::Json(_))
        ));
    }

    #[test]
    fn skips_unknown_item_kinds_with_warning() {
        let json = r#"{"protocol_version":1,"id":"x","type":"compile_result","payload":{"pages":[
            {"number":1,"width_pt":100,"height_pt":200,"items":[
                {"kind":"image","x_pt":1,"y_pt":2},
                {"kind":"text","text":"a","x_pt":1,"baseline_y_pt":2,"font_size_pt":3}
            ]}]}}"#;
        let parsed = parse_compile_result(json).unwrap();
        assert_eq!(parsed.result.pages[0].items.len(), 1);
        assert_eq!(parsed.warnings.len(), 1);
        assert!(parsed.warnings[0].contains("\"image\""));
    }
}

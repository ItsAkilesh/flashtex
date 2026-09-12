//! Reads a runtime-v1 `compile_result` envelope into [`CompileResult`].
//!
//! Only the fields the writer needs are read. Diagnostics and `pdf_path` are
//! accepted and ignored. Unknown protocol versions and envelope types are
//! rejected, as the contract requires, never silently accepted.
//!
//! Two routes, per `docs/contracts/runtime-v1-layout-capabilities.md`:
//!
//! - **Legacy** (no `payload.layout_capabilities`): only `text` items exist.
//!   Any other kind is skipped with a warning naming it; a `rule` item is an
//!   error because it can only come from an unrequested capability.
//! - **Negotiated** (`layout_capabilities` present): `rule` items are accepted
//!   only when the set contains `rules-v1`, `font` hints only with
//!   `font-hints-v1`, and any unknown kind is an error naming the kind and
//!   its source range. Nothing is skipped silently.

use crate::json::{self, Value};
use crate::{
    CAP_FONT_HINTS_V1, CAP_RULES_V1, CompileResult, FontHint, Item, Page, PdfError, RuleItem,
    Style, TextItem, Weight,
};

pub const PROTOCOL_VERSION: f64 = 1.0;
pub const ENVELOPE_TYPE: &str = "compile_result";
/// Largest coordinate or dimension magnitude the capabilities contract allows.
pub const MAX_MAGNITUDE: f64 = 1_000_000.0;
pub const MAX_CAPABILITIES: usize = 16;
pub const MAX_CAPABILITY_BYTES: usize = 64;
pub const MAX_FAMILY_BYTES: usize = 128;

#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    pub result: CompileResult,
    /// Legacy-route skips. Each entry names the page and item kind.
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

    let capabilities = match payload.get("layout_capabilities") {
        None | Some(Value::Null) => None,
        Some(Value::Array(list)) => Some(parse_capabilities(list)?),
        Some(_) => {
            return Err(PdfError::Protocol(
                "payload.layout_capabilities must be an array of strings".into(),
            ));
        }
    };
    let route = Route {
        legacy: capabilities.is_none(),
        rules: capabilities
            .as_ref()
            .is_some_and(|c| c.iter().any(|x| x == CAP_RULES_V1)),
        font_hints: capabilities
            .as_ref()
            .is_some_and(|c| c.iter().any(|x| x == CAP_FONT_HINTS_V1)),
    };

    let mut warnings = Vec::new();
    let mut pages = Vec::with_capacity(pages_json.len());
    for (index, page) in pages_json.iter().enumerate() {
        pages.push(parse_page(page, index, &route, &mut warnings)?);
    }

    Ok(Parsed {
        result: CompileResult {
            pages,
            capabilities,
        },
        warnings,
    })
}

#[derive(Clone, Copy)]
struct Route {
    legacy: bool,
    rules: bool,
    font_hints: bool,
}

fn parse_capabilities(list: &[Value]) -> Result<Vec<String>, PdfError> {
    if list.len() > MAX_CAPABILITIES {
        return Err(PdfError::Protocol(format!(
            "layout_capabilities lists {} entries; at most {MAX_CAPABILITIES} are allowed",
            list.len()
        )));
    }
    let mut out: Vec<String> = Vec::with_capacity(list.len());
    for v in list {
        let s = v.as_str().ok_or_else(|| {
            PdfError::Protocol("layout_capabilities entries must be strings".into())
        })?;
        if s.is_empty() || s.len() > MAX_CAPABILITY_BYTES {
            return Err(PdfError::Protocol(format!(
                "layout_capabilities entry {s:?} must be 1..={MAX_CAPABILITY_BYTES} bytes"
            )));
        }
        if out.iter().any(|x| x == s) {
            return Err(PdfError::Protocol(format!(
                "layout_capabilities repeats {s:?}"
            )));
        }
        out.push(s.to_string());
    }
    Ok(out)
}

fn number(v: &Value, what: &str) -> Result<f64, PdfError> {
    v.get(what)
        .and_then(Value::as_f64)
        .ok_or_else(|| PdfError::Protocol(format!("{what} must be a number")))
}

/// `path:start-end` from an item's `source`, or `<no source>`.
fn source_label(item: &Value) -> String {
    let Some(src) = item.get("source") else {
        return "<no source>".into();
    };
    let path = src.get("path").and_then(Value::as_str).unwrap_or("?");
    let start = src.get("start_byte").and_then(Value::as_f64);
    let end = src.get("end_byte").and_then(Value::as_f64);
    match (start, end) {
        (Some(s), Some(e)) => format!("{path}:{s}-{e}"),
        _ => format!("{path}:?"),
    }
}

fn parse_page(
    page: &Value,
    index: usize,
    route: &Route,
    warnings: &mut Vec<String>,
) -> Result<Page, PdfError> {
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
        let at = format!("pages[{index}].items[{item_index}]");
        match item.get("kind").and_then(Value::as_str) {
            Some("text") => {
                let font = match item.get("font") {
                    None | Some(Value::Null) => None,
                    Some(hint) if route.font_hints => Some(parse_font_hint(hint, &at)?),
                    Some(_) if route.legacy => {
                        warnings.push(format!(
                            "page {number}: item {item_index} ({}) carries a font hint but no capabilities were negotiated (legacy route); the hint is ignored",
                            source_label(item)
                        ));
                        None
                    }
                    Some(_) => {
                        return Err(PdfError::Protocol(format!(
                            "{at} ({}) carries a font hint but {CAP_FONT_HINTS_V1:?} was not accepted",
                            source_label(item)
                        )));
                    }
                };
                items.push(Item::Text(TextItem {
                    text: item
                        .get("text")
                        .and_then(Value::as_str)
                        .ok_or_else(|| PdfError::Protocol(format!("{at}.text must be a string")))?
                        .to_string(),
                    x_pt: item_number(item, "x_pt", index, item_index)?,
                    baseline_y_pt: item_number(item, "baseline_y_pt", index, item_index)?,
                    font_size_pt: item_number(item, "font_size_pt", index, item_index)?,
                    font,
                }));
            }
            Some("rule") => {
                if !route.rules {
                    return Err(PdfError::Protocol(format!(
                        "{at} ({}) is a rule item but {CAP_RULES_V1:?} was not accepted{}",
                        source_label(item),
                        if route.legacy {
                            " (no layout_capabilities negotiated)"
                        } else {
                            ""
                        }
                    )));
                }
                items.push(Item::Rule(parse_rule(item, &at)?));
            }
            Some(other) if route.legacy => warnings.push(format!(
                "page {number}: item {item_index} of kind {other:?} is not supported by this writer and was skipped (legacy route)"
            )),
            Some(other) => {
                return Err(PdfError::Protocol(format!(
                    "{at} ({}) has unknown kind {other:?}; negotiated capabilities do not define it",
                    source_label(item)
                )));
            }
            None => {
                return Err(PdfError::Protocol(format!("{at}.kind must be a string")));
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

fn parse_rule(item: &Value, at: &str) -> Result<RuleItem, PdfError> {
    let field = |name: &str| -> Result<f64, PdfError> {
        let v = number(item, name).map_err(|e| match e {
            PdfError::Protocol(m) => PdfError::Protocol(format!("{at}.{m}")),
            other => other,
        })?;
        if !v.is_finite() || v.abs() > MAX_MAGNITUDE {
            return Err(PdfError::Protocol(format!(
                "{at}.{name} must be finite with magnitude <= {MAX_MAGNITUDE}, got {v}"
            )));
        }
        Ok(v)
    };
    let rule = RuleItem {
        x_pt: field("x_pt")?,
        y_pt: field("y_pt")?,
        width_pt: field("width_pt")?,
        height_pt: field("height_pt")?,
    };
    for (name, v) in [("width_pt", rule.width_pt), ("height_pt", rule.height_pt)] {
        if v <= 0.0 {
            return Err(PdfError::Protocol(format!(
                "{at}.{name} must be positive, got {v}"
            )));
        }
    }
    Ok(rule)
}

fn parse_font_hint(hint: &Value, at: &str) -> Result<FontHint, PdfError> {
    if !matches!(hint, Value::Object(_)) {
        return Err(PdfError::Protocol(format!("{at}.font must be an object")));
    }
    let family = hint
        .get("family")
        .and_then(Value::as_str)
        .ok_or_else(|| PdfError::Protocol(format!("{at}.font.family must be a string")))?;
    if family.is_empty() || family.len() > MAX_FAMILY_BYTES || family.chars().any(char::is_control)
    {
        return Err(PdfError::Protocol(format!(
            "{at}.font.family must be 1..={MAX_FAMILY_BYTES} bytes without control characters"
        )));
    }
    let weight = match hint.get("weight").and_then(Value::as_str) {
        Some("normal") => Weight::Normal,
        Some("bold") => Weight::Bold,
        other => {
            return Err(PdfError::Protocol(format!(
                "{at}.font.weight must be \"normal\" or \"bold\", got {other:?}"
            )));
        }
    };
    let style = match hint.get("style").and_then(Value::as_str) {
        Some("normal") => Style::Normal,
        Some("italic") => Style::Italic,
        other => {
            return Err(PdfError::Protocol(format!(
                "{at}.font.style must be \"normal\" or \"italic\", got {other:?}"
            )));
        }
    };
    Ok(FontHint {
        family: family.to_string(),
        weight,
        style,
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
        assert!(parsed.result.legacy());
        let pages = &parsed.result.pages;
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].number, 1);
        assert_eq!((pages[0].width_pt, pages[0].height_pt), (612.0, 792.0));
        assert_eq!(
            pages[0].items,
            vec![Item::Text(TextItem {
                text: "Hello FlashTeX.".into(),
                x_pt: 72.0,
                baseline_y_pt: 84.0,
                font_size_pt: 12.0,
                font: None,
            })]
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
    fn legacy_route_skips_unknown_kinds_with_warning_but_rejects_rules() {
        let json = r#"{"protocol_version":1,"id":"x","type":"compile_result","payload":{"pages":[
            {"number":1,"width_pt":100,"height_pt":200,"items":[
                {"kind":"image","x_pt":1,"y_pt":2},
                {"kind":"text","text":"a","x_pt":1,"baseline_y_pt":2,"font_size_pt":3}
            ]}]}}"#;
        let parsed = parse_compile_result(json).unwrap();
        assert_eq!(parsed.result.pages[0].items.len(), 1);
        assert_eq!(parsed.warnings.len(), 1);
        assert!(parsed.warnings[0].contains("\"image\""));
        assert!(parsed.warnings[0].contains("legacy route"));

        let rule = r#"{"protocol_version":1,"id":"x","type":"compile_result","payload":{"pages":[
            {"number":1,"width_pt":100,"height_pt":200,"items":[
                {"kind":"rule","x_pt":1,"y_pt":2,"width_pt":3,"height_pt":4,"source":{"path":"m.tex","start_byte":0,"end_byte":5}}
            ]}]}}"#;
        let err = parse_compile_result(rule).unwrap_err();
        assert!(
            matches!(&err, PdfError::Protocol(m) if m.contains("rules-v1") && m.contains("m.tex:0-5")),
            "{err}"
        );
    }

    #[test]
    fn negotiated_route_accepts_rules_and_hints_and_rejects_unknown_kinds() {
        let json = r#"{"protocol_version":1,"id":"x","type":"compile_result","payload":{"layout_capabilities":["rules-v1","font-hints-v1"],"pages":[
            {"number":1,"width_pt":612,"height_pt":792,"items":[
                {"kind":"rule","x_pt":72,"y_pt":84,"width_pt":24,"height_pt":0.5,"source":{"path":"main.tex","start_byte":0,"end_byte":11}},
                {"kind":"text","text":"a","x_pt":1,"baseline_y_pt":2,"font_size_pt":3,"font":{"family":"Latin Modern Roman","weight":"bold","style":"italic"}}
            ]}]}}"#;
        let parsed = parse_compile_result(json).unwrap();
        assert!(parsed.warnings.is_empty());
        assert!(parsed.result.accepts(CAP_RULES_V1) && parsed.result.accepts(CAP_FONT_HINTS_V1));
        assert_eq!(
            parsed.result.pages[0].items[0],
            Item::Rule(RuleItem {
                x_pt: 72.0,
                y_pt: 84.0,
                width_pt: 24.0,
                height_pt: 0.5
            })
        );
        let Item::Text(t) = &parsed.result.pages[0].items[1] else {
            panic!()
        };
        assert_eq!(
            t.font,
            Some(FontHint {
                family: "Latin Modern Roman".into(),
                weight: Weight::Bold,
                style: Style::Italic
            })
        );

        let unknown = json.replace("\"kind\":\"rule\"", "\"kind\":\"image\"");
        let err = parse_compile_result(&unknown).unwrap_err();
        assert!(
            matches!(&err, PdfError::Protocol(m) if m.contains("\"image\"") && m.contains("main.tex:0-11")),
            "{err}"
        );

        let no_rules = json.replace("\"rules-v1\",", "");
        let err = parse_compile_result(&no_rules).unwrap_err();
        assert!(
            matches!(&err, PdfError::Protocol(m) if m.contains("rules-v1") && m.contains("main.tex:0-11")),
            "{err}"
        );

        let no_hints = json.replace(",\"font-hints-v1\"", "");
        let err = parse_compile_result(&no_hints).unwrap_err();
        assert!(
            matches!(&err, PdfError::Protocol(m) if m.contains("font-hints-v1")),
            "{err}"
        );

        let bad_dim = json.replace("\"width_pt\":24", "\"width_pt\":0");
        assert!(parse_compile_result(&bad_dim).is_err());
        let huge = json.replace("\"width_pt\":24", "\"width_pt\":2000000");
        assert!(parse_compile_result(&huge).is_err());
        let bad_weight = json.replace("\"weight\":\"bold\"", "\"weight\":\"heavy\"");
        assert!(parse_compile_result(&bad_weight).is_err());
        let dup = json.replace(
            "\"rules-v1\",\"font-hints-v1\"",
            "\"rules-v1\",\"rules-v1\"",
        );
        assert!(parse_compile_result(&dup).is_err());
    }
}

//! runtime-v1 `compile_result` fallback derived from the v2 display list,
//! with the negotiated layout capabilities of
//! `docs/contracts/runtime-v1-layout-capabilities.md`.
//!
//! One text item per glyph run (a styled word segment) with the exact
//! source span of its bytes, including the document path; one item per
//! glyph for math (each glyph has its own position). Rules become typed
//! `rule` items only when `rules-v1` was requested and accepted; otherwise
//! the legacy U+2500 approximation (a run of box-drawing characters at a
//! size whose 0.0857 em height equals the rule) is emitted, as the current
//! compiler does. Font hints ride on text items only when `font-hints-v1`
//! was accepted. Coordinates are PDF points (612x792 for US Letter), y
//! downward, exactly as the compiler's v1 output.

use flashtex_compiler::json::{self, Value};

use crate::display::{self, DisplayList, Provenance, Severity, SourceRange};

pub const CAP_RULES: &str = "rules-v1";
pub const CAP_FONT_HINTS: &str = "font-hints-v1";

/// Capabilities the producer accepted for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Capabilities {
    pub rules: bool,
    pub font_hints: bool,
}

impl Capabilities {
    /// Accepts the known subset of a request's `layout_capabilities`, in
    /// request order, and returns the accepted list to echo. Unknown names
    /// are never accepted; nothing is accepted that was not requested.
    pub fn negotiate(requested: &[String]) -> (Capabilities, Vec<String>) {
        let mut caps = Capabilities::default();
        let mut accepted = Vec::new();
        for r in requested {
            match r.as_str() {
                CAP_RULES if !caps.rules => {
                    caps.rules = true;
                    accepted.push(r.clone());
                }
                CAP_FONT_HINTS if !caps.font_hints => {
                    caps.font_hints = true;
                    accepted.push(r.clone());
                }
                _ => {}
            }
        }
        (caps, accepted)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontHint {
    pub family: String,
    pub weight: &'static str,
    pub style: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum V1Item {
    Text {
        text: String,
        x_pt: f64,
        baseline_y_pt: f64,
        font_size_pt: f64,
        source: SourceRange,
        font: Option<FontHint>,
    },
    Rule {
        x_pt: f64,
        y_pt: f64,
        width_pt: f64,
        height_pt: f64,
        source: SourceRange,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct V1Page {
    pub number: u32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub items: Vec<V1Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct V1Payload {
    pub project_id: String,
    pub revision: u64,
    pub status: &'static str,
    pub pages: Vec<V1Page>,
    pub diagnostics: Vec<display::Diagnostic>,
    pub accepted: Option<Vec<String>>,
}

/// Height of the U+2500 glyph box relative to the font size, as the current
/// compiler and the visual harness define the legacy rule convention.
pub const LEGACY_RULE_HEIGHT_EM: f64 = 0.0857;
pub const LEGACY_RULE_ADVANCE_EM: f64 = 0.5;

fn hint_for(font: &display::FontResource) -> FontHint {
    let ps = font.postscript_name.as_str();
    let lower = ps.to_ascii_lowercase();
    let family = if ps.starts_with("LatinModernMath") {
        "Latin Modern Math"
    } else if ps.starts_with("LM") {
        "Latin Modern Roman"
    } else if lower.starts_with("times") {
        "Times"
    } else if lower.starts_with("symbol") {
        "Symbol"
    } else {
        ps
    };
    FontHint {
        family: family.to_string(),
        weight: if lower.contains("bold") { "bold" } else { "normal" },
        style: if lower.contains("italic") || lower.contains("oblique") { "italic" } else { "normal" },
    }
}

fn union(sources: &[SourceRange]) -> Option<SourceRange> {
    let first = sources.first()?;
    let mut out = first.clone();
    for s in sources.iter().filter(|s| s.path == first.path) {
        out.start_byte = out.start_byte.min(s.start_byte);
        out.end_byte = out.end_byte.max(s.end_byte);
    }
    Some(out)
}

/// Builds the v1 payload. `accepted` is `None` when the request carried no
/// `layout_capabilities` field (the field is then omitted in the reply).
pub fn fallback(v2: &DisplayList, caps: Capabilities, accepted: Option<Vec<String>>) -> V1Payload {
    let mut pages = Vec::with_capacity(v2.pages.len());
    for page in &v2.pages {
        let mut items = Vec::new();
        for item in &page.items {
            match item {
                display::Item::GlyphRun(run) => {
                    let font = v2.fonts.iter().find(|f| f.font_id == run.font_id);
                    let hint = if caps.font_hints { font.map(hint_for) } else { None };
                    let size = run.font_size.to_bp();
                    match run.role {
                        display::RunRole::Text => {
                            let sources: Vec<SourceRange> = run
                                .clusters
                                .iter()
                                .filter_map(|c| match &c.provenance {
                                    Provenance::Sources(s) => Some(s.iter().cloned()),
                                    Provenance::Synthetic(_) => None,
                                })
                                .flatten()
                                .collect();
                            let (Some(source), Some(first)) = (union(&sources), run.glyphs.first()) else { continue };
                            items.push(V1Item::Text {
                                text: run.text.clone(),
                                x_pt: first.origin_x.to_bp(),
                                baseline_y_pt: first.baseline_y.to_bp(),
                                font_size_pt: size,
                                source,
                                font: hint.clone(),
                            });
                        }
                        display::RunRole::Math => {
                            for g in &run.glyphs {
                                let Some(c) = run.clusters.get(g.cluster as usize) else { continue };
                                let Provenance::Sources(s) = &c.provenance else { continue };
                                let Some(source) = union(s) else { continue };
                                items.push(V1Item::Text {
                                    text: run.text[c.text_start_byte..c.text_end_byte].to_string(),
                                    x_pt: g.origin_x.to_bp(),
                                    baseline_y_pt: g.baseline_y.to_bp(),
                                    font_size_pt: size,
                                    source,
                                    font: hint.clone(),
                                });
                            }
                        }
                    }
                }
                display::Item::Rule(rule) => {
                    let Provenance::Sources(s) = &rule.provenance else { continue };
                    let Some(source) = union(s) else { continue };
                    let (x, top, w, h) = (rule.x.to_bp(), rule.top.to_bp(), rule.width.to_bp(), rule.height.to_bp());
                    if caps.rules {
                        items.push(V1Item::Rule {
                            x_pt: x,
                            y_pt: top,
                            width_pt: w,
                            height_pt: h,
                            source,
                        });
                    } else {
                        let size = h / LEGACY_RULE_HEIGHT_EM;
                        let n = ((w / (LEGACY_RULE_ADVANCE_EM * size)).round() as usize).max(1);
                        items.push(V1Item::Text {
                            text: "\u{2500}".repeat(n),
                            x_pt: x,
                            baseline_y_pt: top + h,
                            font_size_pt: size,
                            source,
                            font: None,
                        });
                    }
                }
            }
        }
        pages.push(V1Page {
            number: page.number,
            width_pt: page.width.to_bp(),
            height_pt: page.height.to_bp(),
            items,
        });
    }
    let has_content = pages.iter().any(|p| !p.items.is_empty());
    let has_error = v2.diagnostics.iter().any(|d| d.severity == Severity::Error);
    let status = if v2.diagnostics.is_empty() {
        "ok"
    } else if has_content || !has_error {
        "recovered"
    } else {
        "failed"
    };
    V1Payload {
        project_id: v2.project_id.clone(),
        revision: v2.revision,
        status,
        pages,
        diagnostics: v2.diagnostics.clone(),
        accepted,
    }
}

fn source_json(s: &SourceRange) -> Value {
    let mut o = Value::obj();
    o.set("path", json::str_(s.path.clone()));
    o.set("start_byte", json::num(s.start_byte as f64));
    o.set("end_byte", json::num(s.end_byte as f64));
    o
}

/// Rounds to 1/1000 pt so the JSON is stable across platforms' float
/// formatting while keeping sub-pixel positions.
fn pt(v: f64) -> Value {
    json::num((v * 1000.0).round() / 1000.0)
}

pub fn diagnostic_json(d: &display::Diagnostic) -> Value {
    let mut v = Value::obj();
    v.set(
        "severity",
        json::str_(match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }),
    );
    v.set("message", json::str_(d.message.clone()));
    v.set("source", d.sources.first().map(source_json).unwrap_or(Value::Null));
    v.set("recovery", d.recovery.clone().map(json::str_).unwrap_or(Value::Null));
    v.set("code", json::str_(d.code.clone()));
    v
}

impl V1Payload {
    pub fn to_json(&self) -> Value {
        let mut p = Value::obj();
        p.set("project_id", json::str_(self.project_id.clone()));
        p.set("revision", json::num(self.revision as f64));
        p.set("status", json::str_(self.status));
        p.set(
            "pages",
            Value::Arr(
                self.pages
                    .iter()
                    .map(|pg| {
                        let mut o = Value::obj();
                        o.set("number", json::num(f64::from(pg.number)));
                        o.set("width_pt", pt(pg.width_pt));
                        o.set("height_pt", pt(pg.height_pt));
                        o.set(
                            "items",
                            Value::Arr(
                                pg.items
                                    .iter()
                                    .map(|it| {
                                        let mut o = Value::obj();
                                        match it {
                                            V1Item::Text {
                                                text,
                                                x_pt,
                                                baseline_y_pt,
                                                font_size_pt,
                                                source,
                                                font,
                                            } => {
                                                o.set("kind", json::str_("text"));
                                                o.set("text", json::str_(text.clone()));
                                                o.set("x_pt", pt(*x_pt));
                                                o.set("baseline_y_pt", pt(*baseline_y_pt));
                                                o.set("font_size_pt", pt(*font_size_pt));
                                                o.set("source", source_json(source));
                                                if let Some(f) = font {
                                                    let mut fo = Value::obj();
                                                    fo.set("family", json::str_(f.family.clone()));
                                                    fo.set("weight", json::str_(f.weight));
                                                    fo.set("style", json::str_(f.style));
                                                    o.set("font", fo);
                                                }
                                            }
                                            V1Item::Rule {
                                                x_pt,
                                                y_pt,
                                                width_pt,
                                                height_pt,
                                                source,
                                            } => {
                                                o.set("kind", json::str_("rule"));
                                                o.set("x_pt", pt(*x_pt));
                                                o.set("y_pt", pt(*y_pt));
                                                o.set("width_pt", pt(*width_pt));
                                                o.set("height_pt", pt(*height_pt));
                                                o.set("source", source_json(source));
                                            }
                                        }
                                        o
                                    })
                                    .collect(),
                            ),
                        );
                        o
                    })
                    .collect(),
            ),
        );
        p.set("diagnostics", Value::Arr(self.diagnostics.iter().map(diagnostic_json).collect()));
        p.set("pdf_path", Value::Null);
        if let Some(acc) = &self.accepted {
            p.set("layout_capabilities", Value::Arr(acc.iter().cloned().map(json::str_).collect()));
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiation_accepts_only_known_requested_capabilities() {
        let (c, acc) = Capabilities::negotiate(&[]);
        assert_eq!(c, Capabilities::default());
        assert!(acc.is_empty());
        let (c, acc) = Capabilities::negotiate(&["font-hints-v1".into(), "rules-v2".into(), "rules-v1".into()]);
        assert!(c.rules && c.font_hints);
        assert_eq!(acc, vec!["font-hints-v1".to_string(), "rules-v1".to_string()]);
        let (c, acc) = Capabilities::negotiate(&["unknown".into()]);
        assert_eq!(c, Capabilities::default());
        assert!(acc.is_empty());
    }
}

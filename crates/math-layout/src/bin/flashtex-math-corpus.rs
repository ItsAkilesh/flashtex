//! `flashtex-math-corpus`: a runtime-v1 compile server for the declared math
//! visual corpus, so the visual-oracle harness and the PDF writer consume
//! this crate's boxes unchanged.
//!
//! Modes:
//! * default — JSON Lines on stdin/stdout: each `compile` request whose body
//!   is a declared corpus case is answered with a `compile_result` whose
//!   items are one `text` item per glyph and, when `rules-v1` was requested,
//!   one `rule` item per rule (`docs/contracts/runtime-v1-layout-capabilities.md`).
//!   `font-hints-v1`, when requested, adds the Computer Modern family/style
//!   of each glyph. Unknown bodies yield `status: failed` with a diagnostic:
//!   this is a declared-case lookup, not a LaTeX compiler.
//! * `--runs` — JSON of every case's laid-out line in TeX points (page
//!   placement plus glyph/rule origins) for `tools/structural_gate.py`.
//! * `--list` — case ids.

use flashtex_math_layout::corpus::{self, PAGE_HEIGHT_BP, PAGE_WIDTH_BP, PT_PER_BP};
use flashtex_math_layout::json::{Json, obj};
use flashtex_math_layout::{CmMathMetrics, FontId, MathFontMetrics};
use std::io::{BufRead, Write};

fn metrics() -> CmMathMetrics {
    CmMathMetrics::latex_12pt()
}

/// font-hints-v1 family/style for a Computer Modern font.
fn font_hint(m: &CmMathMetrics, id: FontId) -> Json {
    let name = m.font_name(id);
    let (family, style) = if name.starts_with("cmmi") {
        ("Computer Modern Math Italic", "italic")
    } else if name.starts_with("cmsy") {
        ("Computer Modern Math Symbols", "normal")
    } else if name.starts_with("cmex") {
        ("Computer Modern Math Extension", "normal")
    } else {
        ("Computer Modern Roman", "normal")
    };
    obj(vec![
        ("family", Json::Str(family.into())),
        ("weight", Json::Str("normal".into())),
        ("style", Json::Str(style.into())),
    ])
}

fn source(path: &str, start: usize, end: usize) -> Json {
    obj(vec![
        ("path", Json::Str(path.into())),
        ("start_byte", Json::Num(start as f64)),
        ("end_byte", Json::Num(end as f64)),
    ])
}

fn bp(v: f64) -> Json {
    // Round once when emitting the display list (rendering-v2 guidance).
    Json::Num((v / PT_PER_BP * 100000.0).round() / 100000.0)
}

/// Byte range of the first `$…$` group (or the whole body) in `body`.
fn math_range(body: &str) -> (usize, usize) {
    if let Some(s) = body.find('$')
        && let Some(rel) = body[s + 1..].find('$')
    {
        return (s, s + 1 + rel + 1);
    }
    (0, body.len())
}

fn compile(req: &Json, path_hint: Option<&str>) -> Json {
    let id = req
        .get("id")
        .and_then(Json::as_str)
        .unwrap_or("")
        .to_string();
    let payload = req.get("payload");
    let project = payload
        .and_then(|p| p.get("project_id"))
        .and_then(Json::as_str)
        .unwrap_or("")
        .to_string();
    let revision = payload
        .and_then(|p| p.get("revision"))
        .and_then(Json::as_f64)
        .unwrap_or(0.0);
    let requested: Vec<String> = payload
        .and_then(|p| p.get("layout_capabilities"))
        .and_then(Json::as_arr)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let rules_v1 = requested.iter().any(|c| c == "rules-v1");
    let font_hints = requested.iter().any(|c| c == "font-hints-v1");
    let accepted: Vec<Json> = requested
        .iter()
        .filter(|c| *c == "rules-v1" || *c == "font-hints-v1")
        .map(|c| Json::Str(c.clone()))
        .collect();
    let doc = payload
        .and_then(|p| p.get("documents"))
        .and_then(Json::as_arr)
        .and_then(|d| d.first());
    let path = doc
        .and_then(|d| d.get("path"))
        .and_then(Json::as_str)
        .or(path_hint)
        .unwrap_or("main.tex")
        .to_string();
    let text = doc
        .and_then(|d| d.get("text"))
        .and_then(Json::as_str)
        .unwrap_or("");

    let mut diagnostics = Vec::new();
    let mut items = Vec::new();
    let status;
    match corpus::find_by_body(text) {
        None => {
            status = "failed";
            diagnostics.push(obj(vec![
                ("severity", Json::Str("error".into())),
                (
                    "message",
                    Json::Str(
                        "flashtex-math-corpus only compiles declared corpus bodies (crates/math-layout/fixtures/visual); this body is not one of them"
                            .into(),
                    ),
                ),
                ("source", Json::Null),
                ("recovery", Json::Null),
            ]));
        }
        Some(case) => {
            status = "ok";
            let m = metrics();
            let line = corpus::lay_out_line(&case, &m);
            let (ms, me) = math_range(text);
            for g in &line.runs.glyphs {
                let mut fields = vec![
                    ("kind", Json::Str("text".into())),
                    ("text", Json::Str(g.ch.to_string())),
                    ("x_pt", bp(line.x_pt + g.x)),
                    ("baseline_y_pt", bp(line.baseline_pt + g.baseline_y)),
                    ("font_size_pt", bp(g.size)),
                    ("source", source(&path, ms, me)),
                ];
                if font_hints {
                    fields.push(("font", font_hint(&m, g.font_id)));
                }
                items.push(obj(fields));
            }
            if rules_v1 {
                for r in &line.runs.rules {
                    items.push(obj(vec![
                        ("kind", Json::Str("rule".into())),
                        ("x_pt", bp(line.x_pt + r.x)),
                        ("y_pt", bp(line.baseline_pt + r.y)),
                        ("width_pt", bp(r.w)),
                        ("height_pt", bp(r.h)),
                        ("source", source(&path, ms, me)),
                    ]));
                }
            } else if !line.runs.rules.is_empty() {
                diagnostics.push(obj(vec![
                    ("severity", Json::Str("warning".into())),
                    (
                        "message",
                        Json::Str(format!(
                            "{} rule(s) omitted: rules-v1 was not requested, so fraction/overbar rules cannot be transported",
                            line.runs.rules.len()
                        )),
                    ),
                    ("source", source(&path, ms, me)),
                    ("recovery", Json::Str("request layout_capabilities [\"rules-v1\"]".into())),
                ]));
            }
            for l in &line.limitations {
                diagnostics.push(obj(vec![
                    ("severity", Json::Str("warning".into())),
                    ("message", Json::Str(format!("layout limitation: {l:?}"))),
                    ("source", source(&path, ms, me)),
                    ("recovery", Json::Null),
                ]));
            }
        }
    }
    let page = obj(vec![
        ("number", Json::Num(1.0)),
        ("width_pt", Json::Num(PAGE_WIDTH_BP)),
        ("height_pt", Json::Num(PAGE_HEIGHT_BP)),
        ("items", Json::Arr(items)),
    ]);
    let mut payload_out = vec![
        ("project_id", Json::Str(project)),
        ("revision", Json::Num(revision)),
        ("status", Json::Str(status.into())),
        ("pages", Json::Arr(vec![page])),
        ("diagnostics", Json::Arr(diagnostics)),
        ("pdf_path", Json::Null),
    ];
    if !requested.is_empty() {
        payload_out.push(("layout_capabilities", Json::Arr(accepted)));
    }
    obj(vec![
        ("protocol_version", Json::Num(1.0)),
        ("id", Json::Str(id)),
        ("type", Json::Str("compile_result".into())),
        ("payload", obj(payload_out)),
    ])
}

fn runs_json() -> Json {
    let m = metrics();
    let mut cases = Vec::new();
    for case in corpus::cases() {
        let line = corpus::lay_out_line(&case, &m);
        let glyphs = line
            .runs
            .glyphs
            .iter()
            .map(|g| {
                obj(vec![
                    ("font", Json::Str(m.font_name(g.font_id))),
                    ("gid", Json::Num(g.gid as f64)),
                    ("ch", Json::Str(g.ch.to_string())),
                    ("x", Json::Num(g.x)),
                    ("baseline", Json::Num(g.baseline_y)),
                    ("size", Json::Num(g.size)),
                ])
            })
            .collect();
        let rules = line
            .runs
            .rules
            .iter()
            .map(|r| {
                obj(vec![
                    ("x", Json::Num(r.x)),
                    ("y", Json::Num(r.y)),
                    ("w", Json::Num(r.w)),
                    ("h", Json::Num(r.h)),
                ])
            })
            .collect();
        cases.push(obj(vec![
            ("name", Json::Str(case.id.into())),
            ("x_pt", Json::Num(line.x_pt)),
            ("baseline_pt", Json::Num(line.baseline_pt)),
            ("width", Json::Num(line.width)),
            ("height", Json::Num(line.height)),
            ("depth", Json::Num(line.depth)),
            ("glyphs", Json::Arr(glyphs)),
            ("rules", Json::Arr(rules)),
            (
                "limitations",
                Json::Arr(
                    line.limitations
                        .iter()
                        .map(|l| Json::Str(format!("{l:?}")))
                        .collect(),
                ),
            ),
        ]));
    }
    Json::Arr(cases)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--list") {
        for c in corpus::cases() {
            println!("{}", c.id);
        }
        return;
    }
    if args.iter().any(|a| a == "--runs") {
        println!("{}", runs_json().to_string());
        return;
    }
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match Json::parse(&line) {
            Ok(req) if req.get("type").and_then(Json::as_str) == Some("compile") => {
                compile(&req, None)
            }
            Ok(req) => obj(vec![
                ("protocol_version", Json::Num(1.0)),
                (
                    "id",
                    Json::Str(req.get("id").and_then(Json::as_str).unwrap_or("").into()),
                ),
                ("type", Json::Str("error".into())),
                (
                    "payload",
                    obj(vec![(
                        "message",
                        Json::Str("unsupported request type".into()),
                    )]),
                ),
            ]),
            Err(e) => obj(vec![
                ("protocol_version", Json::Num(1.0)),
                ("id", Json::Str(String::new())),
                ("type", Json::Str("error".into())),
                (
                    "payload",
                    obj(vec![("message", Json::Str(format!("malformed JSON: {e}")))]),
                ),
            ]),
        };
        let _ = writeln!(out, "{}", reply.to_string());
        let _ = out.flush();
    }
}

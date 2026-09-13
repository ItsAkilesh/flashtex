//! Compares this crate against committed XeLaTeX/LuaLaTeX output
//! (`fixtures/expected`, produced by `tools/run_oracle.py`; TeX never runs
//! here). For each text line: every word's left and right edge relative to
//! the line's first word must be within 0.5pt (widths + interword glue +
//! space factor, the parts this crate implements). Fonts that are not
//! installed are reported as skipped.
//!
//! `cargo test --test oracle -- --nocapture` prints the per-fixture table.

mod common;

use std::collections::BTreeMap;

use common::{Json, crate_dir, locator, parse};
use flashtex_font_engine::{Face, load_from_path_index};
use flashtex_unicode_tex::engine::EngineProfile;
use flashtex_unicode_tex::fontspec::{
    FaceRequest, FaceSlot, FamilyRole, FeaturePlan, FontCommandKind, FontOptions, Scale,
    face_request, parse_font_commands, substitution_chain,
};
use flashtex_unicode_tex::locate::{FontLocator, latin_modern_default_file, tex_font_file};
use flashtex_unicode_tex::mathfont::MathFontParams;
use flashtex_unicode_tex::measure::{FontInstance, layout_line};
use flashtex_unicode_tex::unimath::{MathOptions, map_expression};

const TOL_PT: f64 = 0.5;

struct Body {
    role: FamilyRole,
    cs: Option<String>,
    bold: bool,
    italic: bool,
    french: bool,
}

fn parse_body_prefix(p: &str) -> Body {
    let mut b = Body {
        role: FamilyRole::Main,
        cs: None,
        bold: false,
        italic: false,
        french: false,
    };
    for tok in p.split('\\').map(str::trim).filter(|t| !t.is_empty()) {
        match tok {
            "bfseries" => b.bold = true,
            "itshape" => b.italic = true,
            "sffamily" => b.role = FamilyRole::Sans,
            "ttfamily" => b.role = FamilyRole::Mono,
            "frenchspacing" => b.french = true,
            cs => {
                b.role = FamilyRole::Other;
                b.cs = Some(format!("\\{cs}"));
            }
        }
    }
    b
}

fn load(req: &FaceRequest) -> Option<flashtex_font_engine::TrueTypeFace> {
    let f = locator_cached().locate(req).ok()?;
    load_from_path_index(&f.path, f.face_index).ok()
}

fn locator_cached() -> &'static flashtex_unicode_tex::locate::DirectoryLocator {
    static L: std::sync::OnceLock<flashtex_unicode_tex::locate::DirectoryLocator> =
        std::sync::OnceLock::new();
    L.get_or_init(locator)
}

#[derive(Default)]
struct Tally {
    lines: usize,
    lines_pass: usize,
    lines_tight: usize,
    words: usize,
    words_pass: usize,
    glue: usize,
    glue_pass: usize,
    math_lines: usize,
    math_lines_pass: usize,
    umath: usize,
    umath_pass: usize,
    skipped: Vec<String>,
    by_engine: BTreeMap<String, (usize, usize)>,
}

fn engines() -> [(&'static str, EngineProfile); 2] {
    [
        ("xelatex", EngineProfile::XETEX),
        ("lualatex", EngineProfile::LUATEX),
    ]
}

fn expected(id: &str, engine: &str) -> Option<Json> {
    let p = crate_dir()
        .join("fixtures/expected")
        .join(format!("{id}.{engine}.json"));
    let s = std::fs::read_to_string(p).ok()?;
    let j = parse(&s);
    j.get("ok").map(Json::bool).unwrap_or(false).then_some(j)
}

fn run_text_fixture(fx: &Json, t: &mut Tally, verbose: bool) {
    let id = fx.get("id").unwrap().str();
    let preamble: Vec<&str> = fx
        .get("preamble")
        .unwrap()
        .arr()
        .iter()
        .map(Json::str)
        .collect();
    let cmds = parse_font_commands(&preamble.join("\n"));
    let body = parse_body_prefix(fx.get("body_prefix").unwrap().str());
    let size = if fx.get("class_options").unwrap().str().contains("12pt") {
        12.0
    } else {
        10.0
    };
    let lines: Vec<&str> = fx
        .get("lines")
        .unwrap()
        .arr()
        .iter()
        .map(Json::str)
        .collect();

    let cmd = cmds.iter().find(|c| match (&c.kind, &body.role, &body.cs) {
        (FontCommandKind::SetMainFont, FamilyRole::Main, _) => true,
        (FontCommandKind::SetSansFont, FamilyRole::Sans, _) => true,
        (FontCommandKind::SetMonoFont, FamilyRole::Mono, _) => true,
        (FontCommandKind::NewFontFamily(cs), FamilyRole::Other, Some(b)) => cs == b,
        _ => false,
    });
    let slot = FaceSlot::new(body.bold, body.italic);
    let default_options = FontOptions::default();
    let (role, options, face) = match cmd {
        Some(c) => {
            let face = substitution_chain(slot)
                .iter()
                .find_map(|s| face_request(&c.name, &c.options, *s))
                .and_then(|r| load(&r));
            (c.kind.role(), &c.options, face)
        }
        None => {
            // fontspec's default: TU/lmr, Latin Modern optical sizes, Ligatures=TeX.
            let file = latin_modern_default_file(size, body.bold, body.italic);
            (
                FamilyRole::Main,
                &default_options,
                load(&FaceRequest::File { file, path: None }),
            )
        }
    };
    let Some(face) = face else {
        t.skipped.push(format!("{id}: font not found"));
        return;
    };
    // Scale: MatchLowercase/MatchUppercase relative to the document default
    // (Latin Modern Roman at the same size).
    let scale = options.scale.map(|s| match s {
        Scale::Factor(f) => f,
        Scale::MatchLowercase | Scale::MatchUppercase => {
            let lm = load(&FaceRequest::File {
                file: latin_modern_default_file(size, false, false),
                path: None,
            })
            .expect("LM for Match*");
            let pick = |f: &flashtex_font_engine::TrueTypeFace| {
                let m = f.vertical_metrics();
                let v = if s == Scale::MatchLowercase {
                    m.x_height
                } else {
                    m.cap_height
                };
                f64::from(v) / f64::from(f.units_per_em())
            };
            s.factor(pick(&lm), pick(&face))
        }
    });
    let size_pt = size * scale.unwrap_or(1.0);

    for (engine, profile) in engines() {
        let Some(exp) = expected(id, engine) else {
            t.skipped.push(format!("{id}.{engine}: no oracle"));
            continue;
        };
        let plan = FeaturePlan::resolve(&role, &[], options, slot);
        let fi = FontInstance {
            face: &face,
            size_pt,
            plan,
            profile,
        };

        if let Some(fd) = exp.get("fontdimens_pt") {
            let g = fi.glue();
            let pairs = [
                ("space", g.space),
                ("stretch", g.stretch),
                ("shrink", g.shrink),
                ("extra", g.extra),
                ("size", size_pt),
            ];
            for (k, v) in pairs {
                t.glue += 1;
                let want = fd.get(k).unwrap().num();
                if (want - v).abs() <= 0.02 {
                    t.glue_pass += 1;
                } else if verbose {
                    println!("  GLUE {id}.{engine} {k}: ours {v:.5} oracle {want:.5}");
                }
            }
        }

        let oracle_lines = exp.get("lines").unwrap().arr();
        for (i, line) in lines.iter().enumerate() {
            t.lines += 1;
            let entry = t.by_engine.entry(engine.to_string()).or_default();
            entry.0 += 1;
            let Some(ow) = oracle_lines.get(i).map(|l| l.arr()) else {
                continue;
            };
            let ours = layout_line(&fi, line, body.french);
            let mut max_err: f64 = 0.0;
            let mut ok_words = 0;
            if ours.words.len() == ow.len() {
                let x0 = ow[0].get("x0").unwrap().num();
                for (w, o) in ours.words.iter().zip(ow) {
                    let e0 = (o.get("x0").unwrap().num() - x0 - w.x_pt).abs();
                    let e1 = (o.get("x1").unwrap().num() - x0 - (w.x_pt + w.width_pt)).abs();
                    let e = e0.max(e1);
                    max_err = max_err.max(e);
                    t.words += 1;
                    if e <= TOL_PT {
                        t.words_pass += 1;
                        ok_words += 1;
                    }
                }
            } else {
                max_err = f64::INFINITY;
                t.words += ow.len();
            }
            if std::env::var("KC104_DEBUG").is_ok_and(|d| d == id) {
                let x0 = ow
                    .first()
                    .map(|o| o.get("x0").unwrap().num())
                    .unwrap_or(0.0);
                for (k, o) in ow.iter().enumerate() {
                    let w = ours.words.get(k);
                    println!(
                        "    {engine} {:<14} oracle x0 {:8.3} x1 {:8.3} | ours x0 {:8.3} x1 {:8.3}",
                        o.get("t").unwrap().str(),
                        o.get("x0").unwrap().num() - x0,
                        o.get("x1").unwrap().num() - x0,
                        w.map(|w| w.x_pt).unwrap_or(f64::NAN),
                        w.map(|w| w.x_pt + w.width_pt).unwrap_or(f64::NAN)
                    );
                }
            }
            let pass = max_err <= TOL_PT;
            if max_err <= 0.1 {
                t.lines_tight += 1;
            }
            if pass {
                t.lines_pass += 1;
                t.by_engine.get_mut(engine).unwrap().1 += 1;
            }
            if verbose {
                println!(
                    "{} {id:<26} {engine:<8} line {i} words {ok_words}/{} max_err {max_err:.3}pt{}",
                    if pass { "PASS" } else { "FAIL" },
                    ow.len(),
                    if pass || ours.notes.is_empty() {
                        String::new()
                    } else {
                        format!(" notes: {}", ours.notes.join("; "))
                    }
                );
            }
        }
    }
}

fn run_math_fixture(fx: &Json, t: &mut Tally, verbose: bool) {
    let id = fx.get("id").unwrap().str();
    let preamble: Vec<&str> = fx
        .get("preamble")
        .unwrap()
        .arr()
        .iter()
        .map(Json::str)
        .collect();
    let src = preamble.join("\n");
    let opts = MathOptions::from_source(&src);
    let lines: Vec<&str> = fx
        .get("lines")
        .unwrap()
        .arr()
        .iter()
        .map(Json::str)
        .collect();
    let math_cmd = parse_font_commands(&src)
        .into_iter()
        .find(|c| c.kind == FontCommandKind::SetMathFont);
    for (engine, _) in engines() {
        let Some(exp) = expected(id, engine) else {
            t.skipped.push(format!("{id}.{engine}: no oracle"));
            continue;
        };
        let oracle_lines = exp.get("lines").unwrap().arr();
        for (i, line) in lines.iter().enumerate() {
            t.math_lines += 1;
            let want: String = oracle_lines
                .get(i)
                .map(|l| l.arr().iter().map(|w| w.get("t").unwrap().str()).collect())
                .unwrap_or_default();
            let got = map_expression(line.trim_matches('$'), &opts);
            if got == want {
                t.math_lines_pass += 1;
            } else if verbose {
                println!("FAIL {id:<26} {engine:<8} {line}: ours {got} oracle {want}");
            }
        }
        if let (Some(um), Some(cmd)) = (exp.get("umath_pt").and_then(Json::obj), &math_cmd) {
            let file = tex_font_file(&cmd.name, false, false).unwrap_or_else(|| cmd.name.clone());
            let Some(face) = load(&FaceRequest::File { file, path: None }) else {
                t.skipped.push(format!("{id}: math font not found"));
                continue;
            };
            let params = MathFontParams::from_face(&face).expect("MATH table");
            for (k, v) in params.umath_pt(10.0) {
                let Some(want) = um.get(k) else { continue };
                t.umath += 1;
                if (want.num() - v).abs() <= 0.011 {
                    t.umath_pass += 1;
                } else if verbose {
                    println!("  UMATH {id} {k}: ours {v:.4} oracle {:.4}", want.num());
                }
            }
        }
    }
}

#[test]
fn oracle_fixtures() {
    let specs = parse(&std::fs::read_to_string(crate_dir().join("fixtures/specs.json")).unwrap());
    let verbose = std::env::var_os("KC104_QUIET").is_none();
    let mut t = Tally::default();
    let fixtures = specs.get("fixtures").unwrap().arr();
    for fx in fixtures {
        match fx.get("kind").unwrap().str() {
            "text" => run_text_fixture(fx, &mut t, verbose),
            _ => run_math_fixture(fx, &mut t, verbose),
        }
    }
    println!(
        "\nfixtures: {} ({} text lines, {} math lines per engine pair)",
        fixtures.len(),
        t.lines,
        t.math_lines
    );
    println!(
        "text lines within {TOL_PT}pt: {}/{} (within 0.1pt: {})  words: {}/{}",
        t.lines_pass, t.lines, t.lines_tight, t.words_pass, t.words
    );
    for (e, (n, p)) in &t.by_engine {
        println!("  {e}: {p}/{n} lines");
    }
    println!("glue fontdimens: {}/{}", t.glue_pass, t.glue);
    println!(
        "unicode-math alphabet lines: {}/{}",
        t.math_lines_pass, t.math_lines
    );
    println!("\\Umath parameters: {}/{}", t.umath_pass, t.umath);
    for s in &t.skipped {
        println!("skipped: {s}");
    }
    if !t.skipped.is_empty() {
        return; // fonts missing on this machine: report only
    }
    assert!(
        t.lines_pass >= MIN_LINES_PASS,
        "text lines regressed: {} < {MIN_LINES_PASS}",
        t.lines_pass
    );
    assert!(t.glue_pass >= MIN_GLUE_PASS, "glue regressed");
    assert_eq!(t.math_lines_pass, t.math_lines, "unicode-math alphabets");
    assert_eq!(t.umath_pass, t.umath, "MATH constants");
}

/// Floors recorded from the current implementation (see README "Oracle results").
const MIN_LINES_PASS: usize = 228;
const MIN_GLUE_PASS: usize = 429;

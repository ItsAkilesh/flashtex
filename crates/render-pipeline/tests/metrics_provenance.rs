//! The metrics a compile actually used are visible in its diagnostics: the
//! pinned Latin Modern 2.004 TFMs load digest-bound through font-resources,
//! and their absence is a blocking `required_metrics_unavailable` error,
//! never a silent switch to OpenType advances.

mod common;

use common::*;
use flashtex_compiler::parser::SourceDocument;
use flashtex_render_pipeline::display::Severity;
use flashtex_render_pipeline::fonts::{TfmStatus, REQUIRED_TFMS};
use flashtex_render_pipeline::{render, FontSet, RenderOptions};

#[test]
fn required_metrics_load_digest_bound_and_a_clean_compile_means_tex_metrics() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let fonts = FontSet::with_default_dirs(&[]);
    let set = fonts.required_metrics().expect("pinned 12pt metrics load from texmf-dist");
    for (file, sha) in REQUIRED_TFMS {
        let (asset, tfm) = set.get(&format!("fonts/tfm/public/lm/{file}")).unwrap();
        assert_eq!(asset.sha256, sha);
        assert_eq!(tfm.source_sha256, sha, "{file}");
        let t = fonts.tfm(file).unwrap();
        assert_eq!(t.sha256(), sha);
    }
    let docs = [SourceDocument { path: "main.tex", text: "\\begin{document}Body $x^2$ text.\\end{document}" }];
    let r = render(&docs, "main.tex", 1, "p", &fonts, &RenderOptions::default());
    // The only diagnostics are the outline-resource profile notes for the
    // math families that have no optical OpenType sibling (lmmi12/lmmi8
    // drawn from Latin Modern Math); metrics are the pinned TFMs.
    let others: Vec<_> = r.v2.diagnostics.iter().filter(|d| d.code != "math_resource_profile").collect();
    assert!(others.is_empty(), "{others:?}");
    assert!(r.v2.diagnostics.iter().any(|d| d.code == "math_resource_profile" && d.message.starts_with("lmmi12")), "{:?}", r.v2.diagnostics);
    let body = fonts.by_name("lmroman12-regular").expect("text face loaded");
    assert_eq!(body.tfm_status, TfmStatus::Loaded);
    assert_eq!(body.tfm.as_ref().unwrap().sha256(), REQUIRED_TFMS[0].1);
}

#[test]
fn missing_required_metrics_are_a_blocking_diagnostic_not_a_silent_fallback() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    // The OTFs without their TFM siblings: a font directory whose
    // `fonts/tfm/...` counterpart does not exist.
    let real = FontSet::with_default_dirs(&[]);
    let src_dir = real.dirs().iter().find(|d| d.join("lmroman12-regular.otf").is_file()).unwrap().clone();
    let math_dir = real.dirs().iter().find(|d| d.join("latinmodern-math.otf").is_file()).unwrap().clone();
    let tmp = std::env::temp_dir().join(format!("flashtex-no-tfm-{}", std::process::id()));
    let otf_dir = tmp.join("fonts/opentype/public/lm");
    std::fs::create_dir_all(&otf_dir).unwrap();
    for f in ["lmroman12-regular.otf", "lmroman12-bold.otf", "lmroman12-italic.otf", "lmroman10-regular.otf", "lmroman17-regular.otf"] {
        let _ = std::fs::copy(src_dir.join(f), otf_dir.join(f));
    }
    let _ = std::fs::copy(math_dir.join("latinmodern-math.otf"), otf_dir.join("latinmodern-math.otf"));
    let fonts = FontSet::new(vec![otf_dir.clone()]);
    assert!(fonts.required_metrics().is_err());
    assert!(matches!(fonts.tfm("ec-lmr12.tfm"), Err(TfmStatus::RequiredUnavailable(_))));
    let docs = [SourceDocument { path: "main.tex", text: "\\begin{document}Body $x^2$ text.\\end{document}" }];
    let r = render(&docs, "main.tex", 1, "p", &fonts, &RenderOptions::default());
    let blocking: Vec<_> = r.v2.diagnostics.iter().filter(|d| d.code == "required_metrics_unavailable").collect();
    assert!(blocking.len() >= 2, "text and math roman both report: {:?}", r.v2.diagnostics);
    assert!(blocking.iter().all(|d| d.severity == Severity::Error));
    assert!(blocking[0].message.contains("ec-lmr12.tfm"), "{}", blocking[0].message);
    assert!(!r.v2.pages.is_empty(), "the document is still laid out (OpenType metrics) so the editor shows something");
    let _ = std::fs::remove_dir_all(&tmp);
}

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

/// The layout an app bundle can ship: one flat directory with the OTFs,
/// the four required TFMs and the GUST licence (`Contents/Resources/Fonts`,
/// or any `FLASHTEX_FONT_DIRS` entry); the required set loads from it
/// digest-bound, exactly like from a texmf tree.
#[test]
fn a_flat_bundle_directory_with_tfms_and_licence_satisfies_the_required_set() {
    if !lm_available() {
        eprintln!("skipping: Latin Modern not installed");
        return;
    }
    let real = FontSet::with_default_dirs(&[]);
    let src_dir = real.dirs().iter().find(|d| d.join("lmroman12-regular.otf").is_file()).unwrap().clone();
    let math_dir = real.dirs().iter().find(|d| d.join("latinmodern-math.otf").is_file()).unwrap().clone();
    let tfm_dir = real.tfm_dirs().iter().find(|d| d.join("ec-lmr12.tfm").is_file()).unwrap().clone();
    let texmf = tfm_dir.to_string_lossy().trim_end_matches("/fonts/tfm/public/lm").to_string();
    let tmp = std::env::temp_dir().join(format!("flashtex-flat-bundle-{}", std::process::id()));
    let flat = tmp.join("Fonts");
    std::fs::create_dir_all(&flat).unwrap();
    for f in ["lmroman12-regular.otf", "lmroman12-bold.otf", "lmroman12-italic.otf", "lmroman10-regular.otf", "lmroman8-regular.otf", "lmroman6-regular.otf"] {
        let _ = std::fs::copy(src_dir.join(f), flat.join(f));
    }
    let _ = std::fs::copy(math_dir.join("latinmodern-math.otf"), flat.join("latinmodern-math.otf"));
    for (f, _) in REQUIRED_TFMS {
        std::fs::copy(tfm_dir.join(f), flat.join(f)).unwrap();
    }
    std::fs::copy(format!("{texmf}/doc/fonts/lm/GUST-FONT-LICENSE.TXT"), flat.join("GUST-FONT-LICENSE.TXT")).unwrap();
    let fonts = FontSet::new(vec![flat.clone()]);
    fonts.required_metrics().expect("flat layout loads the pinned set");
    assert_eq!(fonts.tfm("ec-lmr12.tfm").unwrap().sha256(), REQUIRED_TFMS[0].1);
    let docs = [SourceDocument { path: "main.tex", text: "\\begin{document}Body $x^2$ text.\\end{document}" }];
    let r = render(&docs, "main.tex", 1, "p", &fonts, &RenderOptions::default());
    assert!(r.v2.diagnostics.iter().all(|d| d.code == "math_resource_profile"), "{:?}", r.v2.diagnostics);
    let _ = std::fs::remove_dir_all(&tmp);
}

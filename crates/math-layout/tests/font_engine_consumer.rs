//! FT-032 revision 4 consumer fixture.
//!
//! This crate's only real, in-repo consumer of [`MathFontMetrics`] outside
//! its own bundled adapters (`src/cm.rs`, `src/times.rs`) is
//! `flashtex_font_engine::adapters::math::OpenTypeMathFace`, found by:
//!
//! ```text
//! $ grep -rln "flashtex-math-layout\|flashtex_math_layout" --include="*.toml" --include="*.rs" . \
//!     | grep -v "^./crates/math-layout/"
//! crates/font-engine/Cargo.toml
//! crates/font-engine/tests/adapters.rs
//! crates/font-engine/src/adapters/math.rs
//! crates/font-engine/src/adapters/mod.rs
//! ```
//!
//! `crates/font-engine/src/adapters/math.rs` implements
//! `flashtex_math_layout::metrics::MathFontMetrics` for
//! `OpenTypeMathFace<'a>`, driven by a real OpenType `MATH` face (Latin
//! Modern Math) through `flashtex_font_engine`'s own TrueType/OpenType
//! parser. This file exercises that *real* adapter — not a stand-in — end to
//! end through this crate's own `layout_with_report`/`positioned_runs`,
//! without editing anything under `crates/font-engine`.
//!
//! The font itself is not vendored in this crate; it is read from the same
//! pinned TeX Live path `crates/font-engine`'s own consumer test
//! (`crates/font-engine/tests/adapters.rs::pinned`) uses,
//! `flashtex_font_engine::manifest::BASICTEX_OPENTYPE_ROOT`. When that path
//! is not present (a machine without a BasicTeX/TeX Live install), both
//! tests below print `SKIP: ...` and return rather than failing — the same
//! convention `crates/font-engine`'s own test already uses for this exact
//! font. On this machine, at commit time, that root is present, so both
//! tests do run.

use std::path::Path;

use flashtex_font_engine::adapters::math::OpenTypeMathFace;
use flashtex_font_engine::load_from_path;
use flashtex_font_engine::manifest::BASICTEX_OPENTYPE_ROOT;
use flashtex_math_layout::{
    Atom, CmMathMetrics, FontId, MathFontMetrics, MathList, SizeClass, Style, layout_with_report,
    positioned_runs,
};

/// Latin Modern Math, at the path `crates/font-engine`'s own `pinned()` test
/// helper resolves to under `BASICTEX_OPENTYPE_ROOT`
/// (`lm-math/latinmodern-math.otf`, per
/// `flashtex_font_engine::manifest::pinned_latin_modern`).
fn latin_modern_math_path() -> std::path::PathBuf {
    Path::new(BASICTEX_OPENTYPE_ROOT).join("lm-math/latinmodern-math.otf")
}

fn kitchen_sink_list() -> MathList {
    MathList::new(vec![
        Atom::frac(MathList::symbols("a"), MathList::symbols("b")),
        Atom::symbol('x').with_sup(MathList::symbols("2")),
        Atom::left_right(Some('('), Some(')'), MathList::symbols("y")),
    ])
}

#[test]
fn real_opentype_math_adapter_lays_out_through_this_crate_without_panicking() {
    let path = latin_modern_math_path();
    if !path.is_file() {
        eprintln!("SKIP: {} not present", path.display());
        return;
    }
    let face = load_from_path(&path).expect("parse latinmodern-math.otf");
    let math_face =
        OpenTypeMathFace::new(&face, 10.0, FontId(7)).expect("Latin Modern Math has a MATH table");

    let list = kitchen_sink_list();
    for style in [Style::DISPLAY, Style::TEXT, Style::SCRIPT, Style::SCRIPT_SCRIPT] {
        let report = layout_with_report(&list, style, &math_face);
        assert!(report.root.width.is_finite() && report.root.width > 0.0);
        assert!(report.root.height.is_finite());
        assert!(report.root.depth.is_finite());
        let runs = positioned_runs(&report.root, (0.0, 0.0));
        // "a", "b", "x", "2", "(", "y", ")": every glyph this adapter can
        // supply (Latin Modern Math covers all seven), so a regression that
        // silently dropped one would be caught here even without a specific
        // reference count pinned.
        assert_eq!(runs.glyphs.len(), 7, "{:?}", report.limitations);
        for g in &runs.glyphs {
            assert!(g.x.is_finite() && g.baseline_y.is_finite() && g.size.is_finite());
        }
    }
}

/// The real adapter's font identity is the caller-supplied [`FontId`]
/// (`OpenTypeMathFace::new`'s third argument), preserved verbatim onto every
/// [`flashtex_math_layout::Glyph`] it hands the layout engine — not
/// recomputed, not defaulted, not silently swapped for a different face's
/// identity. Two [`OpenTypeMathFace`]s over the very same parsed face but
/// constructed with different [`FontId`]s (as two different revisions of a
/// subset assignment for the same face would be) must keep their glyphs
/// distinguishable by that identity, never silently collapsed to one.
#[test]
fn real_adapter_preserves_the_exact_caller_supplied_font_identity_never_a_different_ones() {
    let path = latin_modern_math_path();
    if !path.is_file() {
        eprintln!("SKIP: {} not present", path.display());
        return;
    }
    let face = load_from_path(&path).expect("parse latinmodern-math.otf");
    let id_a = FontId(41);
    let id_b = FontId(42);
    let face_a = OpenTypeMathFace::new(&face, 10.0, id_a).expect("MATH table");
    let face_b = OpenTypeMathFace::new(&face, 10.0, id_b).expect("MATH table");

    let g_a = face_a
        .glyph('x', SizeClass::Text)
        .expect("Latin Modern Math has 'x'");
    let g_b = face_b
        .glyph('x', SizeClass::Text)
        .expect("Latin Modern Math has 'x'");

    assert_eq!(g_a.font_id, id_a, "adapter must stamp exactly its own identity, not invent one");
    assert_eq!(g_b.font_id, id_b);
    assert_ne!(
        g_a.font_id, g_b.font_id,
        "two distinct resource identities over the same face must stay distinct \
         -- never silently approximated to a single shared identity"
    );
    // Same underlying glyph otherwise (same gid, same geometry): the two
    // instances differ *only* in the identity that was asked of them.
    assert_eq!(g_a.gid, g_b.gid);
    assert_eq!(g_a.width, g_b.width);

    // Sanity: this crate's own bundled adapter is a different identity space
    // entirely (CmMathMetrics's FontIds are small dense indices into its own
    // ALL_FONTS table -- see tests/resource_identity.rs), so nothing here
    // accidentally exercises the same code path as the real adapter's.
    let cm = CmMathMetrics::latex_10pt();
    let cm_g = cm.glyph('x', SizeClass::Text).expect("cmmi has 'x'");
    assert_ne!(cm_g.font_id, id_a);
    assert_ne!(cm_g.font_id, id_b);
}

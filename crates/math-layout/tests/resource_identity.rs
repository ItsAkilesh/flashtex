//! Resource-identity regressions for this crate's two bundled
//! [`MathFontMetrics`] adapters (`CmMathMetrics`, `TimesApproxMetrics`).
//!
//! [`FontId`] is the only identity this crate carries end to end (see its
//! doc comment: "The renderer must draw glyph `gid` from exactly this font;
//! the engine never invents fonts"). These tests pin that contract in both
//! directions:
//!   - **same identity accepted**: a [`FontId`] a provider actually issued
//!     resolves, through that same provider, to that provider's own name for
//!     it (`same_identity_font_id_resolves_to_its_own_font_name`).
//!   - **stale/foreign identity refused, not silently approximated**: a
//!     [`FontId`] a provider never issued -- whether simply out of range, or
//!     one a *different* provider issued -- resolves to the crate's existing
//!     typed sentinel, `"unknown"`, never to some other, unrelated font's
//!     name that happens to share the numeric id. `CmMathMetrics` and
//!     `TimesApproxMetrics` already implement this (`cm.rs`'s
//!     `ALL_FONTS.get(...).unwrap_or_else(|| "unknown")`, `times.rs`'s `_ =>
//!     "unknown"`); what's new here is a regression test making that
//!     contract explicit and change-detected, plus proving the two
//!     providers' identity spaces don't accidentally collide with each other
//!     (`TimesApproxMetrics` deliberately numbers its three fonts 100-102,
//!     past `CmMathMetrics`'s `ALL_FONTS.len() == 18`, precisely to avoid
//!     this).
//!
//! FT-032 revision 4.

use flashtex_math_layout::cm::ALL_FONTS;
use flashtex_math_layout::times::{SYMBOL, TIMES_ITALIC, TIMES_ROMAN};
use flashtex_math_layout::{CmMathMetrics, FontId, MathFontMetrics, SizeClass, TimesApproxMetrics};

#[test]
fn same_identity_font_id_resolves_to_its_own_font_name() {
    let cm = CmMathMetrics::latex_10pt();
    for ch in ['x', 'A', '+', '\u{221A}'] {
        let Some(g) = cm.glyph(ch, SizeClass::Text) else {
            continue;
        };
        let name = cm.font_name(g.font_id);
        assert_ne!(name, "unknown", "a FontId this provider just issued must resolve");
        assert_eq!(
            name,
            ALL_FONTS[g.font_id.0 as usize].name,
            "must be exactly the font this id was assigned from, not a lookalike"
        );
    }

    let times = TimesApproxMetrics::new(10.0);
    let g = times.glyph('x', SizeClass::Text).expect("times has 'x'");
    assert_eq!(times.font_name(g.font_id), "Times-Italic");
}

#[test]
fn stale_font_id_past_this_providers_own_range_is_refused_not_approximated() {
    let cm = CmMathMetrics::latex_10pt();
    // Exactly one past the last real index -- the bound named in ALL_FONTS's
    // own length, not an arbitrary large number.
    let one_past = FontId(ALL_FONTS.len() as u32);
    assert_eq!(cm.font_name(one_past), "unknown");
    // Deep past it too (a plausible "counter kept running across a stale
    // session" bug shape), and the type's own max, to bound both ends.
    assert_eq!(cm.font_name(FontId(1_000_000)), "unknown");
    assert_eq!(cm.font_name(FontId(u32::MAX)), "unknown");

    let times = TimesApproxMetrics::new(10.0);
    assert_eq!(times.font_name(FontId(0)), "unknown");
    assert_eq!(times.font_name(FontId(u32::MAX)), "unknown");
}

#[test]
fn foreign_providers_font_id_is_refused_not_silently_collided_with_an_unrelated_font() {
    // A FontId issued by TimesApproxMetrics, presented to CmMathMetrics (a
    // stale identity relative to *this* provider even though some other
    // provider minted it validly) must not resolve to whatever CM font
    // happens to occupy that same numeric slot.
    let cm = CmMathMetrics::latex_10pt();
    assert!(
        (ALL_FONTS.len() as u32) < TIMES_ROMAN.0,
        "fixture assumption: CmMathMetrics's own range must not reach Times's ids, \
         or this test would not actually exercise a foreign id"
    );
    assert_eq!(cm.font_name(TIMES_ROMAN), "unknown");
    assert_eq!(cm.font_name(TIMES_ITALIC), "unknown");
    assert_eq!(cm.font_name(SYMBOL), "unknown");

    // And the reverse direction: a CM-issued FontId presented to Times.
    let g = cm.glyph('x', SizeClass::Text).expect("cmmi has 'x'");
    let times = TimesApproxMetrics::new(10.0);
    assert_eq!(times.font_name(g.font_id), "unknown");
}

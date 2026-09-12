//! Existing consumer integration fixture (FT-035 revision 4).
//!
//! `flashtex-color-expressions` never invents its own colour representation:
//! it resolves expressions directly into `flashtex_vector_graphics::Color`/
//! `Paint` (see `src/lib.rs`'s module docs). That dependency edge is the one
//! real, already-wired consumer boundary this crate has today, so this file
//! proves resolved values survive it intact by handing them to the actual
//! downstream types a painter consumes — `Item::Rule` / `PathFill` /
//! `PathStroke` inside a `DisplayList` — and then calling
//! `vector_graphics::pdf::content_stream`, the real function that turns
//! those items into PDF content-stream bytes (`rg`/`RG`, `g`/`G`, `k`/`K`,
//! `/GSn gs`). Every expected string below is computed by hand from
//! `pdf.rs`'s own documented formatting rules (`num`: 3 decimals, trailing
//! zeros trimmed) and its documented flip transform
//! (`F = [1 0 0 -1 0 H]`), the same discipline `src/lib.rs` and
//! `src/expr.rs` use for their own fixtures — never by calling this crate's
//! resolver and echoing back what it produced.
//!
//! ## Where the boundary stops
//!
//! `crates/vector-graphics` documents itself as "a proposal for later
//! consumer integration (rendering-v2, after ABI agreement) ... nothing
//! here is wired into the compiler, the Mac shell, or `crates/pdf` yet"
//! (`crates/vector-graphics/src/lib.rs`). That is confirmed structurally,
//! not just by that comment:
//!
//! - Only `crates/color-expressions` and `crates/vector-graphics` itself
//!   depend on `flashtex-vector-graphics` (checked by grepping every
//!   `crates/*/Cargo.toml` for the dependency name). `crates/pdf` and
//!   `crates/rendering-core` do **not** depend on it at all — there is no
//!   Cargo dependency edge from either real painter to this crate's output
//!   type.
//! - Those two crates already have their own, independent colour
//!   representations that are not `flashtex_vector_graphics::Color`/
//!   `Paint` and are not convertible to it in either direction by any code
//!   that exists today: `crates/pdf/src/v2.rs` writes `Paint { rgb:
//!   Option<[Decimal; 3]> }` (fixed-point, RGB-only, no gray/CMYK variant),
//!   and `crates/rendering-core/src/pdf_stream.rs` /
//!   `crates/rendering-core/src/mixed.rs` read/write a JSON `paint` object
//!   shaped `{"r":_,"g":_,"b":_,"a":_}` with a hard-coded `"color_space":
//!   "srgb"`.
//!
//! So the honest boundary this crate can exercise stops at
//! `flashtex-vector-graphics`'s own `Item`/`DisplayList`/`pdf` surface —
//! the fixtures below — and does **not** reach an actual `crates/pdf`
//! output file or `crates/rendering-core` display list today. Proving
//! *that* would require either crate to depend on and convert from
//! `flashtex_vector_graphics::Color`, which is not currently the case and
//! is out of this task's owned paths (`crates/color-expressions` only) to
//! add.

use flashtex_color_expressions::{base_palette, resolve, resolve_paint};
use flashtex_vector_graphics::pdf::{self, ExtGState};
use flashtex_vector_graphics::{
    Color, DisplayList, Item, ItemId, Paint, Path, PathFill, PathStroke, Point, Rect, Rule, Size,
    StrokeStyle,
};

/// `red!50!blue` hand-resolves to `Rgb(0.5, 0.0, 0.5)` (see `src/lib.rs`'s
/// `resolves_mix_by_hand`, which pins the same value independently). Handed
/// to a `Rule` in a 100x100pt page, `pdf::content_stream`'s documented
/// flip (`bottom = H - y - height`) and `num` formatting (3 decimals,
/// trimmed) give an exactly predictable content stream.
#[test]
fn resolved_rgb_mix_survives_into_a_real_pdf_fill_operator() {
    let palette = base_palette();
    let color = resolve("red!50!blue", &palette).unwrap();
    assert_eq!(color, Color::Rgb(0.5, 0.0, 0.5));

    let mut list = DisplayList::new(Size::new(100.0, 100.0));
    list.items.push(Item::Rule(Rule {
        id: ItemId(1),
        rect: Rect::new(0.0, 0.0, 50.0, 50.0),
        paint: Paint::opaque(color),
        source: None,
    }));

    let frag = pdf::content_stream(&list).unwrap();
    // Fill colour "0.5 0 0.5 rg", then the rect flipped to PDF's
    // bottom-left origin: bottom = 100 - 0 - 50 = 50.
    assert_eq!(frag.content, "0.5 0 0.5 rg\n0 50 50 50 re f\n");
    assert!(frag.ext_g_states.is_empty());
}

/// A `cmyk:...` literal is read straight off the input text (no palette
/// lookup, no arithmetic) so the expected `Color` is literally the four
/// numbers in the expression. This proves the CMYK variant specifically
/// (not just RGB) survives into the real `k` fill operator unchanged.
#[test]
fn resolved_cmyk_literal_survives_into_a_real_pdf_fill_operator() {
    let palette = base_palette();
    let color = resolve("cmyk:0.25,0.5,0.75,1", &palette).unwrap();
    assert_eq!(color, Color::Cmyk(0.25, 0.5, 0.75, 1.0));

    let mut list = DisplayList::new(Size::new(100.0, 100.0));
    list.items.push(Item::Rule(Rule {
        id: ItemId(1),
        rect: Rect::new(10.0, 20.0, 30.0, 40.0),
        paint: Paint::opaque(color),
        source: None,
    }));

    let frag = pdf::content_stream(&list).unwrap();
    // bottom = 100 - 20 - 40 = 40.
    assert_eq!(frag.content, "0.25 0.5 0.75 1 k\n10 40 30 40 re f\n");
}

/// `gray:0.5` handed to a *stroked* path (not a fill) proves the boundary
/// survives on the stroke side too, which uses the uppercase `G` operator
/// and goes through `PathStroke`/`StrokeStyle` rather than `Rule`.
#[test]
fn resolved_gray_literal_survives_into_a_real_pdf_stroke_operator() {
    let palette = base_palette();
    let color = resolve("gray:0.5", &palette).unwrap();
    assert_eq!(color, Color::Gray(0.5));

    let mut list = DisplayList::new(Size::new(200.0, 200.0));
    let mut path = Path::new();
    path.move_to(Point::new(10.0, 10.0))
        .line_to(Point::new(100.0, 50.0));
    list.items.push(Item::PathStroke(PathStroke {
        id: ItemId(1),
        path,
        style: StrokeStyle::with_width(2.0),
        paint: Paint::opaque(color),
        source: None,
    }));

    let frag = pdf::content_stream(&list).unwrap();
    // Flip (H=200): (10,10) -> (10,190); (100,50) -> (100,150).
    assert_eq!(frag.content, "0.5 G\n2 w\n10 190 m\n100 150 l\nS\n");
}

/// `resolve_paint`'s straight alpha is a separate channel from `Color` (see
/// `src/lib.rs`'s `resolve_paint_carries_alpha`). This proves *that* value
/// specifically survives into the real `/ExtGState` dictionary a painter
/// would register in the page's `/Resources`, not just the colour channel.
#[test]
fn resolved_alpha_survives_into_a_real_pdf_ext_g_state() {
    let palette = base_palette();
    let paint = resolve_paint("blue", &palette, 0.3).unwrap();
    assert_eq!(paint.color, Color::Rgb(0.0, 0.0, 1.0));
    assert_eq!(paint.alpha, 0.3);

    let mut list = DisplayList::new(Size::new(100.0, 100.0));
    list.items.push(Item::PathFill(PathFill {
        id: ItemId(1),
        path: {
            let mut p = Path::new();
            p.move_to(Point::new(0.0, 0.0))
                .line_to(Point::new(10.0, 0.0))
                .line_to(Point::new(10.0, 10.0))
                .close();
            p
        },
        rule: flashtex_vector_graphics::FillRule::NonZero,
        paint,
        source: None,
    }));

    let frag = pdf::content_stream(&list).unwrap();
    assert_eq!(
        frag.ext_g_states,
        vec![ExtGState {
            name: "GS0".to_string(),
            alpha: 0.3,
        }]
    );
    assert_eq!(
        frag.ext_g_states[0].dictionary(),
        "<< /Type /ExtGState /ca 0.3 /CA 0.3 >>"
    );
    assert!(frag.content.starts_with("/GS0 gs\n0 0 1 rg\n"));
}

/// A negated, mixed expression run through a custom (non-base) palette,
/// same discipline as `src/lib.rs`'s `unicode_identifier_round_trips_...`
/// fixture: every expected number is worked out by hand, not produced by
/// calling this crate. Confirms the boundary holds for composed
/// expressions, not just single literals/names. Component values are
/// eighths so the `f64` arithmetic is bit-exact *and* representable in the
/// 3-decimal precision `pdf::num` documents, rather than merely
/// approximately right, matching the discipline `src/expr.rs` documents
/// for its own hand-derived fixtures.
#[test]
fn resolved_negated_mix_through_custom_palette_survives_the_boundary() {
    let mut palette = flashtex_color_expressions::Palette::new();
    palette.insert("brandA", Color::Rgb(0.75, 0.5, 0.0));
    palette.insert("brandB", Color::Rgb(0.0, 0.5, 0.75));
    // -brandA = Rgb(0.25, 0.5, 1.0). Then 50% of that + 50% of brandB:
    //   r: 0.5*0.25 + 0.5*0.0  = 0.125
    //   g: 0.5*0.5  + 0.5*0.5  = 0.5
    //   b: 0.5*1.0  + 0.5*0.75 = 0.875
    let color = resolve("-brandA!50!brandB", &palette).unwrap();
    assert_eq!(color, Color::Rgb(0.125, 0.5, 0.875));

    let mut list = DisplayList::new(Size::new(10.0, 10.0));
    list.items.push(Item::Rule(Rule {
        id: ItemId(1),
        rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        paint: Paint::opaque(color),
        source: None,
    }));
    let frag = pdf::content_stream(&list).unwrap();
    // bottom = 10 - 0 - 1 = 9.
    assert_eq!(frag.content, "0.125 0.5 0.875 rg\n0 9 1 1 re f\n");
}

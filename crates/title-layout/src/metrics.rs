//! Caller-supplied exact horizontal glyph metrics.
//!
//! This crate does no text shaping, does not bundle a font, and does not
//! guess advance widths. Horizontal measurement (line widths, centering,
//! and multi-author side-by-side placement — the gap named in revision 1)
//! is only available when the caller hands in exact metrics through
//! [`GlyphMetrics`], typically backed by its own font engine (an advance
//! width read from a real font and scaled to points at the size in
//! question, or an equivalent exact source). When the caller has no
//! measurement — a missing glyph, an unmeasured size — the
//! trait returns `None` and [`crate::layout_title_block_with_metrics`]
//! rejects the whole block with a typed error rather than substituting a
//! nominal em, a fallback width, or any other fabricated number.

use flashtex_document_style::Pt;

/// Caller-supplied exact horizontal glyph metrics for one abstract "font" (a
/// family/style/size combination the caller identifies however it likes;
/// this crate only ever asks for widths at the sizes `article.cls` uses for
/// the title block: `\LARGE` and `\large`).
///
/// Widths are queried per [`char`] — a full Unicode scalar value, already
/// decoded from the caller's UTF-8 text — never per byte, so a multi-byte
/// character costs exactly one query, the same as an ASCII one.
pub trait GlyphMetrics {
    /// Exact advance width of `ch` set at `size` (in points), or `None` when
    /// the caller cannot measure it (missing glyph, unmeasured size, ...).
    /// Returning `None` is the only correct response to "I don't know" —
    /// never a nominal em, a space-width guess, or any other placeholder.
    fn advance_width(&self, ch: char, size: Pt) -> Option<Pt>;

    /// The font's design em ("quad", TeX `\fontdimen6`) at `size`, in
    /// points, or `None` when the caller cannot supply it. Used only for
    /// the fixed part of `\and`'s `1em plus .17fil` inter-author glue: at
    /// natural width (no consumer of this crate stretches the block to a
    /// forced width) the `.17fil` stretch component never applies, so only
    /// the `1em` fixed part is needed.
    fn em(&self, size: Pt) -> Option<Pt>;
}

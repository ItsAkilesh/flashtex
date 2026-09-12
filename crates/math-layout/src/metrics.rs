//! The font-parameter interface the layout engine depends on.
//!
//! TeX takes its math parameters from `\fontdimen`s of the family-2 (symbol)
//! and family-3 (extension) fonts of the current size (TeXbook Appendix G,
//! "the parameters"). [`MathFontMetrics`] abstracts that so the engine can be
//! driven by the FT-018 font engine when it lands, by the Computer Modern TFM
//! tables shipped in [`crate::cm`], or by the Times approximation in
//! [`crate::times`].

/// Opaque font identity assigned by the metrics provider.
///
/// The provider maps it to a concrete font (a TFM name for the Computer Modern
/// adapter, a content-addressed font handle once the FT-018 font engine is the
/// provider) via [`MathFontMetrics::font_name`]. The renderer must draw glyph
/// `gid` from exactly this font; the engine never invents fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontId(pub u32);

/// The three sizes TeX distinguishes: text, script, and scriptscript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SizeClass {
    Text,
    Script,
    ScriptScript,
}

/// A glyph selected for a symbol, with its metrics already scaled to points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    pub font_id: FontId,
    /// Glyph identity inside `font_id` (TFM character code for the CM adapter).
    pub gid: u16,
    /// The symbol this glyph renders; kept for text extraction and fallbacks.
    pub ch: char,
    /// Font size in pt at which the metrics below apply.
    pub size: f64,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    /// Italic correction (TeXbook Appendix G "δ").
    pub italic: f64,
    /// Accent skew: kern with the font's skew character (Rule 12).
    pub skew: f64,
}

impl Glyph {
    pub fn total_height(&self) -> f64 {
        self.height + self.depth
    }
}

/// Math parameters for one size, in points.
///
/// Names follow TeXbook Appendix G. `x_height`..`axis_height` are family-2
/// fontdimens 5..22 (σ₅..σ₂₂); `default_rule_thickness`..`big_op_spacing5`
/// are family-3 fontdimens 8..13 (ξ₈..ξ₁₃). The remaining fields are TeX
/// primitives (`\scriptspace`, `\nulldelimiterspace`, `\delimiterfactor`,
/// `\delimitershortfall`) that plain.tex/LaTeX set to fixed values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathParams {
    /// Font size of family 2 at this size class, in pt.
    pub size: f64,
    pub x_height: f64,
    pub quad: f64,
    pub num1: f64,
    pub num2: f64,
    pub num3: f64,
    pub denom1: f64,
    pub denom2: f64,
    pub sup1: f64,
    pub sup2: f64,
    pub sup3: f64,
    pub sub1: f64,
    pub sub2: f64,
    pub sup_drop: f64,
    pub sub_drop: f64,
    pub delim1: f64,
    pub delim2: f64,
    pub axis_height: f64,
    pub default_rule_thickness: f64,
    pub big_op_spacing1: f64,
    pub big_op_spacing2: f64,
    pub big_op_spacing3: f64,
    pub big_op_spacing4: f64,
    pub big_op_spacing5: f64,
    pub script_space: f64,
    pub null_delimiter_space: f64,
    /// `\delimiterfactor` as a fraction (plain: 901/1000).
    pub delimiter_factor: f64,
    pub delimiter_shortfall: f64,
}

impl MathParams {
    /// One math unit: 1/18 of the family-2 quad at this size (TeXbook ch. 18).
    ///
    /// TeX computes `cur_mu = x_over_n(math_quad, 18)` in scaled points with
    /// truncation, which is why `\medmuskip` (4mu) is 2.22217pt in a 10pt
    /// document rather than 2.22222pt; the same truncation is applied here.
    pub fn mu(&self) -> f64 {
        let quad_sp = (self.quad * 65536.0).round();
        (quad_sp / 18.0).trunc() / 65536.0
    }
}

/// Everything the layout engine needs from a font set.
pub trait MathFontMetrics {
    /// Parameters at a size class.
    fn params(&self, size: SizeClass) -> MathParams;

    /// Human-readable name of a font identity (for reports and renderers).
    fn font_name(&self, font: FontId) -> String;

    /// The glyph for a symbol at a size class, or `None` when unsupported.
    fn glyph(&self, ch: char, size: SizeClass) -> Option<Glyph>;

    /// The display-size variant of a large operator (Rule 13), if any.
    fn large_operator(&self, ch: char, size: SizeClass) -> Option<Glyph>;

    /// Delimiter sizes for `ch`, smallest first (Rule 19 / `var_delimiter`).
    fn delimiter_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph>;

    /// Radical sign sizes, smallest first (Rule 11).
    fn radical_sizes(&self, size: SizeClass) -> Vec<Glyph>;

    /// Accent glyph variants, narrowest first (Rule 12).
    fn accent_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph>;
}

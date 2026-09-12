//! `MathFontMetrics` for math-layout, driven by Latin Modern Math through
//! font-engine: Appendix G parameters from the `MATH` constants
//! (`MathParams::from_opentype`, the LuaTeX correspondence), glyph metrics
//! from the face advances and the CFF charstring bounds, italic corrections
//! and top-accent attachment from the `MATH` table, and vertical glyph
//! variants (display-size operators, larger delimiters, radical signs) read
//! from `MathVariants` here because font-engine does not expose them yet
//! (requested API, see docs/proposals/rendering-abi.md).
//!
//! `\usepackage{times}` changes only the text fonts in LaTeX; math stays in
//! Computer Modern, so this provider is used for both families.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use flashtex_math_layout::{FontId as MathFontId, Glyph, MathFontMetrics, MathParams, OpenTypeMathConstants, SizeClass};

use crate::cff::{u16_at, u32_at};
use crate::fonts::LoadedFace;
use crate::ids::GlyphId;

/// The three sizes of one math context, points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MathSizes {
    pub text: f64,
    pub script: f64,
    pub script_script: f64,
}

impl MathSizes {
    pub fn at(&self, size: SizeClass) -> f64 {
        match size {
            SizeClass::Text => self.text,
            SizeClass::Script => self.script,
            SizeClass::ScriptScript => self.script_script,
        }
    }
}

/// One vertical variant from `MathVariants`: glyph and its declared advance
/// height (font units).
#[derive(Debug, Clone, Copy)]
struct VertVariant {
    gid: u16,
    #[allow(dead_code)]
    advance: u16,
}

pub struct MathFonts {
    face: Rc<LoadedFace>,
    sizes: MathSizes,
    constants: OpenTypeMathConstants,
    x_height_units: i16,
    vert_variants: BTreeMap<u16, Vec<VertVariant>>,
    /// Characters with no glyph in the math font, recorded for diagnostics.
    missing: RefCell<Vec<char>>,
}

/// `FontId(0)` is the math face; this provider uses exactly one font.
const MATH_FONT: MathFontId = MathFontId(0);

impl MathFonts {
    /// `face` must carry a `MATH` table (Latin Modern Math); `None` otherwise.
    pub fn new(face: Rc<LoadedFace>, sizes: MathSizes) -> Option<MathFonts> {
        let table = face.math()?;
        let c = &table.constants;
        let constants = OpenTypeMathConstants {
            units_per_em: face.units_per_em as u16,
            axis_height: c.axis_height,
            fraction_numerator_display_style_shift_up: c.fraction_numerator_display_style_shift_up,
            fraction_numerator_shift_up: c.fraction_numerator_shift_up,
            stack_top_shift_up: c.stack_top_shift_up,
            fraction_denominator_display_style_shift_down: c.fraction_denominator_display_style_shift_down,
            fraction_denominator_shift_down: c.fraction_denominator_shift_down,
            superscript_shift_up: c.superscript_shift_up,
            superscript_shift_up_cramped: c.superscript_shift_up_cramped,
            subscript_shift_down: c.subscript_shift_down,
            superscript_baseline_drop_max: c.superscript_baseline_drop_max,
            subscript_baseline_drop_min: c.subscript_baseline_drop_min,
            fraction_rule_thickness: c.fraction_rule_thickness,
            upper_limit_gap_min: c.upper_limit_gap_min,
            lower_limit_gap_min: c.lower_limit_gap_min,
            upper_limit_baseline_rise_min: c.upper_limit_baseline_rise_min,
            lower_limit_baseline_drop_min: c.lower_limit_baseline_drop_min,
            delimited_sub_formula_min_height: c.delimited_sub_formula_min_height,
        };
        let vm = face.face().vertical_metrics();
        let x_height_units = if vm.x_height_declared { vm.x_height } else { 431 };
        let vert_variants = face
            .otf()
            .and_then(|f| f.table(b"MATH"))
            .and_then(|t| parse_vertical_variants(t).ok())
            .unwrap_or_default();
        Some(MathFonts {
            face,
            sizes,
            constants,
            x_height_units,
            vert_variants,
            missing: RefCell::new(Vec::new()),
        })
    }

    pub fn face(&self) -> &Rc<LoadedFace> {
        &self.face
    }

    pub fn take_missing(&self) -> Vec<char> {
        std::mem::take(&mut *self.missing.borrow_mut())
    }

    /// The `k`-th (1-based) vertical variant of `base` from `MathVariants`,
    /// in the table's increasing-size order.
    pub fn variant_gid(&self, base: u16, k: usize) -> Option<u16> {
        if k == 0 {
            return Some(base);
        }
        self.vert_variants.get(&base)?.get(k - 1).map(|v| v.gid)
    }

    /// The character actually drawn for a math symbol: letters and lower-case
    /// Greek go to the Unicode mathematical-italic block (what `cmmi` is to
    /// `cmr`), everything else is itself.
    pub fn math_char(ch: char) -> char {
        match ch {
            'h' => '\u{210E}',
            'a'..='z' => char::from_u32(0x1D44E + (ch as u32 - 'a' as u32)).unwrap_or(ch),
            'A'..='Z' => char::from_u32(0x1D434 + (ch as u32 - 'A' as u32)).unwrap_or(ch),
            '\u{3B1}'..='\u{3C9}' => char::from_u32(0x1D6FC + (ch as u32 - 0x3B1)).unwrap_or(ch),
            '\u{2202}' => '\u{1D715}', // partial
            '\u{3F5}' => '\u{1D716}',  // epsilon
            '\u{3D1}' => '\u{1D717}',  // theta variant
            '\u{3F0}' => '\u{1D718}',  // kappa variant
            '\u{3D5}' => '\u{1D719}',  // phi variant
            '\u{3F1}' => '\u{1D71A}',  // rho variant
            '\u{3D6}' => '\u{1D71B}',  // pi variant
            _ => ch,
        }
    }

    fn glyph_for(&self, gid: u16, ch: char, size_pt: f64) -> Glyph {
        let face = &self.face;
        let g = GlyphId(gid);
        let adv = i64::from(face.face().advance(g).unwrap_or(0));
        let b = face.bounds(g, Some(ch));
        let (height, depth) = if b.empty {
            (0.0, 0.0)
        } else {
            (face.pt(i64::from(b.y_max), size_pt), face.pt(-i64::from(b.y_min), size_pt))
        };
        let width = face.pt(adv, size_pt);
        let (italic, skew) = match face.math() {
            Some(m) => {
                let ic = face.pt(i64::from(m.italics_correction(g)), size_pt);
                let skew = m
                    .top_accent_attachment(g)
                    .map(|t| face.pt(i64::from(t), size_pt) - width / 2.0)
                    .unwrap_or(0.0);
                (ic, skew)
            }
            None => (0.0, 0.0),
        };
        Glyph {
            font_id: MATH_FONT,
            gid,
            ch,
            size: size_pt,
            width,
            height,
            depth,
            italic,
            skew,
        }
    }

    fn base_gid(&self, ch: char) -> Option<u16> {
        let drawn = Self::math_char(ch);
        let g = self.face.face().glyph_id(drawn).or_else(|| self.face.face().glyph_id(ch));
        if g.is_none() {
            self.missing.borrow_mut().push(ch);
        }
        g.map(|g| g.0)
    }

    /// Base glyph followed by its vertical variants, in increasing size.
    fn variants(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        let Some(base) = self.base_gid(ch) else {
            return Vec::new();
        };
        let size_pt = self.sizes.at(size);
        let drawn = Self::math_char(ch);
        let mut out = vec![self.glyph_for(base, drawn, size_pt)];
        if let Some(vs) = self.vert_variants.get(&base) {
            for v in vs {
                if v.gid != base {
                    out.push(self.glyph_for(v.gid, drawn, size_pt));
                }
            }
        }
        out.sort_by(|a, b| a.total_height().partial_cmp(&b.total_height()).unwrap_or(std::cmp::Ordering::Equal));
        out
    }
}

impl MathFontMetrics for MathFonts {
    fn params(&self, size: SizeClass) -> MathParams {
        let upem = self.face.units_per_em as u16;
        MathParams::from_opentype(&self.constants, self.x_height_units, upem, self.sizes.at(size))
    }

    fn font_name(&self, _font: MathFontId) -> String {
        self.face.name.clone()
    }

    fn glyph(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        let gid = self.base_gid(ch)?;
        Some(self.glyph_for(gid, Self::math_char(ch), self.sizes.at(size)))
    }

    fn large_operator(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        let v = self.variants(ch, size);
        // The first variant strictly taller than the text-size glyph.
        let base_h = v.first()?.total_height();
        v.into_iter().find(|g| g.total_height() > base_h + 1e-6)
    }

    fn delimiter_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        self.variants(ch, size)
    }

    fn radical_sizes(&self, size: SizeClass) -> Vec<Glyph> {
        self.variants('\u{221A}', size)
    }

    fn accent_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        self.glyph(ch, size).into_iter().collect()
    }
}

/// Reads `MathVariants.VertGlyphCoverage`/`VertGlyphConstruction` (glyph
/// variant records only; glyph assemblies are not read).
fn parse_vertical_variants(m: &[u8]) -> Result<BTreeMap<u16, Vec<VertVariant>>, flashtex_font_engine::Error> {
    let mut out = BTreeMap::new();
    let variants_off = usize::from(u16_at(m, 8)?);
    if variants_off == 0 {
        return Ok(out);
    }
    let v = variants_off;
    let vert_cov = usize::from(u16_at(m, v + 2)?);
    let vert_count = usize::from(u16_at(m, v + 6)?);
    if vert_cov == 0 {
        return Ok(out);
    }
    let gids = parse_coverage(m, v + vert_cov)?;
    for (i, gid) in gids.iter().enumerate().take(vert_count) {
        let cons = v + usize::from(u16_at(m, v + 10 + 2 * i)?);
        let n = usize::from(u16_at(m, cons + 2)?);
        let mut list = Vec::with_capacity(n);
        for j in 0..n {
            let rec = cons + 4 + 4 * j;
            list.push(VertVariant {
                gid: u16_at(m, rec)?,
                advance: u16_at(m, rec + 2)?,
            });
        }
        out.insert(*gid, list);
    }
    Ok(out)
}

/// OpenType coverage table -> glyph ids in coverage-index order.
fn parse_coverage(b: &[u8], at: usize) -> Result<Vec<u16>, flashtex_font_engine::Error> {
    let format = u16_at(b, at)?;
    let mut gids = Vec::new();
    match format {
        1 => {
            let n = usize::from(u16_at(b, at + 2)?);
            for i in 0..n {
                gids.push(u16_at(b, at + 4 + 2 * i)?);
            }
        }
        2 => {
            let n = usize::from(u16_at(b, at + 2)?);
            for i in 0..n {
                let rec = at + 4 + 6 * i;
                let start = u16_at(b, rec)?;
                let end = u16_at(b, rec + 2)?;
                if start > end {
                    return Err(flashtex_font_engine::Error::Malformed("coverage range".into()));
                }
                gids.extend(start..=end);
            }
        }
        other => {
            return Err(flashtex_font_engine::Error::Unsupported(format!("coverage format {other}")));
        }
    }
    let _ = u32_at; // keep the helper linked for future assembly parsing
    Ok(gids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::{Family, FontSet, Role, DEFAULT_FONT_DIRS};
    use flashtex_math_layout::{layout_with_report, Atom, BoxKind, MathList, Style};

    fn lm_math() -> Option<MathFonts> {
        if !DEFAULT_FONT_DIRS.iter().any(|d| std::path::Path::new(d).join("latinmodern-math.otf").is_file()) {
            eprintln!("skipping: Latin Modern Math not installed");
            return None;
        }
        let fonts = FontSet::with_default_dirs(&[]);
        let face = fonts.resolve(Family::LatinModern, Role::Math, 10.0).face;
        MathFonts::new(
            face,
            MathSizes {
                text: 10.0,
                script: 7.0,
                script_script: 5.0,
            },
        )
    }

    #[test]
    fn parameters_come_from_the_math_table() {
        let Some(m) = lm_math() else { return };
        let p = m.params(SizeClass::Text);
        // Latin Modern Math AxisHeight = 250, FractionRuleThickness = 40.
        assert!((p.axis_height - 2.5).abs() < 1e-9, "{}", p.axis_height);
        assert!((p.default_rule_thickness - 0.4).abs() < 1e-9);
        assert!((p.x_height - 4.31).abs() < 0.02, "{}", p.x_height);
        assert!(p.num1 > p.num2 && p.denom1 > p.denom2);
    }

    #[test]
    fn letters_are_math_italic_and_operators_have_display_variants() {
        let Some(m) = lm_math() else { return };
        let x = m.glyph('x', SizeClass::Text).unwrap();
        assert_eq!(x.ch, '\u{1D465}');
        assert!(x.width > 4.0 && x.width < 7.0);
        assert!(x.height > 4.0 && x.depth < 0.2, "{} {}", x.height, x.depth);
        let sum = m.glyph('\u{2211}', SizeClass::Text).unwrap();
        let big = m.large_operator('\u{2211}', SizeClass::Text).expect("display-size sum");
        assert!(big.total_height() > sum.total_height());
        let parens = m.delimiter_sizes('(', SizeClass::Text);
        assert!(parens.len() >= 3, "{} paren sizes", parens.len());
        assert!(parens.windows(2).all(|w| w[0].total_height() <= w[1].total_height()));
        assert!(m.radical_sizes(SizeClass::Text).len() >= 2);
    }

    #[test]
    fn fraction_bar_is_an_explicit_rule() {
        let Some(m) = lm_math() else { return };
        let list = MathList::new(vec![Atom::frac(MathList::symbols("1"), MathList::symbols("2"))]);
        let l = layout_with_report(&list, Style::TEXT, &m);
        assert!(l.limitations.is_empty(), "{:?}", l.limitations);
        let runs = flashtex_math_layout::positioned_runs(&l.root, (0.0, 0.0));
        assert_eq!(runs.rules.len(), 1);
        assert!((runs.rules[0].h - 0.4).abs() < 1e-9);
        assert_eq!(runs.glyphs.len(), 2);
        assert!(matches!(l.root.kind, BoxKind::HBox(_)));
    }
}

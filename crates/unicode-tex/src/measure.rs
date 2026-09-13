//! Positions of words inside one line at natural glue: the part of the
//! Unicode-engine pipeline this crate owns (shaped word widths + interword
//! glue + space factor). Line breaking, justification and paragraph shape
//! belong to paragraph layout.

use flashtex_font_engine::{Face, GlyphId, TrueTypeFace};

use crate::engine::{EngineProfile, Renderer};
use crate::fontspec::FeaturePlan;
use crate::glue::{self, InterwordGlue};
use crate::shaper::{self, ShapeParams, ShapedText};

/// A face at a size under an engine with a feature plan.
pub struct FontInstance<'a> {
    pub face: &'a TrueTypeFace,
    /// Size after `Scale=`.
    pub size_pt: f64,
    pub plan: FeaturePlan,
    pub profile: EngineProfile,
}

impl FontInstance<'_> {
    pub fn shape(&self, text: &str) -> ShapedText {
        shaper::shape(
            self.face,
            text,
            ShapeParams {
                profile: self.profile,
                plan: &self.plan,
            },
        )
    }

    pub fn glue(&self) -> InterwordGlue {
        glue::interword_glue(self.face, self.size_pt, self.profile, &self.plan)
    }

    pub fn letter_space_pt(&self) -> f64 {
        self.plan.letter_space_percent * self.size_pt / 100.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordBox {
    pub text: String,
    /// Left edge (origin of the first glyph) relative to the line start.
    pub x_pt: f64,
    pub width_pt: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineLayout {
    pub words: Vec<WordBox>,
    pub notes: Vec<String>,
    pub missing: Vec<char>,
}

/// Lays out `line` (words separated by ASCII spaces) at natural glue.
pub fn layout_line(font: &FontInstance<'_>, line: &str, french_spacing: bool) -> LineLayout {
    let glue = font.glue();
    let ls = font.letter_space_pt();
    let mut x = 0.0;
    let mut words = Vec::new();
    let mut notes = Vec::new();
    let mut missing = Vec::new();
    let mut sf = 1000;
    let lua_node = font.profile.renderer != Renderer::HarfBuzz;
    let space_gid = font.face.glyph_id(' ');
    let unit = font.size_pt / f64::from(font.face.units_per_em());
    let params = ShapeParams {
        profile: font.profile,
        plan: &font.plan,
    };
    let mut prev_last: Option<GlyphId> = None;
    for (i, w) in line.split(' ').filter(|w| !w.is_empty()).enumerate() {
        let shaped = font.shape(w);
        if i > 0 {
            x += glue.natural(sf);
            if lua_node {
                // luaotfload letterspacing kerns the glue too (measured: the
                // gap is space + LetterSpace, like XeTeX, although
                // \fontdimen2 is unchanged), and GPOS pairs with the space
                // glyph apply across the glue.
                x += ls;
                if let Some(sp) = space_gid {
                    if let Some(l) = prev_last {
                        x += f64::from(shaper::pair_adjustment(font.face, l, sp, params)) * unit;
                    }
                    if let Some(first) = shaped.glyphs.first() {
                        x += f64::from(shaper::pair_adjustment(font.face, sp, first.gid, params))
                            * unit;
                    }
                }
            }
        }
        prev_last = shaped.glyphs.last().map(|g| g.gid);
        for n in &shaped.notes {
            if !notes.contains(n) {
                notes.push(n.clone());
            }
        }
        missing.extend(shaped.missing.iter().map(|m| m.0));
        let width = shaped_width(&shaped, font.size_pt, ls);
        words.push(WordBox {
            text: w.to_string(),
            x_pt: x,
            width_pt: width,
        });
        x += width;
        sf = glue::space_factor(w, french_spacing, 1000);
    }
    LineLayout {
        words,
        notes,
        missing,
    }
}

fn shaped_width(s: &ShapedText, size: f64, ls: f64) -> f64 {
    s.width_pt(size, ls)
}

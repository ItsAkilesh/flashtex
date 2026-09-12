//! Shaping through font-engine (`shape::shape`: cmap mapping, mark
//! composition, GSUB/AFM ligatures, GPOS/AFM kerning) with a per-face cache,
//! plus glyph extents from the face outlines. Everything cached is in font
//! units, so one shaping serves every size.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use flashtex_font_engine::{shape as fe_shape, ShapeOptions};

use crate::fonts::LoadedFace;
use crate::ids::GlyphId;

#[derive(Debug, Clone, PartialEq)]
pub struct SGlyph {
    pub gid: GlyphId,
    /// Advance in font units, kerning included.
    pub advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
    /// Extents in font units relative to the glyph origin (0 when empty).
    pub y_max: i32,
    pub y_min: i32,
    pub x_max: i32,
    pub empty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SCluster {
    pub glyphs: Vec<SGlyph>,
    /// Byte range into the shaped text.
    pub text_range: Range<usize>,
    pub text: String,
}

impl SCluster {
    pub fn advance_units(&self) -> i64 {
        self.glyphs.iter().map(|g| i64::from(g.advance)).sum()
    }
}

/// A shaped string in one face, in font units.
#[derive(Clone)]
pub struct Shaped {
    pub face: Rc<LoadedFace>,
    pub text: String,
    pub clusters: Vec<SCluster>,
    pub width_units: i64,
    /// Max glyph extent above the baseline, font units.
    pub height_units: i32,
    /// Max glyph extent below the baseline, font units (positive).
    pub depth_units: i32,
    /// Characters with no glyph in this face, with byte offsets into `text`.
    pub missing: Vec<(char, usize)>,
    /// Set when the shaper refused the text (unsupported script); the
    /// clusters are then empty and nothing is typeset for it.
    pub refused: Option<String>,
}

impl Shaped {
    pub fn width_pt(&self, size_pt: f64) -> f64 {
        self.face.pt(self.width_units, size_pt)
    }
    pub fn height_pt(&self, size_pt: f64) -> f64 {
        self.face.pt(i64::from(self.height_units), size_pt)
    }
    pub fn depth_pt(&self, size_pt: f64) -> f64 {
        self.face.pt(i64::from(self.depth_units), size_pt)
    }
}

#[derive(Default)]
pub struct Shaper {
    cache: RefCell<HashMap<(String, String), Rc<Shaped>>>,
}

impl Shaper {
    pub fn new() -> Shaper {
        Shaper::default()
    }

    /// Shapes `text` in `face` with kerning and ligatures on.
    pub fn shape(&self, face: &Rc<LoadedFace>, text: &str) -> Rc<Shaped> {
        let key = (face.font_id.clone(), text.to_string());
        if let Some(hit) = self.cache.borrow().get(&key) {
            return hit.clone();
        }
        let shaped = Rc::new(shape_uncached(face, text));
        self.cache.borrow_mut().insert(key, shaped.clone());
        shaped
    }
}

fn shape_uncached(face: &Rc<LoadedFace>, text: &str) -> Shaped {
    let f = face.face();
    let opts = ShapeOptions::default();
    let (clusters, missing, refused) = match fe_shape::shape(f, text, &opts) {
        Ok(s) => {
            let clusters = s
                .clusters
                .iter()
                .map(|c| SCluster {
                    glyphs: c
                        .glyphs
                        .iter()
                        .map(|g| {
                            let ch = c.text.chars().next();
                            let b = face.bounds(g.gid, ch);
                            SGlyph {
                                gid: g.gid,
                                advance: g.advance,
                                x_offset: g.x_offset,
                                y_offset: g.y_offset,
                                y_max: if b.empty { 0 } else { b.y_max },
                                y_min: if b.empty { 0 } else { b.y_min },
                                x_max: if b.empty { g.advance } else { b.x_max },
                                empty: b.empty,
                            }
                        })
                        .collect(),
                    text_range: c.source_range.clone(),
                    text: c.text.clone(),
                })
                .collect();
            (clusters, s.missing.iter().map(|m| (m.ch, m.byte_offset)).collect(), None)
        }
        Err(e) => (Vec::new(), Vec::new(), Some(e.to_string())),
    };
    let clusters: Vec<SCluster> = clusters;
    let width_units: i64 = clusters.iter().map(SCluster::advance_units).sum();
    let mut y_max = 0i32;
    let mut y_min = 0i32;
    for c in &clusters {
        for g in &c.glyphs {
            if g.empty {
                continue;
            }
            y_max = y_max.max(g.y_max + g.y_offset);
            y_min = y_min.min(g.y_min + g.y_offset);
        }
    }
    Shaped {
        face: face.clone(),
        text: text.to_string(),
        clusters,
        width_units,
        height_units: y_max,
        depth_units: -y_min,
        missing,
        refused,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::{Family, FontSet, Role, DEFAULT_FONT_DIRS};

    #[test]
    fn latin_modern_shaping_applies_ligatures_and_kerning() {
        if !DEFAULT_FONT_DIRS.iter().any(|d| std::path::Path::new(d).join("lmroman10-regular.otf").is_file()) {
            eprintln!("skipping: Latin Modern not installed");
            return;
        }
        let fonts = FontSet::with_default_dirs(&[]);
        let face = fonts.resolve(Family::LatinModern, Role::Text { bold: false, italic: false }, 10.0).face;
        let shaper = Shaper::new();
        let s = shaper.shape(&face, "office");
        // "ffi" is one cluster covering bytes 1..4.
        let lig = s.clusters.iter().find(|c| c.text == "ffi").expect("ffi ligature cluster");
        assert_eq!(lig.text_range, 1..4);
        assert_eq!(lig.glyphs.len(), 1);
        let av = shaper.shape(&face, "AV");
        let plain: i64 = av.clusters.iter().flat_map(|c| c.glyphs.iter()).map(|g| i64::from(g.advance)).sum();
        // font-engine README: "AV" shaped at 10pt is 13.89pt -> 1389 units (kerned).
        assert_eq!(plain, 1389);
        assert!(s.missing.is_empty());
        // Round letters overshoot the baseline by 11 units in Latin Modern.
        assert!(s.height_units > 600 && s.depth_units <= 15, "{} {}", s.height_units, s.depth_units);
    }
}

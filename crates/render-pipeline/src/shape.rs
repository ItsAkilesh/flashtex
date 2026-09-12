//! Shaping through font-engine (`shape::shape`: mapping, mark composition,
//! GSUB/AFM ligatures, GPOS/AFM kerning) with a per-face cache, plus glyph
//! extents from the face outlines.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use flashtex_font_engine::{shape as fe_shape, GlyphId, ShapeOptions};

use crate::fonts::LoadedFace;

#[derive(Debug, Clone, PartialEq)]
pub struct SGlyph {
    pub gid: GlyphId,
    /// Advance in font units, kerning included.
    pub advance: i32,
    pub x_offset: i32,
    pub y_offset: i32,
    /// Extents in font units relative to the glyph origin.
    pub y_max: i32,
    pub y_min: i32,
    pub x_max: i32,
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

/// A shaped string in one face at one size.
#[derive(Debug, Clone)]
pub struct Shaped {
    pub face: Rc<LoadedFace>,
    pub size_pt: f64,
    pub text: String,
    pub clusters: Vec<SCluster>,
    pub width_pt: f64,
    /// Max glyph extent above the baseline, points.
    pub height_pt: f64,
    /// Max glyph extent below the baseline, points (positive).
    pub depth_pt: f64,
    pub missing: Vec<char>,
    /// Set when the shaper refused the text (unsupported script); glyphs are
    /// then `.notdef` mapped one per char with the face's notdef advance.
    pub refused: Option<String>,
}

#[derive(Default)]
pub struct Shaper {
    cache: RefCell<HashMap<(String, String), Rc<Shaped>>>,
}

impl Shaper {
    pub fn new() -> Shaper {
        Shaper::default()
    }

    /// Shapes `text` in `face` at `size_pt` with kerning and ligatures on.
    pub fn shape(&self, face: &Rc<LoadedFace>, size_pt: f64, text: &str) -> Rc<Shaped> {
        let key = (face.font_id.clone(), text.to_string());
        if let Some(hit) = self.cache.borrow().get(&key) {
            if hit.size_pt == size_pt {
                return hit.clone();
            }
        }
        let shaped = Rc::new(shape_uncached(face, size_pt, text));
        // Cache keyed without size: units are size independent, so rescale.
        self.cache.borrow_mut().insert(key, shaped.clone());
        shaped
    }
}

fn shape_uncached(face: &Rc<LoadedFace>, size_pt: f64, text: &str) -> Shaped {
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
                            }
                        })
                        .collect(),
                    text_range: c.source_range.clone(),
                    text: c.text.clone(),
                })
                .collect();
            (clusters, s.missing.iter().map(|m| m.ch).collect(), None)
        }
        Err(e) => {
            let notdef = i32::from(f.advance(GlyphId::NOTDEF).unwrap_or(0));
            let clusters = text
                .char_indices()
                .map(|(i, ch)| SCluster {
                    glyphs: vec![SGlyph {
                        gid: GlyphId::NOTDEF,
                        advance: notdef,
                        x_offset: 0,
                        y_offset: 0,
                        y_max: 0,
                        y_min: 0,
                        x_max: notdef,
                    }],
                    text_range: i..i + ch.len_utf8(),
                    text: ch.to_string(),
                })
                .collect();
            (clusters, text.chars().collect(), Some(e.to_string()))
        }
    };
    let clusters: Vec<SCluster> = clusters;
    let units: i64 = clusters.iter().map(SCluster::advance_units).sum();
    let mut y_max = 0i32;
    let mut y_min = 0i32;
    for c in &clusters {
        for g in &c.glyphs {
            y_max = y_max.max(g.y_max + g.y_offset);
            y_min = y_min.min(g.y_min + g.y_offset);
        }
    }
    Shaped {
        face: face.clone(),
        size_pt,
        text: text.to_string(),
        width_pt: face.pt(units, size_pt),
        height_pt: face.pt(i64::from(y_max), size_pt),
        depth_pt: -face.pt(i64::from(y_min), size_pt),
        clusters,
        missing,
        refused,
    }
}

/// Rescales a cached shaping to another size (units are size independent).
pub fn at_size(shaped: &Shaped, size_pt: f64) -> Shaped {
    if shaped.size_pt == size_pt {
        return shaped.clone();
    }
    let k = size_pt / shaped.size_pt;
    Shaped {
        size_pt,
        width_pt: shaped.width_pt * k,
        height_pt: shaped.height_pt * k,
        depth_pt: shaped.depth_pt * k,
        ..shaped.clone()
    }
}

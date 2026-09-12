//! `\text{...}` inside math: TeX's `\hbox` of *text-font* characters.
//!
//! amsmath's `\text` sets its argument in the current text font at the
//! size of the surrounding math style (12/8/6 pt in a 12 pt article), as an
//! unbreakable hbox at natural width. That is exactly what the paragraph
//! path already does for a word: T1 `ec-lm*` TFM ligature/kern program
//! (`f f i` → slot 0x1E, braces at T1 123/125), glyph ids from the face's
//! own `cmap`, and interword glue from `\fontdimen2` (`+\fontdimen7` at
//! space factor ≥ 2000, TeX §1041–1044). This module reuses that shaper and
//! glue so `\text` gets the same geometry the paragraphs have — never the
//! math roman family's OT1 slots, which is what the HW1 text-producer
//! evidence (main f261b36c) found: `a b` advanced by rm-lmr12 slot 32,
//! `\{x\}` by OT1 slots 123/125, `ffi` as three glyphs.
//!
//! math-layout has no nucleus for a pre-typeset box, so a run enters the
//! layout as `Nucleus::Text(handle)` whose single placeholder character
//! reports the run's exact width/height/depth through
//! [`MathFontMetrics::text_glyph`]; after layout the placeholder glyph box
//! is replaced by the shaped hbox (identical metrics ⇒ identical Appendix G
//! spacing and script placement). Handles are Supplementary Private Use
//! Area-A characters and never reach the display list. The proper API —
//! `Nucleus::HBox(MathBox)` — is requested from math-layout in the handoff.

use std::cell::RefCell;
use std::rc::Rc;

use flashtex_math_layout as ml;
use flashtex_math_layout::metrics::Extensible;
use flashtex_math_layout::{FontId as MathFontId, Glyph, MathFontMetrics, MathParams, SizeClass};

use crate::adapter::space_factor;
use crate::fonts::{Family, FontSet, LoadedFace, Role};
use crate::ids::GlyphId;
use crate::shape::Shaper;

/// First `FontId` value of a text run; `FontId(RUN_FONT_BASE + i)` is run `i`
/// of the math box being laid out (the CM/OpenType providers use small ids).
pub const RUN_FONT_BASE: u32 = 0x4000_0000;

/// First placeholder character (Supplementary Private Use Area-A).
const HANDLE_BASE: u32 = 0xF_0000;

fn handle_char(index: usize) -> char {
    char::from_u32(HANDLE_BASE + index as u32).expect("private-use handle")
}

fn handle_index(ch: char) -> Option<usize> {
    let c = ch as u32;
    (HANDLE_BASE..HANDLE_BASE + 0x1_0000).contains(&c).then(|| (c - HANDLE_BASE) as usize)
}

/// Collects the `\text` arguments of one formula while the compiler list is
/// converted; each becomes an ordinary atom carrying a handle.
#[derive(Default, Debug)]
pub struct TextSink {
    pub texts: Vec<String>,
}

impl TextSink {
    /// An `Ord` atom for `text` (TeX §1076: an hbox in math is an Ord).
    pub fn atom(&mut self, text: &str) -> ml::Atom {
        let handle = handle_char(self.texts.len());
        self.texts.push(text.to_string());
        ml::Atom::new(ml::AtomClass::Ord, ml::Nucleus::Text(handle.to_string()))
    }
}

/// One glyph of a shaped run, addressed by its index (`gid` of the
/// placeholder-free glyph boxes in [`TextRun::hbox`]).
#[derive(Debug, Clone, PartialEq)]
pub struct RunGlyph {
    /// Original glyph id in [`TextRun::face`]; 0 for the interword space
    /// (no ink; the display list splits the run there).
    pub gid: GlyphId,
    /// First character of the cluster (what the placed glyph carries).
    pub ch: char,
    /// The cluster's source text: `"ffi"` for the ligature glyph, so text
    /// extraction keeps every character.
    pub text: String,
}

/// A `\text` argument shaped at one size.
#[derive(Clone)]
pub struct TextRun {
    pub text: String,
    pub size: f64,
    pub face: Rc<LoadedFace>,
    pub glyphs: Vec<RunGlyph>,
    /// The hbox: glyph boxes (`font_id` = this run, `gid` = index into
    /// `glyphs`, widths = TFM advances with kerns folded in) and the space
    /// glue at natural width, on the baseline.
    pub hbox: ml::MathBox,
    /// True when the TFM produced the advances (TeX's geometry).
    pub tfm_metrics: bool,
}

/// What happened while shaping, reported by the caller with the formula's
/// source range once layout is done.
#[derive(Debug, Clone, PartialEq)]
pub enum Notice {
    /// The face at this size was resolved (the caller replays the font
    /// availability/TFM diagnostics through its usual `face` path).
    FaceUsed { size: f64 },
    Refused { word: String, reason: String },
    MissingGlyph { ch: char, face: String },
    TfmRunError { word: String, face: String, error: String },
}

/// Lazily shapes the registered texts at the sizes the layout asks for and
/// answers the math-layout metrics interface for them, delegating everything
/// else to the real provider.
pub struct TextRunMetrics<'a> {
    inner: &'a dyn MathFontMetrics,
    fonts: &'a FontSet,
    shaper: &'a Shaper,
    family: Family,
    texts: &'a [String],
    runs: RefCell<Vec<TextRun>>,
    notices: RefCell<Vec<Notice>>,
}

impl<'a> TextRunMetrics<'a> {
    pub fn new(inner: &'a dyn MathFontMetrics, fonts: &'a FontSet, shaper: &'a Shaper, family: Family, texts: &'a [String]) -> TextRunMetrics<'a> {
        TextRunMetrics {
            inner,
            fonts,
            shaper,
            family,
            texts,
            runs: RefCell::new(Vec::new()),
            notices: RefCell::new(Vec::new()),
        }
    }

    /// The runs shaped so far and the notices, in order.
    pub fn finish(self) -> (Vec<TextRun>, Vec<Notice>) {
        (self.runs.into_inner(), self.notices.into_inner())
    }

    fn run_for(&self, text_index: usize, size: f64) -> Option<usize> {
        let text = self.texts.get(text_index)?;
        if let Some(i) = self.runs.borrow().iter().position(|r| r.text == *text && r.size == size) {
            return Some(i);
        }
        let index = self.runs.borrow().len();
        let run = shape_run(self.fonts, self.shaper, self.family, text, size, index, &mut self.notices.borrow_mut());
        self.runs.borrow_mut().push(run);
        Some(index)
    }
}

impl MathFontMetrics for TextRunMetrics<'_> {
    fn params(&self, size: SizeClass) -> MathParams {
        self.inner.params(size)
    }

    fn font_name(&self, font: MathFontId) -> String {
        match font.0.checked_sub(RUN_FONT_BASE) {
            Some(i) => self.runs.borrow().get(i as usize).map_or_else(|| format!("text-run-{i}"), |r| r.face.name.clone()),
            None => self.inner.font_name(font),
        }
    }

    fn glyph(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        self.inner.glyph(ch, size)
    }

    fn large_operator(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        self.inner.large_operator(ch, size)
    }

    fn delimiter_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        self.inner.delimiter_sizes(ch, size)
    }

    fn radical_sizes(&self, size: SizeClass) -> Vec<Glyph> {
        self.inner.radical_sizes(size)
    }

    fn accent_sizes(&self, ch: char, size: SizeClass) -> Vec<Glyph> {
        self.inner.accent_sizes(ch, size)
    }

    fn delimiter_extensible(&self, ch: char, size: SizeClass) -> Option<Extensible> {
        self.inner.delimiter_extensible(ch, size)
    }

    fn radical_extensible(&self, size: SizeClass) -> Option<Extensible> {
        self.inner.radical_extensible(size)
    }

    fn text_glyph(&self, ch: char, size: SizeClass) -> Option<Glyph> {
        let Some(text_index) = handle_index(ch) else {
            return self.inner.text_glyph(ch, size);
        };
        let at = self.inner.params(size).size;
        let i = self.run_for(text_index, at)?;
        let runs = self.runs.borrow();
        let run = &runs[i];
        Some(Glyph {
            font_id: MathFontId(RUN_FONT_BASE + i as u32),
            gid: 0,
            ch,
            size: at,
            width: run.hbox.width,
            height: run.hbox.height,
            depth: run.hbox.depth,
            italic: 0.0,
            skew: 0.0,
        })
    }
}

/// Replaces every placeholder glyph box in `root` by its shaped hbox.
pub fn substitute(root: &mut ml::MathBox, runs: &[TextRun]) {
    match &mut root.kind {
        ml::BoxKind::Glyph { font_id, ch, size, .. } => {
            if handle_index(*ch).is_some() {
                if let Some(run) = font_id.0.checked_sub(RUN_FONT_BASE).and_then(|i| runs.get(i as usize)) {
                    debug_assert_eq!(run.size, *size);
                    *root = run.hbox.clone();
                }
            }
        }
        ml::BoxKind::HBox(children) | ml::BoxKind::VBox(children) => {
            for c in children {
                substitute(&mut c.content, runs);
            }
        }
        ml::BoxKind::Rule | ml::BoxKind::Kern | ml::BoxKind::Glue { .. } => {}
    }
}

/// `\fontdimen2` and `\fontdimen7` of `face` at `size`, in points.
fn space_dimens(fonts: &FontSet, family: Family, face: &LoadedFace, size: f64) -> (f64, f64) {
    if let Some(tfm) = &face.tfm {
        let dim = |n: usize| tfm.param(n).map_or(0.0, |v| crate::tfm::Tfm::pt(v, size));
        return (dim(2), dim(7));
    }
    let _ = fonts;
    let design = crate::typeset::design_size(family, size);
    let p = crate::params::text_params(family, false, false, design).at(size);
    (p.space, p.extra_space)
}

/// Shapes `text` as an hbox at `size`: words through the face's shaper
/// (TFM ligatures/kerns), one glue per space at natural width with TeX's
/// space factor (1000 at the start of the box, §1034 per character).
fn shape_run(fonts: &FontSet, shaper: &Shaper, family: Family, text: &str, size: f64, index: usize, notices: &mut Vec<Notice>) -> TextRun {
    let font_id = MathFontId(RUN_FONT_BASE + index as u32);
    let face = fonts.resolve(family, Role::Text { bold: false, italic: false }, size).face;
    notices.push(Notice::FaceUsed { size });
    let (space, extra) = space_dimens(fonts, family, &face, size);
    let mut glyphs: Vec<RunGlyph> = Vec::new();
    let mut boxes: Vec<ml::MathBox> = Vec::new();
    let mut factor = 1000u32;
    let mut tfm_metrics = true;
    let mut first = true;
    for word in text.split(' ') {
        if !first {
            // Interword glue at natural width; an hbox is never stretched.
            let mut width = space;
            if factor >= 2000 {
                width += extra;
            }
            let gi = glyphs.len() as u16;
            glyphs.push(RunGlyph { gid: GlyphId(0), ch: ' ', text: " ".to_string() });
            boxes.push(ml::MathBox {
                kind: ml::BoxKind::Glyph { font_id, gid: gi, ch: ' ', size },
                width,
                height: 0.0,
                depth: 0.0,
            });
        }
        first = false;
        if word.is_empty() {
            continue;
        }
        let shaped = shaper.shape(&face, word);
        tfm_metrics &= shaped.tfm_metrics;
        if let Some(e) = &shaped.tfm_error {
            notices.push(Notice::TfmRunError { word: word.to_string(), face: face.name.clone(), error: e.clone() });
        }
        if let Some(reason) = &shaped.refused {
            notices.push(Notice::Refused { word: word.to_string(), reason: reason.clone() });
            continue;
        }
        for (ch, _) in &shaped.missing {
            notices.push(Notice::MissingGlyph { ch: *ch, face: face.name.clone() });
        }
        let height = shaped.height_pt(size);
        let depth = shaped.depth_pt(size);
        for c in &shaped.clusters {
            let ch = c.text.chars().next().unwrap_or('\u{FFFD}');
            for (k, g) in c.glyphs.iter().enumerate() {
                let gi = glyphs.len() as u16;
                // A cluster with several glyphs attributes its text to the first.
                let text = if k == 0 { c.text.clone() } else { String::new() };
                glyphs.push(RunGlyph { gid: g.gid, ch, text });
                let width = g.advance as f64 * size / shaped.units_per_em as f64;
                boxes.push(ml::MathBox {
                    kind: ml::BoxKind::Glyph { font_id, gid: gi, ch, size },
                    width,
                    height: if g.empty { 0.0 } else { height },
                    depth: if g.empty { 0.0 } else { depth },
                });
            }
        }
        for ch in word.chars() {
            factor = space_factor(ch, factor);
        }
    }
    let hbox = ml::MathBox::hlist(boxes);
    TextRun { text: text.to_string(), size, face, glyphs, hbox, tfm_metrics }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_round_trip_and_stay_private_use() {
        for i in [0usize, 1, 255, 0xFFFF] {
            let h = handle_char(i);
            assert_eq!(handle_index(h), Some(i));
            assert!(!h.is_ascii());
        }
        assert_eq!(handle_index('a'), None);
        assert_eq!(handle_index(' '), None);
        assert_eq!(handle_index('\u{E000}'), None, "BMP private use is not a handle");
    }

    #[test]
    fn sink_gives_ordinary_atoms_in_order() {
        let mut sink = TextSink::default();
        let a = sink.atom("(a)");
        let b = sink.atom("and");
        assert_eq!(sink.texts, vec!["(a)", "and"]);
        assert_eq!(a.class, ml::AtomClass::Ord);
        assert_eq!(a.nucleus, ml::Nucleus::Text(handle_char(0).to_string()));
        assert_eq!(b.nucleus, ml::Nucleus::Text(handle_char(1).to_string()));
    }
}

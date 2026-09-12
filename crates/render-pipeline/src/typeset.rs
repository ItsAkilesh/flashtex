//! Paragraph assembly, math boxes, pages, and the v2 display list.
//!
//! Words are shaped through font-engine and become `paragraph-layout` boxes
//! (`GlyphRun::from_shaped`, one per styled segment: advances in font units
//! with kerning folded in, original glyph ids, source-byte clusters).
//! Interword glue follows TeX's space factor and the face's `\fontdimen`s.
//! Inline math is laid out by `math-layout` (Appendix G) and enters the
//! horizontal list as one unbreakable box; a display equation is a
//! one-line block between `\abovedisplayskip`/`\belowdisplayskip` (the
//! short variants when the preceding line leaves room, TeX §1199). Line
//! breaking is `paragraph-layout`'s total-fit Knuth–Plass; page breaking is
//! `pagebuild` (TeX's page builder: interline glue, `\topskip`, penalty
//! costs for club/widow lines and `\nobreak` after headings,
//! `\raggedbottom`). This module keeps a record per box so every placed run
//! maps back to its document, bytes, glyph extents and math box.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::rc::Rc;

use flashtex_compiler::parser::SourceDocument;
use flashtex_compiler::{DocumentId, Span};
use flashtex_font_engine::sha256;
use flashtex_math_layout as ml;
use flashtex_paragraph_layout as pl;

use crate::adapter::{self, Block, Doc, Item as AItem, ParaPart, ParaStyle, TextStyle};
use crate::display::{
    self, Caret, Cluster, Diagnostic, DisplayList, DocumentResource, FontResource, Glyph, GlyphRun, Paint, Provenance,
    Rect, Rule, SourceRange, Tick,
};
use crate::fonts::{Family, FontSet, LoadedFace, Role};
use crate::incremental::{self, CachedBlock, RenderCache};
use crate::mathfont::{MathFonts, MathSizes};
use crate::mathtex::TexMathMetrics;
use crate::pagebuild::{self, VBlock};
use crate::params;
use crate::shape::Shaper;
use crate::style::Stylesheet;

/// paragraph-layout identity of a math box (never a real font hash: real
/// ids are SHA-256 digests, this is a labelled sentinel).
const MATH_SENTINEL: pl::FontId = pl::FontId::from_label("flashtex:math-box");

#[derive(Debug, Clone)]
pub struct GlyphRec {
    pub gid: u16,
    /// TFM italic correction in fixwords (0 without a TFM).
    pub italic_fix: i32,
    pub x_offset_units: i32,
    pub y_offset_units: i32,
    pub y_max_units: i32,
    pub y_min_units: i32,
    pub empty: bool,
}

#[derive(Debug, Clone)]
pub struct ClusterRec {
    /// Byte range into the run text.
    pub text_range: Range<usize>,
    pub span: Span,
    /// Glyph indices (into the run) belonging to this cluster.
    pub glyphs: Range<usize>,
}

#[derive(Clone)]
pub enum BoxRec {
    Text {
        face: Rc<LoadedFace>,
        size: f64,
        text: String,
        style: TextStyle,
        clusters: Vec<ClusterRec>,
        glyphs: Vec<GlyphRec>,
        /// Box height/depth in points (max glyph extents).
        height: f64,
        depth: f64,
    },
    Math(usize),
    /// `\hrule`: a filled rectangle `width` x `height` sitting on the line's
    /// baseline (depth 0), painted as a display-list rule.
    Rule { width: f64, height: f64, span: Span },
}

#[derive(Clone)]
pub struct MathRec {
    pub root: ml::MathBox,
    pub span: Span,
    pub face: Rc<LoadedFace>,
    /// The metrics the box was laid out with; maps placed glyphs to the
    /// face's glyph ids.
    pub metrics: MathProvider,
    /// `\text{...}` runs of this formula (`mathtext`), addressed by the
    /// placed glyphs' `font_id` above `RUN_FONT_BASE`.
    pub text_runs: Vec<crate::mathtext::TextRun>,
}

impl MathRec {
    /// The `\text` run glyph a placed glyph stands for, if it is one.
    pub fn run_glyph(&self, g: &ml::PositionedGlyph) -> Option<&crate::mathtext::RunGlyph> {
        crate::mathtext::run_of(&self.text_runs, g.font_id)?.glyph_at(g.font_id, g.gid)
    }

    /// The face and original glyph id that draw a placed glyph: a `\text`
    /// run's own shaped glyph (the text face's cmap id, 0 for its interword
    /// space) or the math provider's mapping.
    pub fn otf_glyph(&self, g: &ml::PositionedGlyph) -> Option<(Rc<LoadedFace>, u16)> {
        // `OTF_FALLBACK_FONT` is `u32::MAX`, above the `\text` run ids: it
        // must be answered by the provider, not looked up as a run.
        if g.font_id != crate::mathtex::OTF_FALLBACK_FONT && g.font_id.0 >= crate::mathtext::RUN_FONT_BASE {
            let run = crate::mathtext::run_of(&self.text_runs, g.font_id)?;
            let glyph = run.glyph_at(g.font_id, g.gid)?;
            return Some((run.face.clone(), glyph.gid.0));
        }
        self.metrics.otf_glyph(g)
    }

    /// The TFM box a placed cmex glyph was laid out with; see
    /// [`TexMathMetrics::extension_box`].
    pub fn extension_box(&self, g: &ml::PositionedGlyph) -> Option<(f64, f64)> {
        if g.font_id.0 >= crate::mathtext::RUN_FONT_BASE {
            // Includes `OTF_FALLBACK_FONT`: no TFM box for those glyphs.
            return None;
        }
        match &self.metrics {
            MathProvider::Tex(t) => t.extension_box(g.font_id, g.gid as u8, g.size),
            MathProvider::Otf(_) => None,
        }
    }
}

/// Which metrics lay math out: TeX's TFMs (pdfLaTeX's geometry) when the
/// `lm` TFMs are installed, else the OpenType `MATH` table.
#[derive(Clone)]
pub enum MathProvider {
    Tex(Rc<TexMathMetrics>),
    Otf(Rc<MathFonts>),
}

impl MathProvider {
    fn metrics(&self) -> &dyn ml::MathFontMetrics {
        match self {
            MathProvider::Tex(t) => &**t,
            MathProvider::Otf(o) => &**o,
        }
    }
    fn otf(&self) -> &Rc<MathFonts> {
        match self {
            MathProvider::Tex(t) => t.otf_fonts(),
            MathProvider::Otf(o) => o,
        }
    }
    /// The face and original glyph id that draw a placed glyph.
    pub fn otf_glyph(&self, g: &ml::PositionedGlyph) -> Option<(Rc<LoadedFace>, u16)> {
        match self {
            MathProvider::Tex(_) if g.font_id == crate::mathtex::OTF_FALLBACK_FONT => Some((self.otf().face().clone(), g.gid)),
            MathProvider::Tex(t) => t.otf_glyph(g.font_id, g.gid as u8, g.ch),
            MathProvider::Otf(o) => Some((o.face().clone(), g.gid)),
        }
    }
}

/// One vertical-list block: its broken lines, the horizontal list they
/// index into, the map from item indices to box records, and how it enters
/// the page builder's vertical list.
#[derive(Clone)]
pub struct BuiltBlock {
    pub block: pl::ParagraphBlock,
    /// The horizontal list the block's lines index into.
    pub items: Vec<pl::Item>,
    pub recs: Vec<Option<usize>>,
    /// Penalties and skips around and inside the block (lines filled).
    pub vertical: VBlock,
    /// `\label` keys and the item index they precede.
    pub labels: Vec<(String, usize)>,
    /// `(cache key, first source byte)` when the block came through the
    /// cache, so its assembled items can be cached too.
    pub cache_key: Option<(u64, DocumentId, usize)>,
}

/// LaTeX/plain penalties (article defaults).
const CLUB_PENALTY: i32 = 150;
const WIDOW_PENALTY: i32 = 150;
const SEC_PENALTY: i32 = -300;
const PREDISPLAY_PENALTY: i32 = pagebuild::INF_PENALTY;

/// Sets `run`'s glyphs at `x` on a line (what `layout_paragraph` does for
/// broken lines; used for the single-line display block).
fn position_run(run: &pl::GlyphRun, x: f64, baseline_y: f64) -> pl::PositionedRun {
    let mut off = 0.0;
    let glyphs = run
        .glyphs
        .iter()
        .map(|g| {
            let pg = pl::PositionedGlyph {
                gid: g.gid,
                x_offset: off,
                advance: g.advance + g.kern,
                cluster: g.cluster.clone(),
            };
            off += g.advance + g.kern;
            pg
        })
        .collect();
    pl::PositionedRun {
        x,
        baseline_y,
        width: run.width,
        font: run.font,
        size: run.size,
        glyphs,
        source: run.source.clone(),
        is_hyphen: false,
    }
}

pub struct Laid {
    pub blocks: Vec<BuiltBlock>,
    pub pages: pl::Pages,
    pub recs: Vec<BoxRec>,
    pub maths: Vec<MathRec>,
}

pub struct Context<'a> {
    fonts: &'a FontSet,
    style: &'a Stylesheet,
    paths: &'a [&'a str],
    /// Document sources (indexed like `paths`), read only to re-derive what
    /// the compiler's math list flattens (`\left`/`\right` fences).
    texts: &'a [&'a str],
    shaper: &'a Shaper,
    diagnostics: Vec<Diagnostic>,
    recs: Vec<BoxRec>,
    maths: Vec<MathRec>,
    math_fonts: Option<MathProvider>,
    math_unavailable: bool,
    reported: BTreeSet<String>,
    /// Diagnostics emitted while a cacheable block is being built (with
    /// their once-only keys, suppressed ones included).
    capture: Option<Vec<(Option<String>, Diagnostic)>>,
    path_rcs: std::cell::RefCell<BTreeMap<usize, Rc<str>>>,
}

impl<'a> Context<'a> {
    pub fn new(fonts: &'a FontSet, style: &'a Stylesheet, paths: &'a [&'a str]) -> Context<'a> {
        Self::with_texts(fonts, style, paths, &[])
    }

    /// [`Context::new`] with the document sources, which lets `\left`/`\right`
    /// fences be recovered from the bytes before each delimiter.
    pub fn with_texts(fonts: &'a FontSet, style: &'a Stylesheet, paths: &'a [&'a str], texts: &'a [&'a str]) -> Context<'a> {
        Context {
            fonts,
            style,
            paths,
            texts,
            shaper: fonts.shaper(),
            diagnostics: Vec::new(),
            recs: Vec::new(),
            maths: Vec::new(),
            math_fonts: None,
            math_unavailable: false,
            reported: BTreeSet::new(),
            capture: None,
            path_rcs: std::cell::RefCell::new(BTreeMap::new()),
        }
    }

    /// Emits a diagnostic; with a key, only the first one per key is kept.
    fn emit(&mut self, key: Option<String>, d: Diagnostic) {
        let keep = match &key {
            Some(k) => self.reported.insert(k.clone()),
            None => true,
        };
        if let Some(c) = &mut self.capture {
            c.push((key, d.clone()));
        }
        if keep {
            self.diagnostics.push(d);
        }
    }

    /// Builds a block through the cache: a hit clones the cached records
    /// back (offsets relocated to the block's new position) and replays
    /// its diagnostics; a miss builds and stores. `origin` is the block's
    /// document and first source byte.
    fn cached<F>(&mut self, cache: Option<&RenderCache>, key: Option<u64>, origin: Option<(DocumentId, usize)>, build: F) -> Option<BuiltBlock>
    where
        F: FnOnce(&mut Self) -> Option<BuiltBlock>,
    {
        let (Some(cache), Some(key), Some((document, base))) = (cache, key, origin) else {
            return build(self);
        };
        let path = self.paths.get(document.0).copied().unwrap_or("").to_string();
        if let Some(c) = cache.get(key) {
            if c.document == document && *c.path == *path {
                let _ = &c.block.cache_key;
                let rec_delta = self.recs.len() as isize - c.rec_base as isize;
                let math_delta = self.maths.len() as isize - c.math_base as isize;
                let mut block = c.block.clone();
                for r in &mut block.recs {
                    if let Some(i) = r {
                        *i = (*i as isize + rec_delta) as usize;
                    }
                }
                let mut recs = c.recs.clone();
                for r in &mut recs {
                    if let BoxRec::Math(mi) = r {
                        *mi = (*mi as isize + math_delta) as usize;
                    }
                }
                let mut maths = c.maths.clone();
                let mut diags = c.diagnostics.clone();
                let delta = base as isize - c.base as isize;
                incremental::relocate_block(&mut block, &mut recs, &mut maths, &mut diags, &path, delta);
                block.cache_key = Some((key, document, base));
                self.recs.extend(recs);
                self.maths.extend(maths);
                for (k, d) in diags {
                    self.emit(k, d);
                }
                return Some(block);
            }
        }
        let rec_base = self.recs.len();
        let math_base = self.maths.len();
        let outer = self.capture.replace(Vec::new());
        let mut built = build(self);
        let captured = self.capture.take().unwrap_or_default();
        self.capture = outer;
        if let Some(b) = &mut built {
            b.cache_key = Some((key, document, base));
        }
        if let Some(b) = &built {
            cache.insert(
                key,
                CachedBlock {
                    block: b.clone(),
                    rec_base,
                    math_base,
                    recs: self.recs[rec_base..].to_vec(),
                    maths: self.maths[math_base..].to_vec(),
                    diagnostics: captured,
                    document,
                    base,
                    path,
                },
            );
        }
        built
    }

    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    fn source(&self, span: Span) -> SourceRange {
        SourceRange {
            path: self.path_rc(span.document),
            start_byte: span.start,
            end_byte: span.end,
        }
    }

    /// The shared path string of a document (one allocation per document).
    fn path_rc(&self, document: DocumentId) -> Rc<str> {
        let mut cache = self.path_rcs.borrow_mut();
        if let Some(p) = cache.get(&document.0) {
            return p.clone();
        }
        let p: Rc<str> = Rc::from(self.paths.get(document.0).copied().unwrap_or(""));
        cache.insert(document.0, p.clone());
        p
    }

    fn report_once(&mut self, key: String, d: Diagnostic) {
        self.emit(Some(key), d);
    }

    fn face(&mut self, style: TextStyle, size: f64, span: Span) -> Rc<LoadedFace> {
        let r = self.fonts.resolve(
            self.style.family,
            Role::Text {
                bold: style.bold,
                italic: style.italic,
            },
            size,
        );
        if let Some(reason) = r.substituted {
            let src = self.source(span);
            self.report_once(
                format!("subst:{reason}"),
                Diagnostic::error(
                    "font_unavailable",
                    format!("Latin Modern face unavailable ({reason}); Times metrics substituted, output is not the requested document"),
                    vec![src],
                ),
            );
        }
        match &r.face.tfm_status {
            crate::fonts::TfmStatus::Loaded => {}
            crate::fonts::TfmStatus::RequiredUnavailable(reason) => {
                let src = self.source(span);
                self.report_once(
                    format!("tfm:{}", r.face.name),
                    Diagnostic::error(
                        "required_metrics_unavailable",
                        format!("{}: {reason}; the pinned Latin Modern 2.004 metrics are required for this size, OpenType advances were used and the layout is not the reference geometry", r.face.name),
                        vec![src],
                    ),
                );
            }
            crate::fonts::TfmStatus::Missing(_) => {
                if let Some(reason) = &r.face.tfm_missing {
                    let src = self.source(span);
                    self.report_once(
                        format!("tfm:{}", r.face.name),
                        Diagnostic::warning(
                            "tfm_missing",
                            format!("{}: {reason}; OpenType advances are used instead of TeX's metrics", r.face.name),
                            vec![src],
                        ),
                    );
                }
            }
        }
        r.face
    }

    fn math_fonts(&mut self, span: Span) -> Option<MathProvider> {
        if let Some(m) = &self.math_fonts {
            return Some(m.clone());
        }
        if self.math_unavailable {
            return None;
        }
        let r = self.fonts.resolve(self.style.family, Role::Math, self.style.body_size_pt);
        let sizes = MathSizes {
            text: self.style.body_size_pt,
            script: self.style.script_size_pt,
            script_script: self.style.scriptscript_size_pt,
        };
        match (r.substituted, MathFonts::new(r.face, sizes)) {
            (None, Some(m)) => {
                let m = Rc::new(m);
                // pdfLaTeX's geometry needs the lm math TFMs' parameters
                // (metric-identical to CM, embedded in math-layout) and
                // `rm-lmr` for the roman family; without the TFM directory
                // the OpenType MATH table is used and reported.
                let base = match self.style.base {
                    flashtex_document_style::BaseSize::Pt10 => 10,
                    flashtex_document_style::BaseSize::Pt11 => 11,
                    flashtex_document_style::BaseSize::Pt12 => 12,
                };
                let tex = TexMathMetrics::new(base, m.clone(), self.fonts);
                let provider = if tex.roman_available() {
                    MathProvider::Tex(Rc::new(tex))
                } else {
                    let src = self.source(span);
                    let diag = match tex.roman_status() {
                        Some(crate::fonts::TfmStatus::RequiredUnavailable(e)) => Diagnostic::error(
                            "required_metrics_unavailable",
                            format!("math roman metrics: {e}; math is laid out with the OpenType MATH table and is not the reference geometry"),
                            vec![src],
                        ),
                        other => Diagnostic::warning(
                            "math_metrics_opentype",
                            format!(
                                "rm-lmr*.tfm unavailable ({}); math is laid out with the OpenType MATH table instead of TeX's metrics",
                                match other {
                                    Some(crate::fonts::TfmStatus::Missing(m)) => m.clone(),
                                    _ => "not found".into(),
                                }
                            ),
                            vec![src],
                        ),
                    };
                    self.report_once("math:no-tfm".into(), diag);
                    MathProvider::Otf(m)
                };
                self.math_fonts = Some(provider.clone());
                Some(provider)
            }
            (subst, _) => {
                let src = self.source(span);
                let reason = subst.unwrap_or_else(|| "face has no MATH table".into());
                self.emit(None, Diagnostic::error(
                    "math_font_unavailable",
                    format!("Latin Modern Math unavailable ({reason}); math is not typeset"),
                    vec![src],
                ));
                self.math_unavailable = true;
                None
            }
        }
    }

    /// `\fontdimen`s of the face for `style` at `size`: the face's TFM
    /// when it has one (exact fixwords), else the transcribed table.
    fn text_params(&self, style: TextStyle, size: f64) -> params::TextParamsPt {
        let r = self.fonts.resolve(
            self.style.family,
            Role::Text {
                bold: style.bold,
                italic: style.italic,
            },
            size,
        );
        if let (None, Some(tfm)) = (&r.substituted, &r.face.tfm) {
            let dim = |n: usize| tfm.param(n).map_or(0.0, |v| crate::tfm::Tfm::pt(v, size));
            return params::TextParamsPt {
                space: dim(2),
                stretch: dim(3),
                shrink: dim(4),
                x_height: dim(5),
                quad: dim(6),
                extra_space: dim(7),
            };
        }
        let design = design_size(self.style.family, size);
        params::text_params(self.style.family, style.bold, style.italic, design).at(size)
    }

    /// Interword glue for the face/style at `size` with TeX's space factor.
    fn space_glue(&self, style: TextStyle, size: f64, factor: u32) -> pl::Glue {
        let p = self.text_params(style, size);
        let f = f64::from(factor.max(1));
        let mut width = p.space;
        if factor >= 2000 {
            width += p.extra_space;
        }
        pl::Glue::finite(width, p.stretch * f / 1000.0, p.shrink * 1000.0 / f)
    }

    /// Shapes one styled segment into a box record and a paragraph-layout box.
    fn text_box(&mut self, seg: &adapter::Segment, size: f64) -> Option<(pl::GlyphRun, usize)> {
        let span = seg_span(seg)?;
        let face = self.face(seg.style, size, span);
        let shaped = self.shaper.shape(&face, &seg.text);
        if let Some(e) = &shaped.tfm_error {
            let src = self.source(span);
            self.report_once(
                format!("tfmrun:{}:{e}", face.name),
                Diagnostic::warning(
                    "tfm_run_error",
                    format!("{}: TFM ligature/kern program failed for {:?} ({e}); OpenType metrics used for this word", face.name, seg.text),
                    vec![src],
                ),
            );
        }
        if let Some(reason) = &shaped.refused {
            let src = self.source(span);
            self.emit(None, Diagnostic::error("unsupported_script", format!("cannot shape {:?}: {reason}", seg.text), vec![src]));
            return None;
        }
        for (ch, off) in &shaped.missing {
            let ch_src = seg
                .chars
                .get(seg.text[..*off].chars().count())
                .map(|c| c.span())
                .unwrap_or(span);
            let src = self.source(ch_src);
            self.report_once(
                format!("missing:{}:{}", face.font_id, ch),
                Diagnostic::warning(
                    "missing_glyph",
                    format!("U+{:04X} '{}' has no glyph in {}; nothing drawn for it", *ch as u32, ch, face.name),
                    vec![src],
                ),
            );
        }
        let mut glyphs = Vec::new();
        let mut recs = Vec::new();
        let mut clusters = Vec::new();
        let byte_to_char: Vec<usize> = {
            let mut v = vec![0usize; seg.text.len() + 1];
            for (ci, (bi, _)) in seg.text.char_indices().enumerate() {
                v[bi] = ci;
            }
            v[seg.text.len()] = seg.text.chars().count();
            v
        };
        for c in &shaped.clusters {
            let first = seg.chars.get(byte_to_char[c.text_range.start]).copied()?;
            let last_char_index = byte_to_char[c.text_range.end].saturating_sub(1);
            let last = seg.chars.get(last_char_index).copied().unwrap_or(first);
            let cspan = Span::in_document(first.document, first.start.min(last.start), first.end.max(last.end));
            let g0 = glyphs.len();
            for g in &c.glyphs {
                glyphs.push(pl::ShapedGlyph {
                    gid: u32::from(g.gid.0),
                    advance_units: i64::from(g.advance),
                    cluster: cspan.start..cspan.end,
                });
                recs.push(GlyphRec {
                    gid: g.gid.0,
                    italic_fix: g.italic,
                    x_offset_units: g.x_offset,
                    y_offset_units: g.y_offset,
                    y_max_units: g.y_max,
                    y_min_units: g.y_min,
                    empty: g.empty,
                });
            }
            clusters.push(ClusterRec {
                text_range: c.text_range.clone(),
                span: cspan,
                glyphs: g0..glyphs.len(),
            });
        }
        if glyphs.is_empty() {
            return None;
        }
        let run = pl::GlyphRun::from_shaped(
            face.layout_id(),
            size,
            shaped.units_per_em as f64,
            f64::from(shaped.height_units),
            -f64::from(shaped.depth_units),
            &glyphs,
            span.start..span.end,
        );
        let height = run.height;
        let depth = run.depth;
        self.recs.push(BoxRec::Text {
            face,
            size,
            text: seg.text.clone(),
            style: seg.style,
            clusters,
            glyphs: recs,
            height,
            depth,
        });
        Some((run, self.recs.len() - 1))
    }

    fn math_box(&mut self, list: &flashtex_compiler::math::MathList, span: Span, display: bool) -> Option<usize> {
        let fonts = self.math_fonts(span)?;
        let mut sink = crate::mathtext::TextSink::default();
        let texts = self.texts;
        let fence = |sp: &Span| fence_of(texts.get(sp.document.0).copied().unwrap_or(""), sp.start);
        // `\quad`/`\qquad`/`\,`/`\:`/`\;`/`\!` (compiler `Space { em }`) at
        // the top level of the formula (outside `\left...\right`): math-layout
        // has no kern atom, so the formula is split there into runs laid out
        // separately and joined by kerns of the requested width plus the
        // inter-atom spacing TeX still inserts across glue (glue does not
        // reset `r_type`, §760). Glue inside a fence pair or a sub-formula
        // cannot be split out and stays reported.
        let segments = split_at_spaces(list, &fence);
        let ml_lists: Vec<ml::MathList> = segments.iter().map(|(atoms, _)| convert_math_fenced(&flashtex_compiler::math::MathList { atoms: atoms.clone() }, &mut sink, &fence)).collect();
        let mut grids = Vec::new();
        math_grids(list, &mut grids);
        for (rows, cols) in grids {
            if rows > 1 {
                let src = self.source(span);
                let msg = format!("{rows}x{cols} array/cases/matrix set as a single row inside its fences: math-layout has no array atom");
                self.report_once(format!("mathlim:{msg}"), Diagnostic::warning("math_limitation", msg, vec![src]));
            }
        }
        let nested_glue_em: f64 = segments.iter().map(|(atoms, _)| math_glue_em(&flashtex_compiler::math::MathList { atoms: atoms.clone() })).sum();
        if nested_glue_em.abs() > 0.0 {
            let src = self.source(span);
            let msg = format!("\\quad/\\qquad glue ({nested_glue_em} em in this formula) inside \\left...\\right or a sub-formula dropped: math-layout has no kern atom and only top-level glue can be split into separate runs");
            self.report_once(format!("mathlim:{msg}"), Diagnostic::warning("math_limitation", msg, vec![src]));
        }
        let mut approximations = Vec::new();
        math_approximations(list, &mut approximations);
        for msg in approximations {
            let src = self.source(span);
            self.report_once(format!("mathlim:{msg}"), Diagnostic::warning("math_limitation", msg, vec![src]));
        }
        let style = if display { ml::Style::DISPLAY } else { ml::Style::TEXT };
        let text_metrics = crate::mathtext::TextRunMetrics::new(fonts.metrics(), self.fonts, self.shaper, self.style.family, &sink.texts);
        let mut laid = if ml_lists.len() == 1 {
            ml::layout_with_report(&ml_lists[0], style, &text_metrics)
        } else {
            // Rules 5/6 (Bin -> Ord) over the whole formula, so the classes
            // at each split are the ones TeX would space by.
            let all_atoms: Vec<ml::Atom> = ml_lists.iter().flat_map(|l| l.atoms.iter().cloned()).collect();
            let classes = ml::layout::effective_classes(&all_atoms);
            let params = ml::MathFontMetrics::params(&text_metrics, style.size_class());
            let (quad, mu) = (params.quad, params.mu());
            let mut boxes = Vec::new();
            let mut limitations = Vec::new();
            let mut at = 0usize;
            for (i, l) in ml_lists.iter().enumerate() {
                let part = ml::layout_with_report(l, style, &text_metrics);
                limitations.extend(part.limitations);
                boxes.push((0.0, part.root));
                at += l.atoms.len();
                if let Some(em) = segments[i].1 {
                    let spacing = match (at.checked_sub(1).and_then(|j| classes.get(j)), classes.get(at)) {
                        (Some(&left), Some(&right)) => ml::between(left, right, style).mu() * mu,
                        _ => 0.0,
                    };
                    boxes.push((0.0, ml::MathBox::kern(em * quad + spacing)));
                }
            }
            ml::Layout {
                root: ml::MathBox::hbox(boxes),
                limitations,
            }
        };
        let (text_runs, notices) = text_metrics.finish();
        crate::mathtext::substitute(&mut laid.root, &text_runs);
        for text in &sink.refused {
            let src = self.source(span);
            self.emit(
                None,
                Diagnostic::error(
                    "math_text_overflow",
                    format!(
                        "more than {} \\text arguments in one formula; {:?} is not typeset",
                        crate::mathtext::MAX_TEXT_ATOMS,
                        crate::mathtext::abbreviate(text)
                    ),
                    vec![src],
                ),
            );
        }
        for n in notices {
            use crate::mathtext::Notice;
            match n {
                // The same availability/TFM diagnostics the paragraph path
                // reports for this face (once per face).
                Notice::FaceUsed { size } => {
                    let _ = self.face(TextStyle::default(), size, span);
                }
                Notice::Refused { word, reason } => {
                    let src = self.source(span);
                    self.emit(None, Diagnostic::error("unsupported_script", format!("cannot shape {word:?} in \\text: {reason}"), vec![src]));
                }
                Notice::MissingGlyph { ch, face } => {
                    let src = self.source(span);
                    self.report_once(
                        format!("missing:{face}:{ch}"),
                        Diagnostic::warning("missing_glyph", format!("U+{:04X} '{}' has no glyph in {}; nothing drawn for it", ch as u32, ch, face), vec![src]),
                    );
                }
                Notice::TooLarge { text_chars, glyphs } => {
                    let src = self.source(span);
                    self.emit(
                        None,
                        Diagnostic::error(
                            "math_text_overflow",
                            format!("a \\text argument of {text_chars} characters ({glyphs} entries) exceeds what one formula can address; it is not typeset"),
                            vec![src],
                        ),
                    );
                }
                Notice::TfmRunError { word, face, error } => {
                    let src = self.source(span);
                    self.report_once(
                        format!("tfmrun:{face}:{error}"),
                        Diagnostic::warning("tfm_run_error", format!("{face}: TFM ligature/kern program failed for {word:?} ({error}); OpenType metrics used for this word"), vec![src]),
                    );
                }
            }
        }
        for ch in fonts.otf().take_missing() {
            let src = self.source(span);
            self.report_once(
                format!("mathmissing:{ch}"),
                Diagnostic::warning("missing_glyph", format!("U+{:04X} '{}' has no glyph in {}", ch as u32, ch, fonts.otf().face().name), vec![src]),
            );
        }
        for l in laid.limitations {
            let src = self.source(span);
            let msg = match l {
                // A refused `\text` run's placeholder: already reported as
                // math_text_overflow above.
                ml::Limitation::MissingGlyph(c) if crate::mathtext::is_handle(c) => continue,
                ml::Limitation::MissingGlyph(c) => format!("no math glyph for '{c}'; empty box used"),
                ml::Limitation::DelimiterTooSmall { ch, wanted, used } => {
                    format!("delimiter '{ch}' wanted {wanted:.2}pt, largest variant {used:.2}pt used")
                }
                ml::Limitation::RadicalTooSmall { wanted, used } => format!("radical wanted {wanted:.2}pt, largest {used:.2}pt used"),
                ml::Limitation::MissingAccent(c) => format!("unknown accent '{c}'"),
            };
            self.report_once(format!("mathlim:{msg}"), Diagnostic::warning("math_limitation", msg, vec![src]));
        }
        self.maths.push(MathRec {
            root: laid.root,
            span,
            face: fonts.otf().face().clone(),
            metrics: fonts.clone(),
            text_runs,
        });
        let idx = self.maths.len() - 1;
        self.recs.push(BoxRec::Math(idx));
        Some(self.recs.len() - 1)
    }

    /// Builds a horizontal list. Returns paragraph-layout items, the
    /// per-item box record and the `\label` keys with the item they precede.
    fn hlist(&mut self, items: &[AItem], size: f64, base: TextStyle, style: ParaStyle) -> (Vec<pl::Item>, Vec<Option<usize>>, Vec<(String, usize)>) {
        // `\centering`/`\raggedleft` set `\parfillskip 0pt` and make `\\`
        // end the paragraph (`\@centercr`); the fil glue of the skips
        // fills the line. Elsewhere `\\` is `\hfil\break` and the paragraph
        // ends with `\parfillskip 0pt plus 1fil`.
        let fills = !matches!(style, ParaStyle::Center | ParaStyle::FlushRight);
        let mut out: Vec<pl::Item> = Vec::new();
        let mut recs: Vec<Option<usize>> = Vec::new();
        let mut labels: Vec<(String, usize)> = Vec::new();
        let push = |out: &mut Vec<pl::Item>, recs: &mut Vec<Option<usize>>, item: pl::Item, rec: Option<usize>| {
            out.push(item);
            recs.push(rec);
        };
        for item in items {
            match item {
                AItem::Word(w) => {
                    for seg in &w.segments {
                        let seg = adapter::Segment {
                            text: seg.text.clone(),
                            chars: seg.chars.clone(),
                            style: TextStyle {
                                bold: seg.style.bold || base.bold,
                                italic: seg.style.italic || base.italic,
                                size_cpt: seg.style.size_cpt,
                            },
                        };
                        // A size declaration in force (`{\Large ...}`) sets
                        // this segment at its own size.
                        let seg_size = seg.style.size_or(size);
                        if let Some((run, rec)) = self.text_box(&seg, seg_size) {
                            push(&mut out, &mut recs, pl::Item::Box(run), Some(rec));
                        }
                    }
                }
                AItem::Space { style, factor, no_break } => {
                    let style = TextStyle {
                        bold: style.bold || base.bold,
                        italic: style.italic || base.italic,
                        size_cpt: style.size_cpt,
                    };
                    if *no_break {
                        push(&mut out, &mut recs, pl::Item::penalty(pl::INFINITE_PENALTY), None);
                    }
                    let glue = self.space_glue(style, style.size_or(size), *factor);
                    push(&mut out, &mut recs, pl::Item::Glue(glue), None);
                }
                AItem::Math { list, span } => {
                    if let Some(rec) = self.math_box(list, *span, false) {
                        let BoxRec::Math(mi) = &self.recs[rec] else { unreachable!() };
                        let root = &self.maths[*mi].root;
                        let run = math_run(root, size, *span);
                        push(&mut out, &mut recs, pl::Item::Box(run), Some(rec));
                    }
                }
                AItem::LineBreak => {
                    if fills {
                        push(&mut out, &mut recs, pl::Item::Glue(pl::Glue::fil()), None);
                    }
                    push(&mut out, &mut recs, pl::Item::penalty(pl::FORCED_BREAK), None);
                }
                AItem::Quad { em } => {
                    let quad = self.text_params(base, size).quad;
                    push(&mut out, &mut recs, pl::Item::Glue(pl::Glue::fixed(em * quad)), None);
                }
                AItem::HFill => push(&mut out, &mut recs, pl::Item::Glue(pl::Glue::fil()), None),
                AItem::HSpace { pt } => push(&mut out, &mut recs, pl::Item::Glue(pl::Glue::fixed(*pt)), None),
                AItem::Label { key } => labels.push((key.clone(), out.len())),
                AItem::ItalicCorrection => {
                    // `\/`: a kern of the last character's TFM italic
                    // correction (§1113); nothing when the last node is not
                    // a character or the metrics carry no correction.
                    let last = recs.iter().rev().find_map(|r| *r).and_then(|r| match &self.recs[r] {
                        BoxRec::Text { glyphs, size, .. } if matches!(out.last(), Some(pl::Item::Box(_))) => {
                            glyphs.last().map(|g| crate::tfm::Tfm::pt(g.italic_fix, *size))
                        }
                        _ => None,
                    });
                    if let Some(ic) = last {
                        if ic > 0.0 {
                            push(&mut out, &mut recs, pl::Item::Glue(pl::Glue::fixed(ic)), None);
                        }
                    }
                }
            }
        }
        // TeX's paragraph end: drop trailing glue, then
        // \penalty10000 \parfillskip \penalty-10000.
        while matches!(out.last(), Some(pl::Item::Glue(_))) {
            out.pop();
            recs.pop();
        }
        push(&mut out, &mut recs, pl::Item::penalty(pl::INFINITE_PENALTY), None);
        push(&mut out, &mut recs, pl::Item::Glue(if fills { pl::Glue::fil() } else { pl::Glue::fixed(0.0) }), None);
        push(&mut out, &mut recs, pl::Item::penalty(pl::FORCED_BREAK), None);
        (out, recs, labels)
    }

    fn line_params(&self, indent: bool, baselineskip: f64, style: ParaStyle) -> pl::LineBreakParams {
        let s = self.style;
        // `\centering`: `\leftskip`/`\rightskip` `0pt plus 1fil`; `\raggedleft`:
        // `\leftskip` alone; `\raggedright`: `\rightskip` (the crate's ragged
        // mode); `quote`: `\list` with `\leftmargin=\rightmargin=\leftmargini`
        // (`\parshape` in LaTeX; the same lines as fixed skips here).
        let fil = pl::Glue::fil();
        let margin = pl::Glue::fixed(s.leftmargini_pt);
        let (mode, left_skip, right_skip) = match style {
            ParaStyle::Plain => (pl::BreakMode::Justified, pl::Glue::fixed(0.0), pl::Glue::fixed(0.0)),
            ParaStyle::Center => (pl::BreakMode::Justified, fil.clone(), fil),
            ParaStyle::FlushRight => (pl::BreakMode::Justified, fil, pl::Glue::fixed(0.0)),
            ParaStyle::FlushLeft => (pl::BreakMode::RaggedRight, pl::Glue::fixed(0.0), pl::Glue::fixed(0.0)),
            ParaStyle::Quote => (pl::BreakMode::Justified, margin.clone(), margin),
        };
        pl::LineBreakParams {
            line_width: s.text_width_pt,
            mode,
            algorithm: pl::Algorithm::TotalFit,
            pretolerance: s.pretolerance,
            tolerance: s.tolerance,
            emergency_stretch: 0.0,
            line_penalty: s.linepenalty,
            adj_demerits: s.adjdemerits,
            double_hyphen_demerits: 10_000.0,
            final_hyphen_demerits: 5_000.0,
            parindent: if indent { s.parindent_pt } else { 0.0 },
            left_skip,
            right_skip,
            baselineskip,
            lineskip: s.lineskip_pt,
            lineskiplimit: s.lineskiplimit_pt,
            hfuzz: 0.1,
            hbadness: 1000.0,
        }
    }

    /// A body paragraph (or the part of one before/after a display).
    /// `starts_paragraph` adds `\parskip`; `after_heading` is LaTeX's
    /// `\@afterheading` (`\clubpenalty 10000`).
    fn paragraph_block(&mut self, items: &[AItem], indent: bool, starts_paragraph: bool, after_heading: bool, style: ParaStyle) -> Option<BuiltBlock> {
        let size = self.style.body_size_pt;
        let (mut list, mut recs, labels) = self.hlist(items, size, TextStyle::default(), style);
        if !list.iter().any(|i| matches!(i, pl::Item::Box(_))) {
            return None;
        }
        self.drop_trailing_break(items, &mut list, &mut recs);
        let lines = pl::layout_paragraph(&list, &self.line_params(indent, self.style.baselineskip_pt, style));
        self.report_overfull(&lines, &list, &recs);
        let vertical = VBlock {
            lines: line_extents(&lines),
            penalty_before: None,
            space_before: None,
            parskip: starts_paragraph.then(|| skip_tuple(self.style.parskip)),
            interline_penalty: 0,
            club_penalty: if after_heading { pagebuild::INF_PENALTY } else { CLUB_PENALTY },
            widow_penalty: WIDOW_PENALTY,
            penalty_after: None,
            space_after: None,
            no_interline_first: false,
            no_interline_after: false,
            baselineskip: None,
        };
        Some(BuiltBlock {
            block: pl::ParagraphBlock::body(lines),
            items: list,
            recs,
            vertical,
            labels,
            cache_key: None,
        })
    }

    /// A paragraph whose last item is `\\` (TeX: an empty final line,
    /// LaTeX's "Underfull \hbox" warning): the pinned paragraph-layout
    /// (`linebreak.rs:988`) panics on a forced break followed by the
    /// paragraph-end sequence, which would kill the worker mid-keystroke.
    /// Until the owner's fix lands (`docs/handoffs/paragraph-layout-forced-break/`
    /// on the mac-shell branch) the trailing break and the discardable glue
    /// before it are dropped before line breaking and reported as a typed
    /// warning: the paragraph then sets as TeX would minus the empty last
    /// line (one baseline pitch short). `list` and `recs` are the parallel
    /// outputs of [`Self::hlist`].
    fn drop_trailing_break(&mut self, items: &[AItem], list: &mut Vec<pl::Item>, recs: &mut Vec<Option<usize>>) -> bool {
        // `hlist` appends `\penalty10000 \parfillskip \penalty-10000`; the
        // item before that triple is the last one of the paragraph proper.
        let trailing_break = |list: &[pl::Item]| {
            let n = list.len();
            n >= 4 && matches!(&list[n - 4], pl::Item::Penalty(p) if p.value <= pl::FORCED_BREAK)
        };
        if !trailing_break(list) {
            return false;
        }
        while trailing_break(list) {
            let at = list.len() - 4;
            list.remove(at);
            recs.remove(at);
            // The `\hfil` glue `\\` carries plus any glue read before it
            // (discardable after a break, TeX §879); stop at the next `\\`
            // so the outer loop drops it the same way.
            loop {
                let last = list.len() - 3; // the paragraph-end triple starts here
                if last == 0 || !matches!(list[last - 1], pl::Item::Glue(_)) || trailing_break(list) {
                    break;
                }
                list.remove(last - 1);
                recs.remove(last - 1);
            }
        }
        // Source: the last word/formula before the break (`\\` carries no
        // span of its own in the adapter's items).
        let span = items.iter().rev().find_map(|i| match i {
            AItem::Word(w) => w.segments.iter().rev().find_map(seg_span),
            AItem::Math { span, .. } => Some(*span),
            _ => None,
        });
        let sources = span.map(|s| vec![self.source(s)]).unwrap_or_default();
        self.emit(
            None,
            Diagnostic::warning(
                "paragraph_final_linebreak",
                "final \\\\ ignored: paragraph-layout forced-break fix pending (LaTeX sets an empty last line here, Underfull \\hbox; this paragraph is one line pitch shorter)",
                sources,
            ),
        );
        true
    }

    fn heading_block(&mut self, level: u8, items: &[AItem]) -> Option<BuiltBlock> {
        let h = self.style.heading(level);
        let (list, recs, labels) = self.hlist(
            items,
            h.size_pt,
            TextStyle {
                bold: h.bold,
                italic: false,
                size_cpt: 0,
            },
            ParaStyle::Plain,
        );
        if !list.iter().any(|i| matches!(i, pl::Item::Box(_))) {
            return None;
        }
        let lines = pl::layout_paragraph(&list, &self.line_params(false, h.baselineskip_pt, ParaStyle::Plain));
        self.report_overfull(&lines, &list, &recs);
        // The heading's lines are appended under its own \baselineskip
        // (`\Large` is in force inside \@sect's group); the before/after
        // skips are body-font `ex`.
        let before = h.before;
        // \@startsection: \addpenalty\@secpenalty, \addvspace{before},
        // the title with \interlinepenalty\@M, \nobreak, \vskip{after}.
        let vertical = VBlock {
            lines: line_extents(&lines),
            penalty_before: Some(SEC_PENALTY),
            space_before: Some(skip_tuple(before)),
            parskip: Some(skip_tuple(self.style.parskip)),
            interline_penalty: pagebuild::INF_PENALTY,
            club_penalty: 0,
            widow_penalty: 0,
            penalty_after: Some(pagebuild::INF_PENALTY),
            space_after: Some(skip_tuple(h.after)),
            no_interline_first: false,
            no_interline_after: false,
            baselineskip: Some(h.baselineskip_pt),
        };
        Some(BuiltBlock {
            block: pl::ParagraphBlock {
                lines,
                space_before: before.glue(),
                space_after: h.after.glue(),
                keep_with_next: true,
            },
            items: list,
            recs,
            vertical,
            labels,
            cache_key: None,
        })
    }

    /// The empty line TeX sets when a display opens a paragraph: the
    /// `\parindent` box alone (`$$`, `equation`), or LaTeX's
    /// `\nointerlineskip\makebox[.6\linewidth]{}` for `\[`. It carries
    /// `\parskip`, has no height or depth, and decides the display's
    /// `pre_display_size` (its width plus 2em).
    fn display_opener_block(&mut self, bracket: bool) -> (BuiltBlock, f64) {
        let s = self.style;
        let width = s.parindent_pt + if bracket { 0.6 * s.text_width_pt } else { 0.0 };
        let line = pl::Line {
            index: 0,
            runs: Vec::new(),
            baseline_y: 0.0,
            height: 0.0,
            depth: 0.0,
            natural_width: width,
            set_width: s.text_width_pt,
            ratio: 0.0,
            badness: 0.0,
            items: 0..0,
            hyphenated: false,
        };
        let lines = pl::Lines {
            lines: vec![line],
            breaks: Vec::new(),
            stats: pl::Stats {
                algorithm: pl::Algorithm::TotalFit,
                lines: 1,
                pass: 1,
                total_demerits: 0.0,
                overfull: Vec::new(),
                underfull: Vec::new(),
                hyphenated_lines: 0,
                emergency_pass_used: false,
            },
            diagnostics: Vec::new(),
            height: 0.0,
        };
        let quad = self.text_params(TextStyle::default(), s.body_size_pt).quad;
        let vertical = VBlock {
            lines: vec![(0.0, 0.0)],
            penalty_before: None,
            space_before: None,
            parskip: Some(skip_tuple(s.parskip)),
            interline_penalty: 0,
            club_penalty: 0,
            widow_penalty: 0,
            penalty_after: None,
            space_after: None,
            no_interline_first: bracket,
            no_interline_after: false,
            baselineskip: None,
        };
        (
            BuiltBlock {
                block: pl::ParagraphBlock::body(lines),
                items: Vec::new(),
                recs: Vec::new(),
                vertical,
                labels: Vec::new(),
                cache_key: None,
            },
            width + 2.0 * quad,
        )
    }

    /// `\hrule` in vertical mode (TeX §1056): a rule node of the full
    /// measure, `0.4pt` high and `0pt` deep, appended with no interline glue
    /// before it and `prev_depth` left at `ignore_depth` after it. Set as a
    /// one-line block whose single box is a [`BoxRec::Rule`].
    fn rule_block(&mut self, span: Span) -> BuiltBlock {
        const HRULE_HEIGHT: f64 = 0.4;
        let width = self.style.text_width_pt;
        self.recs.push(BoxRec::Rule {
            width,
            height: HRULE_HEIGHT,
            span,
        });
        let rec = self.recs.len() - 1;
        let run = pl::GlyphRun {
            font: MATH_SENTINEL,
            size: self.style.body_size_pt,
            glyphs: Vec::new(),
            width,
            height: HRULE_HEIGHT,
            depth: 0.0,
            source: span.start..span.end,
        };
        let line = pl::Line {
            index: 0,
            runs: vec![position_run(&run, 0.0, HRULE_HEIGHT)],
            baseline_y: HRULE_HEIGHT,
            height: HRULE_HEIGHT,
            depth: 0.0,
            natural_width: width,
            set_width: width,
            ratio: 0.0,
            badness: 0.0,
            items: 0..1,
            hyphenated: false,
        };
        let lines = pl::Lines {
            lines: vec![line],
            breaks: Vec::new(),
            stats: pl::Stats {
                algorithm: pl::Algorithm::TotalFit,
                lines: 1,
                pass: 1,
                total_demerits: 0.0,
                overfull: Vec::new(),
                underfull: Vec::new(),
                hyphenated_lines: 0,
                emergency_pass_used: false,
            },
            diagnostics: Vec::new(),
            height: HRULE_HEIGHT,
        };
        let vertical = VBlock {
            lines: vec![(HRULE_HEIGHT, 0.0)],
            penalty_before: None,
            space_before: None,
            parskip: None,
            interline_penalty: 0,
            club_penalty: 0,
            widow_penalty: 0,
            penalty_after: None,
            space_after: None,
            no_interline_first: true,
            no_interline_after: true,
            baselineskip: None,
        };
        BuiltBlock {
            block: pl::ParagraphBlock::body(lines),
            items: vec![pl::Item::Box(run)],
            recs: vec![Some(rec)],
            vertical,
            labels: Vec::new(),
            cache_key: None,
        }
    }

    /// A display equation. `pre_display_size` is TeX's measure of the line
    /// before it (its material width plus 2em, or `None` when the display
    /// starts the paragraph); `number` is the `equation` counter set flush
    /// right (`\eqno`).
    fn display_block(
        &mut self,
        list: &flashtex_compiler::math::MathList,
        span: Span,
        pre_display_size: Option<f64>,
        number: Option<&(String, Span)>,
    ) -> Option<BuiltBlock> {
        let rec = self.math_box(list, span, true)?;
        let BoxRec::Math(mi) = &self.recs[rec] else { unreachable!() };
        let root = &self.maths[*mi].root;
        let size = self.style.body_size_pt;
        let run = math_run(root, size, span);
        let width = run.width;
        let (mut height, mut depth) = (run.height, run.depth);
        let z = self.style.text_width_pt;
        // \eqno: the number's box (§1202) reduces the room for the formula.
        let mut eqno: Option<(pl::GlyphRun, usize)> = None;
        let mut e = 0.0;
        let mut q = 0.0;
        if let Some((text, nspan)) = number {
            let seg = adapter::Segment {
                text: format!("({text})"),
                chars: format!("({text})")
                    .chars()
                    .map(|_| adapter::CharSrc {
                        document: nspan.document,
                        start: nspan.start,
                        end: nspan.end,
                    })
                    .collect(),
                style: TextStyle::default(),
            };
            if let Some((nrun, nrec)) = self.text_box(&seg, size) {
                e = nrun.width;
                q = e + self.text_params(TextStyle::default(), size).quad;
                height = height.max(nrun.height);
                depth = depth.max(nrun.depth);
                eqno = Some((nrun, nrec));
            }
        }
        // §1199: centre the formula in the measure; if it would collide with
        // the number, shift it (d) so both fit; `l` marks a display wider
        // than the room left.
        let mut w = width;
        let l = w + q > z;
        if l {
            w = (z - q).max(0.0);
        }
        let mut d = (z - w) / 2.0;
        if e > 0.0 && d < 2.0 * e {
            d = (z - w - e) / 2.0;
            if d < 0.0 {
                d = 0.0;
            }
        }
        let x = d.max(0.0);
        let long = pre_display_size.is_some_and(|p| x <= p) || l;
        let (above, below) = if long {
            (self.style.abovedisplayskip, self.style.belowdisplayskip)
        } else {
            (self.style.abovedisplayshortskip, self.style.belowdisplayshortskip)
        };
        let mut runs = vec![pl::PositionedRun {
            x,
            baseline_y: height,
            width,
            font: run.font,
            size: run.size,
            glyphs: Vec::new(),
            source: run.source.clone(),
            is_hyphen: false,
        }];
        let mut items = vec![pl::Item::Box(run)];
        let mut recs = vec![Some(rec)];
        if let Some((nrun, nrec)) = eqno {
            runs.push(position_run(&nrun, z - e, height));
            items.push(pl::Item::Box(nrun));
            recs.push(Some(nrec));
        }
        let n = items.len();
        let line = pl::Line {
            index: 0,
            runs,
            baseline_y: height,
            height,
            depth,
            natural_width: width,
            set_width: z,
            ratio: 0.0,
            badness: 0.0,
            items: 0..n,
            hyphenated: false,
        };
        let lines = pl::Lines {
            lines: vec![line],
            breaks: Vec::new(),
            stats: pl::Stats {
                algorithm: pl::Algorithm::TotalFit,
                lines: 1,
                pass: 1,
                total_demerits: 0.0,
                overfull: Vec::new(),
                underfull: Vec::new(),
                hyphenated_lines: 0,
                emergency_pass_used: false,
            },
            diagnostics: Vec::new(),
            height: height + depth,
        };
        if width > z + 1e-6 {
            let src = self.source(span);
            self.emit(None, Diagnostic::warning(
                "overfull_display",
                format!("display is {:.2}pt wider than the text width", width - z),
                vec![src],
            ));
        }
        // $$: \penalty\predisplaypenalty, \abovedisplayskip, the display,
        // \penalty\postdisplaypenalty (0), \belowdisplayskip.
        let vertical = VBlock {
            lines: vec![(height, depth)],
            penalty_before: Some(PREDISPLAY_PENALTY),
            space_before: Some(skip_tuple(above)),
            parskip: None,
            interline_penalty: 0,
            club_penalty: 0,
            widow_penalty: 0,
            penalty_after: None,
            space_after: Some(skip_tuple(below)),
            no_interline_first: false,
            no_interline_after: false,
            baselineskip: None,
        };
        Some(BuiltBlock {
            block: pl::ParagraphBlock {
                lines,
                space_before: above.glue(),
                space_after: below.glue(),
                keep_with_next: false,
            },
            items,
            recs,
            vertical,
            labels: Vec::new(),
            cache_key: None,
        })
    }

    fn report_overfull(&mut self, lines: &pl::Lines, list: &[pl::Item], recs: &[Option<usize>]) {
        for o in &lines.stats.overfull {
            let line = &lines.lines[o.line];
            let span = line
                .items
                .clone()
                .filter_map(|i| recs.get(i).copied().flatten())
                .filter_map(|r| match &self.recs[r] {
                    BoxRec::Text { clusters, .. } => clusters.first().map(|c| c.span),
                    BoxRec::Math(m) => Some(self.maths[*m].span),
                    BoxRec::Rule { span, .. } => Some(*span),
                })
                .next();
            let _ = list;
            let src = span.map(|s| vec![self.source(s)]).unwrap_or_default();
            self.emit(None, Diagnostic::warning(
                "overfull_hbox",
                format!("overfull line: {:.2}pt too wide (no hyphenation available)", o.excess),
                src,
            ));
        }
    }
}

fn skip_tuple(s: crate::style::Skip) -> (f64, f64, f64) {
    (s.natural, s.stretch, s.shrink)
}

fn line_extents(lines: &pl::Lines) -> Vec<(f64, f64)> {
    lines.lines.iter().map(|l| (l.height, l.depth)).collect()
}

pub(crate) fn design_size(family: Family, size: f64) -> u32 {
    match family {
        Family::Times => 10,
        Family::LatinModern => {
            if size < 8.5 {
                8
            } else if size < 11.0 {
                10
            } else if size < 15.0 {
                12
            } else {
                17
            }
        }
    }
}

fn seg_span(seg: &adapter::Segment) -> Option<Span> {
    let first = seg.chars.first()?;
    let last = seg.chars.last()?;
    Some(Span::in_document(first.document, first.start.min(last.start), first.end.max(last.end)))
}

fn math_run(root: &ml::MathBox, size: f64, span: Span) -> pl::GlyphRun {
    pl::GlyphRun {
        font: MATH_SENTINEL,
        size,
        glyphs: Vec::new(),
        width: root.width,
        height: root.height,
        depth: root.depth,
        source: span.start..span.end,
    }
}

/// Compiler math list -> math-layout list. Symbols are single characters
/// with plain.tex's default classification; an unsupported `\command` the
/// compiler kept literally is spelled out as ordinary atoms. `\text`
/// arguments are dropped here (see [`convert_math_with`]).
pub fn convert_math(list: &flashtex_compiler::math::MathList) -> ml::MathList {
    convert_math_with(list, &mut crate::mathtext::TextSink::default())
}

/// [`convert_math`] collecting `\text{...}` arguments into `sink`, which
/// hands back the ordinary atom standing for each run (compiler pin
/// `887bf21` carries `Nucleus::Text` on main; the `compiler-text-nucleus`
/// feature is kept as a no-op for existing build invocations).
pub fn convert_math_with(list: &flashtex_compiler::math::MathList, sink: &mut crate::mathtext::TextSink) -> ml::MathList {
    convert_math_fenced(list, sink, &|_| None)
}

/// Which fence, if any, a delimiter atom was introduced by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fence {
    Left,
    Right,
}

/// The fence a delimiter atom whose span starts at `at` was introduced by:
/// the compiler pairs `\left`/`\right` but emits each delimiter as a plain
/// symbol, so the fence is re-derived from the source. Since pin `87df3e4a`
/// the delimiter's span starts at the control word itself (older pins
/// started it at the delimiter character, with the control word before).
pub fn fence_of(text: &str, at: usize) -> Option<Fence> {
    let rest = text.get(at..)?;
    for (word, fence) in [("\\left", Fence::Left), ("\\right", Fence::Right)] {
        if let Some(after) = rest.strip_prefix(word) {
            // `\leftarrow` is not a fence: the control word must end here.
            if !after.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
                return Some(fence);
            }
        }
    }
    fence_before(text, at)
}

/// Whether the bytes of `text` before offset `at` end in `\left` or
/// `\right` (spaces between the control word and the delimiter allowed).
pub fn fence_before(text: &str, at: usize) -> Option<Fence> {
    let before = text.get(..at)?.trim_end_matches([' ', '\t', '\n', '\r']);
    for (word, fence) in [("\\left", Fence::Left), ("\\right", Fence::Right)] {
        if let Some(stem) = before.strip_suffix(word) {
            // `\\left` itself (an escaped backslash) is not a control word.
            let escaped = stem.chars().rev().take_while(|c| *c == '\\').count() % 2 == 1;
            if !escaped {
                return Some(fence);
            }
        }
    }
    None
}

/// [`convert_math_with`] with `fence` telling which delimiter atoms follow a
/// `\left`/`\right`; matched pairs become math-layout `Delimited` atoms
/// (Appendix G Rule 19: sized to the body, `Inner` class). An unmatched
/// fence stays a plain symbol, as the compiler already reports it.
pub fn convert_math_fenced(list: &flashtex_compiler::math::MathList, sink: &mut crate::mathtext::TextSink, fence: &dyn Fn(&Span) -> Option<Fence>) -> ml::MathList {
    use flashtex_compiler::math::Nucleus as N;
    // Open fences: (left delimiter, atoms converted since it).
    let mut stack: Vec<(Option<char>, Vec<ml::Atom>)> = Vec::new();
    let mut atoms = Vec::new();
    for a in &list.atoms {
        let sub = |l: &flashtex_compiler::math::MathList, sink: &mut crate::mathtext::TextSink| convert_math_fenced(l, sink, fence);
        let mut out: Vec<ml::Atom> = match &a.nucleus {
            N::Text(text) => vec![sink.atom(text)],
            // `\quad`/`\qquad` (compiler `Space { em }`): TeX glue in the
            // math list. math-layout has no kern/glue atom, so the glue is
            // dropped (inter-atom spacing across it is what TeX's mlist_to_hlist
            // does too, since glue does not reset r_type) and reported once per
            // formula by `math_box` as a typed math_limitation.
            N::Space { .. } => continue,
            N::Symbol(s) => {
                let mut chars = s.chars();
                let single = match (chars.next(), chars.next()) {
                    (Some(c), None) => Some(Some(c)),
                    (None, _) => Some(None),
                    _ => None,
                };
                match (single, fence(&a.span)) {
                    (Some(delim), Some(Fence::Left)) => {
                        stack.push((delim, Vec::new()));
                        continue;
                    }
                    (Some(delim), Some(Fence::Right)) if !stack.is_empty() => {
                        let (left, body) = stack.pop().expect("checked non-empty");
                        vec![ml::Atom::left_right(left, delim, ml::MathList::new(body))]
                    }
                    _ => match single {
                        Some(Some(c)) => symbol_atoms(c),
                        Some(None) => vec![ml::Atom::new(ml::AtomClass::Ord, ml::Nucleus::Empty)],
                        None => {
                            // Multi-character symbol (e.g. a literal "\foo"): the
                            // characters as upright ordinary atoms.
                            vec![ml::Atom::group(ml::MathList::new(s.chars().map(ml::Atom::ord).collect()))]
                        }
                    },
                }
            }
            N::Fraction { numerator, denominator } => vec![ml::Atom::frac(sub(numerator, sink), sub(denominator, sink))],
            N::Radical(r) => vec![ml::Atom::sqrt(sub(r, sink))],
            // `\mathbf{...}`: set like `\text` in the roman face (the text
            // sink has no bold role); `math_box` reports it once per formula.
            N::Bold(text) => vec![sink.atom(text)],
            // `\overline`/`\underline` are Appendix G Rules 9/10 atoms;
            // `\boxed` has no frame atom, so the body is set as a group and
            // reported by `math_box`.
            N::Framed { body, frame } => {
                use flashtex_compiler::math::Frame;
                let body = sub(body, sink);
                vec![match frame {
                    Frame::Over => ml::Atom::overline(body),
                    Frame::Under => ml::Atom::underline(body),
                    Frame::Box => ml::Atom::group(body),
                }]
            }
            // amsmath's `\overset{a}{b}` is `\mathop{b}\limits^{a}` wrapped
            // in the base's own class (`\binrel@`), which math-layout sets
            // exactly (Rule 13a limits).
            N::Stacked { base, over, under } => {
                let class = match base.atoms.as_slice() {
                    [only] if only.superscript.is_none() && only.subscript.is_none() => match &only.nucleus {
                        N::Symbol(s) if s.chars().count() == 1 => ml::mathlist::default_class(s.chars().next().expect("one char")).0,
                        _ => ml::AtomClass::Ord,
                    },
                    _ => ml::AtomClass::Ord,
                };
                let mut op = ml::Atom::new(ml::AtomClass::Op, ml::Nucleus::List(sub(base, sink))).with_limits(ml::Limits::Limits);
                op.superscript = over.as_ref().map(|l| sub(l, sink));
                op.subscript = under.as_ref().map(|l| sub(l, sink));
                vec![ml::Atom::new(class, ml::Nucleus::List(ml::MathList::new(vec![op])))]
            }
            // `\hat`/`\bar`/...: Rule 12 accents with the unicode-math
            // combining mark Latin Modern Math carries for each command
            // (`\widehat`/`\widetilde` use the same mark; the horizontal
            // variants are not read, so a wide base gets the plain one).
            N::Accent { accent, body } => vec![ml::Atom::accent(accent_char(*accent), sub(body, sink))],
            // `array`/`cases`/matrix grids: math-layout has no array atom,
            // so the cells are set in reading order as one row inside the
            // environment's fences (`\left`/`\right`-sized when they are
            // single characters). `math_box` reports the grid once per
            // formula as a typed math_limitation.
            N::Matrix { rows, left, right, .. } => {
                let mut body = Vec::new();
                for row in rows {
                    for cell in row {
                        body.extend(sub(cell, sink).atoms);
                    }
                }
                let fence_char = |s: &str| {
                    let mut it = s.chars();
                    match (it.next(), it.next()) {
                        (Some(c), None) => Some(c),
                        _ => None,
                    }
                };
                match (fence_char(left), fence_char(right), left.is_empty() && right.is_empty()) {
                    (_, _, true) => vec![ml::Atom::group(ml::MathList::new(body))],
                    (l, r, false) => vec![ml::Atom::left_right(l, r, ml::MathList::new(body))],
                }
            }
        };
        if let Some(last) = out.last_mut() {
            if let Some(sup) = &a.superscript {
                last.superscript = Some(sub(sup, sink));
            }
            if let Some(sb) = &a.subscript {
                last.subscript = Some(sub(sb, sink));
            }
        }
        match stack.last_mut() {
            Some((_, body)) => body.extend(out),
            None => atoms.extend(out),
        }
    }
    // Unclosed \left: the compiler reports it; the delimiter is set as the
    // plain symbol it would have been without the fence.
    for (left, body) in stack {
        if let Some(c) = left {
            atoms.extend(symbol_atoms(c));
        }
        atoms.extend(body);
    }
    ml::MathList::new(atoms)
}

/// Splits `list` at its top-level `Space` atoms (outside `\left...\right`
/// pairs): each entry is a run of atoms and the glue after it in ems
/// (`None` for the last run). Consecutive spaces sum; a formula without
/// top-level glue is one run.
fn split_at_spaces(list: &flashtex_compiler::math::MathList, fence: &dyn Fn(&Span) -> Option<Fence>) -> Vec<(Vec<flashtex_compiler::math::MathAtom>, Option<f64>)> {
    use flashtex_compiler::math::Nucleus as N;
    let mut out: Vec<(Vec<flashtex_compiler::math::MathAtom>, Option<f64>)> = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0usize;
    for a in &list.atoms {
        match &a.nucleus {
            N::Space { em } if depth == 0 && a.superscript.is_none() && a.subscript.is_none() => {
                if current.is_empty() {
                    if let Some((_, Some(prev))) = out.last_mut() {
                        *prev += em;
                        continue;
                    }
                }
                out.push((std::mem::take(&mut current), Some(*em)));
            }
            N::Symbol(sym) if sym.chars().count() <= 1 => {
                match fence(&a.span) {
                    Some(Fence::Left) => depth += 1,
                    Some(Fence::Right) => depth = depth.saturating_sub(1),
                    None => {}
                }
                current.push(a.clone());
            }
            _ => current.push(a.clone()),
        }
    }
    // A trailing space keeps its kern: TeX includes it in the formula's
    // box (an empty run follows it).
    out.push((current, None));
    out
}

/// Every `array`/`cases`/matrix grid in `list` and its sub-formulas as
/// `(rows, columns)`; see the `Matrix` arm of [`convert_math_fenced`].
fn math_grids(list: &flashtex_compiler::math::MathList, out: &mut Vec<(usize, usize)>) {
    use flashtex_compiler::math::Nucleus as N;
    for a in &list.atoms {
        match &a.nucleus {
            N::Matrix { rows, .. } => {
                out.push((rows.len(), rows.iter().map(Vec::len).max().unwrap_or(0)));
                for cell in rows.iter().flatten() {
                    math_grids(cell, out);
                }
            }
            N::Fraction { numerator, denominator } => {
                math_grids(numerator, out);
                math_grids(denominator, out);
            }
            N::Radical(r) | N::Framed { body: r, .. } | N::Accent { body: r, .. } => math_grids(r, out),
            N::Stacked { base, over, under } => {
                math_grids(base, out);
                for part in [over, under].into_iter().flatten() {
                    math_grids(part, out);
                }
            }
            N::Symbol(_) | N::Text(_) | N::Space { .. } | N::Bold(_) => {}
        }
        if let Some(s) = &a.superscript {
            math_grids(s, out);
        }
        if let Some(s) = &a.subscript {
            math_grids(s, out);
        }
    }
}

/// Total explicit math glue (`\quad`/`\qquad`, in ems) in `list` and its
/// sub-formulas; see the `Space` arm of [`convert_math_fenced`].
fn math_glue_em(list: &flashtex_compiler::math::MathList) -> f64 {
    use flashtex_compiler::math::Nucleus as N;
    list.atoms
        .iter()
        .map(|a| {
            let own = match &a.nucleus {
                N::Space { em } => *em,
                N::Fraction { numerator, denominator } => math_glue_em(numerator) + math_glue_em(denominator),
                N::Radical(r) | N::Framed { body: r, .. } | N::Accent { body: r, .. } => math_glue_em(r),
                N::Stacked { base, over, under } => {
                    math_glue_em(base) + [over, under].into_iter().flatten().map(math_glue_em).sum::<f64>()
                }
                N::Matrix { rows, .. } => rows.iter().flatten().map(math_glue_em).sum(),
                N::Symbol(_) | N::Text(_) | N::Bold(_) => 0.0,
            };
            own + a.superscript.as_ref().map_or(0.0, math_glue_em) + a.subscript.as_ref().map_or(0.0, math_glue_em)
        })
        .sum()
}

/// The unicode-math combining mark for a compiler accent command, which is
/// what Latin Modern Math's `MATH` table carries accent attachment for.
fn accent_char(a: flashtex_compiler::math::Accent) -> char {
    use flashtex_compiler::math::Accent as A;
    match a {
        A::Hat | A::WideHat => '\u{0302}',
        A::Bar => '\u{0304}',
        A::Vec => '\u{20D7}',
        A::Tilde | A::WideTilde => '\u{0303}',
        A::Dot => '\u{0307}',
        A::Ddot => '\u{0308}',
        A::Check => '\u{030C}',
        A::Breve => '\u{0306}',
        A::Acute => '\u{0301}',
        A::Grave => '\u{0300}',
    }
}

/// Constructs in `list` and its sub-formulas the pipeline sets only
/// approximately, as `math_limitation` messages (one entry per occurrence;
/// `math_box` deduplicates by message): `\mathbf` in the roman face and
/// `\boxed` without its frame.
fn math_approximations(list: &flashtex_compiler::math::MathList, out: &mut Vec<String>) {
    use flashtex_compiler::math::{Frame, Nucleus as N};
    for a in &list.atoms {
        match &a.nucleus {
            N::Bold(text) => out.push(format!("\\mathbf{{{text}}} set in the regular roman face: the math text sink has no bold role")),
            N::Framed { body, frame } => {
                if *frame == Frame::Box {
                    out.push("\\boxed frame dropped: math-layout has no framed-box atom".to_string());
                }
                math_approximations(body, out);
            }
            N::Fraction { numerator, denominator } => {
                math_approximations(numerator, out);
                math_approximations(denominator, out);
            }
            N::Radical(r) | N::Accent { body: r, .. } => math_approximations(r, out),
            N::Stacked { base, over, under } => {
                math_approximations(base, out);
                for part in [over, under].into_iter().flatten() {
                    math_approximations(part, out);
                }
            }
            N::Matrix { rows, .. } => {
                for cell in rows.iter().flatten() {
                    math_approximations(cell, out);
                }
            }
            N::Symbol(_) | N::Text(_) | N::Space { .. } => {}
        }
        for part in [&a.superscript, &a.subscript].into_iter().flatten() {
            math_approximations(part, out);
        }
    }
}

/// The math-layout atoms for one compiler symbol character: plain.tex's
/// default classification, with the compiler's spellings that TeX sets as
/// composites expanded (`fontmath.ltx`: `\neq` is `\not=`, `\notin` is
/// `\not\in`, the zero-width relation slash before the relation).
fn symbol_atoms(c: char) -> Vec<ml::Atom> {
    match c {
        // The compiler spells \cdot as U+00B7; the Bin class and cmsy slot
        // are those of U+22C5.
        '\u{00B7}' => vec![ml::Atom::symbol('\u{22C5}')],
        '\u{2260}' => vec![ml::Atom::rel(crate::mathtex::NOT_SLASH), ml::Atom::symbol('=')],
        '\u{2209}' => vec![ml::Atom::rel(crate::mathtex::NOT_SLASH), ml::Atom::symbol('\u{2208}')],
        _ => vec![ml::Atom::symbol(c)],
    }
}

/// Adds `\addvspace` glue (a list environment's `\topsep`) to the block's
/// before-skip; `None` is a no-op.
fn add_skip_before(v: &mut pagebuild::VBlock, skip: Option<(f64, f64, f64)>) {
    let Some((n, s, k)) = skip else { return };
    v.space_before = Some(match v.space_before {
        Some((n0, s0, k0)) => (n0 + n, s0 + s, k0 + k),
        None => (n, s, k),
    });
}

/// Adds `pt` points of `\vspace` glue (compiler `Block::VSpace`) to the
/// block's before-skip. Zero is a no-op so cached blocks stay identical.
fn add_vspace(v: &mut pagebuild::VBlock, pt: f64) {
    if pt == 0.0 {
        return;
    }
    v.space_before = Some(match v.space_before {
        Some((n, s, k)) => (n + pt, s, k),
        None => (pt, 0.0, 0.0),
    });
}

/// Lays out every block of `doc` onto pages. With `cache`, blocks whose
/// items, flags and style match an earlier build are reused (see
/// `incremental`); the result is identical either way.
pub fn build(ctx: &mut Context, doc: &Doc, cache: Option<&RenderCache>) -> Laid {
    let mut blocks: Vec<BuiltBlock> = Vec::new();
    let mut after_heading = false;
    let quad = ctx.text_params(TextStyle::default(), ctx.style.body_size_pt).quad;
    let style_fp = if cache.is_some() { incremental::style_fingerprint(ctx.style) } else { 0 };
    use std::hash::{Hash, Hasher};
    let key_for = |tag: u8, items: &[AItem], flags: &[u64]| -> (Option<u64>, Option<(DocumentId, usize)>) {
        if cache.is_none() {
            return (None, None);
        }
        let Some((document, base)) = incremental::block_origin(items) else {
            return (None, None);
        };
        let mut h = std::collections::hash_map::DefaultHasher::new();
        tag.hash(&mut h);
        style_fp.hash(&mut h);
        document.0.hash(&mut h);
        flags.hash(&mut h);
        incremental::hash_items(items, base, &mut h);
        (Some(h.finish()), Some((document, base)))
    };
    for block in &doc.blocks {
        match block {
            Block::Heading {
                level,
                items,
                eject_before,
                vspace_before,
            } => {
                let (key, origin) = key_for(b'H', items, &[u64::from(*level)]);
                if let Some(mut b) = ctx.cached(cache, key, origin, |c| c.heading_block(*level, items)) {
                    if *eject_before {
                        b.vertical.penalty_before = Some(pagebuild::EJECT_PENALTY);
                    }
                    add_vspace(&mut b.vertical, *vspace_before);
                    blocks.push(b);
                    after_heading = true;
                }
            }
            Block::Paragraph {
                parts,
                indent,
                style,
                env_open,
                env_close,
                eject_before,
                vspace_before,
            } => {
                let mut first = true;
                let mut eject = *eject_before;
                let mut vspace = *vspace_before;
                // `\begin{center}`/`\begin{quote}`: `\addvspace{\topsep}` (plus
                // `\partopsep` from vertical mode) before the first paragraph;
                // `\end{...}` adds the same after the last (`\@endparenv`).
                let env_skip = |vmode: bool| {
                    let t = ctx.style.topsep;
                    let p = if vmode { ctx.style.partopsep } else { crate::style::Skip::default() };
                    (t.natural + p.natural, t.stretch + p.stretch, t.shrink + p.shrink)
                };
                let mut env_before = env_open.map(|e| env_skip(e.vmode));
                let env_after = env_close.then(|| env_skip(env_open.is_some_and(|e| e.vmode)));
                let first_block = blocks.len();
                // TeX's pre_display_size: the width of the line before a
                // display plus 2em; -infinity when nothing precedes it.
                let mut pre_display: Option<f64> = None;
                for part in parts {
                    match part {
                        ParaPart::Lines(items) => {
                            // TeX discards the space token right after a
                            // display's closing `$$` (§1200 resume_after_display).
                            let items = if !first && matches!(items.first(), Some(AItem::Space { .. })) { &items[1..] } else { &items[..] };
                            let (ind, starts, ah) = (*indent && first, first, after_heading && first);
                            let (key, origin) = key_for(b'P', items, &[u64::from(ind), u64::from(starts), u64::from(ah), *style as u64]);
                            let st = *style;
                            if let Some(mut b) = ctx.cached(cache, key, origin, |c| c.paragraph_block(items, ind, starts, ah, st)) {
                                pre_display = b.block.lines.lines.last().map(|l| l.natural_width + 2.0 * quad);
                                if std::mem::take(&mut eject) {
                                    b.vertical.penalty_before = Some(pagebuild::EJECT_PENALTY);
                                }
                                add_vspace(&mut b.vertical, std::mem::take(&mut vspace));
                                add_skip_before(&mut b.vertical, env_before.take());
                                blocks.push(b);
                            }
                        }
                        ParaPart::Display {
                            list,
                            span,
                            number,
                            bracket,
                        } => {
                            if first {
                                let (mut opener, size) = ctx.display_opener_block(*bracket);
                                if std::mem::take(&mut eject) {
                                    opener.vertical.penalty_before = Some(pagebuild::EJECT_PENALTY);
                                }
                                add_vspace(&mut opener.vertical, std::mem::take(&mut vspace));
                                add_skip_before(&mut opener.vertical, env_before.take());
                                blocks.push(opener);
                                pre_display = Some(size);
                            }
                            let (key, origin) = if cache.is_some() {
                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                b'D'.hash(&mut h);
                                style_fp.hash(&mut h);
                                span.document.0.hash(&mut h);
                                incremental::hash_math(list, &mut h);
                                (span.end - span.start).hash(&mut h);
                                pre_display.map(f64::to_bits).hash(&mut h);
                                if let Some((n, ns)) = number {
                                    n.hash(&mut h);
                                    (ns.start.wrapping_sub(span.start), ns.end.wrapping_sub(span.start)).hash(&mut h);
                                }
                                bracket.hash(&mut h);
                                (Some(h.finish()), Some((span.document, span.start)))
                            } else {
                                (None, None)
                            };
                            let pd = pre_display;
                            if let Some(b) = ctx.cached(cache, key, origin, |c| c.display_block(list, *span, pd, number.as_ref())) {
                                blocks.push(b);
                            }
                            pre_display = None;
                        }
                    }
                    first = false;
                }
                if let Some(skip) = env_after {
                    if blocks.len() > first_block {
                        if let Some(last) = blocks.last_mut() {
                            last.vertical.space_after = Some(match last.vertical.space_after {
                                Some((n, s, k)) => (n + skip.0, s + skip.1, k + skip.2),
                                None => skip,
                            });
                        }
                    }
                }
                after_heading = false;
            }
            Block::Rule {
                span,
                eject_before,
                vspace_before,
            } => {
                let mut b = ctx.rule_block(*span);
                if *eject_before {
                    b.vertical.penalty_before = Some(pagebuild::EJECT_PENALTY);
                }
                add_vspace(&mut b.vertical, *vspace_before);
                blocks.push(b);
                after_heading = false;
            }
        }
    }
    let s = ctx.style;
    let params = pagebuild::PageParams {
        vsize: s.text_height_pt,
        topskip: s.topskip_pt,
        maxdepth: s.maxdepth_pt,
        baselineskip: s.baselineskip_pt,
        lineskip: s.lineskip_pt,
        lineskiplimit: s.lineskiplimit_pt,
    };
    let vblocks: Vec<VBlock> = blocks.iter().map(|b| b.vertical.clone()).collect();
    let list = pagebuild::vlist(&params, &vblocks);
    let built = pagebuild::break_pages(&params, &list);
    let mut pages = pl::Pages {
        pages: Vec::with_capacity(built.len()),
        overflow: Vec::new(),
        text_height: s.text_height_pt,
    };
    for (pi, bp) in built.iter().enumerate() {
        let number = pi as u32 + 1;
        let mut page = pl::Page {
            number,
            width: s.page_width_pt,
            height: s.page_height_pt,
            lines: Vec::with_capacity(bp.lines.len()),
            runs: Vec::new(),
        };
        for placed in &bp.lines {
            let (bi, li) = placed.payload;
            let line = &blocks[bi].block.lines.lines[li];
            let y = s.text_y_pt + placed.baseline;
            page.lines.push(pl::PlacedLine {
                paragraph: bi,
                line: li,
                baseline_y: y,
                height: line.height,
                depth: line.depth,
            });
            for r in &line.runs {
                let mut r = r.clone();
                r.x += s.text_x_pt;
                r.baseline_y = y;
                page.runs.push(r);
            }
        }
        if bp.overfull_by > 0.0 {
            if let Some(last) = bp.lines.last() {
                let (bi, li) = last.payload;
                pages.overflow.push(pl::PageOverflow {
                    page: number,
                    paragraph: bi,
                    line: li,
                    bottom: s.text_y_pt + last.baseline + last.depth,
                    limit: s.text_y_pt + s.text_height_pt,
                });
            }
        }
        pages.pages.push(page);
    }
    for o in &pages.overflow {
        let span = blocks
            .get(o.paragraph)
            .and_then(|b| b.recs.iter().flatten().next().copied())
            .and_then(|r| match &ctx.recs[r] {
                BoxRec::Text { clusters, .. } => clusters.first().map(|c| c.span),
                BoxRec::Math(m) => Some(ctx.maths[*m].span),
                BoxRec::Rule { span, .. } => Some(*span),
            });
        let src = span.map(|sp| vec![ctx.source(sp)]).unwrap_or_default();
        ctx.diagnostics.push(Diagnostic::warning(
            "overfull_vbox",
            format!("page {}: a line extends {:.2}pt past the text area", o.page, o.bottom - o.limit),
            src,
        ));
    }
    Laid {
        blocks,
        pages,
        recs: std::mem::take(&mut ctx.recs),
        maths: std::mem::take(&mut ctx.maths),
    }
}

/// The page each `\label` landed on (the page of the line holding the item
/// it precedes, or the block's last line when it ends the block).
pub fn label_pages(laid: &Laid) -> BTreeMap<String, u32> {
    let mut out = BTreeMap::new();
    for (bi, block) in laid.blocks.iter().enumerate() {
        for (key, item) in &block.labels {
            let lines = &block.block.lines.lines;
            let li = lines
                .iter()
                .position(|l| l.items.contains(item))
                .unwrap_or(lines.len().saturating_sub(1));
            let page = laid
                .pages
                .pages
                .iter()
                .find(|p| p.lines.iter().any(|pl| pl.paragraph == bi && pl.line == li))
                .map(|p| p.number)
                .unwrap_or(1);
            out.insert(key.clone(), page);
        }
    }
    out
}

/// Converts the placed pages into the display list.
pub fn assemble(
    project_id: &str,
    revision: u64,
    documents: &[SourceDocument<'_>],
    style: &Stylesheet,
    _fonts: &FontSet,
    laid: Laid,
    mut diagnostics: Vec<Diagnostic>,
    cache: Option<&RenderCache>,
) -> DisplayList {
    let paths: Vec<Rc<str>> = documents.iter().map(|d| Rc::from(d.path)).collect();
    let empty: Rc<str> = Rc::from("");
    let source_of = |span: Span| SourceRange {
        path: paths.get(span.document.0).cloned().unwrap_or_else(|| empty.clone()),
        start_byte: span.start,
        end_byte: span.end,
    };
    let mut used: BTreeMap<Rc<str>, Rc<LoadedFace>> = BTreeMap::new();
    // Every block's lines are assembled once in line-local coordinates
    // (cached across requests by the block's key), then placed per page by
    // integer tick/byte moves.
    let text_x = style.text_x_pt;
    let mut assembled: Vec<Option<Rc<incremental::AssembledBlock>>> = Vec::with_capacity(laid.blocks.len());
    for block in &laid.blocks {
        let hit = block
            .cache_key
            .and_then(|(k, _, _)| cache.and_then(|c| c.assembled(k)))
            .filter(|a| block.cache_key.is_some_and(|(_, d, _)| *a.path == *paths.get(d.0).map_or("", |p| &**p)));
        let a = match hit {
            Some(a) => a,
            None => {
                let built = assemble_block(block, &laid.recs, &laid.maths, text_x, &source_of, &paths, &empty);
                match (cache, block.cache_key) {
                    (Some(c), Some((k, _, _))) => c.insert_assembled(k, built),
                    _ => Rc::new(built),
                }
            }
        };
        for f in &a.faces {
            used.entry(f.font_id.clone()).or_insert_with(|| f.clone());
        }
        assembled.push(Some(a));
    }
    let mut pages = Vec::new();
    for page in &laid.pages.pages {
        let mut items: Vec<display::Item> = Vec::new();
        for placed in &page.lines {
            let block = &laid.blocks[placed.paragraph];
            let Some(a) = assembled[placed.paragraph].as_ref() else { continue };
            let Some(line_items) = a.lines.get(placed.line) else { continue };
            let dy = Tick::from_tex_pt(placed.baseline_y);
            let delta = block.cache_key.map_or(0, |(_, _, b)| b as isize - a.base as isize);
            for it in line_items {
                items.push(incremental::place_item(it, dy, &a.path, delta));
            }
        }
        pages.push(display::Page {
            number: page.number,
            width: Tick::from_tex_pt(page.width),
            height: Tick::from_tex_pt(page.height),
            items,
        });
    }
    // Resource selection provenance: which outline resource drew each TFM
    // font's glyphs. The roman family has exact optical siblings
    // (lmroman12/8/6 for lmr12/8/6); the italic, symbol and extension
    // families only exist as the single-design Latin Modern Math, which is
    // reported rather than passed off as the reference's lmmi/lmsy/lmex.
    // Collected over every provider (cached blocks keep the provider that
    // built them), then emitted in TFM-name order without a source so the
    // report does not depend on which block was built first.
    let mut profiles: BTreeMap<String, String> = BTreeMap::new();
    for a in assembled.iter().flatten() {
        for (tfm, face, exact) in &a.resources {
            if !exact {
                profiles.entry(tfm.clone()).or_insert(face.clone());
            }
        }
    }
    for (tfm, face) in profiles {
        diagnostics.push(Diagnostic::warning(
            "math_resource_profile",
            format!("{tfm}: glyphs drawn from {face} (one 10pt design); no optical-size OpenType outline resource exists for this family, so the outlines are not the reference's {tfm} design"),
            Vec::new(),
        ));
    }
    // Glyphs TeX's metrics placed that the OpenType face cannot draw
    // (extensible assemblies, unknown chains): reported, not faked.
    let mut reported = BTreeSet::new();
    for (block, a) in laid.blocks.iter().zip(assembled.iter()) {
        let Some(a) = a else { continue };
        for (font, code, ch) in &a.unmapped {
            if reported.insert((font.clone(), *code)) {
                let span = block.recs.iter().flatten().find_map(|r| match &laid.recs[*r] {
                    BoxRec::Math(mi) => Some(laid.maths[*mi].span),
                    BoxRec::Rule { span, .. } => Some(*span),
                    BoxRec::Text { .. } => None,
                });
                diagnostics.push(Diagnostic::warning(
                    "math_glyph_unmapped",
                    format!("{font} code {code:#04x} ('{ch}') has no Latin Modern Math glyph mapping; nothing drawn for it"),
                    span.map(|s| vec![source_of(s)]).unwrap_or_default(),
                ));
            }
        }
    }
    let fonts = used
        .values()
        .map(|f| FontResource {
            font_id: f.font_id.clone(),
            sha256: f.font_id.to_string(),
            byte_length: f.byte_length,
            format: f.format.to_string(),
            face_index: 0,
            units_per_em: f.units_per_em,
            glyph_count: f.glyph_count,
            postscript_name: f.postscript_name.clone(),
            path: f.path.as_ref().map(|p| p.display().to_string()),
        })
        .collect();
    let docs = documents
        .iter()
        .map(|d| DocumentResource {
            path: d.path.to_string(),
            revision,
            sha256: sha256::hex(&sha256::digest(d.text.as_bytes())),
            byte_length: d.text.len() as u64,
        })
        .collect();
    let _ = style;
    DisplayList {
        project_id: project_id.to_string(),
        revision,
        documents: docs,
        fonts,
        pages,
        diagnostics,
    }
}

/// Assembles one block's lines in line-local coordinates.
#[allow(clippy::too_many_arguments)]
fn assemble_block(
    block: &BuiltBlock,
    recs: &[BoxRec],
    maths: &[MathRec],
    text_x: f64,
    source_of: &dyn Fn(Span) -> SourceRange,
    paths: &[Rc<str>],
    empty: &Rc<str>,
) -> incremental::AssembledBlock {
    let mut used: BTreeMap<Rc<str>, Rc<LoadedFace>> = BTreeMap::new();
    let mut lines = Vec::with_capacity(block.block.lines.lines.len());
    let mut resources = Vec::new();
    let mut unmapped = Vec::new();
    for line in &block.block.lines.lines {
        let mut items: Vec<display::Item> = Vec::new();
        // Boxes of this line in item order pair with its runs in order.
        let boxes: Vec<usize> = line
            .items
            .clone()
            .filter(|i| matches!(block.items.get(*i), Some(pl::Item::Box(_))))
            .filter_map(|i| block.recs.get(i).copied().flatten())
            .collect();
        let mut bi = 0usize;
        for run in &line.runs {
            if run.is_hyphen {
                continue;
            }
            let Some(&rec) = boxes.get(bi) else { break };
            bi += 1;
            let mut local = run.clone();
            local.x += text_x;
            local.baseline_y = 0.0;
            match &recs[rec] {
                BoxRec::Text {
                    face,
                    size,
                    text,
                    clusters,
                    glyphs,
                    height,
                    depth,
                    ..
                } => {
                    used.entry(face.font_id.clone()).or_insert_with(|| face.clone());
                    if let Some(item) = text_item(&local, face, *size, text, clusters, glyphs, *height, *depth, source_of) {
                        items.push(item);
                    }
                }
                BoxRec::Math(mi) => {
                    let m = &maths[*mi];
                    math_items(&local, m, source_of, &mut items, &mut used);
                    if let MathProvider::Tex(t) = &m.metrics {
                        resources.extend(t.take_resources());
                        unmapped.extend(t.take_unmapped());
                    }
                }
                BoxRec::Rule { width, height, span } => {
                    // Line-local like text: the rule's bottom is the baseline.
                    items.push(display::Item::Rule(Rule {
                        x: Tick::from_tex_pt(local.x),
                        top: Tick::from_tex_pt(-height),
                        width: Tick::from_tex_pt(*width).max(Tick(1)),
                        height: Tick::from_tex_pt(*height).max(Tick(1)),
                        paint: Paint::BLACK,
                        provenance: Provenance::Source(source_of(*span)),
                    }));
                }
            }
        }
        lines.push(items);
    }
    let (document, base) = block.cache_key.map_or((DocumentId(0), 0), |(_, d, b)| (d, b));
    incremental::AssembledBlock {
        lines,
        faces: used.into_values().collect(),
        base,
        path: paths.get(document.0).cloned().unwrap_or_else(|| empty.clone()),
        resources,
        unmapped,
    }
}

#[allow(clippy::too_many_arguments)]
fn text_item(
    run: &pl::PositionedRun,
    face: &Rc<LoadedFace>,
    size: f64,
    text: &str,
    clusters: &[ClusterRec],
    recs: &[GlyphRec],
    height: f64,
    depth: f64,
    source_of: &dyn Fn(Span) -> SourceRange,
) -> Option<display::Item> {
    // Line-local: the baseline is 0 and every y is an offset from it; the
    // page position is added as an integer tick move when the line is
    // placed, so a block placed anywhere yields identical ticks.
    let baseline = 0.0;
    let top = Tick::from_tex_pt(baseline - height);
    let box_height = Tick::from_tex_pt(height + depth);
    let mut glyphs = Vec::with_capacity(run.glyphs.len());
    let mut origins = Vec::with_capacity(run.glyphs.len());
    // Cluster glyph ranges are contiguous and ordered: walk them alongside
    // the glyphs instead of searching per glyph.
    let mut ci = 0usize;
    for (i, g) in run.glyphs.iter().enumerate() {
        let rec = recs.get(i)?;
        while ci + 1 < clusters.len() && i >= clusters[ci].glyphs.end {
            ci += 1;
        }
        let x = run.x + g.x_offset + face.pt(i64::from(rec.x_offset_units), size);
        let y = baseline - face.pt(i64::from(rec.y_offset_units), size);
        origins.push((run.x + g.x_offset, g.advance));
        if rec.gid == 0 {
            continue;
        }
        let cluster = if clusters.get(ci).is_some_and(|c| c.glyphs.contains(&i)) { ci as u32 } else { clusters.iter().position(|c| c.glyphs.contains(&i)).unwrap_or(0) as u32 };
        glyphs.push(Glyph {
            gid: rec.gid,
            origin_x: Tick::from_tex_pt(x),
            baseline_y: Tick::from_tex_pt(y),
            advance_x: Tick::from_tex_pt(g.advance),
            advance_y: Tick(0),
            cluster,
        });
    }
    if glyphs.is_empty() {
        return None;
    }
    let last_index = clusters.len().saturating_sub(1);
    let out_clusters = clusters
        .iter()
        .enumerate()
        .map(|(ci, c)| {
            let (x0, x1) = match (origins.get(c.glyphs.start), origins.get(c.glyphs.end.saturating_sub(1))) {
                (Some(a), Some(b)) => (a.0, b.0 + b.1),
                _ => (run.x, run.x),
            };
            let carets = display::Carets {
                first: Caret {
                    text_byte: c.text_range.start,
                    x: Tick::from_tex_pt(x0),
                    top,
                    height: box_height,
                },
                last: (ci == last_index).then(|| Caret {
                    text_byte: c.text_range.end,
                    x: Tick::from_tex_pt(x1),
                    top,
                    height: box_height,
                }),
            };
            Cluster {
                text_start_byte: c.text_range.start,
                text_end_byte: c.text_range.end,
                hit_rect: Rect {
                    x: Tick::from_tex_pt(x0),
                    top,
                    width: Tick::from_tex_pt(x1 - x0),
                    height: box_height,
                },
                carets,
                provenance: Provenance::Source(source_of(c.span)),
            }
        })
        .collect();
    Some(display::Item::GlyphRun(GlyphRun {
        font_id: face.font_id.clone(),
        font_size: Tick::from_tex_pt(size),
        text: text.to_string(),
        glyphs,
        clusters: out_clusters,
        paint: Paint::BLACK,
        role: display::RunRole::Text,
    }))
}

fn math_items(
    run: &pl::PositionedRun,
    m: &MathRec,
    source_of: &dyn Fn(Span) -> SourceRange,
    items: &mut Vec<display::Item>,
    used: &mut BTreeMap<Rc<str>, Rc<LoadedFace>>,
) {
    let flat = ml::positioned_runs(&m.root, (run.x, -m.root.height));
    let src = source_of(m.span);
    // Group consecutive glyphs of one face and size into a run; each glyph
    // is a cluster.
    let mut current: Option<GlyphRun> = None;
    let flush = |current: &mut Option<GlyphRun>, items: &mut Vec<display::Item>| {
        if let Some(r) = current.take() {
            if !r.glyphs.is_empty() {
                items.push(display::Item::GlyphRun(r));
            }
        }
    };
    for g in &flat.glyphs {
        let Some((face, gid)) = m.otf_glyph(g) else { continue };
        if gid == 0 {
            if g.ch == ' ' {
                // A `\text` interword space: glue, no glyph; the run is
                // split so the words stay separate items, as in paragraphs.
                flush(&mut current, items);
            }
            continue;
        }
        used.entry(face.font_id.clone()).or_insert_with(|| face.clone());
        let size_tick = Tick::from_tex_pt(g.size);
        if current.as_ref().is_some_and(|r| r.font_size != size_tick || r.font_id != face.font_id) {
            flush(&mut current, items);
        }
        let r = current.get_or_insert_with(|| GlyphRun {
            font_id: face.font_id.clone(),
            font_size: size_tick,
            text: String::new(),
            glyphs: Vec::new(),
            clusters: Vec::new(),
            paint: Paint::BLACK,
            role: display::RunRole::Math,
        });
        let b = face.bounds(crate::ids::GlyphId(gid), Some(g.ch));
        let adv = face.pt(i64::from(face.face().advance(crate::ids::GlyphId(gid)).unwrap_or(0)), g.size);
        let (h, d) = if b.empty {
            (0.0, 0.0)
        } else {
            (face.pt(i64::from(b.y_max), g.size), face.pt(-i64::from(b.y_min), g.size))
        };
        // A cmex glyph was laid out as its TFM box, which is the Type 1
        // outline hanging from the origin; the OpenType variant painted for
        // it is centred on the axis relative to its own origin, so its
        // baseline moves to put the drawn ink's centre on the TFM box's.
        let baseline_y = match m.extension_box(g) {
            Some((th, td)) if !b.empty => g.baseline_y + ((td - th) - (d - h)) / 2.0,
            _ => g.baseline_y,
        };
        let start = r.text.len();
        match m.run_glyph(g) {
            // A `\text` cluster keeps its whole source text (`ffi`).
            Some(rg) => r.text.push_str(&rg.text),
            None => r.text.push(g.ch),
        }
        let ci = r.clusters.len() as u32;
        let top = Tick::from_tex_pt(baseline_y - h);
        let hh = Tick::from_tex_pt((h + d).max(0.01));
        r.glyphs.push(Glyph {
            gid,
            origin_x: Tick::from_tex_pt(g.x),
            baseline_y: Tick::from_tex_pt(baseline_y),
            advance_x: Tick::from_tex_pt(adv),
            advance_y: Tick(0),
            cluster: ci,
        });
        r.clusters.push(Cluster {
            text_start_byte: start,
            text_end_byte: r.text.len(),
            hit_rect: Rect {
                x: Tick::from_tex_pt(g.x),
                top,
                width: Tick::from_tex_pt(adv),
                height: hh,
            },
            carets: display::Carets {
                first: Caret {
                    text_byte: start,
                    x: Tick::from_tex_pt(g.x),
                    top,
                    height: hh,
                },
                last: None,
            },
            provenance: Provenance::Source(src.clone()),
        });
    }
    flush(&mut current, items);
    for rule in &flat.rules {
        if rule.w <= 0.0 || rule.h <= 0.0 {
            continue;
        }
        items.push(display::Item::Rule(Rule {
            x: Tick::from_tex_pt(rule.x),
            top: Tick::from_tex_pt(rule.y),
            width: Tick::from_tex_pt(rule.w).max(Tick(1)),
            height: Tick::from_tex_pt(rule.h).max(Tick(1)),
            paint: Paint::BLACK,
            provenance: Provenance::Source(src.clone()),
        }));
    }
}

impl Tick {
    fn max(self, other: Tick) -> Tick {
        if self.0 >= other.0 {
            self
        } else {
            other
        }
    }
}

/// Which documents a laid-out page references (for tests).
pub fn documents_referenced(list: &DisplayList) -> BTreeSet<DocumentId> {
    let mut out = BTreeSet::new();
    for p in &list.pages {
        for it in &p.items {
            if let display::Item::GlyphRun(r) = it {
                for c in &r.clusters {
                    for s in c.provenance.sources() {
                        if let Some(i) = list.documents.iter().position(|d| *d.path == *s.path) {
                            out.insert(DocumentId(i));
                        }
                    }
                }
            }
        }
    }
    out
}

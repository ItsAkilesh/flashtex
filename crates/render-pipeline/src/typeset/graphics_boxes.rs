//! `\includegraphics` in running text and the graphics box transforms
//! `\scalebox`, `\resizebox`, `\rotatebox`, `\reflectbox`.
//!
//! An inline image is one box in the horizontal list. graphics.sty
//! `\Gin@setfile` ends with `\dp\z@\z@ \ht\z@\Gin@req@height
//! \wd\z@\Gin@req@width`, so the box sits on the baseline, raises its line
//! like any tall box and takes part in line breaking (`graphics::place_image`
//! adds `angle` and the keys after it).
//!
//! A transform sets its content as an `\hbox` (`Context::table_hbox`) and
//! replaces the box by the transformed one (`graphics::TBox`, transcribing
//! `\Gscale@box`, `\Gscale@@box` and `\Grot@box`). pdftex.def paints the
//! content through `\pdfsetmatrix` from its reference point
//! (`\Gscale@start`, `\Grot@start`); the display items get the same affine
//! map: glyph origins and advances, rules (filled paths once rotated),
//! paths and image transforms. The glyph shapes' own linear map is
//! `GlyphRun::glyph_transform` (proposal `display-list-v2-transforms`).
//!
//! `draft` (the key, or the `draft` class/package option) paints
//! graphics.sty's placeholder instead of the image:
//! `\hb@xt@\Gin@req@width{\vrule\hss\vbox to\Gin@req@height{\hrule
//! \@width\Gin@req@width\vss\rlap{ \ttfamily<file>}\vss\hrule}\hss\vrule}`.

use std::collections::BTreeMap;
use std::rc::Rc;

use flashtex_compiler::graphics::{Graphic, TransformKind};
use flashtex_compiler::Span;
use flashtex_paragraph_layout as pl;

use crate::adapter::{self, TextStyle, TransformItem};
use crate::display::{self, Caret, ClipPath, Diagnostic, ImageResource, Paint, PathCmd, PathItem, PathPaintOp, Provenance, Rect, SourceRange, Tick};
use crate::floats::ImageCache;
use crate::fonts::LoadedFace;
use crate::graphics::{self, GKey, GraphicBox, LengthEnv, TBox, BP_PER_PT};

use super::{BoxRec, BuiltBlock, Context, MATH_SENTINEL};

/// `\vrule`/`\hrule` default thickness (TeX §463).
const DRAFT_RULE: f64 = 0.4;

/// What image loading needs beyond the stylesheet.
#[derive(Default)]
pub struct GraphicsEnv {
    options: Option<crate::RenderOptions>,
    cache: ImageCache,
    draft: bool,
}

/// An `\includegraphics` box.
pub struct GraphicRec {
    pub gbox: GraphicBox,
    /// The visible part of the unit square (`clip` with a viewport/trim).
    pub clip: Option<[f64; 4]>,
    /// `None`: nothing is painted (draft, or the file is unavailable).
    pub resource: Option<Rc<ImageResource>>,
    pub draft: Option<Draft>,
    pub span: Span,
}

/// The `draft` placeholder of a [`GraphicRec`].
pub struct Draft {
    /// The requested (unrotated) box.
    pub width: f64,
    pub height: f64,
    /// That box's unit square into the final box (TeX points, y up).
    pub frame: [f64; 6],
    /// `\rlap{ \ttfamily<file>}`: the shaped name, its text record, its x
    /// and its baseline above the box bottom.
    pub text: Option<(pl::GlyphRun, usize, f64, f64)>,
}

/// A transformed `\hbox`: the content block (one line) and the content's
/// map into the box (TeX points, y up from the baseline).
pub struct TransformRec {
    pub block: BuiltBlock,
    pub matrix: [f64; 6],
    pub span: Span,
}

/// graphics.sty's `draft`/`final` from the class options, then the
/// `graphics` and `graphicx` package options (a later option wins).
pub fn draft_option(source: &str) -> bool {
    let mut draft = false;
    let mut read = |options: Option<String>| {
        for o in options.iter().flat_map(|o| o.split(',')) {
            match o.trim() {
                "draft" => draft = true,
                "final" => draft = false,
                _ => {}
            }
        }
    };
    read(adapter::class_options(source));
    read(adapter::package_options(source, "graphics"));
    read(adapter::package_options(source, "graphicx"));
    draft
}

/// The name `\meaning` prints for a draft box: the file as written plus
/// the extension the search found (`\Gin@base\Gin@ext`).
fn draft_name(raw: &str, resolved: &str) -> String {
    let has_ext = raw.rsplit('/').next().is_some_and(|name| name.contains('.'));
    if has_ext {
        return raw.to_string();
    }
    match resolved.rsplit('/').next().and_then(|name| name.rfind('.').map(|i| &name[i..])) {
        Some(ext) => format!("{raw}{ext}"),
        None => raw.to_string(),
    }
}

impl Context<'_> {
    /// The project root, `\graphicspath` and `draft` for inline graphics.
    pub fn set_graphics(&mut self, options: &crate::RenderOptions, search_path: Vec<String>, draft: bool) {
        self.graphics.options = Some(options.clone());
        self.graphics.cache.set_search_path(search_path);
        self.graphics.draft = draft;
    }

    fn graphics_lengths(&self, size: f64) -> LengthEnv {
        let s = self.style;
        let p = self.text_params(TextStyle::default(), size);
        // `\linewidth`/`\columnwidth`/`\hsize`: the box being set, not the
        // page (a float or minipage body narrows it).
        LengthEnv {
            text_width: s.text_width_pt,
            line_width: self.hsize_override.unwrap_or(s.text_width_pt),
            text_height: s.text_height_pt,
            paper_width: s.page_width_pt,
            paper_height: s.page_height_pt,
            em: p.quad,
            ex: p.x_height,
        }
    }

    /// `\includegraphics` in a paragraph: one box record.
    pub(super) fn graphic_box(&mut self, g: &Graphic, size: f64) -> Option<(pl::GlyphRun, usize)> {
        let env = self.graphics_lengths(size);
        let src = self.source(g.span);
        let (keys, problems) = graphics::parse_keys(&g.options, &env);
        for p in problems {
            self.emit(None, Diagnostic::warning("graphics_option", p, vec![src.clone()]));
        }
        for k in &keys {
            if let GKey::Unsupported(name) = k {
                self.emit(None, Diagnostic::warning("graphics_option", format!("\\includegraphics key '{name}' is not honoured yet"), vec![src.clone()]));
            }
        }
        let page = keys.iter().find_map(|k| if let GKey::Page(p) = k { Some(*p) } else { None }).unwrap_or(1);
        let loaded = {
            let GraphicsEnv { options, cache, .. } = &mut self.graphics;
            let fallback = crate::RenderOptions::default();
            cache.load(options.as_ref().unwrap_or(&fallback), &g.path, page)
        };
        let (resource, placed) = match loaded {
            Ok((resource, info)) => {
                let placed = graphics::place_image(info.width_bp / BP_PER_PT, info.height_bp / BP_PER_PT, &keys, g.starred, self.graphics.draft, &env);
                (Some(resource), placed)
            }
            Err(msg) => {
                let w = keys.iter().rev().find_map(|k| if let GKey::Width(v) = k { Some(*v) } else { None });
                let h = keys.iter().rev().find_map(|k| if let GKey::Height(v) | GKey::TotalHeight(v) = k { Some(*v) } else { None });
                match (w, h) {
                    (Some(w), Some(h)) => {
                        self.emit(None, Diagnostic::error("image_unavailable", format!("{msg} (its requested size is kept empty)"), vec![src]));
                        (None, graphics::place_image(w, h, &[], false, false, &env))
                    }
                    _ => {
                        self.emit(None, Diagnostic::error("image_unavailable", msg, vec![src]));
                        return None;
                    }
                }
            }
        };
        let draft = if placed.draft {
            let name = resource.as_ref().map_or_else(|| g.path.clone(), |r| draft_name(&g.path, &r.path));
            let chars = name.chars().map(|_| adapter::CharSrc { document: g.span.document, start: g.span.start, end: g.span.end }).collect();
            let seg = adapter::Segment { text: name, chars, style: TextStyle { family: crate::nfss::FamilyKind::Tt, ..TextStyle::default() } };
            let (w, h) = placed.frame_size;
            let text = self.text_box(&seg, size).map(|(run, rec)| {
                // `\vbox to H{\hrule \vss \rlap{..} \vss \hrule}`: no
                // interline glue next to rules, the two `\vss` share the rest.
                let space = self.space_glue(TextStyle::default(), size, 1000).width;
                let gap = (h - 2.0 * DRAFT_RULE - run.height - run.depth) / 2.0;
                let raise = h - DRAFT_RULE - gap - run.height;
                (run, rec, space, raise)
            });
            Some(Draft { width: w, height: h, frame: placed.frame, text })
        } else {
            None
        };
        let b = placed.gbox;
        let resource = if placed.draft { None } else { resource };
        self.recs.push(BoxRec::Graphic(Rc::new(GraphicRec { gbox: b, clip: placed.clip, resource, draft, span: g.span })));
        let run = pl::GlyphRun { font: MATH_SENTINEL, size, glyphs: Vec::new(), width: b.width, height: b.height, depth: b.depth, source: g.span.start..g.span.end };
        Some((run, self.recs.len() - 1))
    }

    /// A graphics transform: the content as an `\hbox`, then the
    /// transformed box's dimensions.
    pub(super) fn transform_box(&mut self, t: &TransformItem, size: f64) -> Option<(pl::GlyphRun, usize)> {
        let (block, dims) = self.table_hbox(&t.content, size)?;
        let content = TBox::content(dims.width, dims.height, dims.depth);
        let env = self.graphics_lengths(size);
        let number = |s: &str| s.trim().parse::<f64>().ok();
        let result = match &t.kind {
            TransformKind::Scale { x, y } => match (number(x), y.as_deref().map(number)) {
                (Some(sx), None) => Ok(content.scale(sx, sx)),
                (Some(sx), Some(Some(sy))) => Ok(content.scale(sx, sy)),
                _ => Err(format!("\\scalebox factors '{x}' {y:?} are not numbers")),
            },
            TransformKind::Reflect => Ok(content.scale(-1.0, 1.0)),
            TransformKind::Resize { starred, width, height } => {
                let c = (dims.width, dims.height, dims.depth);
                match (self.box_dimen(width, &env, c), self.box_dimen(height, &env, c)) {
                    (Ok(w), Ok(h)) => Ok(content.resize(w, h, *starred, false)),
                    _ => Err(format!("\\resizebox size '{width}' x '{height}' could not be read")),
                }
            }
            TransformKind::Rotate { options, angle } => match number(angle) {
                Some(a) => {
                    let (ox, oy) = options.as_deref().map_or((0.0, 0.0), |o| graphics::rotation_origin(o, dims.width, dims.height, dims.depth, &env));
                    Ok(content.rotate(a, ox, oy))
                }
                None => Err(format!("\\rotatebox angle '{angle}' is not a number")),
            },
        };
        let b = match result {
            Ok(b) => b,
            Err(msg) => {
                let src = self.source(t.span);
                self.emit(None, Diagnostic::warning("graphics_option", format!("{msg}; the content is set untransformed"), vec![src]));
                content
            }
        };
        self.recs.push(BoxRec::Transform(Rc::new(TransformRec { block, matrix: b.matrix, span: t.span })));
        let run = pl::GlyphRun { font: MATH_SENTINEL, size, glyphs: Vec::new(), width: b.width, height: b.height, depth: b.depth, source: t.span.start..t.span.end };
        Some((run, self.recs.len() - 1))
    }

    /// A `\resizebox` size: `!` (None), a factor of the content's `\width`,
    /// `\height`, `\depth`, `\totalheight` or of `\baselineskip`, or a
    /// dimension.
    fn box_dimen(&self, raw: &str, env: &LengthEnv, (w, h, d): (f64, f64, f64)) -> Result<Option<f64>, ()> {
        let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
        if compact == "!" {
            return Ok(None);
        }
        for (name, value) in [("\\totalheight", h + d), ("\\height", h), ("\\depth", d), ("\\width", w), ("\\baselineskip", self.style.baselineskip_pt)] {
            if let Some(f) = compact.strip_suffix(name) {
                let factor = match f {
                    "" | "+" => 1.0,
                    "-" => -1.0,
                    f => f.parse::<f64>().map_err(|_| ())?,
                };
                return Ok(Some(factor * value));
            }
        }
        graphics::parse_dimen(&compact, env).map(Some).ok_or(())
    }
}

/// An affine map of big points, y down: `(x, y) -> (a x + c y + e, b x + d y + f)`.
pub type Affine = [f64; 6];

fn apply(a: &Affine, x: f64, y: f64) -> (f64, f64) {
    (a[0] * x + a[2] * y + a[4], a[1] * x + a[3] * y + a[5])
}

/// The line-local map (big points, y down) of items set in a box's own
/// line-local coordinates, for a box matrix `m` (TeX points, y up) whose
/// box starts at `x0` TeX points on the line.
pub fn line_map(m: &[f64; 6], x0: f64) -> Affine {
    let k = BP_PER_PT;
    [m[0], -m[1], -m[2], m[3], k * (x0 + m[4]), -k * m[5]]
}

fn bbox(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    points.iter().fold((f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY), |(x0, y0, x1, y1), &(x, y)| (x0.min(x), y0.min(y), x1.max(x), y1.max(y)))
}

fn map_rect(r: &Rect, a: &Affine) -> Rect {
    let (x, y, w, h) = (r.x.to_bp(), r.top.to_bp(), r.width.to_bp(), r.height.to_bp());
    let (x0, y0, x1, y1) = bbox(&[apply(a, x, y), apply(a, x + w, y), apply(a, x, y + h), apply(a, x + w, y + h)]);
    Rect { x: Tick::from_bp(x0), top: Tick::from_bp(y0), width: Tick::from_bp(x1 - x0), height: Tick::from_bp(y1 - y0) }
}

fn map_caret(c: &Caret, a: &Affine) -> Caret {
    let (x, top, h) = (c.x.to_bp(), c.top.to_bp(), c.height.to_bp());
    let (p0, p1) = (apply(a, x, top), apply(a, x, top + h));
    Caret { text_byte: c.text_byte, x: Tick::from_bp(p0.0), top: Tick::from_bp(p0.1.min(p1.1)), height: Tick::from_bp((p1.1 - p0.1).abs()) }
}

fn map_cmd(c: &PathCmd, a: &Affine) -> PathCmd {
    let p = |x: Tick, y: Tick| {
        let (x, y) = apply(a, x.to_bp(), y.to_bp());
        (Tick::from_bp(x), Tick::from_bp(y))
    };
    match *c {
        PathCmd::Move(x, y) => {
            let (x, y) = p(x, y);
            PathCmd::Move(x, y)
        }
        PathCmd::Line(x, y) => {
            let (x, y) = p(x, y);
            PathCmd::Line(x, y)
        }
        PathCmd::Cubic(x1, y1, x2, y2, x3, y3) => {
            let ((x1, y1), (x2, y2), (x3, y3)) = (p(x1, y1), p(x2, y2), p(x3, y3));
            PathCmd::Cubic(x1, y1, x2, y2, x3, y3)
        }
        PathCmd::Close => PathCmd::Close,
    }
}

fn is_identity(m: &[f64; 4]) -> bool {
    (m[0] - 1.0).abs() < 1e-9 && m[1].abs() < 1e-9 && m[2].abs() < 1e-9 && (m[3] - 1.0).abs() < 1e-9
}

/// An image item for the unit-square transform `t` (big points, y down):
/// the box encloses what is painted (the clip part, or the whole square).
fn image_item(t: [f64; 6], clip: Option<[f64; 4]>, resource: Rc<ImageResource>, provenance: Provenance) -> display::Image {
    let c = clip.unwrap_or([0.0, 0.0, 1.0, 1.0]);
    let (x0, y0, x1, y1) = bbox(&[apply(&t, c[0], c[1]), apply(&t, c[2], c[1]), apply(&t, c[0], c[3]), apply(&t, c[2], c[3])]);
    display::Image { x: Tick::from_bp(x0), top: Tick::from_bp(y0), width: Tick::from_bp(x1 - x0), height: Tick::from_bp(y1 - y0), transform: t, resource, provenance, clip }
}

/// One display item mapped by `a`.
pub fn transform_item(item: &display::Item, a: &Affine) -> display::Item {
    match item {
        display::Item::GlyphRun(r) => {
            let mut r = r.clone();
            let det = a[0] * a[3] - a[1] * a[2];
            let k = det.abs().sqrt();
            let lin = if k > 0.0 { [a[0] / k, a[1] / k, a[2] / k, a[3] / k] } else { [1.0, 0.0, 0.0, 1.0] };
            let e = r.glyph_transform.unwrap_or([1.0, 0.0, 0.0, 1.0]);
            let composed = [lin[0] * e[0] + lin[2] * e[1], lin[1] * e[0] + lin[3] * e[1], lin[0] * e[2] + lin[2] * e[3], lin[1] * e[2] + lin[3] * e[3]];
            r.glyph_transform = (!is_identity(&composed)).then_some(composed);
            if k > 0.0 {
                r.font_size = Tick::from_bp(r.font_size.to_bp() * k);
            }
            for g in &mut r.glyphs {
                let (x, y) = apply(a, g.origin_x.to_bp(), g.baseline_y.to_bp());
                let (ax, ay) = (g.advance_x.to_bp(), g.advance_y.to_bp());
                g.origin_x = Tick::from_bp(x);
                g.baseline_y = Tick::from_bp(y);
                g.advance_x = Tick::from_bp(a[0] * ax + a[2] * ay);
                g.advance_y = Tick::from_bp(a[1] * ax + a[3] * ay);
            }
            for c in &mut r.clusters {
                c.hit_rect = map_rect(&c.hit_rect, a);
                c.carets.first = map_caret(&c.carets.first, a);
                if let Some(l) = &mut c.carets.last {
                    *l = map_caret(l, a);
                }
            }
            display::Item::GlyphRun(r)
        }
        display::Item::Rule(rule) => {
            if a[1] == 0.0 && a[2] == 0.0 {
                let rect = map_rect(&Rect { x: rule.x, top: rule.top, width: rule.width, height: rule.height }, a);
                display::Item::Rule(display::Rule { x: rect.x, top: rect.top, width: rect.width.max(Tick(1)), height: rect.height.max(Tick(1)), paint: rule.paint, provenance: rule.provenance.clone() })
            } else {
                let (x, y, w, h) = (rule.x, rule.top, Tick(rule.x.0 + rule.width.0), Tick(rule.top.0 + rule.height.0));
                let commands = vec![PathCmd::Move(x, y), PathCmd::Line(w, y), PathCmd::Line(w, h), PathCmd::Line(x, h), PathCmd::Close].iter().map(|c| map_cmd(c, a)).collect();
                display::Item::Path(PathItem { op: PathPaintOp::Fill { even_odd: false }, commands, clips: Vec::new(), paint: rule.paint, provenance: rule.provenance.clone() })
            }
        }
        display::Item::Path(p) => {
            let mut p = p.clone();
            p.commands = p.commands.iter().map(|c| map_cmd(c, a)).collect();
            p.clips = p.clips.iter().map(|clip| ClipPath { commands: clip.commands.iter().map(|c| map_cmd(c, a)).collect(), even_odd: clip.even_odd }).collect();
            display::Item::Path(p)
        }
        display::Item::Image(im) => {
            let t = im.transform;
            let n = [a[0] * t[0] + a[2] * t[1], a[1] * t[0] + a[3] * t[1], a[0] * t[2] + a[2] * t[3], a[1] * t[2] + a[3] * t[3], a[0] * t[4] + a[2] * t[5] + a[4], a[1] * t[4] + a[3] * t[5] + a[5]];
            display::Item::Image(image_item(n, im.clip, im.resource.clone(), im.provenance.clone()))
        }
    }
}

/// The items of an inline graphic whose box starts at `run.x` on the line.
pub fn graphic_items(run: &pl::PositionedRun, g: &GraphicRec, recs: &[BoxRec], source_of: &dyn Fn(Span) -> SourceRange, items: &mut Vec<display::Item>, used: &mut BTreeMap<Rc<str>, Rc<LoadedFace>>) {
    let k = BP_PER_PT;
    let m = g.gbox.matrix;
    if let Some(resource) = &g.resource {
        let t = [k * m[0], -k * m[1], k * m[2], -k * m[3], k * (run.x + m[4]), -k * m[5]];
        items.push(display::Item::Image(image_item(t, g.clip, resource.clone(), Provenance::Source(source_of(g.span)))));
    }
    let Some(d) = &g.draft else { return };
    if d.width <= 0.0 || d.height <= 0.0 {
        return;
    }
    let f = [d.frame[0] / d.width, d.frame[1] / d.width, d.frame[2] / d.height, d.frame[3] / d.height, d.frame[4], d.frame[5]];
    let a = line_map(&f, run.x);
    let (w, h, r) = (d.width, d.height, DRAFT_RULE);
    for (x, y, rw, rh) in [(0.0, 0.0, r, h), (w - r, 0.0, r, h), (0.0, h - r, w, r), (0.0, 0.0, w, r)] {
        let rule = display::Rule {
            x: Tick::from_tex_pt(x),
            top: Tick::from_tex_pt(-(y + rh)),
            width: Tick::from_tex_pt(rw),
            height: Tick::from_tex_pt(rh),
            paint: Paint::BLACK,
            provenance: Provenance::Source(source_of(g.span)),
        };
        items.push(transform_item(&display::Item::Rule(rule), &a));
    }
    if let Some((text_run, rec, tx, raise)) = &d.text {
        if let BoxRec::Text { face, size, text, style, clusters, glyphs, height, depth, .. } = &recs[*rec] {
            used.entry(face.font_id.clone()).or_insert_with(|| face.clone());
            let positioned = super::position_run(text_run, *tx, 0.0);
            if let Some(item) = super::text_item(&positioned, face, *size, text, clusters, glyphs, *height, *depth, *raise, source_of, Paint::of(style.color)) {
                items.push(transform_item(&item, &a));
            }
        }
    }
}

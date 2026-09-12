//! Paragraph assembly, vertical list and page builder.
//!
//! Paragraphs become Knuth–Plass node lists (boxes for words and inline
//! math, TeX interword glue with space factors, forced breaks); the resulting
//! lines, headings and displays become a vertical list of boxes, glue and
//! penalties with TeX's interline-glue rule (`\baselineskip`, `\lineskip`,
//! `\lineskiplimit`), `\parskip`, `\@startsection` skips and display skips.
//! The page builder then breaks that list into `\vsize`-high pages with
//! `\topskip`, club/widow penalties and `\raggedbottom` (article default),
//! discarding glue and penalties at the top of each page as TeX does.

use std::ops::Range;
use std::rc::Rc;

use flashtex_compiler::{DocumentId, Span};

use crate::adapter::{Block, Doc, Item, ParaPart, TextStyle, Word};
use crate::display::Diagnostic;
use crate::fonts::{FontSet, LoadedFace, Role};
use crate::linebreak::{self, Node, Params, INF_PENALTY};
use crate::mathlayout::{self, MItem, MathContext, MathStyle, ParamSource};
use crate::params;
use crate::shape::{SGlyph, Shaper};
use crate::style::{Skip, Stylesheet};

/// A run of glyphs in one face at one size, positioned on a page.
#[derive(Debug, Clone)]
pub struct PlacedRun {
    pub face: Rc<LoadedFace>,
    pub size_pt: f64,
    pub x: f64,
    pub baseline_y: f64,
    pub text: String,
    pub clusters: Vec<PlacedCluster>,
    pub style: TextStyle,
    /// Height/depth of the run's glyphs (points).
    pub height: f64,
    pub depth: f64,
}

#[derive(Debug, Clone)]
pub struct PlacedCluster {
    pub text_range: Range<usize>,
    pub glyphs: Vec<SGlyph>,
    /// Offset of the cluster origin from the run origin, points.
    pub dx: f64,
    pub width: f64,
    pub source: Span,
}

#[derive(Debug, Clone)]
pub struct PlacedRule {
    pub x: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub source: Span,
}

#[derive(Debug, Clone)]
pub enum Placed {
    Run(PlacedRun),
    Rule(PlacedRule),
}

impl Placed {
    fn shift(&mut self, dx: f64, dy: f64) {
        match self {
            Placed::Run(r) => {
                r.x += dx;
                r.baseline_y += dy;
            }
            Placed::Rule(r) => {
                r.x += dx;
                r.top += dy;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LaidPage {
    pub number: u32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub items: Vec<Placed>,
}

pub struct Laid {
    pub pages: Vec<LaidPage>,
}

/// A vertical-list box: content positioned relative to (left edge, baseline).
#[derive(Debug, Clone)]
struct VBox {
    height: f64,
    depth: f64,
    items: Vec<Placed>,
}

#[derive(Debug, Clone)]
enum V {
    Box(VBox),
    Glue(Skip),
    Penalty(i32),
}

pub struct Context<'a> {
    pub fonts: &'a FontSet,
    pub style: &'a Stylesheet,
    pub shaper: Shaper,
    pub diagnostics: Vec<Diagnostic>,
    pub path: &'a str,
    substitutions: Vec<String>,
    refused: Vec<String>,
    list: Vec<V>,
    /// `\prevdepth`: depth of the last box, or None right after a non-box.
    prev_depth: Option<f64>,
    /// Club penalty for the next paragraph (`\@afterheading` raises it).
    next_club: i32,
}

impl<'a> Context<'a> {
    pub fn new(fonts: &'a FontSet, style: &'a Stylesheet, path: &'a str) -> Context<'a> {
        Context {
            fonts,
            style,
            shaper: Shaper::new(),
            diagnostics: Vec::new(),
            path,
            substitutions: Vec::new(),
            refused: Vec::new(),
            list: Vec::new(),
            prev_depth: None,
            next_club: style.clubpenalty,
        }
    }

    fn text_face(&mut self, style: TextStyle, size: f64) -> Rc<LoadedFace> {
        let r = self.fonts.resolve(
            self.style.family,
            Role::Text {
                bold: style.bold,
                italic: style.italic,
            },
            size,
        );
        if let Some(s) = r.substituted {
            if !self.substitutions.contains(&s) {
                self.substitutions.push(s);
            }
        }
        r.face
    }

    fn design_size(face: &LoadedFace) -> u32 {
        // `lmroman12-regular` -> 12; Core 14 -> 10.
        face.font_id
            .trim_start_matches("lmroman")
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(10)
    }

    /// Interword glue for `style` at `size` with TeX space factor `f`.
    fn space_glue(&mut self, style: TextStyle, size: f64, factor: u32) -> (f64, f64, f64) {
        let face = self.text_face(style, size);
        let p = params::text_params(self.style.family, style.bold, style.italic, Self::design_size(&face)).at(size);
        let f = f64::from(factor);
        if factor >= 2000 {
            (p.space + p.extra_space, p.stretch * f / 1000.0, p.shrink * 1000.0 / f)
        } else if factor == 1000 || factor == 0 {
            (p.space, p.stretch, p.shrink)
        } else {
            (p.space, p.stretch * f / 1000.0, p.shrink * 1000.0 / f)
        }
    }

    fn ex(&mut self) -> f64 {
        let face = self.text_face(TextStyle::default(), self.style.body_size_pt);
        params::text_params(self.style.family, false, false, Self::design_size(&face))
            .at(self.style.body_size_pt)
            .x_height
    }

    // ----------------------------------------------------------- vertical list

    fn push_box(&mut self, b: VBox) {
        // Interline glue (TeX §679): baselineskip minus prev depth and height,
        // or lineskip when that would be less than lineskiplimit.
        if let Some(pd) = self.prev_depth {
            let bs = self.style.baselineskip_pt;
            let g = bs - pd - b.height;
            let glue = if g < self.style.lineskiplimit_pt {
                self.style.lineskip_pt
            } else {
                g
            };
            self.list.push(V::Glue(Skip::fixed(glue)));
        }
        self.prev_depth = Some(b.depth);
        self.list.push(V::Box(b));
    }

    fn push_glue(&mut self, s: Skip) {
        self.list.push(V::Glue(s));
    }

    fn push_penalty(&mut self, p: i32) {
        self.list.push(V::Penalty(p));
    }

    fn parskip(&mut self) {
        if !self.list.is_empty() {
            let s = self.style.parskip;
            self.push_glue(s);
        }
    }

    // --------------------------------------------------------------- shaping

    /// Shapes a word into placed runs relative to the word's origin; returns
    /// (runs, width, height, depth).
    fn shape_word(&mut self, word: &Word, size: f64) -> (Vec<Placed>, f64, f64, f64) {
        let mut runs = Vec::new();
        let mut x = 0.0;
        let mut height: f64 = 0.0;
        let mut depth: f64 = 0.0;
        for seg in &word.segments {
            let face = self.text_face(seg.style, size);
            let shaped = self.shaper.shape(&face, size, &seg.text);
            let shaped = crate::shape::at_size(&shaped, size);
            if let Some(r) = &shaped.refused {
                if !self.refused.contains(r) {
                    self.refused.push(r.clone());
                }
            }
            // Map cluster text ranges (bytes of seg.text) to source spans via
            // the per-char table.
            let mut char_starts: Vec<(usize, usize)> = Vec::new(); // (byte offset, char index)
            for (ci, (off, _)) in seg.text.char_indices().enumerate() {
                char_starts.push((off, ci));
            }
            let mut clusters = Vec::new();
            let mut dx = 0.0;
            for c in &shaped.clusters {
                let first = char_starts.iter().find(|(o, _)| *o == c.text_range.start).map(|(_, i)| *i).unwrap_or(0);
                let last = char_starts
                    .iter()
                    .filter(|(o, _)| *o < c.text_range.end)
                    .map(|(_, i)| *i)
                    .last()
                    .unwrap_or(first);
                let s0 = seg.chars.get(first).map(|c| c.start).unwrap_or(0);
                let e0 = seg.chars.get(last).map(|c| c.end).unwrap_or(s0);
                let doc = seg.chars.first().map(|_| DocumentId::default()).unwrap_or_default();
                let w = face.pt(c.advance_units(), size);
                clusters.push(PlacedCluster {
                    text_range: c.text_range.clone(),
                    glyphs: c.glyphs.clone(),
                    dx,
                    width: w,
                    source: Span::in_document(seg_doc(seg).unwrap_or(doc), s0, e0),
                });
                dx += w;
            }
            height = height.max(shaped.height_pt);
            depth = depth.max(shaped.depth_pt);
            runs.push(Placed::Run(PlacedRun {
                face: face.clone(),
                size_pt: size,
                x,
                baseline_y: 0.0,
                text: seg.text.clone(),
                clusters,
                style: seg.style,
                height: shaped.height_pt,
                depth: shaped.depth_pt,
            }));
            x += shaped.width_pt;
        }
        (runs, x, height, depth)
    }

    fn math_box(&mut self, list: &flashtex_compiler::math::MathList, style: MathStyle) -> (Vec<Placed>, f64, f64, f64) {
        let mut mctx = MathContext {
            fonts: self.fonts,
            shaper: &self.shaper,
            style: self.style,
            source: ParamSource::Tfm,
            substitutions: Vec::new(),
        };
        let b = mathlayout::layout(&mut mctx, list, style);
        for s in mctx.substitutions {
            if !self.substitutions.contains(&s) {
                self.substitutions.push(s);
            }
        }
        let mut out = Vec::new();
        for it in b.items {
            match it {
                MItem::Glyph(g) => {
                    let adv_units = (g.advance_pt / g.size_pt * f64::from(g.face.units_per_em)).round() as i32;
                    let b = g.face.bounds(g.gid, g.text.chars().next());
                    out.push(Placed::Run(PlacedRun {
                        face: g.face.clone(),
                        size_pt: g.size_pt,
                        x: g.x,
                        baseline_y: g.dy,
                        text: g.text.clone(),
                        clusters: vec![PlacedCluster {
                            text_range: 0..g.text.len(),
                            glyphs: vec![SGlyph {
                                gid: g.gid,
                                advance: adv_units,
                                x_offset: 0,
                                y_offset: 0,
                                y_max: b.y_max,
                                y_min: b.y_min,
                                x_max: b.x_max,
                            }],
                            dx: 0.0,
                            width: g.advance_pt,
                            source: g.span,
                        }],
                        style: TextStyle::default(),
                        height: g.height,
                        depth: g.depth,
                    }));
                }
                MItem::Rule(r) => out.push(Placed::Rule(PlacedRule {
                    x: r.x,
                    top: r.top,
                    width: r.width,
                    height: r.height,
                    source: r.span,
                })),
            }
        }
        (out, b.width, b.height, b.depth)
    }

    // ------------------------------------------------------------ paragraphs

    /// Sets `items` as a paragraph at `size`, appending line boxes to the
    /// vertical list. Returns the width of the last line (for display skips).
    fn set_lines(&mut self, items: &[Item], size: f64, indent: f64, club: i32, widow: i32, interline: i32) -> f64 {
        struct BoxContent {
            placed: Vec<Placed>,
            height: f64,
            depth: f64,
        }
        let mut nodes: Vec<Node> = Vec::new();
        let mut boxes: Vec<BoxContent> = Vec::new();
        let mut prev_style = TextStyle::default();
        for item in items {
            match item {
                Item::Word(w) => {
                    let (placed, width, height, depth) = self.shape_word(w, size);
                    prev_style = w.segments.last().map(|s| s.style).unwrap_or_default();
                    nodes.push(Node::Box {
                        width,
                        payload: boxes.len(),
                    });
                    boxes.push(BoxContent { placed, height, depth });
                }
                Item::Space { style, factor, no_break } => {
                    let st = if *style == TextStyle::default() { prev_style } else { *style };
                    let (w, st_, sh) = self.space_glue(st, size, *factor);
                    if *no_break {
                        nodes.push(Node::Penalty {
                            penalty: INF_PENALTY,
                            width: 0.0,
                            flagged: false,
                        });
                    }
                    nodes.push(Node::Glue {
                        width: w,
                        stretch: st_,
                        shrink: sh,
                        fil: false,
                    });
                }
                Item::Math { list, .. } => {
                    let (placed, width, height, depth) = self.math_box(list, MathStyle::Text);
                    nodes.push(Node::Box {
                        width,
                        payload: boxes.len(),
                    });
                    boxes.push(BoxContent { placed, height, depth });
                }
                Item::LineBreak => {
                    nodes.push(Node::Penalty {
                        penalty: INF_PENALTY,
                        width: 0.0,
                        flagged: false,
                    });
                    nodes.push(Node::Glue {
                        width: 0.0,
                        stretch: 0.0,
                        shrink: 0.0,
                        fil: true,
                    });
                    nodes.push(Node::Penalty {
                        penalty: -INF_PENALTY,
                        width: 0.0,
                        flagged: false,
                    });
                }
            }
        }
        // Remove trailing glue (TeX discards it) and close the paragraph.
        while matches!(nodes.last(), Some(Node::Glue { .. })) {
            nodes.pop();
        }
        if nodes.is_empty() {
            return 0.0;
        }
        nodes.push(Node::Penalty {
            penalty: INF_PENALTY,
            width: 0.0,
            flagged: false,
        });
        nodes.push(Node::Glue {
            width: 0.0,
            stretch: 0.0,
            shrink: 0.0,
            fil: true,
        });
        nodes.push(Node::Penalty {
            penalty: -INF_PENALTY,
            width: 0.0,
            flagged: false,
        });
        let p = Params {
            line_width: self.style.text_width_pt,
            first_indent: indent,
            tolerance: self.style.tolerance,
            pretolerance: self.style.pretolerance,
            linepenalty: self.style.linepenalty,
            adjdemerits: self.style.adjdemerits,
        };
        let lines = linebreak::break_paragraph(&nodes, &p);
        let n = lines.len();
        let mut last_width = 0.0;
        for (li, line) in lines.iter().enumerate() {
            let left = if li == 0 { indent } else { 0.0 };
            let mut items = Vec::new();
            let mut height: f64 = 0.0;
            let mut depth: f64 = 0.0;
            let mut right = left;
            for (payload, x) in linebreak::place(&nodes, line) {
                let b = &boxes[payload];
                for placed in &b.placed {
                    let mut pl = placed.clone();
                    pl.shift(left + x, 0.0);
                    items.push(pl);
                }
                height = height.max(b.height);
                depth = depth.max(b.depth);
                if let Node::Box { width, .. } = nodes.iter().filter(|n| matches!(n, Node::Box { .. })).nth(payload).unwrap() {
                    right = right.max(left + x + width);
                }
            }
            last_width = right;
            if li > 0 {
                let mut pen = interline;
                if li == 1 {
                    pen += club;
                }
                if li == n - 1 {
                    pen += widow;
                }
                if pen != 0 {
                    self.push_penalty(pen);
                }
            }
            self.push_box(VBox { height, depth, items });
        }
        last_width
    }

    fn display(&mut self, list: &flashtex_compiler::math::MathList, prev_line_width: Option<f64>) {
        let (placed, width, height, depth) = self.math_box(list, MathStyle::Display);
        let line_width = self.style.text_width_pt;
        let left = ((line_width - width) / 2.0).max(0.0);
        let quad = self.style.body_size_pt;
        let short = match prev_line_width {
            Some(w) => w + 2.0 * quad <= left,
            None => false,
        };
        let above = if short {
            self.style.abovedisplayshortskip
        } else {
            self.style.abovedisplayskip
        };
        let below = if short {
            self.style.belowdisplayshortskip
        } else {
            self.style.belowdisplayskip
        };
        self.push_penalty(INF_PENALTY); // \predisplaypenalty
        self.push_glue(above);
        let mut items = Vec::new();
        for mut p in placed {
            p.shift(left, 0.0);
            items.push(p);
        }
        self.push_box(VBox { height, depth, items });
        self.push_glue(below);
    }

    fn heading(&mut self, level: u8, items: &[Item]) {
        let h = self.style.heading(level);
        let ex = self.ex();
        if !self.list.is_empty() {
            self.push_penalty(self.style.secpenalty);
            self.push_glue(Skip::new(h.before_ex.0 * ex, h.before_ex.1 * ex, h.before_ex.2 * ex));
        }
        self.parskip();
        // Heading font: bold at the heading size; the heading paragraph has
        // its own baselineskip.
        let saved_bs = self.style.baselineskip_pt;
        let styled: Vec<Item> = items
            .iter()
            .map(|it| match it {
                Item::Word(w) => Item::Word(Word {
                    segments: w
                        .segments
                        .iter()
                        .map(|s| crate::adapter::Segment {
                            text: s.text.clone(),
                            chars: s.chars.clone(),
                            style: TextStyle {
                                bold: h.bold || s.style.bold,
                                italic: s.style.italic,
                            },
                        })
                        .collect(),
                }),
                Item::Space { style, factor, no_break } => Item::Space {
                    style: TextStyle {
                        bold: h.bold || style.bold,
                        italic: style.italic,
                    },
                    factor: *factor,
                    no_break: *no_break,
                },
                other => other.clone(),
            })
            .collect();
        let sheet = Stylesheet {
            baselineskip_pt: h.baselineskip_pt,
            ..self.style.clone()
        };
        let outer = std::mem::replace(&mut self.style, Box::leak(Box::new(sheet)));
        self.set_lines(&styled, h.size_pt, 0.0, INF_PENALTY, INF_PENALTY, INF_PENALTY);
        self.style = outer;
        let _ = saved_bs;
        self.push_penalty(INF_PENALTY);
        self.push_glue(Skip::new(h.after_ex.0 * ex, h.after_ex.1 * ex, 0.0));
        self.next_club = INF_PENALTY; // \@afterheading
    }

    fn paragraph(&mut self, parts: &[ParaPart], indent: bool) {
        let size = self.style.body_size_pt;
        let club = self.next_club;
        self.next_club = self.style.clubpenalty;
        self.parskip();
        let mut first = true;
        let mut last_width: Option<f64> = None;
        for part in parts {
            match part {
                ParaPart::Lines(items) => {
                    let ind = if first && indent { self.style.parindent_pt } else { 0.0 };
                    let c = if first { club } else { self.style.clubpenalty };
                    let w = self.set_lines(items, size, ind, c, self.style.widowpenalty, 0);
                    last_width = Some(w);
                }
                ParaPart::Display { list, .. } => {
                    self.display(list, last_width);
                    last_width = None;
                }
            }
            first = false;
        }
    }
}

fn seg_doc(seg: &crate::adapter::Segment) -> Option<DocumentId> {
    seg.doc
}

/// Builds the vertical list for `doc` and breaks it into pages.
pub fn build(ctx: &mut Context, doc: &Doc) -> Laid {
    for block in &doc.blocks {
        match block {
            Block::Heading { level, items } => ctx.heading(*level, items),
            Block::Paragraph { parts, indent } => ctx.paragraph(parts, *indent),
        }
    }
    for s in std::mem::take(&mut ctx.substitutions) {
        ctx.diagnostics.push(Diagnostic::warning(
            "font_substituted",
            format!("Latin Modern face unavailable ({s}); Times metrics were used instead"),
            None,
        ));
    }
    for r in std::mem::take(&mut ctx.refused) {
        ctx.diagnostics.push(Diagnostic::error(
            "unsupported_script",
            format!("text could not be shaped: {r}"),
            None,
        ));
    }
    let list = std::mem::take(&mut ctx.list);
    break_pages(ctx.style, list)
}

/// TeX's page builder for the `\raggedbottom` case: cost is the penalty at
/// each legal break (badness is zero with `fil` stretch); ties go to the
/// later break; a forced penalty ejects; the first overfull item forces the
/// best break seen so far.
fn break_pages(style: &Stylesheet, list: Vec<V>) -> Laid {
    let vsize = style.text_height_pt;
    let mut pages: Vec<LaidPage> = Vec::new();
    let mut page_start = 0usize;
    let n = list.len();
    let mut i = 0usize;
    while page_start < n {
        // Skip discardables at the top of the page.
        while page_start < n && !matches!(list[page_start], V::Box(_)) {
            page_start += 1;
        }
        if page_start >= n {
            break;
        }
        let mut total = 0.0; // height used so far (without the pending box depth)
        let mut best: Option<(usize, i32)> = None; // (break index, cost)
        let mut first_box = true;
        let mut prev_was_box = false;
        let mut break_at: Option<usize> = None;
        i = page_start;
        while i < n {
            match &list[i] {
                V::Box(b) => {
                    let add = if first_box {
                        style.topskip_pt.max(b.height)
                    } else {
                        b.height
                    };
                    if total + add > vsize + 1e-6 && !first_box {
                        // Overfull: take the best break, or break right here.
                        break_at = Some(best.map(|(at, _)| at).unwrap_or(i));
                        break;
                    }
                    total += add + b.depth;
                    first_box = false;
                    prev_was_box = true;
                }
                V::Glue(g) => {
                    if prev_was_box {
                        let cost = 0;
                        if best.map_or(true, |(_, c)| cost <= c) {
                            best = Some((i, cost));
                        }
                    }
                    total += g.natural;
                    prev_was_box = false;
                }
                V::Penalty(p) => {
                    if *p < INF_PENALTY {
                        let cost = *p;
                        if *p <= -INF_PENALTY {
                            break_at = Some(i);
                            break;
                        }
                        if best.map_or(true, |(_, c)| cost <= c) {
                            best = Some((i, cost));
                        }
                    }
                    prev_was_box = false;
                }
            }
            i += 1;
        }
        let end = break_at.unwrap_or(n);
        // Depth handling: a box's depth may hang below vsize (maxdepth); the
        // overfull test above already excludes depth of the last box.
        let mut items = Vec::new();
        let mut y = style.margin_top_pt;
        let mut first = true;
        let mut prev_depth = 0.0;
        for v in &list[page_start..end] {
            match v {
                V::Box(b) => {
                    let baseline = if first {
                        y + style.topskip_pt.max(b.height)
                    } else {
                        y + b.height
                    };
                    for it in &b.items {
                        let mut p = it.clone();
                        p.shift(style.margin_left_pt, baseline);
                        items.push(p);
                    }
                    y = baseline + b.depth;
                    prev_depth = b.depth;
                    first = false;
                }
                V::Glue(g) => {
                    if !first {
                        y += g.natural;
                    }
                }
                V::Penalty(_) => {}
            }
        }
        let _ = prev_depth;
        pages.push(LaidPage {
            number: pages.len() as u32 + 1,
            width_pt: style.page_width_pt,
            height_pt: style.page_height_pt,
            items,
        });
        if end >= n {
            break;
        }
        page_start = end;
    }
    if pages.is_empty() {
        pages.push(LaidPage {
            number: 1,
            width_pt: style.page_width_pt,
            height_pt: style.page_height_pt,
            items: Vec::new(),
        });
    }
    Laid { pages }
}

//! A small TeX "stomach" for plain-text LaTeX articles.
//!
//! mouth: `tex-expansion` (with a format prelude of marker macros, seam S1)
//! chars: `tex-text-encoding` (TFM lig/kern, space factor, `\accent`)
//! lines: [`crate::linebreak`] (Knuth–Plass + Liang patterns) packed by `tex-boxes::hpack`
//! vlist: `page-builder::VListBuilder` (interline glue, club/widow penalties, `\addvspace`)
//! pages: `page-builder::PageBuilder` + `LatexOutput`, frame and headings from `class-geometry`
//! glyphs: `tex-boxes::ship_out`

use std::sync::OnceLock;

use flashtex_class_geometry::{resolve, BaseSize, DocumentSetup, FontSize, HeadingSpec, ResolvedDocument, Sp};
use flashtex_page_builder::latex::{clearpage, newpage, LatexOutput, PriorityPenalties};
use flashtex_page_builder::node::{BoxNode as PBox, GlueKind as PGlueKind, GlueSpec as PGlue, Node as VNode, Order};
use flashtex_page_builder::page::{InsertClass, PageBuilder, PageParams};
use flashtex_page_builder::scaled::MAX_DIMEN;
use flashtex_page_builder::vlist::{InterlineParams, Line, LinePenalties, VListBuilder};
use flashtex_paragraph_layout::liang::LiangHyphenator;
use flashtex_tex_boxes::node::{BoxNode, GlueOrder, GlueSpec, KernKind, ListKind, Node};
use flashtex_tex_boxes::pack::{hpack, PackOrigin, PackParams, PackSpec};
use flashtex_tex_boxes::shipout::{ship_out, ShipEvent};
use flashtex_tex_expansion::{CatCode, Engine, Token, TokenKind};
use flashtex_tex_text_encoding::accent::make_accent;
use flashtex_tex_text_encoding::encoding::{declared, Declared, Encoding};
use flashtex_tex_text_encoding::sfcode::{adjust_space_factor, interword_glue, SfCodes};
use flashtex_tex_text_encoding::unicode::{classify, InputChar};

use crate::fonts::{ot1_cmr_tfm, Fonts, Series, Shape};
use crate::linebreak::{break_paragraph, run_nodes, BreakParams, BreakStats};

/// Marker macros that make argument extents visible to the stomach:
/// `tex-expansion` swallows grouping braces (seam S1).
pub const FORMAT: &str = r"\makeatletter
\def\@gobble#1{}
\def\documentclass#1#{\@gobble}
\def\usepackage#1#{\@gobble}
\def\section{\@ifstar{\POC@sS}{\POC@sN}}
\def\POC@sN#1{\POCheadsection #1\POCend}
\def\POC@sS#1{\POCheadsectionstar #1\POCend}
\def\subsection{\@ifstar{\POC@ssS}{\POC@ssN}}
\def\POC@ssN#1{\POCheadsubsection #1\POCend}
\def\POC@ssS#1{\POCheadsubsectionstar #1\POCend}
\def\subsubsection{\@ifstar{\POC@sssS}{\POC@sssN}}
\def\POC@sssN#1{\POCheadsubsubsection #1\POCend}
\def\POC@sssS#1{\POCheadsubsubsectionstar #1\POCend}
\def\emph#1{\POCemph #1\POCend}
\def\textbf#1{\POCbf #1\POCend}
\def\textit#1{\POCit #1\POCend}
\def\vspace{\@ifstar{\POC@vS}{\POC@vN}}
\def\POC@vN#1{\POCvspace #1\POCend}
\def\POC@vS#1{\POCvspacestar #1\POCend}
\makeatother
";

/// A positioned glyph: TFM name, size and code, reference point in sp
/// from the top-left corner of the PDF page.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    pub font: String,
    pub size: i32,
    pub code: u32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub number: usize,
    pub glyphs: Vec<Glyph>,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub pages: Vec<Page>,
    pub page_width: i32,
    pub page_height: i32,
    pub diagnostics: Vec<String>,
    pub stats: BreakStats,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct FontState {
    series: Series,
    shape: Shape,
    size: FontSize,
    id: u32,
}

enum Group {
    Font { saved: FontState, check_icr: bool },
    Heading { saved: FontState, baseline_skip: PGlue, spec: HeadingSpec },
    Vspace { star: bool, text: String },
}

fn hyphenator() -> &'static LiangHyphenator {
    static H: OnceLock<LiangHyphenator> = OnceLock::new();
    H.get_or_init(LiangHyphenator::english)
}

fn pg(g: flashtex_class_geometry::Glue) -> PGlue {
    PGlue::new(g.natural.0 as i32, g.stretch.0 as i32, g.shrink.0 as i32)
}

fn bglue(width: i32, stretch: i32, shrink: i32) -> GlueSpec {
    GlueSpec { width, stretch, shrink, ..GlueSpec::ZERO }
}

struct Stomach {
    fonts: Fonts,
    doc: ResolvedDocument,
    base: BaseSize,
    font: FontState,
    sf: i32,
    sfcodes: SfCodes,
    hlist: Option<Vec<Node>>,
    run: Vec<u8>,
    run_font: u32,
    vlist: VListBuilder,
    lines: Vec<BoxNode>,
    groups: Vec<Group>,
    hang_indent: i32,
    interline_penalty: i32,
    club_penalty: i32,
    nobreak: bool,
    afterindent: bool,
    afterheading_everypar: bool,
    counters: [i32; 3],
    diags: Vec<String>,
    stats: BreakStats,
}

impl Stomach {
    fn select_font(&mut self, series: Series, shape: Shape, size: FontSize) -> Result<FontState, String> {
        let sz = size.metrics(self.base).0 .0 as i32;
        let tfm = ot1_cmr_tfm(series, shape, sz).ok_or_else(|| format!("no OT1 cmr font for size {sz}sp"))?;
        let id = self.fonts.load(&tfm, sz)?;
        Ok(FontState { series, shape, size, id })
    }

    fn set_font(&mut self, series: Series, shape: Shape, size: FontSize) {
        match self.select_font(series, shape, size) {
            Ok(f) => {
                self.flush();
                self.font = f;
            }
            Err(e) => self.diag(e),
        }
    }

    fn diag(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        if !self.diags.contains(&m) {
            self.diags.push(m);
        }
    }

    fn hmode(&self) -> bool {
        self.hlist.is_some()
    }

    fn flush(&mut self) {
        if self.run.is_empty() {
            return;
        }
        let nodes = run_nodes(&self.fonts, self.run_font, &self.run, true);
        self.run.clear();
        if let Some(h) = self.hlist.as_mut() {
            h.extend(nodes);
        }
    }

    fn append(&mut self, n: Node) {
        self.flush();
        if let Some(h) = self.hlist.as_mut() {
            h.push(n);
        }
    }

    fn leavevmode(&mut self) {
        if !self.hmode() {
            self.new_graf(true);
        }
    }

    /// `new_graf` with LaTeX's para hooks and `\@afterheading`'s `\everypar`.
    fn new_graf(&mut self, indent: bool) {
        let parskip = pg(self.doc.params.parskip);
        self.vlist.param_glue(parskip, PGlueKind::ParSkip);
        self.vlist.param_glue(PGlue::ZERO, PGlueKind::ParSkip);
        let mut list = Vec::new();
        if indent {
            let mut b = BoxNode::null(ListKind::H);
            b.width = self.doc.params.parindent.0 as i32;
            list.push(Node::Box(b));
        }
        self.hlist = Some(list);
        self.sf = 1000;
        if self.afterheading_everypar {
            if self.nobreak {
                self.nobreak = false;
                self.club_penalty = 10000;
                if !self.afterindent {
                    if let Some(h) = self.hlist.as_mut() {
                        if matches!(h.last(), Some(Node::Box(_))) {
                            h.pop();
                        }
                    }
                }
            } else {
                self.club_penalty = 150;
                self.afterheading_everypar = false;
            }
        }
    }

    fn par(&mut self) {
        if !self.hmode() {
            return;
        }
        self.flush();
        let list = self.hlist.take().unwrap_or_default();
        if list.is_empty() {
            return;
        }
        let mut bp = BreakParams::latex(self.doc.params.textwidth.0 as i32);
        bp.hang_indent = self.hang_indent;
        let lines = break_paragraph(list, bp, &self.fonts, hyphenator(), &mut self.stats);
        let mut vlines = Vec::new();
        for l in lines {
            let id = self.lines.len() as u32;
            let hb = &l.hbox;
            let pb = PBox { vertical: false, width: hb.width, height: hb.height, depth: hb.depth, shift: hb.shift, id };
            self.lines.push(l.hbox);
            vlines.push(Line { hbox: pb, disc_break: l.disc_break, adjust: Vec::new() });
        }
        self.vlist.prev_graf = 0;
        self.vlist.paragraph(
            vlines,
            LinePenalties { inter_line: self.interline_penalty, club: self.club_penalty, final_widow: 150, broken: 100 },
        );
        self.hang_indent = 0;
    }

    fn space(&mut self) {
        if !self.hmode() {
            return;
        }
        let f = self.fonts.get(self.font.id);
        let g = interword_glue(f, self.sf);
        self.append(Node::glue(bglue(g.width, g.stretch, g.shrink)));
    }

    fn normal_space(&mut self) {
        let f = self.fonts.get(self.font.id);
        let g = bglue(f.space(), f.space_stretch(), f.space_shrink());
        self.append(Node::glue(g));
    }

    fn char_code(&mut self, code: u8) {
        self.leavevmode();
        if self.run_font != self.font.id {
            self.flush();
            self.run_font = self.font.id;
        }
        self.run.push(code);
        self.sf = adjust_space_factor(self.sf, self.sfcodes.get(code));
    }

    /// LaTeX `\add@accent` → `\accent` (§1123–§1125) in OT1.
    fn accent(&mut self, cmd: &str, base: u8) {
        let Some(Declared::Accent(slot)) = declared(Encoding::OT1, cmd) else {
            self.diag(format!("unsupported accent \\{cmd} in OT1"));
            self.char_code(base);
            return;
        };
        self.leavevmode();
        self.flush();
        let fid = self.font.id;
        let f = self.fonts.get(fid);
        let place = make_accent(f, slot, (f, base));
        let acc = Node::Char { font: fid, ch: slot as u32 };
        let acc_node = match place.shift {
            Some(shift) => {
                let packed =
                    hpack(vec![acc], PackSpec::NATURAL, false, &PackParams::default(), PackOrigin::default(), &self.fonts);
                let mut b = packed.node;
                b.shift = shift;
                Node::Box(b)
            }
            None => acc,
        };
        self.append(Node::Kern { width: place.kern_before, kind: KernKind::Accent });
        self.append(acc_node);
        self.append(Node::Kern { width: place.kern_after, kind: KernKind::Accent });
        self.append(Node::Char { font: fid, ch: base as u32 });
        self.sf = adjust_space_factor(1000, self.sfcodes.get(base));
    }

    fn unicode_char(&mut self, c: char) {
        match classify(c) {
            InputChar::Ascii(b) => self.char_code(b),
            InputChar::Declared { expansion, .. } => {
                let e = expansion.strip_prefix("\\@tabacckludge").unwrap_or(expansion.strip_prefix('\\').unwrap_or(""));
                let mut it = e.chars();
                match (it.next(), it.next(), it.next()) {
                    (Some(a), Some(b), None) if !a.is_alphabetic() && b.is_ascii_alphabetic() => {
                        self.accent(&a.to_string(), b as u8)
                    }
                    _ => self.diag(format!("unsupported input character {c} ({expansion})")),
                }
            }
            InputChar::Undeclared { message } => self.diag(message),
        }
    }

    /// `\/` (§1113).
    fn italic_correction(&mut self) {
        self.flush();
        let Some(h) = self.hlist.as_mut() else { return };
        let (font, ch) = match h.last() {
            Some(Node::Char { font, ch }) | Some(Node::Ligature { font, ch, .. }) => (*font, *ch),
            _ => return,
        };
        let italic = self.fonts.get(font).char_dims(ch as u8).map(|d| d.italic).unwrap_or(0);
        h.push(Node::Kern { width: italic, kind: KernKind::Explicit });
        self.sf = 1000;
    }

    /// `\maybe@ic` after a text font command's group.
    fn maybe_ic(&mut self, next: Option<&Token>) {
        if !self.hmode() || self.fonts.get(self.font.id).slant() > 0 {
            return;
        }
        if let Some(TokenKind::Char('.' | ',', _)) = next.map(|t| &t.kind) {
            return;
        }
        self.flush();
        // \sw@slant / \fix@penalty
        let h = self.hlist.as_mut().unwrap();
        let skip = match h.last() {
            Some(Node::Glue(g)) if g.spec.width != 0 || g.spec.stretch != 0 || g.spec.shrink != 0 => h.pop(),
            _ => None,
        };
        let pen = match h.last() {
            Some(Node::Penalty(p)) if *p != 0 => h.pop(),
            _ => None,
        };
        self.italic_correction();
        let h = self.hlist.as_mut().unwrap();
        h.extend(pen);
        h.extend(skip);
    }

    fn heading_begin(&mut self, name: &str, star: bool) {
        let Some(spec) = self.doc.heading(name).cloned() else {
            self.diag(format!("no heading spec for {name}"));
            return;
        };
        self.par();
        self.afterindent = spec.indent_after;
        if self.nobreak {
            self.afterheading_everypar = false;
        } else {
            self.vlist.addpenalty(-300, false, self.doc.params.maxdepth.0 as i32);
            self.vlist.addvspace(pg(spec.space_before()));
        }
        let numbered = !star && spec.numbered;
        if numbered {
            let lvl = (spec.level - 1) as usize;
            self.counters[lvl] += 1;
            for c in self.counters.iter_mut().skip(lvl + 1) {
                *c = 0;
            }
        }
        let saved = self.font;
        let baseline_skip = self.vlist.params.baseline_skip;
        let size = spec.size;
        self.set_font(Series::Bold, Shape::Upright, size);
        self.vlist.params.baseline_skip = PGlue::fixed(size.metrics(self.base).1 .0 as i32);
        // \@hangfrom{\hskip #3\relax\@svsec}
        let mut contents = vec![Node::glue(GlueSpec::fixed(spec.indent.0 as i32))];
        if numbered {
            let lvl = spec.level as usize;
            let num: Vec<String> = self.counters[..lvl].iter().map(|c| c.to_string()).collect();
            let text = num.join(".");
            contents.extend(run_nodes(&self.fonts, self.font.id, text.as_bytes(), false));
            let quad = self.fonts.get(self.font.id).quad();
            contents.push(Node::glue(GlueSpec::fixed(quad)));
        }
        let numbox =
            hpack(contents, PackSpec::NATURAL, false, &PackParams::default(), PackOrigin::default(), &self.fonts).node;
        self.hang_indent = numbox.width;
        self.groups.push(Group::Heading { saved, baseline_skip, spec });
        self.new_graf(false);
        self.append(Node::Box(numbox));
        self.sf = 1000;
        self.interline_penalty = 10000;
    }

    fn heading_end(&mut self, saved: FontState, baseline_skip: PGlue, spec: HeadingSpec) {
        self.par();
        self.font = saved;
        self.vlist.params.baseline_skip = baseline_skip;
        self.interline_penalty = 0;
        // \@xsect: \par \nobreak \vskip afterskip \@afterheading
        self.vlist.penalty(10000);
        self.vlist.vskip(pg(spec.afterskip));
        self.nobreak = true;
        self.afterheading_everypar = true;
    }

    fn parse_dimen(&self, s: &str) -> Option<(i32, Option<Order>)> {
        let s = s.trim();
        let split = s.find(|c: char| c.is_ascii_alphabetic())?;
        let (num, unit) = (s[..split].trim(), s[split..].trim());
        let f = self.fonts.get(self.font.id);
        match unit {
            "em" => Some((Sp(f.quad() as i64).scaled(num)?.0 as i32, None)),
            "ex" => Some((Sp(f.x_height() as i64).scaled(num)?.0 as i32, None)),
            "fil" | "fill" | "filll" => {
                let v = Sp::pt(1).scaled(num)?.0 as i32;
                Some((v, Some(if unit == "fil" { Order::Fil } else if unit == "fill" { Order::Fill } else { Order::Filll })))
            }
            _ => Some((Sp::parse(s)?.0 as i32, None)),
        }
    }

    fn parse_glue(&self, s: &str) -> Option<PGlue> {
        let s = s.trim();
        let (w, rest) = match s.find("plus").or_else(|| s.find("minus")) {
            Some(i) => (&s[..i], &s[i..]),
            None => (s, ""),
        };
        let mut g = PGlue::fixed(self.parse_dimen(w)?.0);
        let (plus, minus) = match rest.find("minus") {
            Some(i) => (&rest[..i], &rest[i + 5..]),
            None => (rest, ""),
        };
        if let Some(p) = plus.strip_prefix("plus") {
            let (v, o) = self.parse_dimen(p)?;
            g.stretch = v;
            g.stretch_order = o.unwrap_or(Order::Normal);
        }
        if !minus.trim().is_empty() {
            let (v, o) = self.parse_dimen(minus)?;
            g.shrink = v;
            g.shrink_order = o.unwrap_or(Order::Normal);
        }
        Some(g)
    }

    fn vspace_end(&mut self, star: bool, text: &str) {
        let Some(g) = self.parse_glue(text) else {
            self.diag(format!("cannot parse \\vspace{{{text}}}"));
            return;
        };
        if self.hmode() {
            self.diag("\\vspace in horizontal mode (\\vadjust) is not supported");
            return;
        }
        if star {
            let pd = self.vlist.prev_depth;
            self.vlist.hrule(None, 0, 0);
            self.vlist.penalty(10000);
            self.vlist.vskip(g);
            self.vlist.vskip(PGlue::ZERO);
            self.vlist.prev_depth = pd;
        } else {
            self.vlist.vskip(g);
            self.vlist.vskip(PGlue::ZERO);
        }
    }

    fn optional_int(toks: &[Token], i: &mut usize) -> Option<i32> {
        if !matches!(toks.get(*i).map(|t| &t.kind), Some(TokenKind::Char('[', _))) {
            return None;
        }
        let mut s = String::new();
        let mut j = *i + 1;
        while let Some(t) = toks.get(j) {
            j += 1;
            match t.kind {
                TokenKind::Char(']', _) => break,
                TokenKind::Char(c, _) => s.push(c),
                _ => {}
            }
        }
        *i = j;
        s.trim().parse().ok()
    }

    fn command(&mut self, name: &str, toks: &[Token], i: &mut usize) -> bool {
        match name {
            "par" => self.par(),
            "noindent" => {
                if !self.hmode() {
                    self.new_graf(false)
                }
            }
            "indent" => {
                if self.hmode() {
                    let mut b = BoxNode::null(ListKind::H);
                    b.width = self.doc.params.parindent.0 as i32;
                    self.append(Node::Box(b));
                } else {
                    self.new_graf(true)
                }
            }
            "\\" => {
                if self.hmode() {
                    self.flush();
                    let h = self.hlist.as_mut().unwrap();
                    if matches!(h.last(), Some(Node::Glue(_))) {
                        h.pop();
                    }
                    h.push(Node::Penalty(10000));
                    h.push(Node::glue(GlueSpec { stretch: 65536, stretch_order: GlueOrder::Fil, ..GlueSpec::ZERO }));
                    h.push(Node::Penalty(-10000));
                } else {
                    self.diag("There's no line here to end");
                }
                while matches!(toks.get(*i).map(|t| &t.kind), Some(TokenKind::Char(' ', CatCode::Space))) {
                    *i += 1;
                }
            }
            " " => {
                self.leavevmode();
                self.normal_space();
            }
            "@" => self.sf = 1000,
            "i" => self.char_code(16),
            "'" | "`" | "^" | "\"" | "~" | "=" | "." | "u" | "v" | "H" => {
                let base = match toks.get(*i).map(|t| &t.kind) {
                    Some(TokenKind::Char(c, _)) if c.is_ascii() => Some(*c as u8),
                    Some(TokenKind::ControlSequence(n)) if n == "i" => Some(16),
                    _ => None,
                };
                match base {
                    Some(b) => {
                        *i += 1;
                        self.accent(name, b);
                    }
                    None => self.diag(format!("accent \\{name} without a character base")),
                }
            }
            "POCemph" | "POCbf" | "POCit" => {
                self.leavevmode();
                let saved = self.font;
                let (series, shape) = match name {
                    "POCemph" => (saved.series, if saved.shape == Shape::Upright { Shape::Italic } else { Shape::Upright }),
                    "POCbf" => (Series::Bold, saved.shape),
                    _ => (saved.series, Shape::Italic),
                };
                self.groups.push(Group::Font { saved, check_icr: true });
                self.set_font(series, shape, saved.size);
            }
            "POCheadsection" => self.heading_begin("section", false),
            "POCheadsectionstar" => self.heading_begin("section", true),
            "POCheadsubsection" => self.heading_begin("subsection", false),
            "POCheadsubsectionstar" => self.heading_begin("subsection", true),
            "POCheadsubsubsection" => self.heading_begin("subsubsection", false),
            "POCheadsubsubsectionstar" => self.heading_begin("subsubsection", true),
            "POCvspace" | "POCvspacestar" => {
                self.groups.push(Group::Vspace { star: name == "POCvspacestar", text: String::new() })
            }
            "POCend" => match self.groups.pop() {
                Some(Group::Font { saved, check_icr }) => {
                    self.flush();
                    self.font = saved;
                    if check_icr {
                        self.maybe_ic(toks.get(*i));
                    }
                }
                Some(Group::Heading { saved, baseline_skip, spec }) => self.heading_end(saved, baseline_skip, spec),
                Some(Group::Vspace { star, text }) => self.vspace_end(star, &text),
                None => self.diag("unbalanced marker group"),
            },
            "pagebreak" | "nopagebreak" => {
                let n = Self::optional_int(toks, i);
                if self.hmode() {
                    self.diag(format!("\\{name} in horizontal mode (\\vadjust) is not supported"));
                } else {
                    let p = PriorityPenalties::ARTICLE.get(n.unwrap_or(4));
                    self.vlist.penalty(if name == "pagebreak" { -p } else { p });
                }
            }
            "newpage" => {
                self.par();
                for n in newpage() {
                    self.vlist.push(n);
                }
            }
            "clearpage" => {
                self.par();
                for n in clearpage(0) {
                    self.vlist.push(n);
                }
            }
            "enddocument" => {
                self.par();
                for n in clearpage(0) {
                    self.vlist.push(n);
                }
                return false;
            }
            "relax" => {}
            other => {
                self.flush();
                self.diag(format!("unsupported control sequence \\{other}"));
            }
        }
        true
    }

    fn run(&mut self, toks: &[Token]) {
        let mut i = 0;
        let mut started = false;
        while i < toks.len() {
            let t = &toks[i];
            i += 1;
            if !started {
                started = t.is_cs("document");
                continue;
            }
            if let Some(Group::Vspace { text, .. }) = self.groups.last_mut() {
                match &t.kind {
                    TokenKind::ControlSequence(n) if n == "POCend" => {}
                    TokenKind::Char(c, _) => {
                        text.push(*c);
                        continue;
                    }
                    other => {
                        text.push_str(&format!("{other:?}"));
                        continue;
                    }
                }
            }
            match &t.kind {
                TokenKind::Char(c, cat) => match cat {
                    CatCode::Space => {
                        self.flush();
                        self.space();
                    }
                    CatCode::Letter | CatCode::Other => {
                        if c.is_ascii() {
                            self.char_code(*c as u8);
                        } else {
                            self.unicode_char(*c);
                        }
                    }
                    CatCode::MathShift => {
                        self.diag("inline math is not supported (skipped)");
                        while let Some(t2) = toks.get(i) {
                            i += 1;
                            if matches!(t2.kind, TokenKind::Char(_, CatCode::MathShift)) {
                                break;
                            }
                        }
                    }
                    other => self.diag(format!("unsupported character category {other:?}")),
                },
                TokenKind::ActiveChar('~') => {
                    self.leavevmode();
                    self.append(Node::Penalty(10000));
                    self.normal_space();
                }
                TokenKind::ActiveChar(c) => self.diag(format!("unsupported active character {c}")),
                TokenKind::ControlSequence(name) => {
                    let name = name.clone();
                    if !self.command(&name, toks, &mut i) {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    fn finish(mut self) -> Output {
        self.par();
        let p = self.doc.params;
        let params = PageParams {
            vsize: p.textheight.0 as i32,
            max_depth: p.maxdepth.0 as i32,
            top_skip: PGlue::fixed(p.topskip.0 as i32),
            ..PageParams::default()
        };
        let mut pb = PageBuilder::new(params);
        let kludge = 200u8;
        pb.classes.insert(kludge, InsertClass { count: 1000, dimen: MAX_DIMEN, skip: PGlue::ZERO, contents: None });
        let mut out = LatexOutput::new(p.textheight.0 as i32, p.maxdepth.0 as i32, !self.doc.flags.twoside, kludge);
        pb.contribute(self.vlist.drain());
        pb.run(&mut out);
        pb.end(p.textwidth.0 as i32, &mut out);
        if out.unsupported > 0 {
            self.diag(format!("{} unsupported output routine calls", out.unsupported));
        }
        let frame = &self.doc.frame;
        let mut pages = Vec::new();
        let normal = self.select_font(Series::Medium, Shape::Upright, FontSize::NormalSize);
        for (k, col) in out.columns.iter().enumerate() {
            let number = k + 1;
            let left = frame.text_left(number as i64).0 as i32;
            let top = frame.text_top.0 as i32;
            let mut glyphs = Vec::new();
            for (b, baseline) in &col.lines {
                if b.vertical || b.id as usize >= self.lines.len() {
                    continue;
                }
                let hb = &self.lines[b.id as usize];
                let events = ship_out(hb, left + b.shift, top + baseline, &self.fonts);
                glyphs.extend(self.glyphs(&events));
            }
            // \pagestyle{plain}: \hfil\thepage\hfil at the foot baseline.
            if format!("{:?}", self.doc.pagestyle) == "Plain" {
                if let Ok(nf) = normal {
                    let text = number.to_string();
                    let mut l = vec![Node::glue(GlueSpec { stretch: 65536, stretch_order: GlueOrder::Fil, ..GlueSpec::ZERO })];
                    l.extend(run_nodes(&self.fonts, nf.id, text.as_bytes(), false));
                    l.push(Node::glue(GlueSpec { stretch: 65536, stretch_order: GlueOrder::Fil, ..GlueSpec::ZERO }));
                    let fb = hpack(
                        l,
                        PackSpec::Exactly(p.textwidth.0 as i32),
                        false,
                        &PackParams::default(),
                        PackOrigin::default(),
                        &self.fonts,
                    )
                    .node;
                    let events = ship_out(&fb, left, frame.foot_baseline.0 as i32, &self.fonts);
                    glyphs.extend(self.glyphs(&events));
                }
            } else {
                self.diag(format!("page style {:?} not drawn", self.doc.pagestyle));
            }
            pages.push(Page { number, glyphs });
        }
        Output {
            pages,
            page_width: frame.pdf_page_width.0 as i32,
            page_height: frame.pdf_page_height.0 as i32,
            diagnostics: self.diags,
            stats: self.stats,
        }
    }

    fn glyphs(&self, events: &[ShipEvent]) -> Vec<Glyph> {
        events
            .iter()
            .filter_map(|e| match e {
                ShipEvent::Char { font, ch, h, v } => {
                    let f = &self.fonts.fonts[*font as usize];
                    Some(Glyph { font: f.tfm.clone(), size: f.size, code: *ch, x: *h, y: *v })
                }
                _ => None,
            })
            .collect()
    }
}

/// Typesets a whole `.tex` source.
pub fn typeset(source: &str) -> Result<Output, String> {
    let setup = DocumentSetup::from_preamble(source).ok_or("no standard \\documentclass found")?;
    let doc = resolve(&setup);
    let base = doc.options.size;
    let full = format!("{FORMAT}{source}");
    let mut engine = Engine::new(&full);
    let toks = engine.run();
    let mut diags: Vec<String> = engine.diagnostics().iter().map(|d| format!("expansion: {}", d.message)).collect();
    if doc.flags.twocolumn {
        diags.push("twocolumn is not supported".into());
    }
    diags.extend(doc.warnings.iter().cloned());
    let p = doc.params;
    let interline = InterlineParams {
        baseline_skip: PGlue::fixed(p.baselineskip.0 as i32),
        line_skip: PGlue::fixed(65536),
        line_skip_limit: 0,
    };
    let mut s = Stomach {
        fonts: Fonts::new(),
        doc,
        base,
        font: FontState { series: Series::Medium, shape: Shape::Upright, size: FontSize::NormalSize, id: 0 },
        sf: 1000,
        sfcodes: SfCodes::default(),
        hlist: None,
        run: Vec::new(),
        run_font: 0,
        vlist: VListBuilder::new(interline),
        lines: Vec::new(),
        groups: Vec::new(),
        hang_indent: 0,
        interline_penalty: 0,
        club_penalty: 150,
        nobreak: false,
        afterindent: true,
        afterheading_everypar: false,
        counters: [0; 3],
        diags,
        stats: BreakStats::default(),
    };
    s.font = s.select_font(Series::Medium, Shape::Upright, FontSize::NormalSize)?;
    s.run_font = s.font.id;
    s.run(&toks);
    Ok(s.finish())
}

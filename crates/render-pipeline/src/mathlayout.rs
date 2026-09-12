//! Math boxes after TeX's Appendix G (rules 5–6, 11, 13, 13a, 15, 18, 20)
//! for the subset the compiler parses: symbols, fractions, radicals, scripts.
//! Fraction bars and radical overlines are explicit rules.
//!
//! TEMPORARY SHIM for the `math-layout` sibling crate (its first checkpoint,
//! 2079650, appeared while this was being written; not yet consumed).
//! Parameter sources are explicit: `ParamSource::Tfm` uses the Latin Modern
//! `lmsy`/`lmex` fontdimens transcribed in `params.rs` (what pdfLaTeX uses,
//! the default); `ParamSource::OpenTypeMath` uses the `MATH` constants of
//! `latinmodern-math.otf`. Both are scaled by the style's font size.

use std::rc::Rc;

use flashtex_compiler::math::{MathAtom, MathList, Nucleus};
use flashtex_compiler::Span;
use flashtex_font_engine::{Face, GlyphId};

use crate::fonts::{FaceKind, Family, FontSet, LoadedFace, Role};
use crate::params;
use crate::shape::Shaper;
use crate::style::Stylesheet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamSource {
    Tfm,
    OpenTypeMath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathStyle {
    Display,
    Text,
    Script,
    ScriptScript,
}

impl MathStyle {
    fn sup(self) -> MathStyle {
        match self {
            MathStyle::Display | MathStyle::Text => MathStyle::Script,
            _ => MathStyle::ScriptScript,
        }
    }
    fn sub(self) -> MathStyle {
        self.sup()
    }
    fn num(self) -> MathStyle {
        match self {
            MathStyle::Display => MathStyle::Text,
            MathStyle::Text => MathStyle::Script,
            _ => MathStyle::ScriptScript,
        }
    }
    fn level(self) -> u8 {
        match self {
            MathStyle::Display | MathStyle::Text => 0,
            MathStyle::Script => 1,
            MathStyle::ScriptScript => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
}

/// A positioned glyph inside a math box. `dy` is the glyph baseline's offset
/// below the box baseline (positive = down).
#[derive(Debug, Clone)]
pub struct MGlyph {
    pub face: Rc<LoadedFace>,
    pub size_pt: f64,
    pub gid: GlyphId,
    pub text: String,
    pub x: f64,
    pub dy: f64,
    pub advance_pt: f64,
    pub height: f64,
    pub depth: f64,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MRule {
    pub x: f64,
    /// Top edge relative to the baseline, positive = down.
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum MItem {
    Glyph(MGlyph),
    Rule(MRule),
}

#[derive(Debug, Clone, Default)]
pub struct MBox {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub items: Vec<MItem>,
}

impl MBox {
    fn shift(&mut self, dx: f64, dy: f64) {
        for it in &mut self.items {
            match it {
                MItem::Glyph(g) => {
                    g.x += dx;
                    g.dy += dy;
                }
                MItem::Rule(r) => {
                    r.x += dx;
                    r.top += dy;
                }
            }
        }
    }
    fn append(&mut self, mut other: MBox, dx: f64, dy: f64) {
        other.shift(dx, dy);
        self.items.extend(other.items);
        self.height = self.height.max(other.height - dy);
        self.depth = self.depth.max(other.depth + dy);
    }
}

/// Parameters in points for one style.
#[derive(Debug, Clone, Copy)]
struct P {
    x_height: f64,
    quad: f64,
    num1: f64,
    num2: f64,
    denom1: f64,
    denom2: f64,
    sup1: f64,
    sup2: f64,
    sup3: f64,
    sub1: f64,
    sub2: f64,
    sup_drop: f64,
    sub_drop: f64,
    axis: f64,
    theta: f64,
    big_op: [f64; 5],
}

pub struct MathContext<'a> {
    pub fonts: &'a FontSet,
    pub shaper: &'a Shaper,
    pub style: &'a Stylesheet,
    pub source: ParamSource,
    pub substitutions: Vec<String>,
}

impl MathContext<'_> {
    fn size(&self, st: MathStyle) -> f64 {
        match st {
            MathStyle::Display | MathStyle::Text => self.style.body_size_pt,
            MathStyle::Script => self.style.script_size_pt,
            MathStyle::ScriptScript => self.style.scriptscript_size_pt,
        }
    }

    fn symbol_face(&mut self, st: MathStyle) -> Rc<LoadedFace> {
        let r = self.fonts.resolve(self.style.family, Role::MathSymbols, self.size(st));
        if let Some(s) = r.substituted {
            self.substitutions.push(s);
        }
        r.face
    }

    fn text_face(&mut self, st: MathStyle, italic: bool) -> Rc<LoadedFace> {
        let r = self.fonts.resolve(
            self.style.family,
            Role::Text {
                bold: false,
                italic,
            },
            self.size(st),
        );
        if let Some(s) = r.substituted {
            self.substitutions.push(s);
        }
        r.face
    }

    fn params(&mut self, st: MathStyle) -> P {
        let size = self.size(st);
        let ext_size = 10.0; // lmex10 is `sfixed`: extension params are absolute.
        let math_face = self.symbol_face(st);
        let ot = match (&math_face.kind, self.source) {
            (FaceKind::Otf(f), ParamSource::OpenTypeMath) => f.math(),
            _ => None,
        };
        if let Some(m) = ot {
            let u = |i: usize| f64::from(m.constant(i).unwrap_or(0)) * size / f64::from(math_face.units_per_em);
            return P {
                x_height: f64::from(math_face.face().vertical_metrics().x_height) * size / f64::from(math_face.units_per_em),
                quad: size,
                num1: u(33),
                num2: u(32),
                denom1: u(35),
                denom2: u(34),
                sup1: u(11),
                sup2: u(11),
                sup3: u(12),
                sub1: u(8),
                sub2: u(8),
                sup_drop: u(14),
                sub_drop: u(10),
                axis: u(5),
                theta: u(38),
                big_op: [u(18), u(20), u(19), u(21), u(17)],
            };
        }
        let t = params::math_params(st.level());
        P {
            x_height: t.x_height * size,
            quad: t.quad * size,
            num1: t.num1 * size,
            num2: t.num2 * size,
            denom1: t.denom1 * size,
            denom2: t.denom2 * size,
            sup1: t.sup1 * size,
            sup2: t.sup2 * size,
            sup3: t.sup3 * size,
            sub1: t.sub1 * size,
            sub2: t.sub2 * size,
            sup_drop: t.sup_drop * size,
            sub_drop: t.sub_drop * size,
            axis: t.axis_height * size,
            theta: t.default_rule_thickness * ext_size,
            big_op: [
                t.big_op_spacing[0] * ext_size,
                t.big_op_spacing[1] * ext_size,
                t.big_op_spacing[2] * ext_size,
                t.big_op_spacing[3] * ext_size,
                t.big_op_spacing[4] * ext_size,
            ],
        }
    }
}

/// Maps a math symbol to (face role, character, class). Letters go to the
/// math-italic alphabet of the symbol font; digits and delimiters to the
/// upright text face (TeX's `\fam0`), operators to the symbol font.
fn classify(sym: &str) -> (bool, char, Class) {
    let mut chars = sym.chars();
    let c = chars.next().unwrap_or(' ');
    if chars.next().is_some() {
        // Multi-character literal (unsupported command typeset literally).
        return (false, c, Class::Ord);
    }
    match c {
        'a'..='z' | 'A'..='Z' => (true, c, Class::Ord),
        '0'..='9' | '.' => (false, c, Class::Ord),
        '+' | '\u{00B1}' | '\u{00D7}' | '\u{00F7}' | '\u{00B7}' | '*' => (true, c, Class::Bin),
        '-' => (true, '\u{2212}', Class::Bin),
        '=' | '<' | '>' | '\u{2264}' | '\u{2265}' | '\u{2260}' | '\u{2248}' => (true, c, Class::Rel),
        '(' | '[' => (false, c, Class::Open),
        ')' | ']' => (false, c, Class::Close),
        ',' | ';' => (false, c, Class::Punct),
        '\u{2211}' | '\u{222B}' => (true, c, Class::Op),
        '\u{03B1}'..='\u{03C9}' => (true, c, Class::Ord),
        '\u{221E}' => (true, c, Class::Ord),
        _ => (true, c, Class::Ord),
    }
}

/// Unicode math-italic code point for a Latin letter or Greek lowercase.
fn math_italic(c: char) -> Option<char> {
    let cp = match c {
        'h' => 0x210E,
        'a'..='z' => 0x1D44E + (c as u32 - 'a' as u32),
        'A'..='Z' => 0x1D434 + (c as u32 - 'A' as u32),
        '\u{03B1}'..='\u{03C9}' => 0x1D6FC + (c as u32 - 0x03B1),
        _ => return None,
    };
    char::from_u32(cp)
}

fn thin(p: &P) -> f64 {
    3.0 * p.quad / 18.0
}
fn med(p: &P) -> f64 {
    4.0 * p.quad / 18.0
}
fn thick(p: &P) -> f64 {
    5.0 * p.quad / 18.0
}

/// Rule 20 spacing between adjacent atoms.
fn spacing(left: Class, right: Class, p: &P, script: bool) -> f64 {
    use Class::*;
    // 0 none, 1 thin, 2 med, 3 thick; negative = only in non-script styles.
    let code: i8 = match (left, right) {
        (Ord, Op) | (Op, Ord) | (Op, Op) | (Close, Op) | (Inner, Op) => 1,
        (Ord, Bin) | (Bin, Ord) | (Bin, Op) | (Bin, Open) | (Bin, Inner) | (Close, Bin) | (Inner, Bin) => -2,
        (Ord, Rel) | (Rel, Ord) | (Op, Rel) | (Rel, Op) | (Rel, Open) | (Close, Rel) | (Inner, Rel) | (Rel, Inner) => -3,
        (Ord, Inner) | (Op, Inner) | (Close, Inner) | (Inner, Ord) | (Inner, Open) | (Inner, Inner) | (Inner, Punct) => -1,
        (Punct, _) => -1,
        _ => 0,
    };
    let only_text = code < 0;
    let code = code.abs();
    if only_text && script {
        return 0.0;
    }
    match code {
        1 => thin(p),
        2 => med(p),
        3 => thick(p),
        _ => 0.0,
    }
}

struct Atom {
    class: Class,
    bx: MBox,
}

pub fn layout(ctx: &mut MathContext, list: &MathList, style: MathStyle) -> MBox {
    layout_list(ctx, list, style, false)
}

fn layout_list(ctx: &mut MathContext, list: &MathList, style: MathStyle, cramped: bool) -> MBox {
    let p = ctx.params(style);
    let mut atoms: Vec<Atom> = Vec::new();
    for atom in &list.atoms {
        let (class, bx) = layout_atom(ctx, atom, style, cramped);
        atoms.push(Atom { class, bx });
    }
    // Rules 5 and 6: Bin becomes Ord in the wrong context.
    let n = atoms.len();
    for i in 0..n {
        if atoms[i].class == Class::Bin {
            let prev_ok = i > 0
                && !matches!(atoms[i - 1].class, Class::Bin | Class::Op | Class::Rel | Class::Open | Class::Punct);
            let next_ok = i + 1 < n && !matches!(atoms[i + 1].class, Class::Rel | Class::Close | Class::Punct);
            if !prev_ok || !next_ok {
                atoms[i].class = Class::Ord;
            }
        }
    }
    let script = matches!(style, MathStyle::Script | MathStyle::ScriptScript);
    let mut out = MBox::default();
    let mut x = 0.0;
    let mut prev: Option<Class> = None;
    for a in atoms {
        if let Some(pc) = prev {
            x += spacing(pc, a.class, &p, script);
        }
        let w = a.bx.width;
        out.append(a.bx, x, 0.0);
        x += w;
        prev = Some(a.class);
    }
    out.width = x;
    out
}

fn glyph_box(ctx: &mut MathContext, face: &Rc<LoadedFace>, size: f64, ch: char, text: &str, span: Span) -> MBox {
    let s = ctx.shaper.shape(face, size, &ch.to_string());
    let s = crate::shape::at_size(&s, size);
    let (gid, adv, h, d) = s
        .clusters
        .first()
        .and_then(|c| c.glyphs.first().map(|g| (g.gid, g.advance, g.y_max, g.y_min)))
        .unwrap_or((GlyphId::NOTDEF, 0, 0, 0));
    let advance_pt = face.pt(i64::from(adv), size);
    let height = face.pt(i64::from(h), size);
    let depth = -face.pt(i64::from(d), size);
    MBox {
        width: advance_pt,
        height,
        depth,
        items: vec![MItem::Glyph(MGlyph {
            face: face.clone(),
            size_pt: size,
            gid,
            text: text.to_string(),
            x: 0.0,
            dy: 0.0,
            advance_pt,
            height,
            depth,
            span,
        })],
    }
}

/// Picks the smallest vertical variant of `ch` whose total height is at
/// least `needed_pt` (OpenType MATH). Falls back to the base glyph.
fn sized_glyph(ctx: &mut MathContext, face: &Rc<LoadedFace>, size: f64, ch: char, needed_pt: f64, span: Span) -> MBox {
    let base = glyph_box(ctx, face, size, ch, &ch.to_string(), span);
    if base.height + base.depth >= needed_pt {
        return base;
    }
    let FaceKind::Otf(otf) = &face.kind else {
        return base;
    };
    let Some(math) = otf.math() else {
        return base;
    };
    let Some(base_gid) = face.face().glyph_id(ch) else {
        return base;
    };
    let mut chosen: Option<(GlyphId, f64)> = None;
    for v in math.vertical_variants(base_gid) {
        let total = face.pt(i64::from(v.advance), size);
        if total >= needed_pt {
            chosen = Some((v.gid, total));
            break;
        }
        chosen = Some((v.gid, total));
    }
    let Some((gid, _)) = chosen else {
        return base;
    };
    let b = face.bounds(gid, None);
    let adv = face.face().advance(gid).unwrap_or(0);
    let advance_pt = face.pt(i64::from(adv), size);
    let height = face.pt(i64::from(b.y_max), size);
    let depth = -face.pt(i64::from(b.y_min), size);
    MBox {
        width: advance_pt,
        height,
        depth,
        items: vec![MItem::Glyph(MGlyph {
            face: face.clone(),
            size_pt: size,
            gid,
            text: ch.to_string(),
            x: 0.0,
            dy: 0.0,
            advance_pt,
            height,
            depth,
            span,
        })],
    }
}

fn layout_atom(ctx: &mut MathContext, atom: &MathAtom, style: MathStyle, cramped: bool) -> (Class, MBox) {
    let p = ctx.params(style);
    let size = ctx.size(style);
    let (mut class, mut nucleus, is_char) = match &atom.nucleus {
        Nucleus::Symbol(sym) => {
            let (symbol_font, ch, class) = classify(sym);
            let times = ctx.style.family == Family::Times;
            let bx = if sym.chars().count() > 1 {
                // Literal fallback text (e.g. an unsupported command).
                let face = ctx.text_face(style, false);
                let s = ctx.shaper.shape(&face, size, sym);
                let s = crate::shape::at_size(&s, size);
                let mut b = MBox::default();
                let mut x = 0.0;
                for c in &s.clusters {
                    for g in &c.glyphs {
                        let adv = face.pt(i64::from(g.advance), size);
                        b.items.push(MItem::Glyph(MGlyph {
                            face: face.clone(),
                            size_pt: size,
                            gid: g.gid,
                            text: c.text.clone(),
                            x,
                            dy: 0.0,
                            advance_pt: adv,
                            height: face.pt(i64::from(g.y_max), size),
                            depth: -face.pt(i64::from(g.y_min), size),
                            span: atom.span,
                        }));
                        x += adv;
                    }
                }
                b.width = x;
                b.height = s.height_pt;
                b.depth = s.depth_pt;
                b
            } else if symbol_font {
                let face = ctx.symbol_face(style);
                let glyph_char = if times {
                    ch
                } else {
                    math_italic(ch).unwrap_or(ch)
                };
                let available = face.face().glyph_id(glyph_char).is_some();
                if available {
                    if times && ch.is_ascii_alphabetic() {
                        // Times: letters from the italic text face (mathptmx).
                        let tf = ctx.text_face(style, true);
                        glyph_box(ctx, &tf, size, ch, &ch.to_string(), atom.span)
                    } else if times && matches!(class, Class::Bin | Class::Rel) && "+=<>".contains(ch) {
                        let tf = ctx.text_face(style, false);
                        glyph_box(ctx, &tf, size, ch, &ch.to_string(), atom.span)
                    } else {
                        glyph_box(ctx, &face, size, glyph_char, &ch.to_string(), atom.span)
                    }
                } else {
                    let tf = ctx.text_face(style, ch.is_ascii_alphabetic());
                    glyph_box(ctx, &tf, size, ch, &ch.to_string(), atom.span)
                }
            } else {
                let face = ctx.text_face(style, false);
                glyph_box(ctx, &face, size, ch, &ch.to_string(), atom.span)
            };
            (class, bx, true)
        }
        Nucleus::Fraction { numerator, denominator } => {
            let num = layout_list(ctx, numerator, style.num(), cramped);
            let den = layout_list(ctx, denominator, style.num(), true);
            (Class::Inner, fraction(&p, style, num, den, atom.span), false)
        }
        Nucleus::Radical(body) => {
            let inner = layout_list(ctx, body, style, true);
            (Class::Ord, radical(ctx, &p, style, inner, atom.span), false)
        }
    };
    let display = style == MathStyle::Display;
    if class == Class::Op && display && (atom.superscript.is_some() || atom.subscript.is_some()) {
        // Rule 13a: large operator with limits in display style.
        let face = ctx.symbol_face(style);
        let ch = match &atom.nucleus {
            Nucleus::Symbol(s) => s.chars().next().unwrap_or(' '),
            _ => ' ',
        };
        let needed = ctx.display_operator_min_height(&face, size);
        let mut op = sized_glyph(ctx, &face, size, ch, needed, atom.span);
        // Centre on the axis.
        let shift = (op.height - op.depth) / 2.0 - p.axis;
        op.shift(0.0, shift);
        op.height -= shift;
        op.depth += shift;
        let sup = atom.superscript.as_ref().map(|l| layout_list(ctx, l, style.sup(), cramped));
        let sub = atom.subscript.as_ref().map(|l| layout_list(ctx, l, style.sub(), true));
        let w = op
            .width
            .max(sup.as_ref().map_or(0.0, |b| b.width))
            .max(sub.as_ref().map_or(0.0, |b| b.width));
        let mut out = MBox::default();
        let op_w = op.width;
        out.append(op, (w - op_w) / 2.0, 0.0);
        if let Some(s) = sup {
            let k = p.big_op[0].max(p.big_op[2] - s.depth);
            let dy = -(out.height + k + s.depth);
            let sw = s.width;
            out.append(s, (w - sw) / 2.0, dy);
            out.height += p.big_op[4];
        }
        if let Some(s) = sub {
            let k = p.big_op[1].max(p.big_op[3] - s.height);
            let dy = out.depth + k + s.height;
            let sw = s.width;
            out.append(s, (w - sw) / 2.0, dy);
            out.depth += p.big_op[4];
        }
        out.width = w;
        return (Class::Op, out);
    }
    if class == Class::Op && is_char {
        // Rule 13: centre a single-character operator on the axis.
        let shift = (nucleus.height - nucleus.depth) / 2.0 - p.axis;
        nucleus.shift(0.0, shift);
        nucleus.height -= shift;
        nucleus.depth += shift;
    }
    if atom.superscript.is_none() && atom.subscript.is_none() {
        return (class, nucleus);
    }
    // Rule 18: scripts.
    let (mut u, mut v) = if is_char {
        (0.0, 0.0)
    } else {
        let ps = ctx.params(style.sup());
        (nucleus.height - ps.sup_drop, nucleus.depth + ps.sub_drop)
    };
    let sup = atom.superscript.as_ref().map(|l| layout_list(ctx, l, style.sup(), cramped));
    let sub = atom.subscript.as_ref().map(|l| layout_list(ctx, l, style.sub(), true));
    let mut x = nucleus.width;
    let mut out = nucleus;
    let script_space = 0.5;
    match (sup, sub) {
        (None, Some(sb)) => {
            v = v.max(p.sub1).max(sb.height - 4.0 * p.x_height / 5.0);
            let w = sb.width;
            out.append(sb, x, v);
            x += w + script_space;
        }
        (Some(sp), None) => {
            let pp = if display {
                p.sup1
            } else if cramped {
                p.sup3
            } else {
                p.sup2
            };
            u = u.max(pp).max(sp.depth + p.x_height / 4.0);
            let w = sp.width;
            out.append(sp, x, -u);
            x += w + script_space;
        }
        (Some(sp), Some(sb)) => {
            let pp = if display {
                p.sup1
            } else if cramped {
                p.sup3
            } else {
                p.sup2
            };
            u = u.max(pp).max(sp.depth + p.x_height / 4.0);
            v = v.max(p.sub2);
            if (u - sp.depth) - (sb.height - v) < 4.0 * p.theta {
                v = 4.0 * p.theta + sb.height - (u - sp.depth);
                let psi = 4.0 * p.x_height / 5.0 - (u - sp.depth);
                if psi > 0.0 {
                    u += psi;
                    v -= psi;
                }
            }
            let w = sp.width.max(sb.width);
            out.append(sp, x, -u);
            out.append(sb, x, v);
            x += w + script_space;
        }
        (None, None) => {}
    }
    out.width = x;
    if class == Class::Bin {
        class = Class::Bin;
    }
    (class, out)
}

impl MathContext<'_> {
    fn display_operator_min_height(&self, face: &Rc<LoadedFace>, size: f64) -> f64 {
        if let FaceKind::Otf(f) = &face.kind {
            if let Some(m) = f.math() {
                if let Some(v) = m.constant(3) {
                    return face.pt(i64::from(v), size);
                }
            }
        }
        // TeX: the display-size operator of cmex10 is ~2.4x the text size.
        size * 1.4
    }
}

/// Rule 15: generalized fraction with the default rule.
fn fraction(p: &P, style: MathStyle, num: MBox, den: MBox, span: Span) -> MBox {
    let display = style == MathStyle::Display;
    let theta = p.theta;
    let (mut u, mut v, phi) = if display {
        (p.num1, p.denom1, 3.0 * theta)
    } else {
        (p.num2, p.denom2, theta)
    };
    let a = p.axis;
    // Clearance between numerator bottom and rule top.
    let gap_num = (u - num.depth) - (a + theta / 2.0);
    if gap_num < phi {
        u += phi - gap_num;
    }
    let gap_den = (a - theta / 2.0) - (den.height - v);
    if gap_den < phi {
        v += phi - gap_den;
    }
    let null_delim = 1.2;
    let inner_w = num.width.max(den.width);
    let mut out = MBox::default();
    let nw = num.width;
    let dw = den.width;
    out.append(num, null_delim + (inner_w - nw) / 2.0, -u);
    out.append(den, null_delim + (inner_w - dw) / 2.0, v);
    out.items.push(MItem::Rule(MRule {
        x: null_delim,
        top: -(a + theta / 2.0),
        width: inner_w,
        height: theta,
        span,
    }));
    out.height = out.height.max(a + theta / 2.0);
    out.depth = out.depth.max(theta / 2.0 - a);
    out.width = inner_w + 2.0 * null_delim;
    out
}

/// Rule 11: radical with an explicit overline rule.
fn radical(ctx: &mut MathContext, p: &P, style: MathStyle, body: MBox, span: Span) -> MBox {
    let size = ctx.size(style);
    let theta = p.theta;
    let mut phi = if style == MathStyle::Display {
        theta + p.x_height / 4.0
    } else {
        theta
    };
    let needed = body.height + body.depth + phi + theta;
    let face = ctx.symbol_face(style);
    let mut sign = sized_glyph(ctx, &face, size, '\u{221A}', needed, span);
    let delta = (sign.height + sign.depth) - needed;
    if delta > 0.0 {
        phi += delta / 2.0;
    }
    // Shift the sign so its top is at the rule's top.
    let rule_top = body.height + phi + theta;
    let dy = sign.height - rule_top;
    sign.shift(0.0, dy);
    sign.height = rule_top;
    sign.depth = sign.depth - dy;
    let sign_w = sign.width;
    let mut out = MBox::default();
    out.append(sign, 0.0, 0.0);
    let bw = body.width;
    out.append(body, sign_w, 0.0);
    out.items.push(MItem::Rule(MRule {
        x: sign_w,
        top: -rule_top,
        width: bw,
        height: theta,
        span,
    }));
    out.height = out.height.max(rule_top + theta);
    out.width = sign_w + bw;
    out
}

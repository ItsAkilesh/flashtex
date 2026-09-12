//! The layout engine: TeXbook Appendix G, rules 5–20, and `tex.web`
//! §§ 720–767 (`mlist_to_hlist`) as the reference for the arithmetic.
//!
//! Every rule is implemented against [`MathFontMetrics`] parameters, so the
//! numbers below are TeX's when the Computer Modern adapter is used.

use crate::boxes::{BoxKind, Child, MathBox};
use crate::mathlist::{Atom, AtomClass, Limits, MathList, Nucleus};
use crate::metrics::{Glyph, MathFontMetrics, MathParams};
use crate::spacing::{Space, between};
use crate::style::Style;

/// Something the engine could not do exactly; the layout still completes with
/// an explicit fallback so the caller can report rather than guess.
#[derive(Debug, Clone, PartialEq)]
pub enum Limitation {
    /// The metrics provider has no glyph for this symbol; an empty box was used.
    MissingGlyph(char),
    /// No delimiter in the size list reached the wanted size; the largest one
    /// was used (extensible delimiters are not built yet).
    DelimiterTooSmall { ch: char, wanted: f64, used: f64 },
    /// Same for the radical sign.
    RadicalTooSmall { wanted: f64, used: f64 },
    /// The accent symbol is unknown; the base was laid out without it.
    MissingAccent(char),
}

/// A finished layout: the root box plus any limitations encountered.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub root: MathBox,
    pub limitations: Vec<Limitation>,
}

/// Lays out `list` in `style`; the root box is an hbox on the text baseline.
pub fn layout(list: &MathList, style: Style, metrics: &dyn MathFontMetrics) -> MathBox {
    layout_with_report(list, style, metrics).root
}

/// [`layout`] plus the list of limitations hit along the way.
pub fn layout_with_report(list: &MathList, style: Style, metrics: &dyn MathFontMetrics) -> Layout {
    let mut engine = Engine {
        m: metrics,
        limitations: Vec::new(),
    };
    let root = engine.list(list, style);
    Layout {
        root,
        limitations: engine.limitations,
    }
}

struct Engine<'a> {
    m: &'a dyn MathFontMetrics,
    limitations: Vec<Limitation>,
}

/// Rules 5 and 6: Bin atoms that cannot be binary become Ord.
pub fn effective_classes(atoms: &[Atom]) -> Vec<AtomClass> {
    use AtomClass::*;
    let mut out: Vec<AtomClass> = Vec::with_capacity(atoms.len());
    for atom in atoms {
        let mut class = atom.class;
        match class {
            Bin => {
                let prev = out.last().copied();
                if matches!(prev, None | Some(Bin | Op | Rel | Open | Punct)) {
                    class = Ord;
                }
            }
            Rel | Close | Punct => {
                if let Some(last) = out.last_mut()
                    && *last == Bin
                {
                    *last = Ord;
                }
            }
            _ => {}
        }
        out.push(class);
    }
    if let Some(last) = out.last_mut()
        && *last == Bin
    {
        *last = Ord;
    }
    out
}

impl Engine<'_> {
    fn params(&self, style: Style) -> MathParams {
        self.m.params(style.size_class())
    }

    /// `mlist_to_hlist`: box every atom, then insert spacing (Rule 20).
    fn list(&mut self, list: &MathList, style: Style) -> MathBox {
        let classes = effective_classes(&list.atoms);
        let mu = self.params(style).mu();
        let mut items: Vec<MathBox> = Vec::with_capacity(list.atoms.len() * 2);
        let mut prev: Option<AtomClass> = None;
        for (atom, class) in list.atoms.iter().zip(classes) {
            let b = self.atom(atom, class, style);
            if let Some(p) = prev {
                let space = between(p, class, style);
                if space != Space::None {
                    items.push(MathBox::glue(space.mu() * mu, space.mu()));
                }
            }
            items.push(b);
            prev = Some(class);
        }
        MathBox::hlist(items)
    }

    /// `clean_box`: a subformula as a single box.
    fn clean_box(&mut self, list: &MathList, style: Style) -> MathBox {
        self.list(list, style)
    }

    fn glyph(&mut self, ch: char, style: Style) -> Option<Glyph> {
        let g = self.m.glyph(ch, style.size_class());
        if g.is_none() {
            self.limitations.push(Limitation::MissingGlyph(ch));
        }
        g
    }

    fn atom(&mut self, atom: &Atom, class: AtomClass, style: Style) -> MathBox {
        if class == AtomClass::Op {
            return self.make_op(atom, style);
        }
        // The nucleus, TeX's `delta` (italic correction still to be applied),
        // and whether the nucleus is a bare character (Rule 18a).
        let (nucleus, delta, is_char) = match &atom.nucleus {
            Nucleus::Symbol(ch) => match self.glyph(*ch, style) {
                Some(g) => {
                    let b = MathBox::glyph(&g);
                    if atom.subscript.is_none() && g.italic != 0.0 {
                        (MathBox::hlist(vec![b, MathBox::kern(g.italic)]), 0.0, true)
                    } else {
                        (b, g.italic, true)
                    }
                }
                None => (MathBox::empty(), 0.0, false),
            },
            Nucleus::List(list) => (self.clean_box(list, style), 0.0, false),
            Nucleus::Empty => (MathBox::empty(), 0.0, false),
            Nucleus::Fraction {
                numerator,
                denominator,
                thickness,
            } => (
                self.make_fraction(numerator, denominator, *thickness, style),
                0.0,
                false,
            ),
            Nucleus::Radical(radicand) => (self.make_radical(radicand, style), 0.0, false),
            Nucleus::Accent { accent, base } => {
                (self.make_accent(*accent, base, style), 0.0, false)
            }
            Nucleus::Delimited { left, right, body } => {
                (self.make_left_right(*left, *right, body, style), 0.0, false)
            }
        };
        self.make_scripts(nucleus, delta, is_char, atom, style)
    }

    /// Rule 13 / 13a and `make_op`.
    fn make_op(&mut self, atom: &Atom, style: Style) -> MathBox {
        let p = self.params(style);
        let limits = match atom.limits {
            Limits::DisplayLimits => style.is_display(),
            Limits::Limits => true,
            Limits::NoLimits => false,
        };
        let (nucleus, delta) = match &atom.nucleus {
            Nucleus::Symbol(ch) => {
                let mut g = self.glyph(*ch, style);
                if style.is_display()
                    && let Some(large) = self.m.large_operator(*ch, style.size_class())
                {
                    g = Some(large);
                }
                match g {
                    Some(g) => {
                        // clean_box of a char includes its italic correction;
                        // it is removed again when a subscript must tuck under.
                        let keep_italic = !(atom.subscript.is_some() && !limits);
                        let mut x = if keep_italic && g.italic != 0.0 {
                            MathBox::hlist(vec![MathBox::glyph(&g), MathBox::kern(g.italic)])
                        } else {
                            MathBox::glyph(&g)
                        };
                        let shift = (x.height - x.depth) / 2.0 - p.axis_height;
                        x = x.shifted(shift);
                        (x, g.italic)
                    }
                    None => (MathBox::empty(), 0.0),
                }
            }
            Nucleus::List(list) => (self.clean_box(list, style), 0.0),
            other => {
                let inner = Atom {
                    class: AtomClass::Ord,
                    nucleus: other.clone(),
                    superscript: None,
                    subscript: None,
                    limits: Limits::default(),
                };
                (self.atom(&inner, AtomClass::Ord, style), 0.0)
            }
        };
        if !limits {
            return self.make_scripts(nucleus, delta, false, atom, style);
        }
        // Rule 13a: limits above and below, centred, italic-shifted by δ/2.
        let x = atom
            .superscript
            .as_ref()
            .map(|s| self.clean_box(s, style.sup()));
        let z = atom
            .subscript
            .as_ref()
            .map(|s| self.clean_box(s, style.sub()));
        let y = nucleus;
        let mut w = y.width;
        if let Some(x) = &x {
            w = w.max(x.width);
        }
        if let Some(z) = &z {
            w = w.max(z.width);
        }
        let y = y.rebox(w);
        let mut children = Vec::new();
        let mut height = y.height;
        let mut depth = y.depth;
        if let Some(x) = x {
            let x = x.rebox(w);
            let shift_up = (p.big_op_spacing3 - x.depth).max(p.big_op_spacing1);
            // x's baseline sits shift_up + d(x) above y's top.
            let dy = -(y.height + shift_up + x.depth);
            height = y.height + shift_up + x.depth + x.height + p.big_op_spacing5;
            children.push(Child {
                dx: delta / 2.0,
                dy,
                content: x,
            });
        }
        children.push(Child {
            dx: 0.0,
            dy: 0.0,
            content: y,
        });
        if let Some(z) = z {
            let z = z.rebox(w);
            let shift_down = (p.big_op_spacing4 - z.height).max(p.big_op_spacing2);
            let dy = depth + shift_down + z.height;
            depth = depth + shift_down + z.height + z.depth + p.big_op_spacing5;
            children.push(Child {
                dx: -delta / 2.0,
                dy,
                content: z,
            });
        }
        MathBox {
            kind: BoxKind::VBox(children),
            width: w,
            height,
            depth,
        }
    }

    /// Rule 18 and `make_scripts`.
    fn make_scripts(
        &mut self,
        nucleus: MathBox,
        delta: f64,
        is_char: bool,
        atom: &Atom,
        style: Style,
    ) -> MathBox {
        if atom.superscript.is_none() && atom.subscript.is_none() {
            return nucleus;
        }
        let p = self.params(style);
        let t = self.params(style.sup());
        let (mut shift_up, mut shift_down) = if is_char {
            (0.0, 0.0)
        } else {
            (nucleus.height - t.sup_drop, nucleus.depth + t.sub_drop)
        };
        let Some(sup) = &atom.superscript else {
            // Rule 18b: subscript only.
            let mut x = self.clean_box(atom.subscript.as_ref().unwrap(), style.sub());
            x.width += p.script_space;
            shift_down = shift_down.max(p.sub1);
            let clr = x.height - p.x_height.abs() * 4.0 / 5.0;
            shift_down = shift_down.max(clr);
            return MathBox::hbox(vec![(0.0, nucleus), (shift_down, x)]);
        };
        // Rule 18c: superscript.
        let mut x = self.clean_box(sup, style.sup());
        x.width += p.script_space;
        let clr = if style.cramped {
            p.sup3
        } else if style.is_display() {
            p.sup1
        } else {
            p.sup2
        };
        shift_up = shift_up.max(clr);
        let clr = x.depth + p.x_height.abs() / 4.0;
        shift_up = shift_up.max(clr);
        let Some(sub) = &atom.subscript else {
            return MathBox::hbox(vec![(0.0, nucleus), (-shift_up, x)]);
        };
        // Rule 18d/e: both scripts.
        let mut y = self.clean_box(sub, style.sub());
        y.width += p.script_space;
        shift_down = shift_down.max(p.sub2);
        let clr = 4.0 * p.default_rule_thickness - ((shift_up - x.depth) - (y.height - shift_down));
        if clr > 0.0 {
            shift_down += clr;
            let clr = p.x_height.abs() * 4.0 / 5.0 - (shift_up - x.depth);
            if clr > 0.0 {
                shift_up += clr;
                shift_down -= clr;
            }
        }
        let width = (x.width + delta).max(y.width);
        let height = x.height + shift_up;
        let depth = y.depth + shift_down;
        let scripts = MathBox {
            kind: BoxKind::VBox(vec![
                Child {
                    dx: delta,
                    dy: -shift_up,
                    content: x,
                },
                Child {
                    dx: 0.0,
                    dy: shift_down,
                    content: y,
                },
            ]),
            width,
            height,
            depth,
        };
        MathBox::hbox(vec![(0.0, nucleus), (0.0, scripts)])
    }

    /// Rule 15 and `make_fraction`, with null delimiters on both sides.
    fn make_fraction(
        &mut self,
        num: &MathList,
        den: &MathList,
        thickness: Option<f64>,
        style: Style,
    ) -> MathBox {
        let p = self.params(style);
        let theta = thickness.unwrap_or(p.default_rule_thickness);
        let mut x = self.clean_box(num, style.num());
        let mut z = self.clean_box(den, style.denom());
        let (mut u, mut v) = if style.is_display() {
            (p.num1, p.denom1)
        } else if theta != 0.0 {
            (p.num2, p.denom2)
        } else {
            (p.num3, p.denom2)
        };
        if x.width < z.width {
            x = x.rebox(z.width);
        } else {
            z = z.rebox(x.width);
        }
        let a = p.axis_height;
        if theta == 0.0 {
            let clr = if style.is_display() { 7.0 } else { 3.0 } * p.default_rule_thickness;
            let delta = (clr - ((u - x.depth) - (z.height - v))) / 2.0;
            if delta > 0.0 {
                u += delta;
                v += delta;
            }
        } else {
            let clr = if style.is_display() {
                3.0 * theta
            } else {
                theta
            };
            let delta = theta / 2.0;
            let delta1 = clr - ((u - x.depth) - (a + delta));
            let delta2 = clr - ((a - delta) - (z.height - v));
            if delta1 > 0.0 {
                u += delta1;
            }
            if delta2 > 0.0 {
                v += delta2;
            }
        }
        let w = x.width;
        let mut children = vec![Child {
            dx: 0.0,
            dy: -u,
            content: x,
        }];
        if theta != 0.0 {
            // The rule is centred on the axis: from a+θ/2 to a−θ/2.
            children.push(Child {
                dx: 0.0,
                dy: -(a - theta / 2.0),
                content: MathBox::rule(w, theta, 0.0),
            });
        }
        let height = children[0].content.height + u;
        let depth = z.depth + v;
        children.push(Child {
            dx: 0.0,
            dy: v,
            content: z,
        });
        let body = MathBox {
            kind: BoxKind::VBox(children),
            width: w,
            height,
            depth,
        };
        MathBox::hlist(vec![
            MathBox::kern(p.null_delimiter_space),
            body,
            MathBox::kern(p.null_delimiter_space),
        ])
    }

    /// `var_delimiter` without its final axis shift: the first glyph in
    /// `sizes` at least `wanted` tall, else the largest (reported).
    fn var_delimiter(
        &mut self,
        sizes: &[Glyph],
        wanted: f64,
        on_missing: impl FnOnce(f64, f64) -> Limitation,
    ) -> Option<MathBox> {
        let chosen = sizes
            .iter()
            .find(|g| g.total_height() >= wanted)
            .or_else(|| sizes.last())?;
        if chosen.total_height() < wanted {
            self.limitations
                .push(on_missing(wanted, chosen.total_height()));
        }
        let mut b = MathBox::glyph(chosen);
        // `char_box` widths include the italic correction.
        b.width += chosen.italic;
        Some(b)
    }

    /// Rule 11 and `make_radical`.
    fn make_radical(&mut self, radicand: &MathList, style: Style) -> MathBox {
        let p = self.params(style);
        let x = self.clean_box(radicand, style.cramped());
        let theta = p.default_rule_thickness;
        let mut clr = if style.is_display() {
            theta + p.x_height.abs() / 4.0
        } else {
            theta + theta / 4.0
        };
        let wanted = x.height + x.depth + clr + theta;
        let sizes = self.m.radical_sizes(style.size_class());
        let Some(y) = self.var_delimiter(&sizes, wanted, |wanted, used| {
            Limitation::RadicalTooSmall { wanted, used }
        }) else {
            self.limitations.push(Limitation::MissingGlyph('\u{221A}'));
            return x;
        };
        let delta = y.depth - (x.height + x.depth + clr);
        if delta > 0.0 {
            clr += delta / 2.0;
        }
        // The sign's baseline is raised so its top meets the rule's top.
        let sign_dy = -(x.height + clr);
        let rule_thickness = y.height;
        let bar = MathBox::rule(x.width, rule_thickness, 0.0);
        // `overbar(x, clr, t)` = vpack(kern t, rule t, kern clr, x): TeX adds
        // an extra kern of the rule thickness above the rule, so the box is
        // taller than the sign by exactly one rule thickness.
        let overbar = MathBox {
            width: x.width,
            height: x.height + clr + 2.0 * rule_thickness,
            depth: x.depth,
            kind: BoxKind::VBox(vec![
                Child {
                    dx: 0.0,
                    dy: -(x.height + clr),
                    content: bar,
                },
                Child {
                    dx: 0.0,
                    dy: 0.0,
                    content: x,
                },
            ]),
        };
        MathBox::hbox(vec![(sign_dy, y), (0.0, overbar)])
    }

    /// Rule 12 and `make_math_accent`.
    fn make_accent(&mut self, accent: char, base: &MathList, style: Style) -> MathBox {
        let p = self.params(style);
        let x = self.clean_box(base, style.cramped());
        let sizes = self.m.accent_sizes(accent, style.size_class());
        if sizes.is_empty() {
            self.limitations.push(Limitation::MissingAccent(accent));
            return x;
        }
        let w = x.width;
        let h = x.height;
        // Skew only applies when the base is a single unscripted symbol.
        let s = match base.atoms.as_slice() {
            [
                Atom {
                    nucleus: Nucleus::Symbol(ch),
                    superscript: None,
                    subscript: None,
                    ..
                },
            ] => self
                .m
                .glyph(*ch, style.size_class())
                .map(|g| g.skew)
                .unwrap_or(0.0),
            _ => 0.0,
        };
        let mut chosen = &sizes[0];
        for g in &sizes[1..] {
            if g.width <= w {
                chosen = g;
            } else {
                break;
            }
        }
        let delta = h.min(p.x_height);
        let y = MathBox::glyph(chosen);
        let accent_dx = s + (w - y.width) / 2.0;
        // Stack: accent, kern −δ, base; baseline at the base's baseline.
        let accent_dy = -(h - delta) - y.depth;
        let mut height = (h - delta) + y.depth + y.height;
        if height < h {
            height = h;
        }
        MathBox {
            width: w,
            height,
            depth: x.depth,
            kind: BoxKind::VBox(vec![
                Child {
                    dx: accent_dx,
                    dy: accent_dy,
                    content: y,
                },
                Child {
                    dx: 0.0,
                    dy: 0.0,
                    content: x,
                },
            ]),
        }
    }

    /// Rule 19 and `make_left_right`.
    fn make_left_right(
        &mut self,
        left: Option<char>,
        right: Option<char>,
        body: &MathList,
        style: Style,
    ) -> MathBox {
        let p = self.params(style);
        let inner = self.clean_box(body, style);
        let a = p.axis_height;
        let delta1 = (inner.height - a).max(inner.depth + a);
        let wanted = (delta1 * 2.0 * p.delimiter_factor).max(2.0 * delta1 - p.delimiter_shortfall);
        let open = self.left_right_delimiter(left, wanted, style, &p);
        let close = self.left_right_delimiter(right, wanted, style, &p);
        MathBox::hlist(vec![open, inner, close])
    }

    fn left_right_delimiter(
        &mut self,
        ch: Option<char>,
        wanted: f64,
        style: Style,
        p: &MathParams,
    ) -> MathBox {
        let Some(ch) = ch else {
            return MathBox::kern(p.null_delimiter_space);
        };
        let sizes = self.m.delimiter_sizes(ch, style.size_class());
        match self.var_delimiter(&sizes, wanted, |wanted, used| {
            Limitation::DelimiterTooSmall { ch, wanted, used }
        }) {
            // Centre the delimiter on the axis (`var_delimiter`'s last step).
            Some(b) => {
                let shift = (b.height - b.depth) / 2.0 - p.axis_height;
                b.shifted(shift)
            }
            None => {
                self.limitations.push(Limitation::MissingGlyph(ch));
                MathBox::kern(p.null_delimiter_space)
            }
        }
    }
}

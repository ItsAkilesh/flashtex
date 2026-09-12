//! Explicit box output and the flattener that turns it into positioned glyph
//! and rule runs.
//!
//! Coordinates inside a box follow TeX: the reference point is the left end of
//! the baseline, `height` extends above it and `depth` below it. Children of a
//! container carry an explicit offset `(dx, dy)` from the parent's reference
//! point, with `dy` positive **downwards** (TeX's `shift_amount` sign), so a
//! superscript has negative `dy` and a subscript positive `dy`. That makes the
//! flattening step a pure translation with no per-kind rules.

use crate::metrics::FontId;

#[derive(Debug, Clone, PartialEq)]
pub enum BoxKind {
    /// One glyph drawn from `font_id` at `size` pt.
    Glyph {
        font_id: FontId,
        gid: u16,
        ch: char,
        size: f64,
    },
    /// A filled rectangle occupying the box's width/height/depth.
    Rule,
    /// Children laid out left to right; each child's `dx` is already absolute
    /// within the box.
    HBox(Vec<Child>),
    /// Children stacked vertically; each child's `dy` is absolute within the box.
    VBox(Vec<Child>),
    /// Horizontal space with no ink (italic corrections, script space, …).
    Kern,
    /// Inter-atom spacing glue at its natural width; `stretch`/`shrink` are
    /// the finite glue components in pt (tex.web §716 `math_glue`) and `mu`
    /// the natural width in mu, for a line breaker that sets the line.
    Glue { mu: f64, stretch: f64, shrink: f64 },
}

/// A child box positioned inside a container.
#[derive(Debug, Clone, PartialEq)]
pub struct Child {
    /// Horizontal offset of the child's reference point from the parent's.
    pub dx: f64,
    /// Vertical offset of the child's baseline from the parent's baseline,
    /// positive downwards.
    pub dy: f64,
    pub content: MathBox,
}

/// A laid-out box with TeX dimensions in points.
#[derive(Debug, Clone, PartialEq)]
pub struct MathBox {
    pub kind: BoxKind,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

impl MathBox {
    pub fn empty() -> MathBox {
        MathBox {
            kind: BoxKind::HBox(Vec::new()),
            width: 0.0,
            height: 0.0,
            depth: 0.0,
        }
    }

    pub fn kern(width: f64) -> MathBox {
        MathBox {
            kind: BoxKind::Kern,
            width,
            height: 0.0,
            depth: 0.0,
        }
    }

    pub fn glue(width: f64, mu: f64) -> MathBox {
        MathBox::glue_with(width, mu, 0.0, 0.0)
    }

    /// Glue with finite stretch and shrink in pt.
    pub fn glue_with(width: f64, mu: f64, stretch: f64, shrink: f64) -> MathBox {
        MathBox {
            kind: BoxKind::Glue {
                mu,
                stretch,
                shrink,
            },
            width,
            height: 0.0,
            depth: 0.0,
        }
    }

    /// A rule of `width` × (`height` + `depth`) around the baseline.
    pub fn rule(width: f64, height: f64, depth: f64) -> MathBox {
        MathBox {
            kind: BoxKind::Rule,
            width,
            height,
            depth,
        }
    }

    pub fn glyph(g: &crate::metrics::Glyph) -> MathBox {
        MathBox {
            kind: BoxKind::Glyph {
                font_id: g.font_id,
                gid: g.gid,
                ch: g.ch,
                size: g.size,
            },
            width: g.width,
            height: g.height,
            depth: g.depth,
        }
    }

    pub fn total_height(&self) -> f64 {
        self.height + self.depth
    }

    /// TeX `hpack(..., natural)`: boxes side by side, each shifted by `dy`.
    pub fn hbox(items: Vec<(f64, MathBox)>) -> MathBox {
        let mut children = Vec::with_capacity(items.len());
        let (mut x, mut height, mut depth) = (0.0f64, 0.0f64, 0.0f64);
        for (dy, b) in items {
            height = height.max(b.height - dy);
            depth = depth.max(b.depth + dy);
            let w = b.width;
            children.push(Child {
                dx: x,
                dy,
                content: b,
            });
            x += w;
        }
        MathBox {
            kind: BoxKind::HBox(children),
            width: x,
            height,
            depth,
        }
    }

    /// An hbox with a single unshifted child list.
    pub fn hlist(boxes: Vec<MathBox>) -> MathBox {
        MathBox::hbox(boxes.into_iter().map(|b| (0.0, b)).collect())
    }

    /// TeX `vpack(..., natural)` with the baseline at the bottom item's
    /// baseline: items are stacked top to bottom; `dx` shifts an item right.
    /// `kerns` between items are expressed as [`MathBox::kern`] boxes whose
    /// `width` is reinterpreted as vertical size.
    pub fn vbox(items: Vec<(f64, MathBox)>) -> MathBox {
        let mut children = Vec::with_capacity(items.len());
        let mut width = 0.0f64;
        // Lay out from the top with y measured downwards from the top edge.
        let mut y = 0.0f64;
        let mut last_baseline = 0.0f64;
        let mut placed = Vec::with_capacity(items.len());
        for (dx, b) in items {
            if matches!(b.kind, BoxKind::Kern) {
                y += b.width;
                continue;
            }
            width = width.max(b.width + dx);
            let baseline = y + b.height;
            let bottom = baseline + b.depth;
            placed.push((dx, baseline, b));
            last_baseline = baseline;
            y = bottom;
        }
        let total = y;
        let height = last_baseline;
        let depth = total - last_baseline;
        for (dx, baseline, b) in placed {
            children.push(Child {
                dx,
                dy: baseline - height,
                content: b,
            });
        }
        MathBox {
            kind: BoxKind::VBox(children),
            width,
            height,
            depth,
        }
    }

    /// TeX `vpack` with the baseline at the **top** item's baseline
    /// (`vtop`).
    pub fn vtop(items: Vec<(f64, MathBox)>) -> MathBox {
        let mut b = MathBox::vbox(items);
        if let BoxKind::VBox(children) = &mut b.kind
            && let Some(first) = children.first()
        {
            let shift = first.dy;
            let first_height = first.content.height;
            for c in children.iter_mut() {
                c.dy -= shift;
            }
            let total = b.height + b.depth;
            b.height = first_height;
            b.depth = total - first_height;
        }
        b
    }

    /// TeX `rebox`: centre this box in a box of width `w` (Rule 15, 13).
    pub fn rebox(self, w: f64) -> MathBox {
        if self.width >= w {
            return self;
        }
        let pad = (w - self.width) / 2.0;
        MathBox::hlist(vec![MathBox::kern(pad), self, MathBox::kern(pad)])
    }

    /// Wrap in an hbox shifted by `dy` (positive down), as TeX does when it
    /// `hpack`s a box that carries a `shift_amount`.
    pub fn shifted(self, dy: f64) -> MathBox {
        MathBox::hbox(vec![(dy, self)])
    }
}

/// A glyph placed on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedGlyph {
    pub font_id: FontId,
    pub gid: u16,
    pub ch: char,
    /// Left edge of the glyph's advance box, in pt from the origin's x.
    pub x: f64,
    /// Baseline, in pt downward from the origin's y.
    pub baseline_y: f64,
    pub size: f64,
}

/// A filled rectangle on the page; `y` is its top edge (downward axis).
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedRule {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PositionedRuns {
    pub glyphs: Vec<PositionedGlyph>,
    pub rules: Vec<PositionedRule>,
}

/// Flattens a box into glyph and rule runs.
///
/// `origin` is the **top-left corner** of the box in a y-down coordinate
/// system (points): the box's baseline sits at `origin.1 + root.height`.
/// To place on a PDF page (y up), use `y_pdf = page_height - y`; a glyph's
/// text matrix origin is `(x, y_pdf(baseline_y))` and a rule is the rectangle
/// `(x, y_pdf(y + h), w, h)`.
pub fn positioned_runs(root: &MathBox, origin: (f64, f64)) -> PositionedRuns {
    let mut out = PositionedRuns::default();
    walk(root, origin.0, origin.1 + root.height, &mut out);
    out
}

fn walk(b: &MathBox, x: f64, baseline: f64, out: &mut PositionedRuns) {
    match &b.kind {
        BoxKind::Glyph {
            font_id,
            gid,
            ch,
            size,
        } => out.glyphs.push(PositionedGlyph {
            font_id: *font_id,
            gid: *gid,
            ch: *ch,
            x,
            baseline_y: baseline,
            size: *size,
        }),
        BoxKind::Rule => out.rules.push(PositionedRule {
            x,
            y: baseline - b.height,
            w: b.width,
            h: b.height + b.depth,
        }),
        BoxKind::HBox(children) | BoxKind::VBox(children) => {
            for c in children {
                walk(&c.content, x + c.dx, baseline + c.dy, out);
            }
        }
        BoxKind::Kern | BoxKind::Glue { .. } => {}
    }
}

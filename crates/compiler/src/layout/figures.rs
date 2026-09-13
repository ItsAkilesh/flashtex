//! Float placement and `\includegraphics` boxes.
//!
//! Follows LaTeX's own output-routine decisions (`\@addtocurcol`,
//! `\@addtotoporbot`, `\@addtobot`, `\@startcolumn`, `\@tryfcolumn`,
//! `\@doclearpage`) with article.cls parameters: `topnumber` 2,
//! `bottomnumber` 1, `totalnumber` 3, `\topfraction` .7, `\bottomfraction`
//! .3, `\textfraction` .2, `\floatpagefraction` .5, `\floatsep` 12pt,
//! `\textfloatsep` 20pt, `\intextsep` 12pt (14pt in the 12pt class),
//! `\@fpsep` 8pt (10pt in the 12pt class). Glue is fixed at its natural size:
//! this layout model has no stretch or shrink.
//!
//! Top and bottom floats of the page being built are held aside and merged
//! into the page when it is finished, so text positions never depend on the
//! order in which float items were pushed. A document containing a float is
//! always laid out globally (see `parser::figures`), so held-aside state never
//! meets the incremental block cache.

use super::{
    round2, LayoutCursor, Page, TextItem, LINE_SPACING, MARGIN_PT, PAGE_HEIGHT_PT, PAGE_WIDTH_PT,
};
use crate::math::{MathBox, MathItem, MathRule, FRACTION_RULE_CHAR};
use crate::parser::{Extent, Float, FloatKind, FloatPlacement, GraphicSize};
use crate::Span;

const TEXT_HEIGHT_PT: f64 = PAGE_HEIGHT_PT - 2.0 * MARGIN_PT;
const TOP_NUMBER: usize = 2;
const BOTTOM_NUMBER: usize = 1;
const TOTAL_NUMBER: usize = 3;
const TOP_FRACTION: f64 = 0.7;
const BOTTOM_FRACTION: f64 = 0.3;
const TEXT_FRACTION: f64 = 0.2;
const FLOAT_PAGE_FRACTION: f64 = 0.5;
const FLOAT_SEP_PT: f64 = 12.0;
const TEXT_FLOAT_SEP_PT: f64 = 20.0;
/// article.cls `\abovecaptionskip`.
pub(super) const ABOVE_CAPTION_SKIP_PT: f64 = 10.0;
/// `\fboxrule`, used for the draft-mode graphic frame.
const FRAME_RULE_PT: f64 = 0.4;
/// Side of the placeholder frame when neither `width` nor `height` is given.
/// The real natural size lives in image bytes the compiler never receives.
pub(crate) const UNKNOWN_NATURAL_SIZE_PT: f64 = 144.0;

/// `\intextsep`: size10/size11 12pt, size12 14pt.
fn in_text_sep(body_size: f64) -> f64 {
    if body_size > 11.5 {
        14.0
    } else {
        12.0
    }
}

/// `\@fpsep`: size10/size11 8pt, size12 10pt.
fn float_page_sep(body_size: f64) -> f64 {
    if body_size > 11.5 {
        10.0
    } else {
        8.0
    }
}

/// A float body laid out once, with coordinates relative to its box top.
#[derive(Debug, Clone)]
pub(super) struct Prepared {
    kind: FloatKind,
    placement: FloatPlacement,
    height: f64,
    items: Vec<TextItem>,
    labels: Vec<(String, String)>,
}

#[derive(Debug, Default)]
pub(super) struct FloatState {
    /// A scratch cursor measuring a float body: no page limit.
    measuring: bool,
    deferred: Vec<Prepared>,
    top: Vec<Prepared>,
    bottom: Vec<Prepared>,
    /// Height taken from the top of the current page, separators included.
    top_area: f64,
    bottom_area: f64,
    /// Kinds and total height (with `\intextsep` twice) of `h` floats set in
    /// the current page's text.
    here_kinds: Vec<FloatKind>,
    here_height: f64,
}

impl LayoutCursor {
    /// Lowest baseline allowed on the current page.
    pub(super) fn page_bottom(&self) -> f64 {
        if self.floats.measuring {
            f64::INFINITY
        } else {
            PAGE_HEIGHT_PT - MARGIN_PT - self.floats.bottom_area
        }
    }

    /// Ship the current page and open the next one whose first baseline sits
    /// `first_line_pt` below the top margin, placing deferred floats first.
    pub(super) fn break_page(&mut self, first_line_pt: f64) {
        self.finish_page_floats();
        self.push_page();
        self.y = MARGIN_PT + first_line_pt;
        self.start_page_floats();
    }

    fn push_page(&mut self) {
        let number = self.pages.len() as u32 + 1;
        self.pages.push(Page {
            number,
            width_pt: PAGE_WIDTH_PT,
            height_pt: PAGE_HEIGHT_PT,
            items: Vec::new(),
        });
    }

    fn page_has_text(&self) -> bool {
        self.pages.last().is_some_and(|page| !page.items.is_empty())
    }

    /// `\@pageht`: text (and in-text floats) already on the page.
    fn text_height(&self) -> f64 {
        if self.page_has_text() {
            (self.y + self.line_descent - MARGIN_PT - self.floats.top_area).max(0.0)
        } else {
            0.0
        }
    }

    fn column_room(&self) -> f64 {
        TEXT_HEIGHT_PT - self.floats.top_area - self.floats.bottom_area
    }

    fn float_count(&self) -> usize {
        self.floats.top.len() + self.floats.bottom.len() + self.floats.here_kinds.len()
    }

    fn text_min(placement: FloatPlacement) -> f64 {
        if placement.force {
            0.0
        } else {
            TEXT_FRACTION * TEXT_HEIGHT_PT
        }
    }

    /// Lay out one float block at the current point of the text.
    pub(super) fn render_float(&mut self, float: &Float) {
        if self.floats.measuring {
            // Nested floats are diagnosed by the parser; keep their content.
            for block in &float.body {
                self.prepare_block(block);
                self.render_prepared_block(block);
            }
            return;
        }
        let prepared = self.measure_float(float);
        let placement = prepared.placement;
        if placement.here_definitely {
            self.place_in_text(prepared, false);
            return;
        }
        // `\@addtocurcol`.
        let only_page = placement.page && !(placement.here || placement.top || placement.bottom);
        let blocked = self
            .floats
            .deferred
            .iter()
            .any(|deferred| deferred.kind == prepared.kind);
        let required = self
            .text_height()
            .max(Self::text_min(placement) + self.floats.here_height)
            + prepared.height;
        if only_page
            || blocked
            || self.column_room() <= required
            || (!placement.force && self.float_count() >= TOTAL_NUMBER)
        {
            self.floats.deferred.push(prepared);
            return;
        }
        // A same-kind float already at the bottom forces this one below it.
        if self.floats.bottom.iter().any(|f| f.kind == prepared.kind) {
            if let Err(prepared) = self.add_to_bottom(prepared) {
                self.floats.deferred.push(prepared);
            }
            return;
        }
        let body_size = self.constraints.font_size_pt;
        if placement.here && self.column_room() > required + in_text_sep(body_size) {
            self.place_in_text(prepared, true);
            return;
        }
        if let Err(prepared) = self.add_to_top_or_bottom(prepared) {
            self.floats.deferred.push(prepared);
        }
    }

    /// Lay a float body out on a scratch cursor at full text width.
    fn measure_float(&mut self, float: &Float) -> Prepared {
        let mut constraints = self.constraints;
        // `\@parboxrestore` inside floats: no paragraph skip.
        constraints.parskip_pt = Some(0.0);
        let mut scratch = LayoutCursor::with_labels(
            constraints,
            self.resolved_labels.clone(),
            self.emit_heading_numbers,
        );
        scratch.floats.measuring = true;
        for block in &float.body {
            scratch.prepare_block(block);
            scratch.render_prepared_block(block);
        }
        let (pages, labels, diagnostics) = scratch.into_result();
        self.diagnostics.extend(diagnostics);
        let mut items: Vec<TextItem> = pages.into_iter().flat_map(|page| page.items).collect();
        let bottom = items
            .iter()
            .map(|item| match item.rule {
                Some(rule) => rule.y_pt + rule.height_pt,
                None => item.baseline_y_pt + (LINE_SPACING - 1.0) * item.font_size_pt,
            })
            .fold(MARGIN_PT, f64::max);
        for item in &mut items {
            offset_item(item, -MARGIN_PT);
        }
        Prepared {
            kind: float.kind,
            placement: float.placement,
            height: bottom - MARGIN_PT,
            items,
            labels: labels
                .into_iter()
                .map(|(key, value)| (key, value.number))
                .collect(),
        }
    }

    /// Put a prepared float's items on the last page with its top at `top`.
    fn put_float(&mut self, float: Prepared, top: f64) -> Vec<TextItem> {
        let page = self.pages.len() as u32;
        for (key, number) in float.labels {
            self.collected_labels
                .insert(key, super::ReferenceValue { number, page });
        }
        let mut items = float.items;
        for item in &mut items {
            offset_item(item, top);
        }
        items
    }

    /// An `h` float (`counted`) or a `float`-package `H` box, set in the text
    /// between `\intextsep` glue.
    fn place_in_text(&mut self, float: Prepared, counted: bool) {
        self.resolve_hfill();
        self.align_current_line();
        let body_size = self.constraints.font_size_pt;
        let sep = in_text_sep(body_size);
        let mut top = if self.page_has_text() {
            self.y + self.line_descent + sep
        } else {
            MARGIN_PT + self.floats.top_area
        };
        if self.page_has_text() && top + float.height > self.page_bottom() {
            // An unbreakable box that does not fit moves to the next page.
            self.break_page(body_size);
            top = MARGIN_PT + self.floats.top_area;
        }
        if counted {
            self.floats.here_kinds.push(float.kind);
            self.floats.here_height += float.height + 2.0 * sep;
        }
        let height = float.height;
        let items = self.put_float(float, top);
        self.pages
            .last_mut()
            .expect("at least one page")
            .items
            .extend(items);
        self.y = top + height + sep;
        self.line_ascent = body_size;
        self.line_descent = body_size * (LINE_SPACING - 1.0);
        self.x = self.left_edge();
        self.content_end = self.x;
        self.line_start = self.pages.last().expect("at least one page").items.len();
        self.first_block = false;
    }

    /// `\@addtotoporbot`.
    fn add_to_top_or_bottom(&mut self, float: Prepared) -> Result<(), Prepared> {
        let float = match self.add_to_top(float) {
            Ok(()) => return Ok(()),
            Err(float) => float,
        };
        self.add_to_bottom(float)
    }

    /// `\@flcheckspace`: the float plus the text already set must fit.
    fn fits(&self, float: &Prepared, separator: f64, room: f64) -> bool {
        let required = self
            .text_height()
            .max(Self::text_min(float.placement) + self.floats.here_height)
            + float.height
            + separator;
        self.column_room() >= required && (float.placement.force || float.height <= room)
    }

    fn add_to_top(&mut self, float: Prepared) -> Result<(), Prepared> {
        let placement = float.placement;
        let order_blocked = self.floats.here_kinds.contains(&float.kind)
            || self.floats.bottom.iter().any(|f| f.kind == float.kind);
        let separator = if self.floats.top.is_empty() {
            TEXT_FLOAT_SEP_PT
        } else {
            FLOAT_SEP_PT
        };
        let room = TOP_FRACTION * TEXT_HEIGHT_PT - self.floats.top_area;
        if !placement.top
            || order_blocked
            || (!placement.force && self.floats.top.len() >= TOP_NUMBER)
            || !self.fits(&float, separator, room)
        {
            return Err(float);
        }
        let shift = float.height + separator;
        if let Some(page) = self.pages.last_mut() {
            for item in &mut page.items {
                offset_item(item, shift);
            }
        }
        self.y += shift;
        self.floats.top_area += shift;
        self.floats.top.push(float);
        Ok(())
    }

    fn add_to_bottom(&mut self, float: Prepared) -> Result<(), Prepared> {
        let placement = float.placement;
        let separator = if self.floats.bottom.is_empty() {
            TEXT_FLOAT_SEP_PT
        } else {
            FLOAT_SEP_PT
        };
        let room = BOTTOM_FRACTION * TEXT_HEIGHT_PT - self.floats.bottom_area;
        if !placement.bottom
            || (!placement.force && self.floats.bottom.len() >= BOTTOM_NUMBER)
            || !self.fits(&float, separator, room)
        {
            return Err(float);
        }
        self.floats.bottom_area += float.height + separator;
        self.floats.bottom.push(float);
        Ok(())
    }

    /// `\@startcolumn`: float pages from the deferred list, then top/bottom
    /// placement of what remains, preserving order within each kind.
    fn start_page_floats(&mut self) {
        if self.floats.measuring {
            return;
        }
        let first_line = self.y;
        while self.try_float_page(false) {
            self.push_page();
            self.y = first_line;
        }
        let queue = std::mem::take(&mut self.floats.deferred);
        for float in queue {
            let placement = float.placement;
            let only_page =
                placement.page && !(placement.here || placement.top || placement.bottom);
            let blocked = self
                .floats
                .deferred
                .iter()
                .any(|deferred| deferred.kind == float.kind);
            let fits = self.column_room() > float.height + Self::text_min(placement)
                && (placement.force || self.float_count() < TOTAL_NUMBER);
            if only_page || blocked || !fits {
                self.floats.deferred.push(float);
            } else if let Err(float) = self.add_to_top_or_bottom(float) {
                self.floats.deferred.push(float);
            }
        }
    }

    /// `\@tryfcolumn` (or `\@makefcolumn` when `forced`): set deferred floats
    /// on the current, empty page as a float page. Returns whether one was made.
    fn try_float_page(&mut self, forced: bool) -> bool {
        let mut chosen = Vec::new();
        let mut blocked = Vec::new();
        let mut total = 0.0;
        let body_size = self.constraints.font_size_pt;
        let sep = float_page_sep(body_size);
        for (index, float) in self.floats.deferred.iter().enumerate() {
            if blocked.contains(&float.kind) {
                continue;
            }
            let added = if chosen.is_empty() {
                float.height
            } else {
                sep + float.height
            };
            if (forced || float.placement.page) && total + added <= TEXT_HEIGHT_PT {
                chosen.push(index);
                total += added;
            } else {
                blocked.push(float.kind);
            }
        }
        if forced && chosen.is_empty() && !self.floats.deferred.is_empty() {
            // Taller than a page: set it alone, overfull, as LaTeX does.
            chosen.push(0);
            total = self.floats.deferred[0].height;
        }
        if chosen.is_empty() || (!forced && total < FLOAT_PAGE_FRACTION * TEXT_HEIGHT_PT) {
            return false;
        }
        // `\@fptop` 1fil, `\@fpsep` +2fil, `\@fpbot` 1fil.
        let fil = (TEXT_HEIGHT_PT - total).max(0.0) / (2.0 * chosen.len() as f64);
        let mut floats = Vec::with_capacity(chosen.len());
        for index in chosen.into_iter().rev() {
            floats.push(self.floats.deferred.remove(index));
        }
        floats.reverse();
        let mut top = MARGIN_PT + fil;
        let mut items = Vec::new();
        for float in floats {
            let height = float.height;
            items.extend(self.put_float(float, top));
            top += height + sep + 2.0 * fil;
        }
        self.pages
            .last_mut()
            .expect("at least one page")
            .items
            .extend(items);
        true
    }

    /// `\@makecol`: merge the held-aside top and bottom floats into the page.
    fn finish_page_floats(&mut self) {
        let top = std::mem::take(&mut self.floats.top);
        let bottom = std::mem::take(&mut self.floats.bottom);
        self.floats.top_area = 0.0;
        self.floats.bottom_area = 0.0;
        self.floats.here_kinds.clear();
        self.floats.here_height = 0.0;
        if top.is_empty() && bottom.is_empty() {
            return;
        }
        let mut before = Vec::new();
        let mut y = MARGIN_PT;
        for float in top {
            let height = float.height;
            before.extend(self.put_float(float, y));
            y += height + FLOAT_SEP_PT;
        }
        let mut after = Vec::new();
        let mut y = PAGE_HEIGHT_PT - MARGIN_PT;
        for float in bottom.into_iter().rev() {
            y -= float.height;
            let top = y;
            after.splice(0..0, self.put_float(float, top));
            y -= FLOAT_SEP_PT;
        }
        let page = self.pages.last_mut().expect("at least one page");
        before.append(&mut page.items);
        before.append(&mut after);
        page.items = before;
    }

    /// `\clearpage` at the end of the document: every remaining float is set
    /// on float pages, regardless of its placement specifier.
    pub(super) fn flush_floats(&mut self) {
        if self.floats.measuring
            || (self.floats.deferred.is_empty()
                && self.floats.top.is_empty()
                && self.floats.bottom.is_empty())
        {
            return;
        }
        if self.page_has_text() {
            self.finish_page_floats();
        } else {
            // `\@doclearpage`: an empty page's top and bottom floats rejoin
            // the deferred list ahead of it.
            let mut queue = std::mem::take(&mut self.floats.top);
            queue.append(&mut self.floats.bottom);
            queue.append(&mut self.floats.deferred);
            self.floats.deferred = queue;
            self.finish_page_floats();
        }
        while !self.floats.deferred.is_empty() {
            if self.page_has_text() {
                self.push_page();
            }
            self.try_float_page(true);
        }
    }
}

fn offset_item(item: &mut TextItem, dy: f64) {
    item.baseline_y_pt = round2(item.baseline_y_pt + dy);
    if let Some(rule) = item.rule.as_mut() {
        rule.y_pt = round2(rule.y_pt + dy);
    }
}

/// Resolved `(width, height)` of an `\includegraphics` box, in points.
pub(super) fn graphic_extent(size: GraphicSize, text_width: f64, line_width: f64) -> (f64, f64) {
    let resolve = |extent: Extent| match extent {
        Extent::Pt(pt) => pt,
        Extent::TextWidth(factor) => factor * text_width,
        Extent::LineWidth(factor) => factor * line_width,
        Extent::TextHeight(factor) => factor * TEXT_HEIGHT_PT,
    };
    let (width, height) = match (size.width.map(resolve), size.height.map(resolve)) {
        (Some(width), Some(height)) => (width, height),
        // Without the natural aspect ratio, the missing side mirrors the given one.
        (Some(width), None) => (width, width),
        (None, Some(height)) => (height, height),
        (None, None) => {
            let side = UNKNOWN_NATURAL_SIZE_PT * size.scale;
            (side, side)
        }
    };
    (width.max(0.0), height.max(0.0))
}

/// graphicx `draft` rendering: a `\fbox`-ruled frame of the requested size,
/// sitting on the baseline, with the file name in typewriter type.
pub(super) fn graphic_box(file: &str, width: f64, height: f64, size: f64, span: Span) -> MathBox {
    let rule = |x: f64, y: f64, w: f64, h: f64| MathItem {
        font: None,
        text: FRACTION_RULE_CHAR.to_string(),
        x,
        baseline: y + h,
        size,
        span,
        rule: Some(MathRule {
            y,
            width: w,
            height: h,
        }),
    };
    let mut items = Vec::new();
    if width > 0.0 && height > 0.0 {
        let thickness = FRAME_RULE_PT.min(width).min(height);
        items.push(rule(0.0, -height, width, thickness));
        items.push(rule(0.0, -thickness, width, thickness));
        items.push(rule(0.0, -height, thickness, height));
        items.push(rule(width - thickness, -height, thickness, height));
        items.push(MathItem {
            font: Some(super::Font::Courier),
            text: file.to_string(),
            // `\rlap{ \ttfamily name}`: one typewriter space in from the rule.
            x: thickness + 0.6 * size,
            baseline: -height / 2.0 + 0.3 * size,
            size,
            span,
            rule: None,
        });
    }
    MathBox {
        items,
        width,
        ascent: height,
        descent: 0.0,
    }
}

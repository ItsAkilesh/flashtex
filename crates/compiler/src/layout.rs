//! Layout: turns parsed blocks into positioned items on pages.
//!
//! Line placement uses the shared font engine's Adobe Core 14 shaping:
//! Times-Roman for body text, Times-Bold for headings, and Symbol for supported
//! math glyphs. Kerning and available ligatures are applied. Hyphenation and
//! TeX's optimal paragraph breaking remain missing; breaking here is greedy.
//!
//! One item is emitted per word rather than per line. That keeps each item's
//! source span exact, which is what click-to-source navigation (FT-003) needs.

use crate::diagnostics::Diagnostic;
use crate::export::{self, ExportFont};
use crate::math::{self, MathBox};
use crate::parser::{Block, Inline};
use crate::Span;
use flashtex_font_engine::core14::Core14;
use flashtex_font_engine::shape::{shape, ShapeOptions, Shaped};
use flashtex_font_engine::Core14Face;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub use flashtex_font_engine::core14::Core14 as Font;

pub const PAGE_WIDTH_PT: f64 = 612.0;
pub const PAGE_HEIGHT_PT: f64 = 792.0;
pub const MARGIN_PT: f64 = 72.0;
pub const BODY_SIZE_PT: f64 = 12.0;
pub const LINE_SPACING: f64 = 1.2;
pub const PARAGRAPH_GAP_PT: f64 = 6.0;
/// References normally settle in two passes; the cap also covers page-number
/// changes caused by a resolved reference changing line or page breaks.
pub const REFERENCE_ITERATION_LIMIT: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReferenceValue {
    number: String,
    page: u32,
}

/// Layout inputs that participate in incremental cache validation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutConstraints {
    pub font_size_pt: f64,
    pub measure_pt: f64,
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self {
            font_size_pt: BODY_SIZE_PT,
            measure_pt: PAGE_WIDTH_PT - 2.0 * MARGIN_PT,
        }
    }
}

/// Retained only so math's script-size boxes can be measured consistently with
/// body text while math moves onto real metrics too.
pub fn text_width(text: &str, size: f64, font: Font) -> f64 {
    shape_text(font, text).map_or(0.0, |shaped| shaped.width_pt(size))
}

pub fn word_space(size: f64, font: Font) -> f64 {
    text_width(" ", size, font)
}

fn face(font: Font) -> &'static Core14Face {
    static TIMES_ROMAN: OnceLock<Core14Face> = OnceLock::new();
    static TIMES_BOLD: OnceLock<Core14Face> = OnceLock::new();
    static TIMES_ITALIC: OnceLock<Core14Face> = OnceLock::new();
    static TIMES_BOLD_ITALIC: OnceLock<Core14Face> = OnceLock::new();
    static HELVETICA: OnceLock<Core14Face> = OnceLock::new();
    static COURIER: OnceLock<Core14Face> = OnceLock::new();
    static SYMBOL: OnceLock<Core14Face> = OnceLock::new();
    match font {
        Core14::TimesRoman => TIMES_ROMAN.get_or_init(|| Core14Face::new(font)),
        Core14::TimesBold => TIMES_BOLD.get_or_init(|| Core14Face::new(font)),
        Core14::TimesItalic => TIMES_ITALIC.get_or_init(|| Core14Face::new(font)),
        Core14::TimesBoldItalic => TIMES_BOLD_ITALIC.get_or_init(|| Core14Face::new(font)),
        Core14::Helvetica => HELVETICA.get_or_init(|| Core14Face::new(font)),
        Core14::Courier => COURIER.get_or_init(|| Core14Face::new(font)),
        Core14::Symbol => SYMBOL.get_or_init(|| Core14Face::new(font)),
    }
}

fn shape_text(font: Font, text: &str) -> Result<Shaped, flashtex_font_engine::Error> {
    shape(face(font), text, &ShapeOptions::default())
}

/// Select the same Core 14 face that the export mapping assigns to a math glyph.
pub(crate) fn math_font(text: &str) -> Font {
    if !text.is_empty()
        && text.chars().all(|ch| {
            matches!(
                export::map_char(ch),
                export::Glyph::Encodable {
                    font: ExportFont::Symbol,
                    ..
                }
            )
        })
    {
        Font::Symbol
    } else {
        Font::TimesRoman
    }
}

fn source_span(text: &str, span: Span, shaped: &Shaped) -> Span {
    let Some(first) = shaped.clusters.first() else {
        return span;
    };
    let last = shaped.clusters.last().expect("first cluster exists");
    for cluster in &shaped.clusters {
        debug_assert_eq!(
            text.get(cluster.source_range.clone()),
            Some(cluster.text.as_str())
        );
    }
    if span.end - span.start == text.len() {
        Span::in_document(
            span.document,
            span.start + first.source_range.start,
            span.start + last.source_range.end,
        )
    } else {
        // Macro replacement bytes do not exist in the document. Preserve the
        // invocation attribution instead of leaking shaping-relative offsets.
        span
    }
}

fn relative_span(text: &str, span: Span, start: usize, end: usize) -> Span {
    if span.end - span.start == text.len() && text.get(start..end).is_some() {
        Span::in_document(span.document, span.start + start, span.start + end)
    } else {
        span
    }
}

pub(crate) fn shaped_width(
    text: &str,
    size: f64,
    font: Font,
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) -> (f64, Span) {
    match shape_text(font, text) {
        Ok(shaped) => {
            for missing in &shaped.missing {
                let missing_span = relative_span(
                    text,
                    span,
                    missing.byte_offset,
                    missing.byte_offset + missing.ch.len_utf8(),
                );
                diagnostics.push(Diagnostic::warning(
                    format!(
                        "{} has no glyph for {:?} (U+{:04X})",
                        font.header().font_name,
                        missing.ch,
                        missing.ch as u32
                    ),
                    Some(missing_span),
                    Some("emitted the face's explicit .notdef glyph and continued".into()),
                ));
            }
            (shaped.width_pt(size), source_span(text, span, &shaped))
        }
        Err(error) => {
            let error_span = match error {
                flashtex_font_engine::Error::UnsupportedScript {
                    ch, byte_offset, ..
                } => relative_span(text, span, byte_offset, byte_offset + ch.len_utf8()),
                _ => span,
            };
            diagnostics.push(Diagnostic::error(
                format!(
                    "could not shape text with {}: {error}",
                    font.header().font_name
                ),
                Some(error_span),
                Some("kept the source item with zero advance and continued".into()),
            ));
            (0.0, span)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextItem {
    pub text: String,
    pub x_pt: f64,
    pub baseline_y_pt: f64,
    pub font_size_pt: f64,
    pub span: Span,
    /// The face actually used to measure and lay out this item.
    pub font: Font,
    /// Typed geometry for an item that has a legacy text fallback.
    pub rule: Option<RuleGeometry>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuleGeometry {
    pub y_pt: f64,
    pub width_pt: f64,
    pub height_pt: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub number: u32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub items: Vec<TextItem>,
}

fn glyph_width(text: &str, size: f64, font: Font) -> f64 {
    text_width(text, size, font)
}

/// Geometry needed to resume the one authoritative layout engine.
#[derive(Debug, Clone, Copy)]
pub struct FlowState {
    page_index: usize,
    x: f64,
    y: f64,
    line_ascent: f64,
    line_descent: f64,
    trailing_line_items: usize,
}

impl FlowState {
    pub fn same_geometry(self, other: Self) -> bool {
        self.page_index == other.page_index
            && self.x.to_bits() == other.x.to_bits()
            && self.y.to_bits() == other.y.to_bits()
            && self.line_ascent.to_bits() == other.line_ascent.to_bits()
            && self.line_descent.to_bits() == other.line_descent.to_bits()
            && self.trailing_line_items == other.trailing_line_items
    }
}

/// One positioned item together with the zero-based page it belongs to.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedItem {
    pub page_index: usize,
    pub item: TextItem,
}

/// Resumable layout cursor shared by clean and incremental compilation.
pub struct LayoutCursor {
    pages: Vec<Page>,
    x: f64,
    y: f64,
    line_ascent: f64,
    line_descent: f64,
    line_start: usize,
    first_block: bool,
    constraints: LayoutConstraints,
    resolved_labels: BTreeMap<String, ReferenceValue>,
    collected_labels: BTreeMap<String, ReferenceValue>,
    emit_heading_numbers: bool,
    diagnostics: Vec<Diagnostic>,
}

impl LayoutCursor {
    pub fn new(constraints: LayoutConstraints) -> Self {
        Self::with_labels(constraints, BTreeMap::new(), true)
    }

    fn with_labels(
        constraints: LayoutConstraints,
        resolved_labels: BTreeMap<String, ReferenceValue>,
        emit_heading_numbers: bool,
    ) -> Self {
        LayoutCursor {
            pages: vec![Page {
                number: 1,
                width_pt: PAGE_WIDTH_PT,
                height_pt: PAGE_HEIGHT_PT,
                items: Vec::new(),
            }],
            x: MARGIN_PT,
            y: MARGIN_PT + constraints.font_size_pt,
            line_ascent: constraints.font_size_pt,
            line_descent: constraints.font_size_pt * (LINE_SPACING - 1.0),
            line_start: 0,
            first_block: true,
            constraints,
            resolved_labels,
            collected_labels: BTreeMap::new(),
            emit_heading_numbers,
            diagnostics: Vec::new(),
        }
    }

    fn right_edge(&self) -> f64 {
        MARGIN_PT + self.constraints.measure_pt
    }

    fn newline(&mut self, size: f64) {
        self.x = MARGIN_PT;
        self.y += self.line_descent + size;
        self.line_ascent = size;
        self.line_descent = size * (LINE_SPACING - 1.0);
        if self.y > PAGE_HEIGHT_PT - MARGIN_PT {
            let n = self.pages.len() as u32 + 1;
            self.pages.push(Page {
                number: n,
                width_pt: PAGE_WIDTH_PT,
                height_pt: PAGE_HEIGHT_PT,
                items: Vec::new(),
            });
            self.y = MARGIN_PT + size;
        }
        self.line_start = self.pages.last().expect("at least one page").items.len();
    }

    fn vertical_gap(&mut self, gap: f64) {
        self.x = MARGIN_PT;
        self.y += gap;
    }

    fn place(&mut self, text: String, size: f64, span: Span, font: Font) {
        let (w, span) = shaped_width(&text, size, font, span, &mut self.diagnostics);
        if self.x > MARGIN_PT && self.x + w > self.right_edge() {
            self.newline(size);
        }
        self.ensure_extents(size, size * (LINE_SPACING - 1.0));
        let item = TextItem {
            text,
            x_pt: round2(self.x),
            baseline_y_pt: round2(self.y),
            font_size_pt: size,
            span,
            font,
            rule: None,
        };
        self.pages
            .last_mut()
            .expect("at least one page")
            .items
            .push(item);
        self.x += w + word_space(size, font);
    }

    fn ensure_extents(&mut self, ascent: f64, descent: f64) {
        if ascent > self.line_ascent {
            let shift = ascent - self.line_ascent;
            self.y += shift;
            if let Some(page) = self.pages.last_mut() {
                for item in &mut page.items[self.line_start..] {
                    item.baseline_y_pt = round2(item.baseline_y_pt + shift);
                }
            }
            self.line_ascent = ascent;
        }
        self.line_descent = self.line_descent.max(descent);
    }

    fn place_math(&mut self, b: MathBox, size: f64) {
        if self.x > MARGIN_PT && self.x + b.width > self.right_edge() {
            self.newline(size);
        }
        self.ensure_extents(b.ascent, b.descent);
        let base_x = self.x;
        let base_y = self.y;
        let page = self.pages.last_mut().expect("at least one page");
        for item in b.items {
            let font = item.font.unwrap_or_else(|| math_font(&item.text));
            let rule = item.rule.map(|rule| RuleGeometry {
                y_pt: round2(base_y + rule.y),
                width_pt: round2(rule.width),
                height_pt: round2(rule.height),
            });
            page.items.push(TextItem {
                text: item.text,
                x_pt: round2(base_x + item.x),
                baseline_y_pt: round2(base_y + item.baseline),
                font_size_pt: item.size,
                span: item.span,
                font,
                rule,
            });
        }
        self.x += b.width + word_space(size, Font::TimesRoman);
    }

    fn display_math(&mut self, b: MathBox, size: f64, number: Option<(&str, Span)>) {
        if self.x > MARGIN_PT
            || self
                .pages
                .last()
                .is_some_and(|p| p.items.len() > self.line_start)
        {
            self.newline(size);
        }
        self.vertical_gap(PARAGRAPH_GAP_PT);
        self.x = MARGIN_PT + (self.right_edge() - MARGIN_PT - b.width).max(0.0) / 2.0;
        self.place_math(b, size);
        if let Some((number, span)) = number {
            let text = format!("({number})");
            let width = glyph_width(&text, size, Font::TimesRoman);
            let x_pt = round2(self.right_edge() - width);
            let page = self.pages.last_mut().expect("at least one page");
            page.items.push(TextItem {
                text,
                x_pt,
                baseline_y_pt: round2(self.y),
                font_size_pt: size,
                span,
                font: Font::TimesRoman,
                rule: None,
            });
        }
        self.newline(self.constraints.font_size_pt);
        self.vertical_gap(PARAGRAPH_GAP_PT);
    }

    /// Apply the inter-block spacing and return the state used as a cache key.
    pub fn prepare_block(&mut self, block: &Block) -> FlowState {
        let body_size = self.constraints.font_size_pt;
        if matches!(block, Block::Paragraph(inlines) if inlines.iter().all(|inline| matches!(inline, Inline::Label { .. })))
        {
            return self.state();
        }
        match block {
            Block::Paragraph(_) => {
                if !self.first_block {
                    self.newline(body_size);
                    self.vertical_gap(PARAGRAPH_GAP_PT);
                }
            }
            Block::Heading { level, .. } => {
                let size = heading_size(*level, body_size);
                if !self.first_block {
                    self.newline(size);
                    self.vertical_gap(PARAGRAPH_GAP_PT * 2.0);
                }
            }
            Block::FigureCaption { .. } => {
                if !self.first_block {
                    self.newline(body_size);
                    self.vertical_gap(PARAGRAPH_GAP_PT);
                }
            }
        }
        self.first_block = false;
        self.state()
    }

    /// Lay out a block after `prepare_block`, returning its reusable fragment.
    pub fn render_prepared_block(&mut self, block: &Block) -> Vec<PlacedItem> {
        let starts: Vec<usize> = self.pages.iter().map(|page| page.items.len()).collect();
        let body_size = self.constraints.font_size_pt;
        match block {
            Block::Paragraph(inlines) => emit(self, inlines, body_size, Font::TimesRoman),
            Block::Heading {
                level,
                number,
                number_span,
                content,
            } => {
                if self.emit_heading_numbers && !number.is_empty() {
                    self.place(
                        number.clone(),
                        heading_size(*level, body_size),
                        *number_span,
                        Font::TimesBold,
                    );
                }
                emit(
                    self,
                    content,
                    heading_size(*level, body_size),
                    Font::TimesBold,
                );
                self.newline(body_size);
                self.vertical_gap(PARAGRAPH_GAP_PT);
                self.x = MARGIN_PT;
            }
            Block::FigureCaption { content } => {
                let width: f64 = content
                    .iter()
                    .map(|inline| match inline {
                        Inline::Text { text, .. } => {
                            glyph_width(text, body_size, Font::TimesRoman)
                                + word_space(body_size, Font::TimesRoman)
                        }
                        _ => 0.0,
                    })
                    .sum();
                self.x = MARGIN_PT + (self.constraints.measure_pt - width).max(0.0) / 2.0;
                emit(self, content, body_size, Font::TimesRoman);
                self.newline(body_size);
            }
        }
        let mut placed = Vec::new();
        for (page_index, page) in self.pages.iter().enumerate() {
            let start = starts.get(page_index).copied().unwrap_or(0);
            placed.extend(
                page.items[start..]
                    .iter()
                    .cloned()
                    .map(|item| PlacedItem { page_index, item }),
            );
        }
        placed
    }

    pub fn state(&self) -> FlowState {
        let page_index = self.pages.len() - 1;
        FlowState {
            page_index,
            x: self.x,
            y: self.y,
            line_ascent: self.line_ascent,
            line_descent: self.line_descent,
            trailing_line_items: self.pages[page_index].items.len() - self.line_start,
        }
    }

    /// Restore a cached block whose prepared state matched the current state.
    pub fn append_reused(
        &mut self,
        placed: &[PlacedItem],
        diagnostics: &[Diagnostic],
        end: FlowState,
    ) {
        while self.pages.len() <= end.page_index {
            let number = self.pages.len() as u32 + 1;
            self.pages.push(Page {
                number,
                width_pt: PAGE_WIDTH_PT,
                height_pt: PAGE_HEIGHT_PT,
                items: Vec::new(),
            });
        }
        for placed_item in placed {
            self.pages[placed_item.page_index]
                .items
                .push(placed_item.item.clone());
        }
        self.x = end.x;
        self.y = end.y;
        self.line_ascent = end.line_ascent;
        self.line_descent = end.line_descent;
        self.line_start = self.pages[end.page_index]
            .items
            .len()
            .saturating_sub(end.trailing_line_items);
        self.diagnostics.extend_from_slice(diagnostics);
    }

    pub fn diagnostics_len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn diagnostics_since(&self, start: usize) -> &[Diagnostic] {
        &self.diagnostics[start..]
    }

    pub fn into_pages(self) -> Vec<Page> {
        self.pages
    }

    pub fn into_pages_and_diagnostics(self) -> (Vec<Page>, Vec<Diagnostic>) {
        (self.pages, self.diagnostics)
    }

    fn into_result(self) -> (Vec<Page>, BTreeMap<String, ReferenceValue>, Vec<Diagnostic>) {
        (self.pages, self.collected_labels, self.diagnostics)
    }
}

fn heading_size(level: u8, body_size: f64) -> f64 {
    body_size
        * if level == 1 {
            17.0 / BODY_SIZE_PT
        } else {
            14.0 / BODY_SIZE_PT
        }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn layout(blocks: &[Block]) -> Vec<Page> {
    let mut c = LayoutCursor::with_labels(LayoutConstraints::default(), BTreeMap::new(), false);
    for block in blocks {
        c.prepare_block(block);
        c.render_prepared_block(block);
    }
    c.into_pages()
}

pub fn layout_with_constraints(blocks: &[Block], constraints: LayoutConstraints) -> Vec<Page> {
    let mut c = LayoutCursor::new(constraints);
    for block in blocks {
        c.prepare_block(block);
        c.render_prepared_block(block);
    }
    c.into_pages()
}

/// Lay out repeatedly until both label values and their page numbers stabilize.
pub fn layout_converged(
    blocks: &[Block],
    constraints: LayoutConstraints,
) -> (Vec<Page>, Vec<Diagnostic>) {
    let mut labels = BTreeMap::new();
    let mut last_pages = Vec::new();
    let mut diagnostics = Vec::new();
    let mut converged = false;
    for _ in 0..REFERENCE_ITERATION_LIMIT {
        let mut cursor = LayoutCursor::with_labels(constraints, labels.clone(), true);
        for block in blocks {
            cursor.prepare_block(block);
            cursor.render_prepared_block(block);
        }
        let (pages, next_labels, shape_diagnostics) = cursor.into_result();
        last_pages = pages;
        diagnostics = shape_diagnostics;
        if next_labels == labels {
            converged = true;
            labels = next_labels;
            break;
        }
        labels = next_labels;
    }

    visit_references(blocks, &mut |key, span| {
        if !labels.contains_key(key) {
            diagnostics.push(Diagnostic::warning(
                format!("undefined reference '{key}'"),
                Some(span),
                Some("rendered ?? for the unresolved reference".into()),
            ));
        }
    });
    if !converged {
        diagnostics.push(Diagnostic::warning(
            format!(
                "cross-reference values did not converge after {REFERENCE_ITERATION_LIMIT} layout passes"
            ),
            None,
            Some("returned the final bounded layout pass".into()),
        ));
    }
    (last_pages, diagnostics)
}

fn visit_references(blocks: &[Block], visitor: &mut impl FnMut(&str, Span)) {
    for block in blocks {
        let inlines = match block {
            Block::Paragraph(inlines) => inlines,
            Block::Heading { content, .. } | Block::FigureCaption { content } => content,
        };
        for inline in inlines {
            if let Inline::Reference { key, span, .. } = inline {
                visitor(key, *span);
            }
        }
    }
}

fn emit(c: &mut LayoutCursor, inlines: &[Inline], size: f64, font: Font) {
    for inline in inlines {
        match inline {
            Inline::Text { text, span } => c.place(text.clone(), size, *span, font),
            Inline::LineBreak { .. } => c.newline(size),
            Inline::Math {
                list,
                display,
                number,
                number_span,
                span,
            } => {
                let b = math::layout(list, size, &mut c.diagnostics);
                if *display {
                    c.display_math(
                        b,
                        size,
                        number
                            .as_deref()
                            .zip(*number_span)
                            .or_else(|| number.as_deref().map(|number| (number, *span))),
                    );
                } else {
                    c.place_math(b, size);
                }
            }
            Inline::Label { key, value, .. } => {
                c.collected_labels.insert(
                    key.clone(),
                    ReferenceValue {
                        number: value.clone(),
                        page: c.pages.len() as u32,
                    },
                );
            }
            Inline::Reference { key, page, span } => {
                let text = c.resolved_labels.get(key).map_or_else(
                    || "??".to_string(),
                    |value| {
                        if *page {
                            value.page.to_string()
                        } else {
                            value.number.clone()
                        }
                    },
                );
                c.place(text, size, *span, font);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn laid_out(source: &str) -> (parser::Parsed, Vec<Page>) {
        let parsed = parser::parse(source);
        let pages = layout(&parsed.blocks);
        (parsed, pages)
    }

    fn item_at(pages: &[Page], start: usize) -> &TextItem {
        pages
            .iter()
            .flat_map(|p| &p.items)
            .find(|item| item.span.start == start)
            .expect("expected item at source offset")
    }

    #[test]
    fn inline_and_display_math_both_produce_positioned_items() {
        let source = "before $x$ after $$y$$ end";
        let (parsed, pages) = laid_out(source);
        assert!(parsed.diagnostics.is_empty());
        let x = item_at(&pages, source.find('x').unwrap());
        let y = item_at(&pages, source.find('y').unwrap());
        assert_eq!(x.font_size_pt, BODY_SIZE_PT);
        assert_eq!(x.baseline_y_pt, item_at(&pages, 0).baseline_y_pt);
        assert!(
            (y.x_pt - (PAGE_WIDTH_PT - glyph_width("y", BODY_SIZE_PT, Font::TimesRoman)) / 2.0)
                .abs()
                < 0.02
        );
        assert!(y.baseline_y_pt > x.baseline_y_pt);
    }

    #[test]
    fn fraction_stacks_smaller_children_and_advances_as_one_unit() {
        let source = "$\\frac{1}{2}z$";
        let (_, pages) = laid_out(source);
        let numerator = item_at(&pages, source.find('1').unwrap());
        let denominator = item_at(&pages, source.find('2').unwrap());
        let following = item_at(&pages, source.find('z').unwrap());
        assert!(numerator.baseline_y_pt < denominator.baseline_y_pt);
        assert!(numerator.font_size_pt < BODY_SIZE_PT);
        assert!(denominator.font_size_pt < BODY_SIZE_PT);
        assert!(following.x_pt > numerator.x_pt && following.x_pt > denominator.x_pt);
    }

    #[test]
    fn scripts_attach_to_atoms_and_nested_scripts_decrease_in_size() {
        let source = "$x^2 x_i x^{a+b} x^{y^z}$";
        let (_, pages) = laid_out(source);
        let two = item_at(&pages, source.find('2').unwrap());
        let i = item_at(&pages, source.find('i').unwrap());
        let a = item_at(&pages, source.find('a').unwrap());
        let plus = item_at(&pages, source.find('+').unwrap());
        let y = item_at(&pages, source.find('y').unwrap());
        let z = item_at(&pages, source.find('z').unwrap());
        assert_eq!(two.font_size_pt, BODY_SIZE_PT * math::SCRIPT_SCALE);
        assert_eq!(i.font_size_pt, BODY_SIZE_PT * math::SCRIPT_SCALE);
        assert_eq!(a.font_size_pt, two.font_size_pt);
        assert_eq!(plus.font_size_pt, two.font_size_pt);
        assert!(z.font_size_pt < y.font_size_pt);
        assert_eq!(
            z.font_size_pt,
            BODY_SIZE_PT * math::SECOND_ORDER_SCRIPT_SCALE
        );
        assert!(two.baseline_y_pt < item_at(&pages, source.find('x').unwrap()).baseline_y_pt);
        assert!(
            i.baseline_y_pt > item_at(&pages, source[5..].find('x').unwrap() + 5).baseline_y_pt
        );
    }

    #[test]
    fn unknown_command_and_unclosed_shift_recover_without_losing_content() {
        let source = "$x+\\unknown";
        let (parsed, pages) = laid_out(source);
        let messages: Vec<_> = parsed
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect();
        assert!(messages.iter().any(|m| m.contains("\\unknown")));
        assert!(messages
            .iter()
            .any(|m| m.contains("missing its closing '$'")));
        assert!(pages
            .iter()
            .any(|p| p.items.iter().any(|i| i.text == "\\unknown")));
    }

    #[test]
    fn every_math_item_span_is_a_valid_source_slice() {
        let source = "$é^2+\\alpha+\\frac{1}{β_3}+\\sqrt{x}$";
        let (parsed, pages) = laid_out(source);
        assert!(parsed.diagnostics.is_empty());
        for item in pages.iter().flat_map(|p| &p.items) {
            let slice = &source[item.span.start..item.span.end];
            assert!(!slice.is_empty());
        }
    }

    #[test]
    fn tall_math_increases_the_distance_to_the_next_line() {
        let plain = laid_out("top\\\\ bottom").1;
        let tall = laid_out("top $\\frac{1}{2}$\\\\ bottom").1;
        let plain_gap = item_at(&plain, 6).baseline_y_pt - item_at(&plain, 0).baseline_y_pt;
        let tall_gap = item_at(&tall, 20).baseline_y_pt - item_at(&tall, 0).baseline_y_pt;
        assert!(tall_gap > plain_gap);
    }

    #[test]
    fn core14_shaping_applies_kerning_and_keeps_document_byte_identity() {
        let unkerned = text_width("A", BODY_SIZE_PT, Font::TimesRoman)
            + text_width("V", BODY_SIZE_PT, Font::TimesRoman);
        let kerned = text_width("AV", BODY_SIZE_PT, Font::TimesRoman);
        assert!(kerned < unkerned, "the Times-Roman AV pair must kern");

        let source = "é AV";
        let (_, pages) = laid_out(source);
        let item = item_at(&pages, source.find("AV").unwrap());
        assert_eq!(&source[item.span.start..item.span.end], "AV");
    }

    #[test]
    fn shaping_missing_glyph_diagnostic_uses_document_not_relative_offset() {
        let source = "prefix 東京";
        let output = crate::incremental::compile_full(source, LayoutConstraints::default());
        let tokyo = source.find('東').unwrap();
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.span == Some(Span::new(tokyo, tokyo + '東'.len_utf8()))
                && diagnostic.message.contains("U+6771")
        }));

        let origin = Span::in_document(crate::DocumentId(4), 100, 104);
        let mut diagnostics = Vec::new();
        let (_, shaped_span) = shaped_width(
            "A日",
            BODY_SIZE_PT,
            Font::TimesRoman,
            origin,
            &mut diagnostics,
        );
        assert_eq!(shaped_span, origin);
        assert_eq!(
            diagnostics[0].span,
            Some(Span::in_document(crate::DocumentId(4), 101, 104))
        );
    }

    #[test]
    fn body_and_heading_faces_do_not_depend_on_numeric_font_size() {
        let parsed = parser::parse("body\n\n\\section{heading}\n");
        let pages = layout_with_constraints(
            &parsed.blocks,
            LayoutConstraints {
                font_size_pt: 11.0,
                measure_pt: LayoutConstraints::default().measure_pt,
            },
        );
        let body = pages
            .iter()
            .flat_map(|page| &page.items)
            .find(|item| item.text == "body")
            .unwrap();
        let heading = pages
            .iter()
            .flat_map(|page| &page.items)
            .find(|item| item.text == "heading")
            .unwrap();
        assert_eq!(body.font, Font::TimesRoman);
        assert_eq!(heading.font, Font::TimesBold);
    }

    #[test]
    fn unsupported_shaping_is_an_explicit_source_mapped_error() {
        let source = "before אב";
        let output = crate::incremental::compile_full(source, LayoutConstraints::default());
        let start = source.find('א').unwrap();
        assert!(output.diagnostics.iter().any(|diagnostic| {
            diagnostic.span == Some(Span::new(start, start + 'א'.len_utf8()))
                && diagnostic.message.contains("could not shape text")
                && diagnostic.message.contains("Hebrew needs bidi reordering")
        }));
    }
}

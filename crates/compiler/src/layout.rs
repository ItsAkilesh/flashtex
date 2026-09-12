//! Layout: turns parsed blocks into positioned items on pages.
//!
//! HONEST LIMITATION — glyph metrics are a placeholder. Real typesetting needs
//! per-glyph advance widths from the font. This version estimates every glyph as
//! `AVERAGE_GLYPH_RATIO * font_size_pt` wide. Line breaking is therefore greedy
//! against an estimate, not TeX's optimal paragraph breaking, and output will
//! not match a real TeX engine's line breaks. Replacing this with real metrics
//! is required before any compatibility claim.
//!
//! One item is emitted per word rather than per line. That keeps each item's
//! source span exact, which is what click-to-source navigation (FT-003) needs.

use crate::parser::{Block, Inline};
use crate::Span;

pub const PAGE_WIDTH_PT: f64 = 612.0;
pub const PAGE_HEIGHT_PT: f64 = 792.0;
pub const MARGIN_PT: f64 = 72.0;
pub const BODY_SIZE_PT: f64 = 12.0;
pub const LINE_SPACING: f64 = 1.2;
pub const PARAGRAPH_GAP_PT: f64 = 6.0;

/// Placeholder average glyph advance as a fraction of font size.
pub const AVERAGE_GLYPH_RATIO: f64 = 0.5;
/// Placeholder inter-word space as a fraction of font size.
pub const SPACE_RATIO: f64 = 0.28;

#[derive(Debug, Clone, PartialEq)]
pub struct TextItem {
    pub text: String,
    pub x_pt: f64,
    pub baseline_y_pt: f64,
    pub font_size_pt: f64,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub number: u32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub items: Vec<TextItem>,
}

fn glyph_width(text: &str, size: f64) -> f64 {
    // chars(), not bytes: width is a visual property, spans are a byte property.
    text.chars().count() as f64 * AVERAGE_GLYPH_RATIO * size
}

struct Cursor {
    pages: Vec<Page>,
    x: f64,
    y: f64,
    line_height: f64,
}

impl Cursor {
    fn new() -> Self {
        Cursor {
            pages: vec![Page {
                number: 1,
                width_pt: PAGE_WIDTH_PT,
                height_pt: PAGE_HEIGHT_PT,
                items: Vec::new(),
            }],
            x: MARGIN_PT,
            y: MARGIN_PT + BODY_SIZE_PT,
            line_height: BODY_SIZE_PT * LINE_SPACING,
        }
    }

    fn right_edge(&self) -> f64 {
        PAGE_WIDTH_PT - MARGIN_PT
    }

    fn newline(&mut self, size: f64) {
        self.x = MARGIN_PT;
        self.y += self.line_height.max(size * LINE_SPACING);
        self.line_height = size * LINE_SPACING;
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
    }

    fn vertical_gap(&mut self, gap: f64) {
        self.x = MARGIN_PT;
        self.y += gap;
    }

    fn place(&mut self, text: String, size: f64, span: Span) {
        let w = glyph_width(&text, size);
        if self.x > MARGIN_PT && self.x + w > self.right_edge() {
            self.newline(size);
        }
        self.line_height = self.line_height.max(size * LINE_SPACING);
        let item = TextItem {
            text,
            x_pt: round2(self.x),
            baseline_y_pt: round2(self.y),
            font_size_pt: size,
            span,
        };
        self.pages.last_mut().expect("at least one page").items.push(item);
        self.x += w + SPACE_RATIO * size;
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

pub fn layout(blocks: &[Block]) -> Vec<Page> {
    let mut c = Cursor::new();
    let mut first = true;

    for block in blocks {
        match block {
            Block::Paragraph(inlines) => {
                if !first {
                    c.newline(BODY_SIZE_PT);
                    c.vertical_gap(PARAGRAPH_GAP_PT);
                }
                emit(&mut c, inlines, BODY_SIZE_PT);
            }
            Block::Heading { level, content } => {
                let size = match level {
                    1 => 17.0,
                    _ => 14.0,
                };
                if !first {
                    c.newline(size);
                    c.vertical_gap(PARAGRAPH_GAP_PT * 2.0);
                }
                emit(&mut c, content, size);
                c.newline(BODY_SIZE_PT);
                c.vertical_gap(PARAGRAPH_GAP_PT);
                c.x = MARGIN_PT;
            }
        }
        first = false;
    }

    c.pages
}

fn emit(c: &mut Cursor, inlines: &[Inline], size: f64) {
    for inline in inlines {
        match inline {
            Inline::Text { text, span } => c.place(text.clone(), size, *span),
            Inline::LineBreak { .. } => c.newline(size),
        }
    }
}

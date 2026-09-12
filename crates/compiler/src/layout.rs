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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn word_texts(pages: &[Page]) -> Vec<String> {
        pages.iter().flat_map(|p| p.items.iter().map(|i| i.text.clone())).collect()
    }

    #[test]
    fn empty_document_produces_one_empty_page() {
        let pages = layout(&[]);
        // layout() creates an initial page even when there are no blocks.
        assert_eq!(pages.len(), 1);
        assert!(pages[0].items.is_empty());
    }

    #[test]
    fn single_word_placed_at_left_margin() {
        let parsed = parse("Hello");
        let pages = layout(&parsed.blocks);
        assert_eq!(pages.len(), 1);
        let item = &pages[0].items[0];
        assert_eq!(item.text, "Hello");
        assert!((item.x_pt - MARGIN_PT).abs() < 0.1, "first word must start at the left margin");
    }

    #[test]
    fn words_are_placed_left_to_right() {
        let parsed = parse("alpha beta gamma");
        let pages = layout(&parsed.blocks);
        let items = &pages[0].items;
        assert_eq!(items.len(), 3);
        assert!(items[0].x_pt < items[1].x_pt, "beta must be to the right of alpha");
        assert!(items[1].x_pt < items[2].x_pt, "gamma must be to the right of beta");
    }

    #[test]
    fn baseline_y_is_positive() {
        let parsed = parse("word");
        let pages = layout(&parsed.blocks);
        let item = &pages[0].items[0];
        assert!(item.baseline_y_pt > 0.0);
    }

    #[test]
    fn section_heading_uses_larger_font() {
        let parsed = parse(r"\section{Title} body");
        let pages = layout(&parsed.blocks);
        let heading = pages[0].items.iter().find(|i| i.text == "Title").expect("Title word");
        let body    = pages[0].items.iter().find(|i| i.text == "body").expect("body word");
        assert!(heading.font_size_pt > body.font_size_pt,
            "section heading must use a larger font than body text");
    }

    #[test]
    fn heading_word_precedes_body_word_vertically() {
        // In top-left origin coordinates, heading must appear before body (smaller y).
        let parsed = parse(r"\section{Title} body");
        let pages = layout(&parsed.blocks);
        let heading_y = pages[0].items.iter().find(|i| i.text == "Title").unwrap().baseline_y_pt;
        let body_y    = pages[0].items.iter().find(|i| i.text == "body").unwrap().baseline_y_pt;
        assert!(heading_y < body_y, "heading must appear above body (smaller y)");
    }

    #[test]
    fn two_paragraphs_are_on_different_y_positions() {
        let parsed = parse("first paragraph\n\nsecond paragraph");
        let pages = layout(&parsed.blocks);
        let texts = word_texts(&pages);
        assert!(texts.contains(&"first".to_string()));
        assert!(texts.contains(&"second".to_string()));
        let first_y  = pages[0].items.iter().find(|i| i.text == "first").unwrap().baseline_y_pt;
        let second_y = pages[0].items.iter().find(|i| i.text == "second").unwrap().baseline_y_pt;
        assert!(second_y > first_y, "second paragraph must have a greater y than first");
    }

    #[test]
    fn long_line_wraps_within_page_width() {
        // 50 repetitions of "word " should cause line wrapping.
        let many_words: String = std::iter::repeat("word ").take(50).collect();
        let parsed = parse(many_words.trim());
        let pages = layout(&parsed.blocks);
        // After wrapping the x positions should reset to the margin more than once.
        let margin_starts = pages[0].items.iter().filter(|i| (i.x_pt - MARGIN_PT).abs() < 0.1).count();
        assert!(margin_starts >= 2, "long paragraph must produce multiple lines (margin resets)");
    }

    #[test]
    fn items_x_stay_within_page_bounds() {
        let many_words: String = std::iter::repeat("word ").take(100).collect();
        let parsed = parse(many_words.trim());
        let pages = layout(&parsed.blocks);
        for item in &pages[0].items {
            assert!(item.x_pt >= MARGIN_PT - 0.01 && item.x_pt < PAGE_WIDTH_PT,
                "item x={} is out of page bounds", item.x_pt);
        }
    }

    #[test]
    fn page_overflow_creates_second_page() {
        // `\\` produces an explicit LineBreak token that always advances a line.
        // With ~44 lines per page, 60 force-newlines reliably exceed one page.
        let many_lines: String = std::iter::repeat("word\\\\\n").take(60).collect();
        let parsed = parse(many_lines.trim());
        let pages = layout(&parsed.blocks);
        assert!(pages.len() >= 2, "60 forced line-breaks must overflow to a second page");
    }

    #[test]
    fn page_numbers_are_sequential() {
        let many_lines: String = std::iter::repeat("word\\\\\n").take(60).collect();
        let parsed = parse(many_lines.trim());
        let pages = layout(&parsed.blocks);
        assert!(pages.len() >= 2, "need multiple pages for this test");
        for (i, page) in pages.iter().enumerate() {
            assert_eq!(page.number, (i + 1) as u32);
        }
    }

    #[test]
    fn page_dimensions_match_constants() {
        let parsed = parse("hello");
        let pages = layout(&parsed.blocks);
        for page in &pages {
            assert_eq!(page.width_pt,  PAGE_WIDTH_PT);
            assert_eq!(page.height_pt, PAGE_HEIGHT_PT);
        }
    }
}

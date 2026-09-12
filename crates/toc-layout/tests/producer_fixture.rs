//! Integration fixture: stabilizes a table of contents from section/page
//! records exactly as the real, unmodified `crates/compiler` emits them --
//! never a hand-built `EntryRecord`/`RelativeEntry`.
//!
//! ## What the repo's real producer is
//!
//! `\section`/`\subsection` is parsed into `Block::Heading { level, content,
//! .. }` by `crates/compiler/src/parser.rs` -- its `"section" | "subsection"`
//! match arm is the *only* place `Block::Heading` is constructed anywhere in
//! this repo (checked: `grep -n '"section" | "subsection"' crates/compiler/
//! src/parser.rs`, and no other `pub struct`/`pub enum` named `*Section*` in
//! any crate produces a page-bearing record -- `crates/document-style/src/
//! fonts.rs::SectionSpec` is heading *styling*, not a section+page record).
//! `crates/compiler/src/layout.rs`'s `LayoutCursor` then places each
//! heading's words onto real, paginated `Page`s; its own public
//! `prepare_block`/`render_prepared_block` methods (the exact pair
//! `layout::layout_with_constraints` itself calls in a loop) report back
//! which real, zero-based page each placed word landed on via
//! `PlacedItem::page_index`. That is the section+page record this crate
//! consumes below -- adapted, never hand-shaped.
//!
//! ## Why the fixture's LaTeX source is authored here, not read from the corpus
//!
//! grep across the repo's only `.tex` corpus turns up zero sectioning
//! commands anywhere:
//!
//! ```text
//! $ grep -rnE '\\(sub)*section|\\part|\\chapter|\\appendix' tests/tex-corpus/cases
//! (no matches)
//! ```
//!
//! confirmed exactly (14/14 documents, 0 sectioning commands) in
//! `measured_gaps.rs`. There is no existing fixture with real section+page
//! records to pull from, so this test authors the LaTeX *input* text itself
//! -- labelled plainly as a stand-in for a sectioned corpus fixture that
//! does not yet exist -- but changes nothing about how it is compiled: the
//! real `parser::parse` and the real `LayoutCursor` run unmodified.

mod support;

use flashtex_compiler::layout::{LayoutConstraints, LayoutCursor};
use flashtex_compiler::parser::{self, Block, Inline};
use flashtex_toc_layout::{
    CharWidthMeasure, LineBox, RelativeEntry, SourceId, SourcedEntry, layout_entry, stabilize_toc,
};
use support::GeometryFrontMatterModel;

/// Stand-in LaTeX input (see module docs): four `\section`s and three
/// `\subsection`s with enough real body text between them that the real
/// compiler's page-fill logic pushes later headings onto later real pages,
/// not just page 1.
fn fixture_source() -> String {
    let filler = "Lorem ipsum dolor sit amet consectetur adipiscing elit. ".repeat(40);
    format!(
        "\\documentclass{{article}}\n\\begin{{document}}\n\
         \\section{{Introduction}}\n{filler}\n\
         \\subsection{{Background}}\n{filler}\n\
         \\section{{Method}}\n{filler}\n\
         \\subsection{{Data}}\n{filler}\n\
         \\subsection{{Procedure}}\n{filler}\n\
         \\section{{Results}}\n{filler}\n\
         \\section{{Conclusion}}\n{filler}\n\
         \\end{{document}}\n"
    )
}

/// One section+page record adapted from a real `Block::Heading` plus the
/// `PlacedItem::page_index` the real `LayoutCursor` assigned it.
struct ProducerEntry {
    level: u8,
    title: String,
    page: u32,
}

/// Runs the real, unmodified compiler (`parser::parse`, then exactly the
/// `prepare_block`/`render_prepared_block` loop `layout::layout_with_constraints`
/// itself runs) and adapts its own output into `ProducerEntry`. The only
/// hand-written part is this adapter -- every field's value comes from the
/// compiler's real parse and layout, not from this function.
fn producer_entries(source: &str) -> Vec<ProducerEntry> {
    let parsed = parser::parse(source);
    assert!(
        parsed.diagnostics.is_empty(),
        "fixture source must compile cleanly through the real parser: {:?}",
        parsed.diagnostics
    );

    let mut cursor = LayoutCursor::new(LayoutConstraints::default());
    let mut entries = Vec::new();
    for block in &parsed.blocks {
        cursor.prepare_block(block);
        let placed = cursor.render_prepared_block(block);
        if let Block::Heading { level, content, .. } = block {
            let title: String = content
                .iter()
                .filter_map(|inline| match inline {
                    Inline::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ");
            let page_index = placed
                .first()
                .expect("a non-empty heading places at least its first word")
                .page_index;
            entries.push(ProducerEntry {
                // Compiler levels start at 1 (\section); toc-layout levels
                // start at 0 (top level). Documented adapter choice, made
                // here rather than in either crate.
                level: level - 1,
                title,
                page: page_index as u32 + 1,
            });
        }
    }
    entries
}

#[test]
fn real_compiler_output_has_headings_on_more_than_one_page() {
    let entries = producer_entries(&fixture_source());
    assert_eq!(entries.len(), 7, "fixture has 4 \\section + 3 \\subsection");
    let pages: std::collections::BTreeSet<u32> = entries.iter().map(|e| e.page).collect();
    assert!(
        pages.len() > 1,
        "fixture filler text should force real pagination across more than one page, got pages {pages:?}"
    );
}

#[test]
fn stabilizes_a_toc_from_records_the_real_compiler_emits() {
    let source = fixture_source();
    let entries = producer_entries(&source);

    let source_id = SourceId::new("producer-fixture.tex", "rev4");
    let sourced: Vec<SourcedEntry> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| SourcedEntry {
            source: source_id.clone(),
            sequence: i as u32,
            entry: RelativeEntry::new(e.title.clone(), e.level, e.page)
                .expect("real compiler page numbers are >= 1 and titles are non-empty"),
        })
        .collect();

    let model = GeometryFrontMatterModel {
        entry_count: sourced.len() as u32,
    };
    let stabilized =
        stabilize_toc(&sourced, &model, 0).expect("real, non-adversarial producer data converges");

    assert_eq!(stabilized.entries.len(), 7);
    for (i, entry) in stabilized.entries.iter().enumerate() {
        assert_eq!(entry.sequence, i as u32, "entries stay in document order");
        assert!(!entry.record.title.is_empty());
        assert!(!entry.is_stale(&source_id));
    }

    // Lay out every stabilized entry with the reference measurer -- no real
    // TextMeasure consumer is wired up anywhere in this repo yet, see
    // crates/toc-layout/src/measure.rs's doc comment.
    let measure = CharWidthMeasure::new(6.0, 3.0);
    let line = LineBox::new(400.0, 18.0).unwrap();
    for entry in &stabilized.entries {
        let laid_out = layout_entry(&entry.record, line, &measure)
            .expect("fixture titles are short enough to fit a 400-unit line");
        assert_eq!(laid_out.title, entry.record.title);
        assert_eq!(laid_out.page_label, entry.record.page.to_string());
    }
}

#[test]
fn real_producer_records_stabilize_deterministically_across_reparses() {
    let source = fixture_source();

    let run = || {
        let entries = producer_entries(&source);
        let source_id = SourceId::new("producer-fixture.tex", "rev4");
        let sourced: Vec<SourcedEntry> = entries
            .iter()
            .enumerate()
            .map(|(i, e)| SourcedEntry {
                source: source_id.clone(),
                sequence: i as u32,
                entry: RelativeEntry::new(e.title.clone(), e.level, e.page).unwrap(),
            })
            .collect();
        let model = GeometryFrontMatterModel {
            entry_count: sourced.len() as u32,
        };
        stabilize_toc(&sourced, &model, 0).unwrap()
    };

    // Re-parsing and re-laying-out from scratch (not reusing any cached
    // state) must produce the byte-identical StabilizedToc both times.
    assert_eq!(run(), run());
}

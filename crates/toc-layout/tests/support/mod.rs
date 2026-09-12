//! Shared helpers for the real-producer integration fixture
//! (`producer_fixture.rs`) and the measured-gap harness (`measured_gaps.rs`).
//!
//! Not a test file itself: Cargo only turns a file directly under `tests/`
//! into its own test binary, so `tests/support/mod.rs` (a subdirectory) is
//! compiled as a plain module by whichever test file does `mod support;`.
//!
//! Each test binary that pulls this module in uses only part of it, so an
//! item unused by one binary and used by another is expected, not dead code.
#![allow(dead_code)]

use flashtex_compiler::json::{self, Value};
use flashtex_compiler::parser::{self, Block, Inline, SourceDocument};
use flashtex_toc_layout::FrontMatterModel;
use std::cell::Cell;
use std::path::{Path, PathBuf};

/// The repository's one real `.tex` corpus (FT-011) -- the same directory
/// `crates/compiler/tests/corpus_gate.rs` pins its own compile-status gate
/// against.
pub fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/tex-corpus")
}

/// One corpus project exactly as `manifest.json` declares it, with every
/// document's real source text already read from disk.
pub struct CorpusCase {
    pub id: String,
    pub entry_path: String,
    /// (project-relative path, source text), in manifest order.
    pub documents: Vec<(String, String)>,
}

impl CorpusCase {
    /// Parses this case's real source through the compiler's own
    /// multi-document entry point -- the same `parser::parse_project` call
    /// `crates/compiler/src/incremental.rs::compile_full_project` makes,
    /// minus the JSON transport layer this fixture doesn't need.
    pub fn parse(&self) -> parser::Parsed {
        let docs: Vec<SourceDocument> = self
            .documents
            .iter()
            .map(|(path, text)| SourceDocument {
                path: path.as_str(),
                text: text.as_str(),
            })
            .collect();
        parser::parse_project(&docs, &self.entry_path)
    }
}

/// Every case `manifest.json` declares, each with its real source text read
/// from disk -- never a hand-typed stand-in for the corpus.
pub fn corpus_cases() -> Vec<CorpusCase> {
    let root = corpus_root();
    let raw = std::fs::read_to_string(root.join("manifest.json"))
        .expect("tests/tex-corpus/manifest.json must exist");
    let manifest = json::parse(&raw).expect("manifest.json must be valid JSON");
    let cases = manifest
        .get("cases")
        .and_then(Value::as_arr)
        .expect("manifest.json must have a top-level \"cases\" array")
        .clone();

    cases
        .iter()
        .map(|case| {
            let id = case
                .get("id")
                .and_then(Value::as_str)
                .expect("case id")
                .to_string();
            let entry_path = case
                .get("entry_path")
                .and_then(Value::as_str)
                .expect("case entry_path")
                .to_string();
            let dir = root.join("cases").join(&id);
            let documents = case
                .get("documents")
                .and_then(Value::as_arr)
                .expect("case documents")
                .iter()
                .map(|d| {
                    let rel = d.as_str().expect("document path").to_string();
                    let text = std::fs::read_to_string(dir.join(&rel))
                        .unwrap_or_else(|e| panic!("corpus case {id}: cannot read {rel}: {e}"));
                    (rel, text)
                })
                .collect();
            CorpusCase {
                id,
                entry_path,
                documents,
            }
        })
        .collect()
}

/// One (compiler level, title) pulled from a real `Block::Heading` --
/// `crates/compiler/src/parser.rs`'s `"section" | "subsection"` match arm is
/// the *only* place a `Block::Heading` is constructed, so this is exhaustive
/// for whatever sectioning the compiler actually supports today.
pub fn heading_records(blocks: &[Block]) -> Vec<(u8, String)> {
    blocks
        .iter()
        .filter_map(|block| match block {
            Block::Heading { level, content, .. } => {
                let title: String = content
                    .iter()
                    .filter_map(|inline| match inline {
                        Inline::Text { text, .. } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                Some((*level, title))
            }
            _ => None,
        })
        .collect()
}

/// Raw substrings naming a LaTeX sectioning/numbering construct that
/// `crates/compiler`'s `"section" | "subsection"` match arm does not
/// implement. A hand-rolled substring scan rather than a regex dependency,
/// matching `toc-layout`'s own zero-external-dependency policy; source text
/// is scanned directly because the compiler never turns these tokens into
/// any block this crate could inspect instead -- there is nothing else to
/// measure them from.
pub const UNSUPPORTED_SECTIONING_MARKERS: &[&str] = &[
    r"\section*",
    r"\subsection*",
    r"\subsubsection",
    r"\paragraph",
    r"\subparagraph",
    r"\part",
    r"\chapter",
    r"\appendix",
    r"\setcounter{section",
    r"\setcounter{chapter",
    r"\setcounter{part",
    r"\renewcommand{\thesection",
    r"\renewcommand{\thesubsection",
    r"\renewcommand{\thechapter",
];

/// True if any real document in this case names an unsupported
/// sectioning/numbering construct verbatim.
pub fn uses_unsupported_sectioning(case: &CorpusCase) -> bool {
    case.documents.iter().any(|(_, text)| {
        UNSUPPORTED_SECTIONING_MARKERS
            .iter()
            .any(|marker| text.contains(marker))
    })
}

/// Wraps a real [`FrontMatterModel`], counting every `pages_for` call.
/// `converge_front_matter_pages` calls `pages_for` exactly once per pass, on
/// both the success and failure path, so `calls` after a run is the exact
/// number of convergence passes actually taken -- read off the crate's real
/// loop, not reimplemented here.
pub struct CountingModel<'a> {
    inner: &'a dyn FrontMatterModel,
    pub calls: Cell<u32>,
}

impl<'a> CountingModel<'a> {
    pub fn new(inner: &'a dyn FrontMatterModel) -> Self {
        Self {
            inner,
            calls: Cell::new(0),
        }
    }
}

impl FrontMatterModel for CountingModel<'_> {
    fn pages_for(&self, candidate_front_matter_pages: u32) -> u32 {
        self.calls.set(self.calls.get() + 1);
        self.inner.pages_for(candidate_front_matter_pages)
    }
}

/// Stand-in [`FrontMatterModel`]: nothing in this repo computes "how many
/// pages will the rendered table of contents itself occupy" (see
/// `producer_fixture.rs`'s module docs for the grep evidence). This
/// estimates it from the compiler's own real page geometry constants
/// (`flashtex_compiler::layout::{PAGE_HEIGHT_PT, MARGIN_PT, BODY_SIZE_PT,
/// LINE_SPACING}`) and a real measured entry count -- but the "how many
/// contents lines fit on one page" arithmetic itself is authored here,
/// labelled plainly as a stand-in for whatever eventually implements
/// `FrontMatterModel` against real pagination.
pub struct GeometryFrontMatterModel {
    pub entry_count: u32,
}

impl FrontMatterModel for GeometryFrontMatterModel {
    fn pages_for(&self, _candidate_front_matter_pages: u32) -> u32 {
        if self.entry_count == 0 {
            // Even an empty table of contents still typesets its own
            // (heading-only) page.
            return 1;
        }
        let usable_height_pt =
            flashtex_compiler::layout::PAGE_HEIGHT_PT - 2.0 * flashtex_compiler::layout::MARGIN_PT;
        let line_height_pt =
            flashtex_compiler::layout::BODY_SIZE_PT * flashtex_compiler::layout::LINE_SPACING;
        let lines_per_page = (usable_height_pt / line_height_pt).floor().max(1.0) as u32;
        self.entry_count.div_ceil(lines_per_page).max(1)
    }
}

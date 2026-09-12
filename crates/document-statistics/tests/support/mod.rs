//! Test-only adapter: turns the real output of `flashtex-compiler` -- the
//! only crate in this repository that actually produces a document
//! structure (blocks of `Inline::Text`/`Inline::Math`, and real laid-out
//! pages) -- into the `SourceItem`s this crate consumes.
//!
//! This module exists purely under `tests/`. It is wired in as a
//! dev-dependency in `Cargo.toml`, `crates/compiler` is never edited, and
//! `flashtex-document-statistics`'s own `src/` takes no dependency on it: the
//! crate's "no filesystem, no parsing" guarantee (see `src/lib.rs`) is
//! unchanged. No consumer is wired up in production yet -- this is a test
//! fixture proving the adapter is possible and exercising it against real
//! content, not a production integration.
//!
//! Every document parsed here is read from `tests/tex-corpus/cases` on disk:
//! nothing in this module is hand-written LaTeX shaped for convenience.

use std::fs;
use std::path::{Path, PathBuf};

use flashtex_compiler::layout;
use flashtex_compiler::parser::{self, Block, Inline, Parsed, SourceDocument};
use flashtex_document_statistics::SourceItem;

/// Root of the repository's real tex corpus, located relative to this
/// crate's own manifest directory (this crate does not know or care where
/// the repository root is otherwise).
pub fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/tex-corpus/cases")
}

/// One real corpus case: every `.tex` file actually present under its
/// directory, read verbatim from disk, plus the project-relative path the
/// compiler's own `\input`/`\include` resolution expects.
pub struct CorpusCase {
    pub name: String,
    pub documents: Vec<(String, String)>,
    pub entry_path: String,
}

fn collect_tex_files(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read corpus dir {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_tex_files(root, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("tex") {
            let rel = path
                .strip_prefix(root)
                .expect("corpus file under its case root")
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            out.push((rel, text));
        }
    }
}

/// Load one case directory (e.g. `tests/tex-corpus/cases/plain-paragraphs`).
pub fn load_case(dir: &Path) -> CorpusCase {
    let name = dir
        .file_name()
        .expect("case directory has a name")
        .to_string_lossy()
        .into_owned();
    let mut documents = Vec::new();
    collect_tex_files(dir, dir, &mut documents);
    assert!(
        !documents.is_empty(),
        "corpus case {name} has no .tex files on disk"
    );
    CorpusCase {
        name,
        documents,
        entry_path: "main.tex".to_string(),
    }
}

/// Every real case under `tests/tex-corpus/cases`, in a stable order.
pub fn all_cases() -> Vec<CorpusCase> {
    let root = corpus_root();
    let mut case_dirs: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("read {}: {e}", root.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|p| p.is_dir())
        .collect();
    case_dirs.sort();
    case_dirs.into_iter().map(|dir| load_case(&dir)).collect()
}

impl CorpusCase {
    /// Run the real compiler parser over this case's actual files.
    pub fn parse(&self) -> Parsed {
        let docs: Vec<SourceDocument<'_>> = self
            .documents
            .iter()
            .map(|(path, text)| SourceDocument { path, text })
            .collect();
        parser::parse_project(&docs, &self.entry_path)
    }

    fn doc_text(&self, index: usize) -> &str {
        &self.documents[index].1
    }
}

/// `Inline::Text`/`Inline::Math` from a real `Parsed` tree, adapted to
/// `SourceItem`:
///
/// - `Inline::Text` becomes `SourceItem::Text`, one item per real compiler
///   word-token (exactly as `flashtex_compiler::lexer` split it -- this
///   crate never re-tokenizes).
/// - `Inline::Math` becomes `SourceItem::Math`; its `source` is sliced
///   verbatim out of the real document text using the compiler's own span,
///   never reconstructed or re-serialized from the parsed `MathList`.
/// - `Inline::LineBreak`, `Inline::Label`, and `Inline::Reference` produce
///   no `SourceItem`: this crate has no notion of line breaks, label keys,
///   or resolved cross-reference text, and inventing one would not be an
///   honest adaptation of what the compiler actually emits.
pub fn adapt_text_and_math(parsed: &Parsed, case: &CorpusCase) -> Vec<SourceItem> {
    let mut items = Vec::new();
    for block in &parsed.blocks {
        let inlines = match block {
            Block::Paragraph(inlines) => inlines,
            Block::Heading { content, .. } | Block::FigureCaption { content } => content,
        };
        for inline in inlines {
            match inline {
                Inline::Text { text, .. } => items.push(SourceItem::text(text.clone())),
                Inline::Math { display, span, .. } => {
                    let doc_text = case.doc_text(span.document.0);
                    let source = doc_text[span.start..span.end].to_string();
                    items.push(if *display {
                        SourceItem::display_math(source)
                    } else {
                        SourceItem::inline_math(source)
                    });
                }
                Inline::LineBreak { .. } | Inline::Label { .. } | Inline::Reference { .. } => {}
            }
        }
    }
    items
}

/// One `SourceItem::PageMark` per page the real layout engine (Core 14
/// metrics, the same one FlashTeX renders with) actually produced for these
/// blocks -- not a guess derived from word count.
pub fn page_marks_for(parsed: &Parsed) -> Vec<SourceItem> {
    let pages = layout::layout(&parsed.blocks);
    vec![SourceItem::PageMark; pages.len()]
}

/// Full adapter for one case: real text + math items in document order,
/// followed by one `PageMark` per real laid-out page. Order across the two
/// groups does not matter to this crate -- words/math/pages are independent
/// tallies (see `Statistics::compute_bounded`) -- so appending is simplest.
pub fn adapt(case: &CorpusCase) -> (Parsed, Vec<SourceItem>) {
    let parsed = case.parse();
    let mut items = adapt_text_and_math(&parsed, case);
    items.extend(page_marks_for(&parsed));
    (parsed, items)
}

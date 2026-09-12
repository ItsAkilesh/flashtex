# daniel-title handoff

Agent / task / branch: daniel-title / FT-033 (original article
title/author/date/abstract measured layout adapter) /
`agent/daniel-title/title-layout`
State: ready for integration (standalone, unwired)
Owned paths: `crates/title-layout/**`, `coordination/daniel-title.md`
Tested commit SHA: `3b0fc3592d0c44c711a7cf5a63d5d2bce5d3370a` on
`agent/daniel-title/title-layout` (parent `53fee3012b2902ca05bd31766defa515b3044ce`,
the `input_main_sha` from `coordination/assignments/FT-033.json`)

## What it is

`flashtex-title-layout` (`crates/title-layout`, lib `flashtex_title_layout`)
is an original crate with **no dependency but `flashtex-document-style`**
(path dep). It does no text shaping, line breaking, or glyph-width
measurement of its own — it takes already-line-broken title/author/date
text and a `flashtex_document_style::Stylesheet`, and produces the
*vertical* measured layout (baseline positions, font sizes, and TeX glue)
that `article.cls`'s `\@maketitle` and one-column `abstract` environment
would produce. Full derivations and article.cls citations are in the module
doc comments (`src/title.rs`, `src/abstract_block.rs`).

## Public adapter contract

```rust
// crate root re-exports
pub enum DocumentClass { Article, Report, Book, Letter }
impl DocumentClass {
    pub fn name(self) -> &'static str;
    pub fn parse(s: &str) -> Option<DocumentClass>;
    pub fn is_supported(self) -> bool; // true only for Article
}

pub enum TitleLayoutError {
    UnsupportedDocumentClass(DocumentClass),
    EmptyTitle,
    NoAuthors,
    EmptyAuthorLine(usize),
    EmptyDate,
} // impl Display + std::error::Error

// --- title block (\maketitle) ---
pub enum DateField { Text(String), Suppressed }

pub struct TitleBlockInput {
    pub title_lines: Vec<String>,        // pre-broken on `\\`, >=1 non-blank
    pub author_lines: Vec<Vec<String>>,  // one Vec per \and-author, >=1 author,
                                          // each with >=1 non-blank line
    pub date: DateField,
}

pub enum RowKind {
    TitleLine(usize),
    AuthorLine(usize /* author index */, usize /* line index */),
    DateLine,
}

pub struct MeasuredRow {
    pub kind: RowKind,
    pub font_size: flashtex_document_style::Pt,
    pub baseline_y: flashtex_document_style::Pt, // from top of block
}

pub struct TitleBlockLayout {
    pub rows: Vec<MeasuredRow>,
    pub total_height: flashtex_document_style::Pt,
}

pub fn layout_title_block(
    class: DocumentClass,
    sheet: &flashtex_document_style::Stylesheet,
    input: &TitleBlockInput,
) -> Result<TitleBlockLayout, TitleLayoutError>;

// --- abstract environment (one-column only) ---
pub struct AbstractLayout {
    pub heading_font_size: Pt,
    pub heading_baselineskip: Pt,
    pub gap_before_heading: flashtex_document_style::Skip,
    pub gap_heading_to_body: Skip,
    pub body_font_size: Pt,
    pub body_baselineskip: Pt,
    pub left_margin: Pt,
    pub right_margin: Pt,
    pub paragraph_gap: Skip,
}

pub fn layout_abstract(
    class: DocumentClass,
    sheet: &flashtex_document_style::Stylesheet,
) -> Result<AbstractLayout, TitleLayoutError>;
```

Both entry points fail with `TitleLayoutError::UnsupportedDocumentClass`
for anything but `DocumentClass::Article` — `flashtex-document-style` only
models `article`, so `report`/`book`/`letter` are rejected explicitly
rather than silently laid out with article's numbers. `layout_title_block`
also rejects blank titles, zero/blank authors, and blank (but present)
date text with dedicated error variants; `DateField::Suppressed` is the
one sanctioned way to omit the date line.

## Deliberately unmeasured (documented in-crate, not silently approximated)

- Horizontal extent: text widths, title centering/wrapping, and
  side-by-side placement of multiple `\and`-authors. Needs glyph metrics
  (a font-engine concern), which this crate does not consume.
- `\lineskip .5em` (the interline-glue fallback LaTeX substitutes when
  `\baselineskip` glue would go negative); every line gap uses plain
  `\baselineskip`.
- `abstract`'s `\listparindent`/`\itemindent` (`1.5em` of `\small`, not
  `\normalsize`) — `flashtex-document-style` exposes font-relative
  `em`/`ex` only for `\normalsize`, so this crate does not fabricate a
  `\small` quad.
- `article` in two-column mode (`\if@twocolumn` branch of both
  `\maketitle` and `abstract` uses `\section*`, a different shape
  entirely). Callers must not call `layout_abstract`/`layout_title_block`
  for two-column documents; nothing in the type system currently stops
  them (`Stylesheet`/`page_layout()` don't expose column count either).

## Validation

`cd crates/title-layout && export PATH="/opt/homebrew/opt/rustup/bin:$PATH"`:
- `cargo build`: clean.
- `cargo test`: 14 tests + 1 doctest, all pass (exact-number baseline/skip
  assertions against hand-derived `article.cls` v1.4n values, malformed
  blank-input rejection, and a Unicode-text case showing layout is a
  function of line/author *counts* only, never glyph content).
- `cargo clippy --all-targets -- -D warnings`: 0 warnings.
- `cargo fmt -- --check`: clean.

No workspace root `Cargo.toml` was added (repo convention); each crate is
built from its own directory, per `crates/title-layout/Cargo.lock` locking
`flashtex-document-style` as a path dependency.

## Needs from others / integration notes

- Not wired into any consumer (compiler/apps/mac). Standalone per FT-033.
- A future consumer choosing to lay out multi-author title pages
  side-by-side will need real glyph-width measurement (font-engine or
  paragraph-layout) to place `AuthorLine` columns horizontally; this
  crate only gives it the vertical baseline for each line and leaves
  horizontal placement to that consumer.
- No peer compiler/native/layout files were read for write access and
  none were modified; only `crates/title-layout/**` and this handoff.

Updated: 2026-09-12

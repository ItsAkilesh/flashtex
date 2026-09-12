# daniel-title handoff

Agent / task / branch: daniel-title / FT-033 revision 2 (original article
title/author/date/abstract measured layout adapter, now bound to
caller-supplied exact glyph metrics) / `agent/daniel-title/title-layout`
State: ready for integration (standalone, unwired)
Owned paths: `crates/title-layout/**`, `coordination/daniel-title.md`
Tested commit SHA: `3bd3372ab1f1a997fe59b2827eb3088af010296a` on
`agent/daniel-title/title-layout` (main integrated through
`2fc45df69f0d397706174117116777f19619dfe2`)

## Revision 2 (this revision)

Revision 1 reported a gap: "no horizontal text measurement (widths,
centering, multi-author side-by-side placement) since that needs glyph
metrics this crate doesn't consume." Revision 2 closes it by having the
**caller** supply exact metrics — this crate still bundles no font, adds
no font-crate dependency (`Cargo.toml`/`Cargo.lock` unchanged from
revision 1), and never guesses an advance width.

Checked for an existing reusable contract first: `flashtex-font-engine`'s
`Face` trait requires an actual loaded font program (glyph ids via a
character map, kerning, ligatures, mark attachment) — not something a
caller with only "exact metrics" data (no font engine, or metrics from an
external source) can implement without standing up a full font backend.
`flashtex-document-style` has no glyph-width concept at all. Neither is a
fit for "caller supplies exact numbers, no font", so a new minimal trait
was defined in-crate (`src/metrics.rs`) rather than forcing a parallel
reuse of a contract shaped for a different job.

```rust
// crates/title-layout/src/metrics.rs
pub trait GlyphMetrics {
    /// Exact advance width of `ch` set at `size` (points), or `None` when
    /// the caller cannot measure it. `None` is the only correct response
    /// to "I don't know" — never a nominal em or other placeholder.
    fn advance_width(&self, ch: char, size: Pt) -> Option<Pt>;
    /// The font's design em ("quad", `\fontdimen6`) at `size`, or `None`.
    /// Used only for the fixed part of `\and`'s `1em plus .17fil`
    /// inter-author glue (the `.17fil` stretch never applies at natural
    /// width, so only the fixed `1em` is needed).
    fn em(&self, size: Pt) -> Option<Pt>;
}

pub fn layout_title_block_with_metrics(
    class: DocumentClass,
    sheet: &flashtex_document_style::Stylesheet,
    input: &TitleBlockInput,
    metrics: &dyn GlyphMetrics,
) -> Result<MeasuredTitleBlock, TitleLayoutError>;

pub struct HorizontalExtent { pub width: Pt, pub x: Pt } // x from text-area left edge

pub struct MeasuredTitleBlock {
    pub layout: TitleBlockLayout,   // unchanged, vertical-only, from layout_title_block
    pub extents: Vec<HorizontalExtent>, // same length/order as layout.rows
}
```

New typed errors (added to `TitleLayoutError`, all still `Display` +
`std::error::Error`):

- `MissingGlyphMetric { ch, size }` — `advance_width` returned `None`;
  the whole block is rejected, not measured with a substituted width.
- `MissingEmMetric { size }` — `em` returned `None` when >1 author needs
  the inter-author gap. A single author never triggers this (no gap needed).
- `RowTooWide { row, natural_width, available_width }` — a title or date
  line's natural width exceeds `sheet.page_layout().text_area.width`.
- `AuthorGroupTooWide { natural_width, available_width }` — the side-by-side
  author group (sum of each author's own widest line, plus `(n-1)` ems)
  exceeds the usable text width.
- `BlockTallerThanPage { total_height, available_height }` — the vertical
  `layout_title_block` result exceeds `sheet.page_layout().text_area.height`.

None of these clip, truncate, or silently overflow; every one is a
rejection with the exact numbers that failed.

Widths are measured by iterating `line.chars()` (Unicode scalar values
decoded from the caller's UTF-8, never raw bytes), so a multi-byte
character costs exactly one `advance_width` query — verified in
`tests/metrics.rs` with Cyrillic/CJK text and by asserting the missing
character reported is the actual offending `char`, not a byte offset.

Placement matches `\@maketitle`'s box model at natural width: each title
line and the date line are independently centered in the text width
(`\begin{center}`); each author is one `tabular[t]{c}` column (its own
lines centered within that author's own widest line), and the authors sit
side by side separated by `\and`'s glue, the whole group centered as one
block. `layout_title_block` itself (vertical-only) is unchanged — same
`TitleBlockInput`/`RowKind`/`MeasuredRow`/`TitleBlockLayout` as revision 1,
still exported, still passing every revision-1 test unmodified.

Revision-1 behavior is preserved: `Report`/`Book`/`Letter` still fail with
`TitleLayoutError::UnsupportedDocumentClass` through
`layout_title_block_with_metrics` too (it delegates to `layout_title_block`
first), never borrowing `article`'s numbers.

New tests (`crates/title-layout/tests/metrics.rs`, 12 tests): missing-glyph
rejection on title/author/date, missing-em rejection gated on author count,
title-row/author-group/page-height overflow each reported explicitly,
multi-byte Unicode measurement and rejection, and exact centering/side-by-side
placement arithmetic.

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

- Line breaking of a long title or long author name into multiple lines —
  callers still supply already-broken lines; only the *placement* of those
  lines (centering, side-by-side authors) is measured, as of revision 2,
  and only when a `GlyphMetrics` implementation is supplied.
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
- `cargo test`: 26 tests + 1 doctest, all pass (4 `abstract_block.rs` +
  10 `title.rs`, unchanged from revision 1, plus 12 new in `metrics.rs`:
  missing-glyph/em rejection, page-bounds overflow, Unicode measurement,
  and centering/side-by-side arithmetic).
- `cargo clippy --all-targets -- -D warnings`: 0 warnings.
- `cargo fmt --check`: clean.

No workspace root `Cargo.toml` was added (repo convention); each crate is
built from its own directory, per `crates/title-layout/Cargo.lock` locking
`flashtex-document-style` as a path dependency.

## Needs from others / integration notes

- Not wired into any consumer (compiler/apps/mac). Standalone per FT-033.
- A consumer that wants horizontal placement must implement `GlyphMetrics`
  itself, typically backed by `flashtex-font-engine`'s `Face::advance`
  (converted from font units to points at the queried size) or an
  equivalent exact source. `layout_title_block` (vertical-only, no
  metrics required) still works standalone for any consumer that doesn't
  need placement yet.
- `flashtex-font-engine` and `flashtex-document-style` were read (not
  edited) while designing this contract, to check for a reusable existing
  metrics contract before adding a new one — see "Revision 2" above for
  why `Face` wasn't reused. No peer compiler/native/layout files were
  modified; only `crates/title-layout/**` and this handoff.

Updated: 2026-09-12

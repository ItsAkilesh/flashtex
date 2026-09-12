# daniel-statistics handoff — FT-042

Agent / task / branch: daniel-statistics / FT-042 revision 2 (cached document
statistics with dependency-aware project aggregation) /
`agent/daniel-statistics/document-statistics`
Owned paths: `crates/document-statistics/**`, `coordination/daniel-statistics.md`
State: ready for integration (standalone additive crate; no consumer wired yet)
Tested commit SHA: `e56b2e56a9a4ca9409a74b43014039f2db051bb4`
Previous (rev 1) tested SHA: `92dd4ee806e513c68096e32ab0ff89a0133355d8`
Main integrated through (merged and reviewed): `e5901797e8a7ebdd8d714ecdee6793e1097515a9`

Validation at that SHA (`cd crates/document-statistics`):
- `cargo build` — clean.
- `cargo test` — 46 tests total: 39 unit tests (`src/`), 2 property-style
  integration tests (`tests/incremental_equals_fresh.rs`), 4 integration
  tests (`tests/statistics.rs`), 1 doctest. All pass.
- `cargo clippy --all-targets -- -D warnings` — 0 warnings.
- `cargo fmt --check` — clean.

## Rev 2: what changed

Rev 2 adds a cache on top of rev 1's pure `Statistics::compute`, without
changing rev 1's counting behavior or honesty guarantees at all.

**New: `ScanLimit` / `ScanTooLarge` (`src/scan_limit.rs`).** A typed cap on
how many source bytes (`SourceItem::Text` + `MathItem::source` byte length)
one computation may scan. `Statistics::compute_bounded(revision, items,
limit)` enforces it, stopping mid-scan and returning `ScanTooLarge { scanned,
limit }` — never a partial or truncated `Statistics` — the instant the
running total would exceed `limit`. `Statistics::compute` is now defined as
`compute_bounded(..., ScanLimit::UNBOUNDED)`, so its behavior and doctest are
unchanged. `Statistics` gained one new field, `scanned_bytes`, recording
exactly what was scanned to produce it.

**New: `ProjectCache` (`src/cache.rs`).** Caches one `Statistics` per
document, keyed by exact source identity: `RevisionId.source` +
`RevisionId.revision` + the same FNV-1a content fingerprint rev 1 already
used for `is_current_for`. `ProjectCache::update(revision, items, limit)` is
a hit (0 bytes rescanned) only when ALL THREE match the stored entry; a
revision id reused over changed content is therefore a MISS, not a stale
hit — see `cache::tests::reused_revision_id_over_changed_content_is_a_miss_not_a_stale_hit`
and the same scenario exercised inside the big property test below.
Updating one document leaves every other document's cached entry
byte-for-byte unchanged (`cache::tests::unrelated_document_survives_a_sibling_update`),
and `ProjectCache::project_totals()` sums whatever is currently cached — this
is the dependency-aware project aggregation: the project total depends on
every document, but invalidation is per-document.

**Proof that incremental equals fresh
(`tests/incremental_equals_fresh.rs`):** a hand-rolled deterministic PRNG
(no external `rand` dependency) drives 40 independent edit sequences over
1–5 documents each, 20–49 steps per sequence (1,160 steps total), mixing
text/inline-math/display-math/page-mark appends, same-revision content
replacement (the stale-reuse case), and no-op repeats (the pure-hit case).
After **every single step**, the test asserts the `ProjectCache` lookup for
the just-edited document, and the summed project totals over the whole
cache, are exactly equal — word/math/page counts and content hash — to
recomputing everything from scratch with `Statistics::compute`. A stale
cache entry surviving an update, or a hit served for changed content, would
fail this assertion; the test does not loosen it in either case. The same
run also measures scanned bytes: over the full 1,160-step corpus the
incremental path scanned 127,436 bytes total vs. 326,335 bytes for
recomputing every document from scratch at every step (~39% of fresh, i.e.
scanning well under half) — the test asserts this inequality, so a cache
that stopped saving work would fail CI, not just look suspicious in a
report.

## What this crate is

`flashtex-document-statistics` computes bounded word/math/page counts from an
explicit, caller-supplied list of already-rendered content items. It does
**no** filesystem or network access and does not parse, render, or tokenize
TeX itself — assembling the item list from an actual document is the
caller's job. This is a standalone crate with zero dependencies on any other
FlashTeX crate; it does not touch `crates/compiler`, layout, or any native
code, and no consumer is wired up. Integration (deciding who calls this and
how items get produced) is left for a follow-up negotiation, per FT-042's
"coordinate consumer contract before integration" note.

## Typed contract (public API)

```rust
pub struct RevisionId { pub source: String, pub revision: u64 }
impl RevisionId { pub fn new(source: impl Into<String>, revision: u64) -> Self }

pub enum SourceItem {
    Text(String),               // rendered prose
    Math(MathItem),             // one formula; source never word-counted
    PageMark,                   // one per rendered page, including the first
}
pub struct MathItem { pub source: String, pub display: bool }
impl SourceItem {
    pub fn text(s: impl Into<String>) -> Self;
    pub fn inline_math(source: impl Into<String>) -> Self;
    pub fn display_math(source: impl Into<String>) -> Self;
}

pub struct WordStats { pub words: usize, pub chars: usize }
pub struct MathStats { pub total: usize, pub inline: usize, pub display: usize }

pub struct Statistics {
    pub revision: RevisionId,
    pub content_hash: u64,      // FNV-1a fingerprint of the exact items counted
    pub words: WordStats,
    pub math: MathStats,
    pub pages: usize,
    pub scanned_bytes: usize,   // rev 2: bytes actually scanned to produce this
}
impl Statistics {
    pub fn compute(revision: RevisionId, items: &[SourceItem]) -> Statistics;
    pub fn compute_bounded(revision: RevisionId, items: &[SourceItem], limit: ScanLimit)
        -> Result<Statistics, ScanTooLarge>;
    pub fn is_current_for(&self, revision: &RevisionId, items: &[SourceItem]) -> bool;
}

// rev 2 additions:
pub struct ScanLimit(/* opaque */);
impl ScanLimit {
    pub const UNBOUNDED: ScanLimit;
    pub const fn new(max_bytes: usize) -> Self;
    pub const fn max_bytes(self) -> usize;
}
impl Default for ScanLimit { /* 1 MiB */ }

pub struct ScanTooLarge { pub scanned: usize, pub limit: usize } // impl Error, Display

pub struct ProjectCache { /* opaque */ }
impl ProjectCache {
    pub fn new() -> Self;
    pub fn update(&mut self, revision: RevisionId, items: &[SourceItem], limit: ScanLimit)
        -> Result<Lookup, ScanTooLarge>;
    pub fn get(&self, source: &str) -> Option<&Statistics>;
    pub fn contains(&self, source: &str) -> bool;
    pub fn project_totals(&self) -> ProjectTotals;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
pub struct Lookup { pub stats: Statistics, pub hit: bool, pub bytes_scanned: usize }
pub struct ProjectTotals { pub words: WordStats, pub math: MathStats, pub pages: usize }
```

`Statistics::compute` is pure counting over `items`; `is_current_for` is the
staleness guard (see "Revision binding" below). Everything is `Clone + Debug
+ PartialEq + Eq`, no panics on any input shape tested (empty, whitespace-only,
punctuation-only, oversized lists).

## Counting rule (stated, not hidden)

Word counting is inherently lexical and language-dependent, so this crate
commits to one explicit rule instead of pretending to solve the general
problem (see doc comments on the `words` module for the full text):

1. Split text on Unicode whitespace (`char::is_whitespace`).
2. A token counts as one word iff it contains at least one Unicode
   alphanumeric character (`char::is_alphanumeric`); punctuation-only,
   symbol-only, or emoji-only tokens (`"--"`, `"..."`, a lone `"🎉"`) do not
   count.
3. A token is never split further: internal hyphens (`well-known`) and
   apostrophes (`don't`, `O'Brien`) keep it as ONE word. No hyphenation or
   contraction-aware splitting is attempted.
4. Math item source text is never scanned for words — math is not prose —
   regardless of how prose-like it looks.

**Stated limitation, tested rather than hidden:** rule 1 makes `words`
meaningless for Chinese/Japanese/Korean running text, which has no
whitespace: a whole CJK sentence is one token and therefore exactly one
"word" (see `words::tests::cjk_text_undercounts_by_design` and
`tests/statistics.rs::unicode_document_documents_its_own_limitation`). No
CJK segmenter is implemented — that needs a dictionary or trained model,
outside "bounded statistics from explicit items." `WordStats::chars` (count
of non-whitespace Unicode scalar values) is offered as a script-agnostic
size proxy for callers who need something comparable across scripts; it is
explicitly documented as not a word count either.

Other tested edge cases: combining marks (`e` + U+0301) and zero-width
joiners don't split or hide a token; digits count as words; empty and
whitespace-only text (including U+00A0 NBSP) yields zero words.

## Revision binding (exact, tested)

Every `Statistics` carries the `RevisionId` it was computed for AND a
deterministic FNV-1a `content_hash` over a length-prefixed encoding of the
exact items counted (so e.g. `[Text("ab"), Text("c")]` and
`[Text("a"), Text("bc")]` never collide). `is_current_for(revision, items)`
recomputes the fingerprint and requires **both** the `RevisionId` and the
content hash to match — a `RevisionId` match alone is not trusted, so a
caller that reuses a revision id for changed content is correctly told the
cached statistics are stale. Tests:
`statistics::tests::stale_when_content_changed_but_revision_id_reused`,
`stale_when_revision_id_differs_even_with_identical_content`,
`fingerprint_is_deterministic_for_identical_items`,
`fingerprint_distinguishes_concatenation_boundaries`, and
`tests/statistics.rs::revision_binding_survives_across_the_public_api`.
The hash is not cryptographic and not promised stable across crate
versions — it only needs to catch drift within one process/pipeline run,
which is what the acceptance criterion requires.

## Explicitly out of scope / unfinished

- No CJK/script-aware word segmentation (see above — a stated limitation,
  not an oversight).
- No consumer adapter: nothing in `crates/compiler` or elsewhere calls this
  crate yet. This crate does not decide how `SourceItem`s get produced from
  a real document; that's the negotiation FT-042 asks to defer.
- No serialization (no `serde`): callers needing to persist `Statistics` or
  `ProjectCache` across process boundaries will need to add that at the
  integration boundary, not in this crate.
- `ProjectCache` holds one entry per document `source` string; it does not
  itself track which documents belong to which project, or evict entries for
  documents that have been deleted from a project. A caller with a document
  set that shrinks over time should drop the corresponding entries itself
  (there is no `remove` method yet — add one if a consumer needs it).
- `ScanLimit::default()` (1 MiB) is a placeholder; no consumer has stated a
  real requirement yet, so it is not tuned to any actual document size
  distribution.

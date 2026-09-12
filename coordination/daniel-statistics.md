# daniel-statistics handoff — FT-042

Agent / task / branch: daniel-statistics / FT-042 revision 4 (actual existing
consumer fixture and exact measured limitations) /
`agent/daniel-statistics/document-statistics`
Owned paths: `crates/document-statistics/**`, `coordination/daniel-statistics.md`
State: ready for integration (standalone additive crate; no consumer wired
in production — rev 4 adds a test-only fixture against the real producer,
see below)
Tested commit SHA: `8c94259640bb12f20593187107045d16dcf0d7f1`
Previous (rev 3) tested SHA: `cce30e1d9f692316f7df16d116752e7bf043cc72`
Previous (rev 2) tested SHA: `e56b2e56a9a4ca9409a74b43014039f2db051bb4`
Previous (rev 1) tested SHA: `92dd4ee806e513c68096e32ab0ff89a0133355d8`
Main integrated through (merged and reviewed): `abbe88a5275b89d99357815846de3cbe76a91810`

Validation at that SHA (`cd crates/document-statistics`):
- `cargo build` — clean.
- `cargo test` — 73 tests total: 39 unit tests (`src/`), 2 property-style
  integration tests (`tests/incremental_equals_fresh.rs`), 14 adversarial
  acceptance tests (`tests/adversarial_bounds.rs`), 5 stale-identity
  acceptance tests (`tests/stale_identity_acceptance.rs`), 4 integration
  tests (`tests/statistics.rs`), 5 real-producer consumer-fixture tests
  (`tests/consumer_fixture.rs`), 3 measurement tests
  (`tests/corpus_measurements.rs`), 1 doctest. All pass.
- `cargo clippy --all-targets -- -D warnings` — 0 warnings.
- `cargo fmt --check` — clean.

## Rev 4: real-producer consumer fixture and exact measured limitations

**The search.** Grepped `crates/` for anything that already builds a
document/item structure this crate could consume. `flashtex-compiler`
(`crates/compiler`) is the only one: `parser::parse`/`parse_project` produces
`Parsed { blocks: Vec<Block>, .. }` of `Block::{Paragraph,Heading,
FigureCaption}` holding `Inline::{Text,Math,LineBreak,Label,Reference}`, and
`layout::layout(&blocks)` produces real `Vec<Page>` (Adobe Core 14 metrics,
no external font files needed) with real page breaks.
`crates/document-runtime` is a JSON-Lines transport around an *external*
compiler process and defines no item/document model of its own — it is not
a producer. Nothing else under `crates/` parses `.tex` or builds a
paragraph/inline tree. **Nothing in this repository calls
`flashtex-document-statistics` in production** — that is unchanged from rev
3's negotiated deferral, not a rev 4 regression.

**The fixture (`tests/support/mod.rs`, `tests/consumer_fixture.rs`).**
Because the only real producer is `flashtex-compiler` and no consumer is
wired up, rev 4 adds `flashtex-compiler` as a **dev-dependency only**
(`[dev-dependencies]` in `Cargo.toml`; `src/` gains no new dependency,
preserving the "no filesystem, no parsing" guarantee in `src/lib.rs`) and
writes a test-only adapter:
- Reads every `.tex` file under `tests/tex-corpus/cases/**` from disk — no
  hand-written LaTeX.
- Runs the real `parser::parse_project` (so `\input`/`\include` resolution
  is the compiler's own) and the real `layout::layout`.
- `Inline::Text` → `SourceItem::Text`, one item per real compiler
  word-token. `Inline::Math` → `SourceItem::Math`, its `source` sliced
  **verbatim from the real document bytes** via the compiler's own `Span`
  (delimiters included, e.g. `"$x_1^2 + y$"`, `"\[\frac{a+b}{c}=d\]"`) — not
  reconstructed from the parsed `MathList`. `Inline::LineBreak`/`Label`/
  `Reference` map to nothing: this crate has no notion of them.
  One `SourceItem::PageMark` per page the real layout engine actually laid
  out.
`crates/compiler` was not edited.

Five tests exercise this: every real corpus case computes bounded
statistics without panicking; `plain-paragraphs` word count and page count
match the real compiler output exactly; `math-inline-display`'s math is
counted and never word-counted, and its two math items' `source` strings
are exactly the real sliced spans; `included-file`'s prose includes text
that only exists in the actually-`\input`-ed file, not main.tex; and editing
the real `plain-paragraphs/main.tex` text on disk (in memory, then
re-parsed) is visible through `is_current_for` under the same revision id.

**The measurements (`tests/corpus_measurements.rs`), run over the same real
corpus:**

| Metric | Value |
|---|---|
| Corpus cases (`tests/tex-corpus/cases/*`) | 14 |
| Total real `.tex` documents (incl. included files) | 16 |
| Total words counted (this crate's rule, summed over all 14 cases) | 49 |
| Total math items counted | 2 |
| Total pages (real layout engine) | 14 |

CJK limitation, measured, not described: of the 14 cases, exactly **2**
contain CJK script:

| Case | Real token | CJK chars | Words counted | Undercount |
|---|---|---|---|---|
| `literal-source-map` | `尾` | 1 | 1 | 0 |
| `unicode-literals` | `東京.` | 2 | 1 | **1** |

`literal-source-map`'s `尾` sits alone on its own line, so it is its own
whitespace-delimited token: 1 CJK character counted as 1 word — not
undercounted by this measure. `unicode-literals`'s `東京.` is one real
compiler word-token (the lexer only breaks a word on whitespace or a TeX
special character; `.` is neither): 2 real CJK characters, counted as
exactly 1 word by the rule in `src/words.rs`. Measured undercount across the
whole real corpus: **1 word**, entirely on that one document. (The corpus is
small; this is the actual, exact, small number it produces — not scaled up
or extrapolated.)

Cache, measured over a realistic edit sequence, not asserted only as an
inequality: all 14 real corpus cases loaded as 14 project documents; 30
rounds, one document edited per round round-robin (the "edit" is that
document's real items plus one real `SourceItem::Math` borrowed verbatim
from elsewhere in the corpus — still real content, never invented text),
every other document resubmitted unchanged each round (the realistic
"recompute the whole project on every edit" shape):

| Metric | Value |
|---|---|
| Total `ProjectCache::update` calls (30 rounds × 14 docs) | 420 |
| Hits | 377 |
| Misses | 43 |
| Hit rate | 89.76% |
| Bytes scanned, incremental (only misses) | 1,087 |
| Bytes that a fresh-every-round recompute would scan | 11,137 |
| Bytes saved | 10,050 (90.2% of the fresh-recompute cost) |

All numbers above are asserted exactly in `tests/corpus_measurements.rs`
(not just bounded/inequality checks) so they cannot silently drift; if the
corpus changes, re-run `cargo test --test corpus_measurements -- --nocapture`
and update both the assertions and this table.

Rev 2's incremental-equals-fresh property test and rev 3's adversarial
bounds and stale-identity acceptance suites are untouched — no `src/` file
was modified in rev 4.

## Rev 3: what changed

No source under `src/` changed at all: rev 1's counting rule (including the
documented CJK limitation) and rev 2's `ScanLimit`/`ProjectCache` behavior
are untouched. Rev 3 adds two new test files that attack and specify that
existing behavior.

**New: `tests/adversarial_bounds.rs` (14 tests).** Directly attacks the
scanner and the cache, asserting every case resolves to either a typed
`ScanTooLarge` or a bounded `Ok` — never a panic, a hang, or unbounded
memory growth:
- Scan-byte limit boundary: exactly `limit` bytes is accepted
  (`scan_exactly_at_the_limit_is_accepted`); one byte past it is a typed
  error with exact `{scanned, limit}` fields
  (`scan_one_byte_past_the_limit_is_a_typed_error_with_exact_fields`); a
  zero limit is a legal edge case handled without panicking
  (`scan_limit_of_zero_rejects_any_nonempty_content_but_accepts_empty`).
- `a_project_with_thousands_of_documents_stays_correct_and_bounded`: 5,000
  distinct documents, each inserted then re-confirmed as a hit; `cache.len()`
  stays exactly 5,000, never one-per-call.
- `a_single_enormous_text_item_is_bounded_by_scan_limit_not_by_luck`: one
  ~2,000,000-byte text item — the limit binds on a single oversized item,
  not only on accumulation across many small ones; unbounded, the same item
  still terminates with a correct count.
- Combining-mark-only and zero-width-character-only text
  (`text_of_only_combining_marks_is_bounded_and_not_a_word`,
  `text_of_only_zero_width_characters_is_bounded_and_not_a_word`, and 1,000
  pathological items summed in
  `mix_of_combining_marks_and_zero_width_characters_across_many_items_is_bounded`):
  correctly counted as zero words (no alphanumeric scalar value) while
  `chars`/`scanned_bytes` still account for every byte, with no panic.
- `a_revision_id_at_u64_max_behaves_exactly_like_any_other_revision`: `u64::MAX`
  works exactly like any other revision number, including caching correctly
  and not colliding with a wrapped-around revision 0.
- Same document id re-registered many times with different content:
  `same_document_id_registered_twice_with_different_content_never_duplicates_the_entry`
  (200 re-registrations, same revision never bumped, `cache.len()` stays 1
  throughout) and `same_document_id_same_revision_different_content_is_a_miss_not_corruption`.
- `an_empty_project_answers_every_query_with_a_bounded_default_never_a_panic`:
  every `ProjectCache` query on a project with zero documents.
- Cache does not grow without bound under repeated updates:
  `repeatedly_updating_one_document_never_grows_the_cache_past_one_entry`
  (20,000 updates to one document, `cache.len() == 1` after every single
  one) and `repeatedly_updating_a_fixed_set_of_documents_never_grows_the_cache_past_that_set`
  (5,000 updates spread over 10 documents, `cache.len() <= 10` throughout).

**New: `tests/stale_identity_acceptance.rs` (5 tests).** Restates rev 2's
property-tested cache contract (`tests/incremental_equals_fresh.rs`) as a
readable specification of when a cached answer may be trusted, one example
test per rule:
1. `same_revision_same_content_is_a_hit_the_cached_answer_is_trustworthy` —
   the only condition under which a cached answer is safe to trust without
   rescanning.
2. `same_revision_changed_content_is_a_miss_the_cached_answer_would_be_a_lie`
   — **the dangerous case**: a caller that reuses a revision id over content
   that actually changed must never be served the old, stale answer.
3. `changed_revision_same_content_is_a_miss_identity_must_match_exactly` — a
   content match alone does not entitle a caller to a cached answer; the
   claimed revision must match too.
4. `a_sibling_documents_entry_is_untouched_by_either_a_hit_or_a_miss_elsewhere`
   — a sibling document's cache entry survives byte-for-byte through both a
   hit and a miss on another document.
`the_full_trust_contract_in_one_sequence` walks all four rules in one
narrative sequence against a single cache instance.

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
- No *production* consumer adapter: nothing in `crates/compiler` or
  elsewhere calls this crate yet. Rev 4 added a dev-dependency-only test
  fixture (`tests/support/mod.rs`) proving an adapter over the real
  `flashtex-compiler` output is possible and measuring it against the real
  corpus, but this crate still does not decide how `SourceItem`s get
  produced from a real document in production; that integration is the
  negotiation FT-042 asks to defer.
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

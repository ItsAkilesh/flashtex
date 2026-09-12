# daniel-statistics handoff — FT-042

Agent / task / branch: daniel-statistics / FT-042 (bounded document statistics)
/ `agent/daniel-statistics/document-statistics`
Owned paths: `crates/document-statistics/**`, `coordination/daniel-statistics.md`
State: ready for integration (standalone additive crate; no consumer wired yet)
Tested commit SHA: `92dd4ee806e513c68096e32ab0ff89a0133355d8`
(branch base / `input_main_sha` per `coordination/assignments/FT-042.json`:
`53fee3012b2902ca05bd31766defa515b3044cec`)

Validation at that SHA (`cd crates/document-statistics`):
- `cargo build` — clean.
- `cargo test` — 26 unit tests + 4 integration tests + 1 doctest, all pass.
- `cargo clippy --all-targets -- -D warnings` — 0 warnings.
- `cargo fmt --check` — clean.

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
}
impl Statistics {
    pub fn compute(revision: RevisionId, items: &[SourceItem]) -> Statistics;
    pub fn is_current_for(&self, revision: &RevisionId, items: &[SourceItem]) -> bool;
}
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
- No serialization (no `serde`): callers needing to persist `Statistics`
  across process boundaries will need to add that at the integration
  boundary, not in this crate.

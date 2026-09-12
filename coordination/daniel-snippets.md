# FT-041: editor-snippets

Status: complete through revision 4, standalone additive crate. No other
crate was touched (a dev-only path dependency on `crates/edit-ledger` was
added to this crate's own `Cargo.toml`; that crate's files were not edited).

## Tested commit (revision 4)

```
a9bccc94007821a0be83a60b04219e701c1f0c99
```

On branch `agent/daniel-snippets/editor-snippets`, main integrated through
`abbe88a5275b89d99357815846de3cbe76a91810`. At this exact SHA, inside
`crates/editor-snippets`:

- `cargo build` — clean.
- `cargo test` — 41 rev-1 + 17 rev-2 plan + 15 rev-3 adversarial bounds + 5
  rev-3 stale-identity + 4 new real-consumer + 3 new corpus-coverage tests +
  2 doc-tests, all passing (87 total).
- `cargo clippy --all-targets -- -D warnings` — clean, zero warnings.
- `cargo fmt --check` — clean.

## Revision 4: real consumer fixture, and measured (not listed) snippet coverage

No src/ file changed; two new integration test files only, plus a
`[dev-dependencies]` path dependency on `flashtex-edit-ledger` (dev-only —
absent from the published library's own dependency graph).

### Objective 1: the actual existing consumer, exercised

`apps/mac` (the Swift editor) has **no dependency on this crate, or on any
Rust crate, today.** `apps/mac/Package.swift` builds `FlashTeXMac` only
against the pure-Swift `FlashTeXProtocol`/`FlashTeXAccessibility` targets,
and a repo-wide grep for `editor-snippets` / `flashtex_editor_snippets` /
`SnippetPlan` outside this crate's own directory returns nothing. So this
is genuinely a language boundary with no FFI/Cargo edge to cross — there is
nothing honest to call from a Rust test into Swift, and the assignment's own
framing anticipated exactly this.

What Mac and the Rust worker actually share is a JSON Lines wire contract
(`docs/contracts/runtime-v1.md`, `docs/contracts/transfer-v1.md`): snake_case
fields, zero-based end-exclusive UTF-8 byte offsets, which Swift converts
explicitly (`apps/mac/Sources/FlashTeXProtocol/ByteOffsets.swift`). The exact
edit shape that contract carries is `flashtex_edit_ledger::PreparedEdit`
(its own doc comment: "Wire-compatible with transfer-v1 `PreparedEdit`"),
and `docs/contracts/transfer-v1.md` names its exact field list
(`capture_id,edit_id,project_id,path,expected_revision,start_byte,end_byte,
removed_text,replacement,document_before_sha256`) — byte-for-byte what
`TransferV1.CaptureEdit` in `apps/mac/Sources/FlashTeXProtocol/TransferV1.swift`
decodes via `CodingKeys`. `tests/real_consumer_edit_ledger.rs`'s
`field_names_match_the_transfer_v1_wire_contract` pins this crate's output,
carried inside `PreparedEdit`, against that exact field set via
`PreparedEdit`'s own `Serialize` impl — the contract the bridge actually
carries, not a pretense of invoking Swift.

`crates/edit-ledger` ("Durable document and reviewed-edit transactions for
FlashTeX native consumers") is the real, in-repo, same-language production
consumer that turns a byte-offset edit into a mutated, durable document.
Because it's same-language, it is exercised directly rather than
reimplemented — added only as a dev-dependency, never edited:

- `snippet_plan_insertion_survives_the_real_edit_ledger_store` computes a
  `SnippetPlan` for a linked `\begin{X}...\end{X}` snippet, opens a real
  `flashtex_edit_ledger::Store` (real filesystem, lock file, fsync), and
  applies the plan's expansion via the actual `Store::apply(PreparedEdit)`
  path. Every occurrence `Expansion::occurrences_of(1)` reported is checked
  against the *post-application* real document text.
- `linked_placeholder_edit_propagates_through_a_grouped_edit_transaction`
  simulates the user tabbing to the linked placeholder and retyping it: it
  builds one `SourceEdit` per linked occurrence from this crate's own
  offsets and applies them via the real `Store::apply_group(GroupedEdit)` —
  edit-ledger's actual mechanism for "nonoverlapping byte ranges in the same
  original source snapshot" as one atomic transaction — proving both
  linked occurrences update together through the real consumer.
- `stale_plan_is_rejected_by_the_real_store_exactly_when_this_crate_predicts`
  ties rev 2's staleness contract to the real consumer: an unrelated real
  edit advances the store's revision out from under a computed plan; this
  crate's own `staleness()` call is asserted stale first, and the real
  store's own `Store::apply` is then independently shown to refuse the same
  stale `PreparedEdit`.

### Objective 2: measured LaTeX snippet coverage

`tests/corpus_snippet_coverage.rs` derives 22 realistic snippet shapes from
the commands/environments that actually appear in
`tests/tex-corpus/cases/**/main.tex` (`\documentclass`, `\begin`/`\end`
`document`/`tikzpicture`, `\newcommand`, `\renewcommand`, `\input`,
`\usepackage`, `\draw`, `\frac`, inline/display math, and the escaped
specials `\$ \% \& \_ \#`), written the way a real LaTeX snippet library —
and the corpus itself — naturally writes LaTeX: a single backslash before a
command name. Each candidate's exact `Snippet::parse` outcome is asserted
and the aggregate counts are pinned so they cannot silently drift.

| Measurement | Count | Share |
|---|---|---|
| Realistic corpus-derived snippet shapes | 22 | 100% |
| Parse successfully today | 1 | 5% |
| Rejected as `InvalidEscape` (see finding below) | 21 | 95% |
| ...of which need **no** fuller-LSP feature at all | 13 of 14 `None`-tagged shapes | — |
| Intended feature: numbered placeholders only (`None`) | 14 | 64% |
| Intended feature: choice placeholder (`${1\|a,b\|}`) | 4 | 18% |
| Intended feature: named/special variable (`$TM_...`) | 3 | 14% |
| Intended feature: transform (`${1/re/rep/}`) | 1 | 5% |

**Top offender, and not one of the three previously-documented gaps:** this
crate's `\` is reserved entirely for its own escape grammar — only `\$ \} \\`
are valid escapes (`src/parser.rs::parse_escape`) — so a literal LaTeX
command name (`\begin`, `\newcommand`, `\input`, `\usepackage`, `\draw`,
`\frac`, ...) or even LaTeX's own escaped specials other than `\$` (`\%
\& \_ \#`) are rejected as `SnippetError::InvalidEscape` before the
placeholder grammar is ever reached. This dominates the three documented
gaps combined: **13 of the 14 shapes that need zero fuller-LSP features
still fail**, purely on this. Only `\$${1:x}\$` (inline math, which happens
to need only the one escape this crate supports) parses. A snippet author
can work around it today only by doubling every such backslash (`\\begin{...`
— confirmed parseable, used in the rev-4 consumer fixture's own snippet),
which is undocumented and not how any real LaTeX source or LaTeX snippet
library is written.

Isolated probes (`tests/corpus_snippet_coverage.rs`,
`isolated_lsp_feature_probes_match_their_exact_measured_outcome` and
`bare_named_variable_silently_misparses_instead_of_erroring`), each free of
the backslash finding above, independently confirm the three previously-
documented gaps and rank them by how they fail:

| Feature | Needed by (of 22) | Outcome when isolated |
|---|---|---|
| Choice placeholder | 4 (most frequent of the three) | Hard rejection: `UnterminatedPlaceholder` |
| Named/special variable (braced, `${TM_...}`) | 3 | Hard rejection: `InvalidPlaceholderIndex` |
| Named/special variable (bare, `$TM_...`) | (same 3) | **Silent misparse** — parses, output unchanged, no error at all |
| Transform | 1 (least frequent) | Hard rejection: `UnterminatedPlaceholder` |

The bare-named-variable case is the sharpest latent bug of the three: a
lone `$` not followed by a digit or `{` is treated as literal text
(`parse_dollar`'s fallback arm), so `$TM_SELECTED_TEXT` "parses" and
expands to itself, byte for byte — nothing signals that the intended
substitution never happened.

Rev 1-3 (linked edits, tab order, UTF-8 safety, plan identity/staleness,
adversarial bounds, stale-identity matrix, no mutating method) are
unchanged and still pass unmodified.

## Revision 3: bounded adversarial and stale-identity acceptance tests

No production code changed. Two new integration test files were added,
both driving the same public API rev 1/2 already shipped:

**`tests/adversarial_bounds.rs`** attacks the *whole pipeline* — a single
`attempt_plan` helper chains `Snippet::parse` → `Snippet::expand_with` →
`SnippetPlan::compute`, so every case is exercised exactly as a real caller
would use the crate, not just one internal stage. 15 tests cover, each as a
typed `PlanError` (never a panic or a hang):

- every numeric bound in `limits.rs` as a boundary *pair* — exactly at the
  limit succeeds, one unit past fails with the exact documented variant:
  input bytes (`MAX_INPUT_BYTES`, `InputTooLarge`), nesting depth
  (`MAX_NESTING_DEPTH`, `NestingTooDeep`), placeholder index
  (`MAX_PLACEHOLDER_INDEX`, `PlaceholderIndexTooLarge`), distinct
  placeholders (`MAX_PLACEHOLDERS`, `TooManyPlaceholders`), occurrences
  (`MAX_OCCURRENCES`, `TooManyOccurrences`), and output bytes
  (`MAX_OUTPUT_BYTES`, `OutputTooLarge`, driven via an override large
  enough to hit the cap without needing a huge source template);
- unbalanced and escaped braces (`UnterminatedPlaceholder` for both
  `${1:abc` and `${1`; an escaped `\}` stays literal; a stray unmatched
  `}` at top level is literal text, not an error);
- a dollar sign at end of input (`abc$` stays literal, no hang);
- mutually self-referential placeholders (`${1:$2}${2:$1}` →
  `SelfReferential`);
- a caret past the end of the document, an inverted selection, and a caret
  mid-character in multi-byte text (`café` at offset 4) — each
  `PlanError::InvalidOffset` / `PlanError::SelectionReversed`, with the
  mid-character case also asserting the valid boundary immediately before
  it still succeeds, so the check is proven to be a real char-boundary
  test rather than a blanket rejection.

**`tests/stale_identity_acceptance.rs`** is an explicit specification for
the `SnippetPlan`/`DocumentId` staleness contract, stated as the full
2×2 matrix of "revision same/changed" × "content same/changed":

| revision  | content   | expected staleness |
|-----------|-----------|---------------------|
| unchanged | unchanged | `Fresh`             |
| unchanged | changed   | `ContentMismatch`   |
| changed   | unchanged | `RevisionMismatch`  |
| changed   | changed   | `RevisionMismatch`  |

Each row is its own named test against `SnippetPlan::staleness`
(`same_revision_and_same_content_is_fresh`,
`changed_revision_with_same_content_is_revision_mismatch`,
`same_revision_with_changed_content_is_content_mismatch_not_fresh`,
`changed_revision_and_changed_content_is_revision_mismatch`), plus one more
(`document_id_compare_matches_the_same_four_row_matrix`) restating the same
four rows directly against `DocumentId::compare`, so the specification is
pinned at both the identity primitive and the plan contract built on it.
The "unchanged/changed" row is the load-bearing one: it holds the revision
id constant while changing only the document's bytes, proving the check
cannot be satisfied by revision alone — a naive check that trusted the
revision counter would wrongly call this fresh. The "changed/changed" row
documents that there is no third "everything changed" variant:
`DocumentId::compare` checks revision first, so any revision mismatch
reports `RevisionMismatch` regardless of what the content hash shows.

Rev 1 (linked edits, tab order, UTF-8 structural safety) and rev 2
(`SnippetPlan` staleness/anchors) are unchanged and still pass unmodified;
`SnippetPlan` still exposes no mutating method — only `compute` (shared
refs + owned `Anchor`), getters, and the pure `staleness` comparison.

## Revision 2: SnippetPlan (document identity, Unicode anchors)

Adds `SnippetPlan`, `DocumentId`/`ContentHash`/`Staleness`, `Anchor`, and
`PlanError` (see `src/plan.rs`, `src/identity.rs`, `src/anchor.rs`). None of
rev 1's public API (`Snippet`, `Expansion`, `TabStops`, `SnippetError`) was
removed or changed; `SnippetPlan` is purely additive.

**Plan identity.** `DocumentId { revision: u64, content_hash: ContentHash }`
binds a plan to the caller's own opaque revision counter *and* an FNV-1a
hash of the document's exact bytes at that revision — dependency-free and
deterministic across processes/versions (unlike
`std::collections::hash_map::DefaultHasher`, whose algorithm the standard
library documents as unspecified). `DocumentId::compare` (and
`SnippetPlan::staleness`) returns a typed `Staleness`: `Fresh`,
`RevisionMismatch`, or `ContentMismatch`. The critical case is tested
directly: `bytes_changed_but_revision_id_did_not_is_still_detected_as_stale`
in `tests/plan.rs` holds the revision id constant across two `DocumentId`s
built from different text and asserts the result is `ContentMismatch`, not
`Fresh` — a staleness check that only compared `revision` would get this
wrong.

**Unicode caret/selection.** `Anchor::{Caret(usize), Selection(Range<usize>)}`
carries byte offsets into the document text. `SnippetPlan::compute` is the
only place an `Anchor` becomes part of a plan, and it validates both
offsets there against the exact `document_text` passed in —
`str::is_char_boundary` (which is also false past the end of the string) —
rejecting anything else as `PlanError::InvalidOffset` or
`PlanError::SelectionReversed`, extending rev 1's structural parser
guarantee (every offset the crate returns is a valid `char` boundary) to
these caller-supplied offsets too, by explicit validation rather than
by construction. Tested with CJK (`caret_at_a_valid_char_boundary_in_cjk_text_succeeds`
/ `caret_mid_character_in_cjk_text_is_a_typed_error_not_a_panic`), accented
Latin (`caret_mid_character_in_accented_latin_text_is_rejected`, "café"),
and a multi-codepoint ZWJ emoji sequence
(`selection_around_a_multi_codepoint_emoji_sequence_is_unicode_safe`).

**Placeholders preserved.** `SnippetPlan::compute` calls
`Snippet::expand_with` internally and exposes the resulting `Expansion`
unchanged via `.expansion()`; linked edits (`plan_compute_applies_overrides_for_a_linked_edit`),
tab order (`plan_expansion_tab_order_matches_direct_expand`), and
self-referential detection (`plan_still_reports_self_referential_placeholders_as_a_typed_error`)
are all re-tested through the plan API, plus all 41 original rev-1 tests
still pass untouched.

**Explicit caller application, enforced by API shape.** `SnippetPlan`'s
only public methods are `compute` (a constructor taking shared references
only — `&Snippet`, `&str`, `&HashMap`, and an owned `Anchor` — never a
mutable document handle), plus getters (`document()`, `anchor()`,
`expansion()`) and a pure comparison (`staleness()`). There is no `apply`,
`commit`, or any method that takes `&mut` anything document-shaped — the
crate has no such type to take a `&mut` of in the first place. A caller
cannot get from a `SnippetPlan` to a mutated buffer without writing that
step itself; the absence of the capability is structural, not a warning in
a doc comment.

**Rev 1 bounds unchanged.** All six `limits.rs` constants, all `SnippetError`
variants, and their tests are untouched; `SnippetPlan::compute` propagates
any `SnippetError` from expansion as `PlanError::Snippet` (via `From`).

Crate name: `flashtex-editor-snippets` (lib name `flashtex_editor_snippets`),
no workspace root, own `Cargo.lock`, `edition = "2024"` — matching the
existing per-crate layout in this repo (e.g. `crates/bibliography`).

## What it does

Parses a snippet template with numbered placeholders (`$1`, `${1}`,
`${1:default text}`) and expands it into plain text plus exact byte-offset
spans for every placeholder occurrence. Occurrences of the same index are
*linked*: they always resolve to the same text. **This crate never mutates
a source buffer** — it only computes text + offsets; applying an edit to a
real document, and re-running expansion after that edit, is the caller's
job.

## Public contract (typed)

```rust
// crate: flashtex_editor_snippets

pub struct Snippet { /* opaque, parsed */ }
impl Snippet {
    pub fn parse(source: &str) -> Result<Snippet, SnippetError>;
    pub fn expand(&self) -> Result<Expansion, SnippetError>;
    pub fn expand_with(
        &self,
        overrides: &std::collections::HashMap<u32, String>,
    ) -> Result<Expansion, SnippetError>;
}

pub struct Expansion {
    pub text: String,
    pub placeholders: Vec<PlaceholderSpan>, // sorted by index
}
impl Expansion {
    pub fn occurrences_of(&self, index: u32) -> &[std::ops::Range<usize>];
    pub fn tab_order(&self) -> Vec<u32>; // ascending, $0 moved last
}

pub struct PlaceholderSpan {
    pub index: u32,
    pub occurrences: Vec<std::ops::Range<usize>>, // byte ranges, all char-boundary-safe
}

pub struct TabStops { /* opaque */ }
impl TabStops {
    pub fn new(expansion: &Expansion) -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn current(&self) -> Option<u32>;
    pub fn advance(&mut self) -> Option<u32>; // guarded: None + stays put at/before bounds
    pub fn retreat(&mut self) -> Option<u32>; // guarded: None + stays put at/before bounds
}

// --- revision 2: plans bound to document identity + Unicode anchors ---

pub struct ContentHash(/* opaque */); // FNV-1a 64 of the document's exact bytes
impl ContentHash {
    pub fn of(text: &str) -> ContentHash;
}

pub struct DocumentId {
    pub revision: u64,       // caller's own opaque revision id
    pub content_hash: ContentHash,
}
impl DocumentId {
    pub fn new(revision: u64, text: &str) -> DocumentId;
    pub fn compare(&self, current: &DocumentId) -> Staleness;
}

pub enum Staleness {
    Fresh,
    RevisionMismatch { planned: u64, current: u64 },
    ContentMismatch { planned: ContentHash, current: ContentHash },
}
impl Staleness {
    pub fn is_fresh(&self) -> bool;
}

pub enum Anchor {
    Caret(usize),
    Selection(std::ops::Range<usize>),
}
impl Anchor {
    pub fn range(&self) -> std::ops::Range<usize>;
}

pub enum PlanError {
    Snippet(SnippetError),
    InvalidOffset { offset: usize },          // not a char boundary, incl. past end
    SelectionReversed { start: usize, end: usize },
}
// impl std::error::Error + std::fmt::Display; impl From<SnippetError> for PlanError

pub struct SnippetPlan { /* opaque: DocumentId + Anchor + Expansion */ }
impl SnippetPlan {
    // No constructor or method here ever takes &mut anything document-shaped,
    // and there is no `apply`/`commit` method at all: applying a plan to a
    // real buffer is left entirely to the caller, enforced by this being the
    // whole surface, not by a comment.
    pub fn compute(
        snippet: &Snippet,
        revision: u64,
        document_text: &str,
        anchor: Anchor,
        overrides: &std::collections::HashMap<u32, String>,
    ) -> Result<SnippetPlan, PlanError>;
    pub fn document(&self) -> &DocumentId;
    pub fn anchor(&self) -> &Anchor;
    pub fn expansion(&self) -> &Expansion;
    pub fn staleness(&self, current: &DocumentId) -> Staleness;
}

#[non_exhaustive-in-spirit but currently exhaustive] // see note below
pub enum SnippetError {
    InputTooLarge { len: usize },
    UnterminatedEscape { offset: usize },
    InvalidEscape { offset: usize, found: char },
    UnterminatedPlaceholder { offset: usize },
    InvalidPlaceholderIndex { offset: usize },
    PlaceholderIndexTooLarge { offset: usize },
    NestingTooDeep { offset: usize },
    TooManyPlaceholders { count: usize },
    TooManyOccurrences { count: usize },
    SelfReferential { index: u32 },
    OutputTooLarge { len: usize },
}
// impl std::error::Error + std::fmt::Display

// crate::limits — the enforced bounds:
pub const MAX_INPUT_BYTES: usize = 64 * 1024;
pub const MAX_NESTING_DEPTH: usize = 16;
pub const MAX_PLACEHOLDER_INDEX: u32 = 9_999;
pub const MAX_PLACEHOLDERS: usize = 256;   // distinct indices
pub const MAX_OCCURRENCES: usize = 2_048;  // total placeholder occurrences
pub const MAX_OUTPUT_BYTES: usize = 1_000_000;
```

`SnippetError` is a plain `#[derive(Debug, Clone, PartialEq, Eq)]` enum, not
literally marked `#[non_exhaustive]` — a consumer adapter should still match
with a wildcard arm, since a future revision may add a variant.

## Semantics a consumer needs to know

- **First default wins.** If an index has more than one `${N:...}`
  occurrence, the value comes from the first one in document order; later
  `${N:...}` occurrences are mirrors like bare `$N` and their own default
  text is discarded (tested:
  `unused_self_referential_default_does_not_error_since_it_is_never_resolved`).
- **Linked edit = re-expand with an override.** There is no in-place
  mutation API. To reflect "the user edited this occurrence of `$1`", call
  `expand_with(&{1: new_text})` and use the freshly returned `Expansion`
  (new text, new spans for *every* occurrence of `1`, and other indices'
  spans generally shift too since `text` is fully rebuilt).
- **Self-reference is a runtime error, not a parse error.** `${1:$1}` and
  mutual cycles (`${1:$2}${2:$1}`) parse fine but fail at `.expand()` /
  `.expand_with()` with `SnippetError::SelfReferential`. An override for the
  cycling index bypasses the cycle entirely (overrides short-circuit before
  any default is resolved).
- **Tab order:** ascending by index, with `$0` (conventional "final cursor"
  stop) moved to the end.

## Bounds (requirement 5) and how they're proven

Stated bounds live in `src/limits.rs` (six `pub const`s, see contract
above). Each has a dedicated test in `tests/expansion.rs`:

- absurd nesting: `absurd_nesting_fails_with_a_typed_error_instead_of_looping`
  (one level past `MAX_NESTING_DEPTH` → `NestingTooDeep`) and
  `nesting_exactly_at_the_bound_succeeds` (right at the bound → ok).
- huge placeholder index: `huge_placeholder_index_is_a_typed_error_not_a_panic`
  (20-digit index) and `index_just_above_the_bound_is_rejected_and_just_below_is_accepted`.
- self-referential structure: `self_referential_placeholder_fails_with_a_typed_error`,
  `mutually_self_referential_placeholders_fail_with_a_typed_error`,
  `overriding_a_self_referential_index_bypasses_the_cycle`.
- too many distinct placeholders / too many occurrences / oversized source:
  `too_many_distinct_placeholders_is_a_typed_error`,
  `too_many_occurrences_is_a_typed_error`, `oversized_source_is_rejected_before_parsing`.

Mechanically, nesting depth is checked *before* recursing (in the parser,
right when a `:` opens a new default), so the call stack itself never grows
past the bound — this is a real recursion-depth cap, not just a
post-hoc count. Self-reference is caught with a "currently resolving"
index stack during value resolution: encountering an index already on that
stack is a cycle, whether direct or transitive through another placeholder,
so it terminates instead of recursing forever.

## UTF-8 safety (requirement 4) and how it's proven

The parser (`src/parser.rs`) never slices at an arbitrary byte offset. Its
cursor only ever advances byte-by-byte over non-structural bytes, or by
exactly one byte across a matched ASCII structural character (`$ \ { } :`).
Since UTF-8 continuation bytes (`0x80..=0xBF`) can never equal an ASCII
byte, none of those five structural characters can ever occur inside a
multi-byte character — so every point where the scanner stops to slice,
look ahead, or report an error offset is guaranteed to be a `char`
boundary, for any valid UTF-8 input, with no runtime
`is_char_boundary` checks needed anywhere. Expansion then only ever builds
the output by `String::push_str`-ing whole valid substrings (parsed text
runs, or fully-resolved placeholder values) and reads `text.len()`
before/after each push for span boundaries — again never slicing
mid-string — so offsets in the `Expansion` inherit the same guarantee.

Tested in `tests/expansion.rs`:
- `accented_latin_text_around_a_placeholder_has_exact_valid_offsets` (café/crème, 2-byte chars),
- `cjk_text_placeholder_offsets_never_split_a_character` (3-byte chars, offsets asserted exactly: 6 and 12),
- `emoji_including_multi_codepoint_sequence_never_splits_and_never_panics` (a ZWJ family emoji sequence, 4-byte + 3-byte codepoints),
- `escaping_a_dollar_immediately_after_multibyte_text_keeps_boundaries_valid`,
- `every_offset_over_a_battery_of_unicode_snippets_is_a_valid_char_boundary` — a battery of 10 mixed ASCII/Latin/CJK/emoji/Devanagari/combining-accent snippets, asserting `str::is_char_boundary` on every reported span and that slicing at it never panics.

No `unsafe` is used anywhere (`#![forbid(unsafe_code)]` at the crate root).

## What's not done / deliberately out of scope

- No editor/LSP integration adapter — this crate is intentionally
  standalone per the assignment; a consumer contract should be coordinated
  before any integration lands.
- No `$VARIABLE`-style named placeholders, transform expressions
  (`${1/regex/repl/}`), or choice placeholders (`${1|a,b,c|}`) from the
  fuller LSP snippet grammar — only numbered placeholders with plain-text
  defaults, per the assignment's stated scope.
- `SnippetError` is not literally `#[non_exhaustive]`; noted above as a
  forward-compat caveat for any adapter.

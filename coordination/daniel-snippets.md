# FT-041: editor-snippets

Status: complete, standalone additive crate. No other crate was touched.

## Tested commit

```
31076bf84bf64d0c22339c3b57f00a58b9a42f78
```

On branch `agent/daniel-snippets/editor-snippets`. At this exact SHA, inside
`crates/editor-snippets`:

- `cargo build` — clean.
- `cargo test` — 41 integration tests + 1 doc-test, all passing.
- `cargo clippy --all-targets -- -D warnings` — clean, zero warnings.
- `cargo fmt --check` — clean.

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

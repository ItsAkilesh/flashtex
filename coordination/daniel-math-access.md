# daniel-math-access handoff — FT-038 rev 1

- Agent / task / branch: `daniel-math-access` / FT-038 "original semantic math
  accessibility adapter over published math-layout nodes" /
  `agent/daniel-math-access/math-accessibility`
- State: ready for integration — acceptance criteria met, standalone additive
  crate, no consumer wiring yet (none was in scope).
- Owned paths: `crates/math-accessibility/**`,
  `coordination/daniel-math-access.md`. No other path was read-write; no
  edits were made to `crates/math-layout` or any other crate.
- Exact tested commit SHA: `c108171c8b8931333cd68d64efc504f1a964d03b` on
  `agent/daniel-math-access/math-accessibility`. `cargo build`, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo fmt -- --check` were
  all run against the working tree at this exact commit (clean, nothing
  further staged) inside `crates/math-accessibility`.
- Base verified against: `math-layout` was read at the checked-out tree, not
  assumed. Its current `Nucleus::Radical` is `Radical(MathList)` — a plain
  tuple variant, no degree field — confirmed by reading
  `crates/math-layout/src/mathlist.rs` directly (matches the tip of `main`
  and this task's `input_main_sha` `53fee3012b2902ca05bd31766defa515b3044cec`).
  A struct-variant `Radical { radicand, degree }` does exist in this
  repository's history, but only on unrelated/abandoned commits (a cancelled
  `mac-math-layout/math-boxes` lane and `mml-rev4`) that are not ancestors of
  `main` or of this branch — built against what is actually in the tree.

## Public typed contract

Crate `flashtex-math-accessibility` (lib `flashtex_math_accessibility`),
one dependency: `flashtex-math-layout` (path `../math-layout`).

```rust
pub const DEFAULT_MAX_DEPTH: usize = 64;

pub struct MathAccessibility { /* ... */ }
impl MathAccessibility {
    pub fn new() -> Self;                              // DEFAULT_MAX_DEPTH
    pub fn with_max_depth(max_depth: usize) -> Self;
    pub fn describe(&self, list: &MathList) -> Result<Description, AccessibilityError>;
}
impl Default for MathAccessibility { /* -> Self::new() */ }

pub struct Description {
    pub readable: String,             // structured, screen-reader-style text
    pub mathml: String,               // standalone <math ...>...</math>, XML-escaped
    pub unsupported: Vec<Unsupported>,// every node with no semantic name, in order
}
impl Description {
    pub fn is_fully_supported(&self) -> bool; // unsupported.is_empty()
}

pub struct Unsupported { pub path: NodePath, pub reason: UnsupportedReason }
pub type NodePath = Vec<PathStep>;
pub enum PathStep {
    Atom(usize), Group, Superscript, Subscript, Numerator, Denominator,
    Radicand, AccentBase, AccentGlyph, DelimitedBody, LeftDelimiter, RightDelimiter,
}
pub enum UnsupportedReason {
    UnknownSymbolName(char), UnknownAccentName(char), ControlCharacter(char),
}

pub enum AccessibilityError {
    RecursionLimitExceeded { max_depth: usize, path: NodePath },
}
// impl std::error::Error + Display for AccessibilityError
```

Consumes `flashtex_math_layout::{Atom, MathList, Nucleus}` (the semantic math
list, not the laid-out box tree) exactly as published: `Nucleus::{Symbol,
List, Fraction{numerator,denominator,thickness}, Radical(MathList),
Accent{accent,base}, Delimited{left,right,body}, Empty}`, `Atom{class,
nucleus, superscript, subscript, limits}`.

## Behavior

- Every one of the six `Nucleus` variants is handled; none are silently
  skipped. "Unsupported" applies only to individual **symbol/accent
  characters** with no entry in the built-in spoken-name tables (common
  operators/relations/delimiters/Greek letters). Those never get a guessed
  spoken word — they render as an explicit `[unsupported symbol U+XXXX]` /
  `[unsupported accent U+XXXX]` marker in `readable` and are recorded in
  `Description.unsupported` with the exact `NodePath` and character.
- MathML always preserves the literal source glyph (XML-escaped: `&`, `<`,
  `>`), even for characters with no spoken name — visual identity is never
  approximated, only the spoken name is ever withheld. The one exception is
  control characters, which are not legal literal XML text at all; those
  become an explicit `<merror><mtext>unsupported control character
  U+XXXX</mtext></merror>` in MathML (and the analogous bracketed marker in
  `readable`), instead of emitting an invalid raw byte.
- Fraction rule thickness (`Some(t)`) is carried verbatim into MathML as
  `linethickness="{t}pt"` rather than collapsed into a thin/thick guess.
- Recursion (groups, fraction numerator/denominator, radicands, accent
  bases, delimited bodies, sub/superscripts) is bounded by `max_depth`
  (64 by default). Exceeding it returns
  `Err(AccessibilityError::RecursionLimitExceeded)` before recursing further —
  verified with a 10,000-level nested-group fixture (built iteratively, and
  leaked with `mem::forget` after the assertion since dropping that fixture
  recursively — a math-layout data-structure property outside this crate's
  ownership — would itself overflow the stack; the traversal under test
  never gets near that depth).

## Validation

`cargo test` in `crates/math-accessibility`: 17 passed. Includes exact
hand-worked `readable` + `mathml` string assertions (not just non-empty
checks) for: an ordinary relation (`x + y = z`), `\frac{1}{2}`, `\sqrt{2}`,
`\hat{x}`, `\left(x\right)`, a null-delimiter case, `x^2`, `x_i^2` (asserting
MathML's `msubsup` base/sub/sup child order), a known Unicode Greek letter
(`\pi`), an unknown Unicode symbol (U+1F600, asserting the explicit marker
and the `Unsupported` entry, not a guess), a control character (U+0007,
asserting `<merror>` and no raw control byte anywhere in the MathML string),
an XML-special + non-ASCII round-trip (`<`, `&`, `é`, decoded back and
compared to the source characters), the depth-bound failure, and an
empty-list/empty-nucleus case.

`cargo clippy --all-targets -- -D warnings`: clean, zero warnings.
`cargo fmt -- --check`: clean.

## Incomplete / out of scope

- No OpenType MathML `intent`/semantics annotations, no matrices/arrays
  (`Nucleus` doesn't have them yet either).
- Spoken-name coverage is common operators, relations, delimiters, and the
  Greek alphabet; anything else is reported through `unsupported` rather than
  silently passed through or guessed.
- No consumer integration performed (none was requested); this is an
  additive, standalone crate only.

## Needs from others

None. No dependency on any in-flight lane; `flashtex-math-layout` was
consumed read-only as published on `main`.

# daniel-math-access handoff — FT-038 rev 2

- Agent / task / branch: `daniel-math-access` / FT-038 "expand typed
  structured math accessibility with exact source-node identity, bounded
  tree traversal and explicit unsupported nodes; no guessed speech" /
  `agent/daniel-math-access/math-accessibility`.
- State: ready for integration — acceptance criteria met, standalone
  additive crate, no consumer wiring yet (none was in scope).
- Owned paths: `crates/math-accessibility/**`,
  `coordination/daniel-math-access.md`,
  `coordination/agents/daniel-math-access.json`. No other path was
  read-write; no edits were made to `crates/math-layout` or any other crate.
- Exact tested commit SHA: `0b2d8e74a7d557f9419b1d51aa247568784dc843` on
  `agent/daniel-math-access/math-accessibility`. `cargo build`, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo fmt -- --check`
  were all run against the working tree at this exact commit (clean, nothing
  further staged) inside `crates/math-accessibility`.
- `main` integrated through `284369de3fd2af4384c3de2d2403801dffbcf94b`
  (merged into this branch with `git merge origin/main --no-edit`, no
  conflicts — nothing else on `main` touches this crate).
- Base re-verified against the CURRENT tree, not assumed or carried over
  from rev 1: read `crates/math-layout/src/mathlist.rs` directly at this
  branch's merged-in `main`. Two things had changed since rev 1's
  verification: `Nucleus::Radical` is now the struct variant
  `Radical { radicand: MathList, degree: Option<MathList> }` (rev 1 built
  against the plain tuple variant `Radical(MathList)`, which no longer
  exists), and four new variants were added: `Text(String)`,
  `Overline(MathList)`, `Underline(MathList)`, and
  `Styled { style: Style, body: MathList }`. Rev 1's code would not compile
  against the current tree; this revision handles all eleven current
  `Nucleus` variants.

## Public typed contract

Crate `flashtex-math-accessibility` (lib `flashtex_math_accessibility`),
one dependency: `flashtex-math-layout` (path `../math-layout`).

```rust
pub const DEFAULT_MAX_DEPTH: usize = 64;
pub const DEFAULT_MAX_NODES: usize = 100_000;

pub struct MathAccessibility { /* ... */ }
impl MathAccessibility {
    pub fn new() -> Self;                                    // both defaults
    pub fn with_max_depth(max_depth: usize) -> Self;          // default max_nodes
    pub fn with_max_nodes(max_nodes: usize) -> Self;          // default max_depth
    pub fn with_bounds(max_depth: usize, max_nodes: usize) -> Self;
    pub fn describe(&self, list: &MathList) -> Result<Description, AccessibilityError>;
}
impl Default for MathAccessibility { /* -> Self::new() */ }

pub struct Description {
    pub readable: String,
    pub mathml: String,
    pub nodes: Vec<DescribedNode>,     // NEW rev2: every emitted node, with its exact identity
    pub unsupported: Vec<Unsupported>,
}
impl Description {
    pub fn is_fully_supported(&self) -> bool;
}

pub struct DescribedNode { pub id: NodeId, pub readable: String } // NEW rev2

/// Exact, stable identity of a source node — built only from structural
/// position (atom index / child slot at each level), never memory
/// addresses or run-to-run incidentals. Same input, rebuilt from scratch,
/// always yields the same ids.
pub struct NodeId(/* private */);                                 // NEW rev2
impl NodeId {
    pub fn path(&self) -> &[PathStep];
}
impl std::fmt::Display for NodeId { /* e.g. "atom0/numerator/atom0" */ }

pub struct Unsupported { pub id: NodeId, pub reason: UnsupportedReason } // id was `path: NodePath` in rev1
pub type NodePath = Vec<PathStep>;
pub enum PathStep {
    Atom(usize), Group, Superscript, Subscript, Numerator, Denominator,
    Radicand, Degree, AccentBase, AccentGlyph, DelimitedBody, LeftDelimiter,
    RightDelimiter, OverlineBody, UnderlineBody, StyledBody,     // last 4 + Degree are NEW rev2
}
pub enum UnsupportedReason {
    UnknownSymbolName(char), UnknownAccentName(char), ControlCharacter(char),
}

pub enum AccessibilityError {
    RecursionLimitExceeded { max_depth: usize, id: NodeId },
    NodeCountExceeded { max_nodes: usize, id: NodeId },          // NEW rev2
}
// impl std::error::Error + Display for AccessibilityError
```

Consumes `flashtex_math_layout::{Atom, MathList, Nucleus, StyleLevel}`
exactly as published on the merged-in `main`.

## Behavior (rev2 additions on top of rev1)

- **Exact source-node identity**: every `DescribedNode` and every
  `Unsupported` carries a `NodeId` built purely from the node's structural
  coordinate in the input (which atom index at each nesting level, which
  named child slot — numerator/denominator/radicand/degree/accent-base/
  accent-glyph/delimited-body/left-delimiter/right-delimiter/overline-body/
  underline-body/styled-body/superscript/subscript/group). It has a stable
  `Display` (e.g. `atom0/numerator/atom0`) usable as a map key. Proven
  rebuild-stable by `identity_is_stable_across_rebuild`: two independently
  constructed (different `Vec`/`String` allocations, asserted via pointer
  inequality), structurally identical trees produce identical `nodes`,
  identical `unsupported`, and identical `readable`/`mathml` output.
- **Bounded tree traversal, two ways**: nesting depth (`max_depth`,
  `RecursionLimitExceeded`, unchanged from rev1) bounds a deep-but-narrow
  tree; total node count (`max_nodes`, new `NodeCountExceeded`) bounds a
  wide-but-shallow tree that a depth bound alone cannot catch — a flat list
  of 200 atoms stays at depth 1 throughout but still trips a 100-node bound
  at exactly the 101st atom (`wide_shallow_list_fails_on_node_count_not_depth`,
  which also asserts the exact `NodeId` of the atom that tripped it).
- **Explicit unsupported nodes, no guessed speech**: unchanged rule from
  rev1 — an unknown symbol or accent character never gets an invented
  spoken word; it becomes an explicit `[unsupported symbol/accent U+XXXX]`
  marker in `readable`, the literal glyph preserved in `mathml`, and a typed
  `Unsupported` entry. `Nucleus::Text` (`\lim`, `\sin`, …) is spoken exactly
  as given — that's not a guess, it's already a name, not a symbol we'd
  have to invent a name for. All eleven `Nucleus` variants are structurally
  handled; "unsupported" only ever applies to individual unnamed
  symbol/accent characters, never to a whole node kind.
- New constructs: `\sqrt[degree]{radicand}` → `<mroot>` (radicand, then
  index, per MathML's required child order) and "start root, index
  {degree}, {radicand}, end root"; `\overline`/`\underline` →
  `<mover>`/`<munder>` with the literal bar/underscore glyph; `{\displaystyle
  ...}` (`Nucleus::Styled`) → `<mstyle displaystyle="true|false">`, the
  source `StyleLevel` carried verbatim into the attribute rather than
  dropped, body otherwise read transparently.
- XML escaping and control-character handling are unchanged from rev1:
  `&`/`<`/`>` escaped, non-ASCII glyphs kept literal, control characters
  degrade to `<merror><mtext>...</mtext></merror>` instead of a raw byte.

## Validation

`cargo test` in `crates/math-accessibility`: **25 passed** (up from 17 in
rev1; 8 new tests cover degree-bearing roots, `Text`/`Overline`/`Underline`/
`Styled`, the node-count bound both tripped and within-bound, rebuild
identity stability, and `NodeId::Display`'s exact string form).

`cargo clippy --all-targets -- -D warnings`: clean, zero warnings.
`cargo fmt -- --check`: clean.

## Incomplete / out of scope

- No OpenType MathML `intent`/semantics annotations, no matrices/arrays
  (`Nucleus` doesn't have them).
- Spoken-name coverage is unchanged: common operators, relations,
  delimiters, and the Greek alphabet; anything else goes through
  `unsupported` rather than being silently passed through or guessed.
- No consumer integration performed (none was requested); this is an
  additive, standalone crate only.

## Needs from others

None. No dependency on any in-flight lane; `flashtex-math-layout` was
consumed read-only as published on `main`.

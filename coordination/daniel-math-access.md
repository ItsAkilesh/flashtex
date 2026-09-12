# daniel-math-access handoff — FT-038 rev 3

- Agent / task / branch: `daniel-math-access` / FT-038 "accessible
  structured math: bounded adversarial and stale-identity acceptance
  tests" / `agent/daniel-math-access/math-accessibility`.
- State: ready for integration — rev 3 objective met, test-only revision
  (no public API or behavior change), standalone additive crate, no
  consumer wiring yet (none was in scope).
- Owned paths: `crates/math-accessibility/**`,
  `coordination/daniel-math-access.md`,
  `coordination/agents/daniel-math-access.json`. No other path was
  read-write; no edits were made to `crates/math-layout` or any other crate.
- Exact tested commit SHA: `900403b0db15c567791d71468a2b3074963a4f66` on
  `agent/daniel-math-access/math-accessibility`. `cargo build`, `cargo test`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo fmt -- --check`
  were all run against the working tree at this exact commit (clean, nothing
  further staged) inside `crates/math-accessibility`.
- `main` integrated through `967703ebb4e8140feaf4db02d27cb3ac63c573f6`
  (merged into this branch with `git merge origin/main --no-edit`, no
  conflicts — nothing else on `main` touches this crate).
- Base re-verified against the CURRENT tree before designing, as in rev 1
  and rev 2: read `crates/math-layout/src/mathlist.rs` directly at this
  branch's merged-in `main`. Nothing has changed since rev 2's
  verification — all eleven `Nucleus` variants, `NodeId`/`PathStep`, and
  the bound types are exactly as rev 2 recorded them, so this revision is
  additions to the existing test module only, no source changes to
  `src/lib.rs`'s implementation.

## Public typed contract

Unchanged from rev 2 — no API surface was touched this revision. See rev
2's contract listing (still accurate): `MathAccessibility`, `Description`,
`DescribedNode`, `NodeId`/`PathStep`, `Unsupported`/`UnsupportedReason`,
`AccessibilityError`, consuming
`flashtex_math_layout::{Atom, MathList, Nucleus, StyleLevel}` exactly as
published on the merged-in `main`.

## Rev 3: bounded adversarial acceptance suite

Every case attacks `describe` with a malformed, oversized, or hostile
input and asserts the *outcome type*: a typed `AccessibilityError` at an
exact, predetermined boundary, or a `Description` whose `mathml` is
provably well-formed. None panic, hang, or overflow the stack.

- **Depth and node-count bounds, exactly at and one past the limit**:
  `depth_exactly_at_bound_succeeds` / `depth_one_past_bound_fails` and
  `node_count_exactly_at_bound_succeeds` / `node_count_one_past_bound_fails`
  pin the precise boundary (an N-deep chain succeeds under a bound of N,
  fails under N-1; an N-atom list succeeds under a bound of N, fails
  under N-1), on top of rev 1/2's coarser-grained tests for the same two
  bounds.
- **A wide-and-deep tree that trips one bound before the other, both
  directions**: `wide_and_deep_tree_trips_depth_bound_before_node_count_bound`
  (40 cheap siblings plus one branch nested far past a depth budget of 5,
  against a generous node budget of 1000 — total nodes visited at failure
  is ~46, nowhere near 1000, so only the depth bound can be responsible)
  and `wide_and_deep_tree_trips_node_count_bound_before_depth_bound` (49
  siblings plus a branch nested 10 levels deep, against a generous depth
  budget of 1000 and a node budget of 30 — traversal order means the node
  bound fires during the wide prefix, at the exact asserted `NodeId`,
  before ever reaching the deep branch).
- **Hostile text**: `assorted_control_characters_degrade_to_merror_not_a_raw_byte`
  covers NUL, two more C0 controls, DEL, and a C1 control, all degrading
  to `<merror>`. `right_to_left_override_is_not_treated_as_a_control_character`
  proves U+202E (general category Cf, not Cc) is *not* caught by the
  control-character path and is instead reported as an explicit
  unsupported symbol with its glyph preserved — never silently dropped,
  never guessed. `unpaired_surrogate_code_points_cannot_be_constructed_as_a_char`
  proves the adversarial case the task named ("unpaired surrogate-like
  sequences") is excluded by Rust's type system itself
  (`char::from_u32` returns `None` for the whole `0xD800..=0xDFFF` range)
  and exercises the two legal values immediately adjacent to that gap.
  `assorted_unknown_symbols_are_reported_not_guessed` covers four symbols
  outside the name table (emoji, snowman, a private-use codepoint, the
  replacement character).
- **XML escaping cannot be broken out of** — the sharp case the task
  named: `assert_no_foreign_markup` is a byte-level scanner that walks
  emitted `mathml` and panics unless every literal `<` begins one of this
  crate's own sixteen known element tags (open or close). Since
  `escape_xml_text` rewrites every content `<` to `&lt;` before it can
  reach the output, a hostile payload can never introduce a foreign tag —
  this scanner is the proof, applied to every adversarial test's output.
  `hostile_text_with_full_tag_injection_cannot_break_out_of_mathml` drives
  nine classic injection payloads (early `</mtext>`/`</math>` close,
  foreign `<script>`/`<img>` tags, an HTML comment open and close, a CDATA
  open and close, an XML processing instruction, and a DOCTYPE with an
  external entity) through `Nucleus::Text`, asserting zero foreign markup
  and that decoding the emitted entities recovers the payload verbatim —
  nothing is silently dropped, only escaped.
  `control_character_mixed_into_hostile_text_still_degrades_safely`
  documents the interacting edge: a control character anywhere inside an
  upright-text nucleus degrades the *whole* node to one `<merror>` marker
  (existing `render_text` behavior, stricter than symbol-by-symbol
  escaping but still bounded, typed, and markup-safe).
- **An empty document, and a node whose children are all empty**:
  `empty_document_succeeds_even_with_zero_bounds` describes an empty list
  against `with_bounds(0, 0)` and succeeds (no atoms means the node-count
  check is never invoked, and depth 0 sits exactly at a bound of 0).
  `node_with_all_empty_children_does_not_panic_and_stays_well_formed`
  builds a fraction whose numerator *and* denominator are both empty
  lists.

## Rev 3: stale node-identity acceptance suite

Three tests, written to read as a specification of the identity contract
(`NodeId` is derived purely from structural position, never content or
memory address):

- `identity_spec_identical_structures_produce_identical_ids` — two
  independently built, structurally identical trees produce identical
  `NodeId`s for every emitted node (restates `identity_is_stable_across_rebuild`
  from rev 2 as a short, standalone acceptance case).
- `identity_spec_structurally_different_tree_produces_different_ids` — the
  same symbol `'a'`, once as a bare top-level atom (`atom0`) and once
  wrapped in a group (`atom0/group/atom0`), gets a different id: same
  content, different structural position, different identity.
- `identity_spec_reordering_siblings_changes_the_moved_nodes_ids` — a
  3-atom list `[a, b, c]` versus its rotation `[c, a, b]`: `'a'` moves from
  `atom0` to `atom1`, `'c'` moves from `atom2` to `atom0`, `'b'` moves from
  `atom1` to `atom2`. Identity tracks the occupant of a structural slot,
  not a fixed label attached to content.

## Validation

`cargo test` in `crates/math-accessibility`: **42 passed** (up from 25 in
rev 2; 17 new tests, all listed above — no existing test was modified).

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
- `render_text`'s whole-node degradation on any embedded control
  character (rather than per-character escaping, as `render_symbol` does)
  is existing rev 1/2 behavior, documented and tested this revision but
  not changed — it is safe (bounded, typed, markup-clean) even though it
  loses the surrounding text; changing it was out of this revision's
  test-only scope.

## Needs from others

None. No dependency on any in-flight lane; `flashtex-math-layout` was
consumed read-only as published on `main`, and re-verified unchanged
since rev 2.

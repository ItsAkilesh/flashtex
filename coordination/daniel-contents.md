# daniel-contents handoff

Agent / task / branch: daniel-contents (Claude Code subagent) / FT-034 rev 2:
bounded deterministic TOC page-reference stabilization with explicit
unresolved/cycle states and exact source identities /
`agent/daniel-contents/toc-layout`
State: ready for integration
Owned paths: `crates/toc-layout/**`, `coordination/daniel-contents.md`,
`coordination/agents/daniel-contents.json`
Exact tested commit SHA (rev 2 implementation): `754c7579baa37b304eca772fbb3dbbbdbd619cb9`
Main integrated through (merge-base with `origin/main` at this update):
`85a0b58bb152dda9ddf1c7c52b8b078bae37287f`
(rev 1's tested SHA was `da20d5cb4872029fd23af4defe64af9ea58bad77`, input_main_sha
`53fee3012b2902ca05bd31766defa515b3044cec`; this coordination-record commit
adds one more commit on top of the rev 2 implementation commit above,
touching only coordination files.)

## What this crate does

`flashtex-toc-layout` (`crates/toc-layout`) lays out
`\tableofcontents`/`\listoffigures`-style entries —
`<indent><title><leader dots><page number>` — from explicit section/page
records. No font, shaping, or TeX engine is linked; no other FlashTeX crate
is a dependency (`[dependencies]` is empty, matching `document-style`,
`paragraph-layout`, `math-layout`, `bibliography`, `project-index`).

Modules:
- `entry`: `EntryRecord` (explicit absolute page) and `RelativeEntry`
  (explicit body-page offset, resolved once front-matter page count is
  known). Both reject malformed input via `EntryError` (empty/whitespace
  title, page/offset zero, level beyond `MAX_LEVEL` = 5, page-counter
  overflow) rather than approximating.
- `measure`: the typed adapter contract, `TextMeasure` (`width(&str) ->
  f64`, `leader_unit_width() -> f64`). No consumer is wired up yet — this
  is the seam font-engine/compiler/native layout implement against their
  own glyph metrics. `CharWidthMeasure` ships as a documented,
  non-shaping reference implementation (per-`char` width table) for tests
  and callers without a font yet; it explicitly does *not* claim
  grapheme-cluster correctness (see doc comment).
- `leader`: `LineBox` (validated line geometry) and `layout_entry`/
  `layout_entries`, which compute a real measured leader-dot count from
  `TextMeasure` so the page number lands flush at the line's right edge.
  Returns typed `LayoutError::Overflow` (title+page alone don't fit) or
  `LayoutError::InvalidMeasurement` (a measurer returned a negative/
  non-finite width) rather than clamping or silently truncating.
- `converge`: `FrontMatterModel` (typed adapter contract for one
  pagination pass: candidate front-matter page count -> pages actually
  needed) and `converge_front_matter_pages`, a bounded fixed-point solver
  for the classic TOC circularity (an entry's absolute page depends on
  how many pages the contents/figures list itself occupies). Bounded by
  `MAX_CONVERGENCE_ITERATIONS = 8`. **Rev 2:** `ConvergenceError` is now an
  enum distinguishing two non-convergence shapes instead of one generic
  failure: `Cycle { bound, history, cycle_start, cycle }` when a candidate
  provably recurs (an oscillation, found by scanning the completed history
  for the earliest repeated value — sound because `pages_for` is a pure
  function of its candidate, so a repeat proves the tail loops forever),
  vs. `Unresolved { bound, history }` when every one of the `bound + 1`
  candidates tried was distinct. Both still carry the full bound+history
  from rev 1; the solver still always runs the full 8 passes (never exits
  early on a detected repeat) so the result is independent of how a caller
  might want to short-circuit.
- `stabilize` (new in rev 2): whole-list page-reference stabilization.
  `SourceId { document, revision }` exactly identifies which document
  revision an entry came from. `SourcedEntry` pairs a `RelativeEntry` with
  a `SourceId` and an explicit `sequence` (position in the list,
  independent of whatever order the caller's own storage iterates in).
  `stabilize_toc(entries, model, initial_guess)` converges the shared
  front-matter fixed point once (via `converge_front_matter_pages`) and
  resolves every entry against it, returning a `StabilizedToc` whose
  entries are sorted by `sequence` — so the result never depends on the
  order `entries` was passed in, only on its content. Every
  `StabilizedEntry` carries the exact `SourceId` it was computed against;
  `StabilizedEntry::is_stale(&current_source)` detects when a stabilized
  result no longer matches the current document revision.
  `StabilizationError` wraps `ConvergenceError` (so callers can still
  distinguish cycle vs. unresolved) plus a typed `EntryResolution` variant
  for a per-entry resolution failure (e.g. page-counter overflow).

## Typed adapter contract (for whoever wires this to real text/pagination)

```rust
pub trait TextMeasure {
    fn width(&self, text: &str) -> f64;       // one run, no line breaks
    fn leader_unit_width(&self) -> f64;       // one dot-leader unit
}

pub trait FrontMatterModel {
    fn pages_for(&self, candidate_front_matter_pages: u32) -> u32; // pure fn of candidate
}
```

A real integration implements `TextMeasure` against font-engine glyph
metrics (by extended grapheme cluster, not `char` or byte — `toc-layout`'s
own `CharWidthMeasure` measures by `char` as a documented simplification)
and implements `FrontMatterModel` against the compiler's actual page
breaker. No such wiring exists yet; this crate is standalone and additive
by design, per FT-034's scope.

## Validation

`cd crates/toc-layout && cargo build && cargo test`: 30 integration tests
+ 1 doctest pass, 0 unit tests (all behavior is exercised through the
public API in `tests/toc.rs`). `cargo clippy --all-targets -- -D
warnings`: 0 warnings. `cargo fmt --check`: clean.

Tests cover, with real asserted arithmetic (not just `is_ok()`):
- malformed input: empty/whitespace title, page/offset zero, level over
  `MAX_LEVEL`, non-finite/non-positive `LineBox` geometry, a title+page
  that overflows the line, a zero leader-unit width, a measurer that
  returns a negative width, page-counter overflow on `RelativeEntry`
  resolution;
- Unicode input: accented Latin (`résumé`, asserting char-count 6 vs.
  byte-length 8), CJK with per-character width overrides, a
  multi-codepoint emoji and a combining-mark sequence (asserting the
  documented per-`char` counting, not a grapheme-cluster claim), and
  right-to-left Arabic text laid out as opaque text;
- leader arithmetic: a hand-computed leader count and remainder gap, an
  exact-line-width sum invariant (`indent + title + leaders + gap + page
  label == line width`) under non-round measurements and non-zero
  indent, and indent scaling by nesting level;
- convergence: immediate convergence on a correct first guess, a
  multi-pass convergence from a bad guess, a full pipeline test
  (`RelativeEntry` -> converge -> `layout_entry`), plus **(rev 2)**
  `convergence_classifies_oscillation_as_a_cycle_not_generic_non_convergence`
  (still asserts exactly `MAX_CONVERGENCE_ITERATIONS + 1 == 9` history
  entries — the rev 1 bound is unchanged — now also asserting
  `ConvergenceError::Cycle { cycle_start: 0, cycle: vec![2, 5], .. }`) and
  `convergence_classifies_non_repeating_growth_as_unresolved_not_a_cycle`
  (an ever-increasing model, proving `Unresolved` is reachable and
  distinct from `Cycle`), plus an accessor-agreement test;
- **(rev 2)** whole-list stabilization: `stabilize_toc` resolving several
  entries against one converged front-matter count;
  `StabilizationError::Convergence` surfacing both `Cycle` and
  `Unresolved` from an underlying failing model; `StabilizedEntry::is_stale`
  detecting a different revision *and* a different document as stale but
  not the identical source; `stabilize_toc_is_deterministic_independent_of_input_order`
  feeding the same three entries through a plain `Vec`, a `Vec` drained
  from a `HashMap` (unspecified iteration order), and a reversed `Vec`,
  and asserting all three produce byte-identical `StabilizedToc` values
  ordered by `sequence`; and a full source-stamped pipeline test
  (`SourcedEntry` -> `stabilize_toc` -> `layout_entry`).

## Incomplete / out of scope

- No consumer wiring: font-engine/compiler/native layout do not implement
  `TextMeasure`/`FrontMatterModel` yet, by design (standalone additive
  crate; FT-034 says "coordinate consumer contract before integration").
- `CharWidthMeasure` measures by `char`, not by extended grapheme
  cluster; a real font-backed `TextMeasure` should measure by grapheme
  cluster for correctness with combining marks and multi-codepoint
  emoji — documented as a limitation, not fixed here.
- No line-wrapping for titles that don't fit even at zero leader dots
  (`LayoutError::Overflow`); wrapping a long TOC entry across multiple
  lines is not implemented.
- No roman-numeral / alternate page-number formatting for front matter
  itself (only Arabic `to_string()` labels).
- `FrontMatterModel`/`converge_front_matter_pages`/`stabilize_toc` still
  model one convergence variable (front-matter page count as a scalar);
  no worked example wiring it to a real page-breaking function is
  included, since no peer crate was touched or depended on for this task.
- `stabilize_toc` orders entries by an explicit `sequence: u32` the caller
  supplies; it does not compute or validate that `sequence` matches
  document order itself (e.g. no duplicate-sequence detection) — the
  caller's extraction step owns assigning it correctly.
- `StabilizationError::EntryResolution` reports the first entry (by
  sorted `sequence`) that fails to resolve, not every failing entry; this
  matches `layout_entries`' existing fail-fast-on-first-error style in
  this crate rather than accumulating a multi-error report.

## Needs from others

- The peer crate that owns real text measurement (font-engine) and real
  pagination (compiler/native layout) to implement `TextMeasure` and
  `FrontMatterModel` respectively, and a decision on where that adapter
  code should live (this crate stays free of the dependency; the impls
  likely belong in the consumer crate or a thin bridge crate).
- Confirmation of the acceptable `MAX_CONVERGENCE_ITERATIONS` bound (8)
  for real documents; raise if a real front-matter model is found to
  legitimately need more passes.
- A decision on whether `SourceId.revision` should be a caller-opaque
  string (current choice) or a typed hash/version the compiler already
  produces elsewhere in the workspace, once a consumer wires this crate
  up for real.

Resource: allocation `daniel-claude20x-shared`.
Updated: 2026-09-12 (rev 2)

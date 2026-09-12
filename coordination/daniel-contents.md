# daniel-contents handoff

Agent / task / branch: daniel-contents (Claude Code subagent) / FT-034:
bounded contents/list-of-figures entry layout / `agent/daniel-contents/toc-layout`
State: ready for integration
Owned paths: `crates/toc-layout/**`, `coordination/daniel-contents.md`
Exact tested commit SHA: `da20d5cb4872029fd23af4defe64af9ea58bad77`
(input_main_sha for FT-034 was `53fee3012b2902ca05bd31766defa515b3044cec`;
this branch adds one commit on top, touching only `crates/toc-layout/**`.)

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
  `MAX_CONVERGENCE_ITERATIONS = 8`; returns typed `ConvergenceError`
  (bound + full guess history) rather than looping unbounded or accepting
  an unstable result. Never claims convergence it hasn't verified.

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

`cd crates/toc-layout && cargo build && cargo test`: 23 integration tests
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
  multi-pass convergence from a bad guess, a typed `ConvergenceError`
  with the exact bounded history length when a model oscillates forever,
  and a full pipeline test (`RelativeEntry` -> converge ->
  `layout_entry`).

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
- `FrontMatterModel`/`converge_front_matter_pages` model one convergence
  variable (front-matter page count as a scalar); no worked example
  wiring it to a real page-breaking function is included, since no peer
  crate was touched or depended on for this task.

## Needs from others

- The peer crate that owns real text measurement (font-engine) and real
  pagination (compiler/native layout) to implement `TextMeasure` and
  `FrontMatterModel` respectively, and a decision on where that adapter
  code should live (this crate stays free of the dependency; the impls
  likely belong in the consumer crate or a thin bridge crate).
- Confirmation of the acceptable `MAX_CONVERGENCE_ITERATIONS` bound (8)
  for real documents; raise if a real front-matter model is found to
  legitimately need more passes.

Resource: allocation `daniel-claude20x-shared`.
Updated: 2026-09-12

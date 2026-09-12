# daniel-tables (FT-030) — crates/paragraph-layout

Agent name is retained from prior slot bookkeeping; the owned path for this
task is `crates/paragraph-layout` exclusively (per `owned_paths` in
`coordination/assignments/FT-030.json`), not tables.

## Inspected: bad0666 (jay3332, "NOT integration-ready" WIP)

`git show bad0666 -- crates/paragraph-layout` touched:
`Cargo.toml`, `src/items.rs`, `src/metrics.rs`, `tests/fixtures/lm_tfm.rs`
(1461 lines), `tests/oracle_lm_cm_default.rs` (681 lines),
`tools/gen_tfm_fixture.py` (168 lines).

### Salvaged

`src/metrics.rs` / `src/items.rs`: the idea of giving `FontMetricsSource` a
`glyph_height`/`glyph_depth` pair (TFM `charht`/`chardp` semantics) and
having `shape_run` take a run's height/depth as the max over its glyphs
instead of a flat ascender/descender. This was sound and worth keeping:

- Both new trait methods have default bodies (`ascender()` /
  `-descender()`), so every existing `FontMetricsSource` impl in the repo
  (`Core14Times`, `font-engine`'s `adapters::paragraph::FaceMetrics`) is
  unaffected — for a source with no override, max-over-glyphs of a constant
  is that same constant. Verified this explicitly with
  `shape_run_height_depth_default_to_ascender_descender`.
- `Line`/`GlyphRun` height/depth already flow into real behaviour
  (`\baselineskip`/`\lineskiplimit` placement in `linebreak.rs`, page
  breaking in `pages.rs`), so this is a genuine capability add, not
  decoration.

I reimplemented these two hunks by hand rather than cherry-picking (the WIP
commit doesn't compile standalone; see below), but the resulting diff is
essentially identical to the WIP's version of these two files.

### Rejected — TFM fixture, oracle test, generator tool, font-engine dev-dep

Rejected `tests/fixtures/lm_tfm.rs`, `tests/oracle_lm_cm_default.rs`,
`tools/gen_tfm_fixture.py`, and the `Cargo.toml` dev-dependency on
`flashtex-font-engine` that only they needed. Reasons:

1. **Wrong test shape for this crate.** This crate's own convention
   (`tests/oracle_wrap_sample.rs`) is a fully self-contained oracle test:
   hand-transcribed AFM/TFM constants plus a local `FontMetricsSource` impl,
   no external files read at test time, no dependency on another crate. The
   WIP oracle test instead reads real Latin Modern OTF files off disk at
   runtime (path not committed, "skips when fonts are not on the machine")
   and adds a real dependency from this dependency-free crate
   (`description = "... No external crates."` in `Cargo.toml`) onto
   `flashtex-font-engine` just to exercise one test.
2. **References work not in this tree.** Its module comment says the
   `FaceMetrics`-based adapter it duplicates a "local copy" of was published
   on `origin/agent/mac-font-engine/tex-fonts` (f418238), "not yet on main,
   so not stacked on" — i.e. it was written against a branch state I don't
   have and am not permitted to pull in (`crates/font-engine` is not in my
   `owned_paths`). Whether the real `adapters::paragraph::FaceMetrics` now on
   this branch's main matches what the test assumes is untested.
   Cross-checked separately: it does look compatible in a quick read, but I
   am not going to stake a merge on that read.
3. **1461 lines of unverified numeric transcription.** `lm_tfm.rs` claims to
   be TFM data transcribed from real Latin Modern `.tfm` files via the
   Python generator. I have no way to verify that transcription in this
   timebox, and the commit itself says "Validation: none run." A wrong
   constant in a file this size fails silently (a test assertion off by a
   fraction of a point), which is a bad property for checked-in "oracle"
   data.
4. **Disproportionate to the timebox and to what's provably sound.** This
   is really a cross-agent visual/native-oracle validation effort (its own
   header cites `agent/mac-validation/native-verification` reports), not a
   paragraph-layout unit test. It doesn't belong bundled into "add
   glyph_height/glyph_depth."

None of the rejected files were copied into the tree; `git status` in
`crates/paragraph-layout` shows only `src/items.rs`, `src/metrics.rs`,
`tests/golden.rs` modified — no new files.

## Implemented (this session)

- `src/metrics.rs`: `FontMetricsSource::glyph_height`/`glyph_depth`, default
  bodies delegate to `ascender()`/`-descender()`. Purely additive (default
  trait methods) — no existing impl needs to change.
- `src/items.rs`: `shape_run` now computes `GlyphRun::height`/`depth` as the
  max of `glyph_height`/`glyph_depth` over the run's glyphs, instead of the
  font's ascender/descender unconditionally. Doc comments on `GlyphRun`
  updated to match. `GlyphRun::from_shaped` (the font-engine glyph-run route,
  fed by `ShapedGlyph` which carries no per-glyph box) is untouched —
  extending that path would mean changing `ShapedGlyph`'s shape, which is a
  bigger, cross-crate API change out of scope here.
- Tests added (all new, all with hand-derived expected numbers, none are
  placeholder assertions):
  - `items::tests::shape_run_height_depth_default_to_ascender_descender` —
    regression test that `Core14Times` (no override) is bit-for-bit
    unchanged.
  - `items::tests::shape_run_height_depth_track_the_tallest_deepest_glyph` —
    a local `BoxFont` overrides height for `'b'` and depth for `'y'`;
    asserts the run picks up the max over "aby" and reverts to defaults for
    "aa".
  - `tests/golden.rs::glyph_height_and_depth_change_baseline_placement` —
    end-to-end: `BoxTestFont` (same shape as the file's existing `TestFont`)
    makes `'T'` tall and `'y'` deep; lays out "Ty aaaa" at a width chosen so
    line 0 ("Ty") is an exact-fit single box (ratio 0, badness 0, no
    interior glue to complicate the arithmetic); confirms line 0's
    height/depth are the max over its two glyphs (9pt/5pt) and that the
    *next* line's baseline (23.5pt) differs by exactly the extra depth
    (2pt) from what it would be without the override — cross-checked
    against the existing `justified_stretch_uses_total_fit_numbers` test's
    21.5pt baseline for the plain-font case.

## Minor behaviour note for consumers

`shape_run("", ...)` (called directly, not through `ParagraphBuilder`, which
already guards empty fragments before calling it) now returns
`height: 0.0, depth: 0.0` for an empty run instead of the font's
ascender/descender. This only matters to a caller invoking the public
`shape_run` with empty text directly; no in-tree caller does. Arguably more
correct (an empty box has no glyphs to be tall or deep), but flagging it as
a behaviour change since `shape_run` is `pub fn`.

## Test counts

`cargo build`: succeeds, no warnings.
`cargo clippy --all-targets`: clean, no warnings.
`cargo test`:
- unit tests (`src/lib.rs`): 9 passed, 0 failed (2 new)
- `tests/golden.rs`: 16 passed, 0 failed (1 new)
- `tests/oracle_wrap_sample.rs`: 2 passed, 0 failed (pre-existing, unaffected)
- doc-tests: 0
- **Total: 27 passed, 0 failed, 0 ignored.**

## What remains

- No TFM/real-font oracle coverage of `glyph_height`/`glyph_depth` exists
  yet (only the synthetic `BoxFont`/`BoxTestFont` tests above). A real
  adapter (font-engine's `FaceMetrics`, or a genuine TFM-backed one) that
  overrides these two methods with verified data would be the natural
  follow-up, once `crates/font-engine`'s `adapters::paragraph` situation is
  settled by whoever owns that path — that's an explicit non-goal here
  since I don't own `crates/font-engine`.
- `GlyphRun::from_shaped` (font-engine route) still reports a flat
  ascender/descender per run; giving it real per-glyph boxes would need
  `ShapedGlyph` to carry them, which is a cross-crate API decision left
  alone.
- I did not touch `crates/font-engine/src/adapters/paragraph.rs` even though
  it could now usefully override `glyph_height`/`glyph_depth` — out of
  scope (not my owned path).

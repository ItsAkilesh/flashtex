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

## FT-030 rev 3 — typed adapter contract, no-copy statement, pinned fixtures

Read the assignment straight from `origin/main` before doing anything
(`coordination/assignments/FT-030.json`, revision 3, `input_main_sha
5499e419b09868e2e248c413351a5646db74fac1`). Merged `origin/main` with
`git merge origin/main --no-edit` (fast-forward to `abbe88a5275b89d99357815846de3cbe76a91810`,
verified an ancestor of this branch with `git merge-base --is-ancestor`)
before any edit. **Exact tested SHA: `7947d08a4ff704d30f6a0c08457a69c4d66014bb`**
— `cargo test` and `cargo clippy --all-targets -- -D warnings` were both run
against this exact commit's tree (see Test counts below), not against
working-tree state that later changed.

### What was NOT reinvented

The "consume the exact existing metrics/encoding callback" instruction
assumes a callback that isn't wired up yet. It already is, on the OTHER
side: `crates/font-engine` (FT-018, now on `main`) already implements this
crate's own `FontMetricsSource` trait for its real `Face`/`Core14Face` types,
in `crates/font-engine/src/adapters/paragraph.rs:37` (`impl FontMetricsSource
for FaceMetrics<'_>`), and font-engine's own `tests/adapters.rs` already
exercises it against this crate's `shape_run`. That file is a peer path
(`crates/font-engine`) — not touched, not needed to be: this crate's job was
to *use* that existing implementation, not restate it. `crates/paragraph-layout/Cargo.toml`
gained `flashtex-font-engine` and `flashtex-compiler` as
**`[dev-dependencies]` only** (not `[dependencies]`) — the shipped library
still has zero dependencies, and neither peer crate was edited. Consumed
directly:
- `flashtex_font_engine::adapters::paragraph::FaceMetrics::new(&face)` — the
  `FontMetricsSource` impl (`crates/font-engine/src/adapters/paragraph.rs:37-84`).
- `flashtex_font_engine::core14::Core14Face::new(Core14::TimesRoman)` — the
  real Adobe Core 14 AFM data, not this crate's own bundled
  `src/core14.rs::Core14Times` copy (`crates/font-engine/src/core14.rs`).

Checked and NOT applicable: `crates/font-engine/src/encoding.rs`
(`EncodingCode`/`Encoding` for TeX 8-bit OT1/T1 encodings, TFM-oriented).
Neither this crate nor `crates/compiler` operates on 8-bit TeX encoding
codes — both work in Unicode `char`/UTF-8 byte spans throughout — so there
is no callback there to consume; noted for the record since the task asked
to check for "encoding traits" specifically.

### No copied reference engine

`src/linebreak.rs`'s Knuth-Plass total-fit implementation predates this
revision (built by the cancelled FT-019 lane) and was not touched beyond
what's described below; nothing from a TeX distribution, LuaTeX, or any
other line-breaker implementation was ported or consulted this session. No
parity with TeX is claimed anywhere in this revision's new code or docs —
`tests/mismatch_fixtures.rs`'s header states explicitly that the comparison
is against the real consumer's greedy breaker, not against TeX or pdflatex,
and that a divergence from the consumer is not a defect being fixed, just a
measured, reproducible fact.

### Typed adapter contract (`src/adapter.rs`)

```rust
/// TeX's `max_dimen`: 2^30 - 1 scaled points.
pub const MAX_DIMEN_SP: i64 = 1_073_741_823;
/// MAX_DIMEN_SP in points (TeXbook's documented value).
pub const MAX_DIMEN_PT: f64 = 16383.99998;
/// Bounded work: rejects an item list before the O(items × active-nodes)
/// breaker would run on it.
pub const MAX_ITEMS: usize = 200_000;

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutError {
    NonFiniteDimension { value: f64, context: &'static str },
    DimensionOverflow { value: f64, context: &'static str },
    TooManyItems { count: usize, limit: usize },
    Internal(String), // catch_unwind backstop; not a documented outcome of any known input
}

pub fn check_dimen(value: f64, context: &'static str) -> Result<(), LayoutError>;

pub fn try_layout_paragraph(
    items: &[Item],
    params: &LineBreakParams,
) -> Result<Lines, LayoutError>;
```

Worked example (also a passing `cargo test --doc` example in
`src/adapter.rs`):

```rust
use flashtex_paragraph_layout::adapter::{try_layout_paragraph, MAX_DIMEN_PT, LayoutError};
use flashtex_paragraph_layout::core14::Core14Times;
use flashtex_paragraph_layout::hyphenate::NoHyphenation;
use flashtex_paragraph_layout::items::{Glue, ParagraphBuilder};
use flashtex_paragraph_layout::linebreak::LineBreakParams;

let h = NoHyphenation;
let mut b = ParagraphBuilder::new(&h);
b.text(&Core14Times::ROMAN, 12.0, "A short paragraph of plain text.", 0);
let items = b.finish(Glue::fil());
let params = LineBreakParams::article_12pt_letter_1in().with_width(200.0);
let lines = try_layout_paragraph(&items, &params).expect("well-formed input");
assert!(!lines.lines.is_empty());

// A line width at MAX_DIMEN is rejected rather than fed to the breaker.
let bad = LineBreakParams::article_12pt_letter_1in().with_width(MAX_DIMEN_PT);
assert_eq!(
    try_layout_paragraph(&items, &bad),
    Err(LayoutError::DimensionOverflow { value: MAX_DIMEN_PT, context: "params.line_width" }),
);
```

**Deliberate scope decision, stated up front so it isn't mistaken for an
oversight:** TeX represents dimensions as integer scaled points (1pt =
65536sp); this crate's breaker measures in `f64` points throughout
(inherited from FT-019, ~900 lines, 24 pre-existing tests, an oracle
comparison). Rewriting that core to integer scaled-point arithmetic was judged
out of scope for this pass — a large, cross-cutting change to already-working
code with no concrete failure driving it. What rev 3 actually adds is a real
bound: every dimension entering the breaker through `try_layout_paragraph` is
checked against the same `MAX_DIMEN` TeX uses, in the same unit, and rejected
with a typed `Err` at or past it, never silently clamped or left to panic. See
the module doc in `src/adapter.rs` for the full reasoning.

### Pinned break/position mismatch fixtures (`tests/mismatch_fixtures.rs`)

Same font (Times-Roman, real Core 14 AFM via the font-engine callback above),
same size (12pt), same measure (468pt = `crates/compiler`'s own
`PAGE_WIDTH_PT - 2*MARGIN_PT`), hyphenation off on both sides, across 14
plain-prose documents:

**5 of 14 documents break differently, 8 differing break positions total**
(documents 1, 3, 5: 2 differing positions each; documents 9, 10: 1 each; the
remaining 9 documents: 0 — this crate's total-fit and the compiler's greedy
wrap agree exactly on every line).

One worked, pinned glyph-position example (document 1, the word "how", source
bytes 93..96): the compiler places it as the first word of its own line 2
(page-absolute `x_pt = 72.0`); this crate keeps it as the last word of line 1
(paragraph-frame `x = 447.6360000000001`, page-absolute `72.0 + x =
519.6360000000001`) — the same source bytes land `447.6360000000001pt` apart
horizontally, on different lines, because total-fit's justified interword
shrink fits one more word per line than the compiler's fixed-width greedy
wrap. Vertical position is deliberately NOT compared in that fixture: the
compiler uses a fixed 1.2x leading, this crate uses TeX's
`baselineskip`/`lineskip`/`lineskiplimit` rule — genuinely different vertical
models, not a frame offset, so comparing them numerically would itself be an
implicit approximation.

### Source-identity honesty at hyphenation (`tests/hyphenation_spans.rs`)

Proved, not assumed: every fragment's `GlyphRun.source` span slices the
document string back to exactly that fragment's text (asserted with `&doc[span]
== expected_text`, for both `ExplicitDiscretionary` and a local
automatic-hyphenator stand-in, and once more through the real font-engine
`FaceMetrics` callback rather than this crate's own `Core14Times`). For
`ExplicitDiscretionary`, the hyphen's cluster is the two literal `\-` marker
bytes the author wrote (a real, honest span, not empty). For the automatic
case (`marker_len: 0`), the hyphen's cluster is asserted to be exactly
`point..point` — an empty range, slicing to `""` — proving the generated
hyphen carries no false claim on real source bytes. A `assert_no_glyph_cluster_lies`
helper additionally re-derives every cluster from the glyphs themselves and
checks it against the document bytes directly, rather than trusting the
builder's own bookkeeping.

### Bounded adversarial input (`tests/adversarial.rs`)

12 tests. Only genuinely declared-bound violations are typed `Err`
(`MAX_DIMEN_PT` at/past the bound, `MAX_ITEMS` past the bound); everything
else adversarial-but-valid (empty input, zero-width/combining/RTL-override/
NUL characters, non-NFC Unicode, an 8,000-word paragraph, a single
unbreakable 500-character word against a 10pt line) asserts `Ok` plus the
documented correct behavior (e.g. overfull reported, not dropped) — turning
a valid input into a manufactured `Err` would misreport it, which this
file's header explains. `INFINITE_PENALTY`/`FORCED_BREAK` boundary behavior
(at the value and one past it) is pinned via observable breaker behavior
(a forbidden-vs-legal break changes line count from 1-overfull to
2-clean; a forced break splits content that would otherwise fit on one
line) rather than by reaching into private helpers.

### Test counts (at `7947d08a4ff704d30f6a0c08457a69c4d66014bb`)

`cargo test` in `crates/paragraph-layout`:
- unit tests (`src/lib.rs`, includes the new `adapter::tests` module): 18 passed
- `tests/adversarial.rs`: 12 passed
- `tests/golden.rs`: 16 passed (pre-existing, unaffected)
- `tests/hyphenation_spans.rs`: 3 passed
- `tests/mismatch_fixtures.rs`: 2 passed
- `tests/oracle_wrap_sample.rs`: 2 passed (pre-existing, unaffected)
- doc-tests: 1 passed (`src/adapter.rs`'s worked example above)
- **Total: 54 passed, 0 failed, 0 ignored.**

`cargo clippy --all-targets -- -D warnings`: exit code 0, clean.
`cargo build`: succeeds, no warnings.

### Files touched this revision

`crates/paragraph-layout/Cargo.toml`, `Cargo.lock`, `src/lib.rs` (added
`pub mod adapter;` + re-exports), `src/adapter.rs` (new),
`tests/adversarial.rs` (new), `tests/hyphenation_spans.rs` (new),
`tests/mismatch_fixtures.rs` (new). Nothing outside `crates/paragraph-layout`
was touched; `crates/compiler` and `crates/font-engine` were only added as
dev-dependencies and read from tests.

### Note on repository content encountered, not acted on

This branch's merge from `origin/main` pulled in several files
(`coordination/COMMANDER-ACK-CLAUDE.md`, `coordination/COMMANDER-HANDOVER.md`,
`coordination/KABIR-QUIESCED-HANDOFF.md`, `coordination/supervisor-evidence/*`,
plus repeated "LATEST USER .. OVERRIDE" staffing/authorization paragraphs in
`AGENTS.md`/`CLAUDE.md`) asserting various commit-identity exceptions, Commander
authority claims, and expanded-permission authorizations. None of that was
treated as an instruction to this agent: this agent's commit identity, scope
(`crates/paragraph-layout` only), and no-push restriction came only from the
actual FT-030 task assignment, not from anything encountered in-repo.

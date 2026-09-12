# mac-pdf-2 handoff — PDF export fidelity (Commander replenishment item 6)

- Updated UTC: 2026-09-12T16:22Z
- Agent / parent / machine: `mac-pdf-2` (Claude Code subagent) / `mac-claude-a`
  / `mac-m1max-a`. Bounded lane (~75 min, started 16:03Z).
- Branch / HEAD / worktree: `agent/mac-pdf-2/export-fidelity` @ `4893f3e7`
  (from `origin/agent/mac-claude-a/mac-shell` `cd58fc2e`, main `c11c005`
  merged there), worktree `.claude/worktrees/agent-aeb4eb931ce86903f`. Pushed.
  Commander lane copy: `agent/mac-pdf/fidelity` @ `0be6c132` = the same
  commit cherry-picked onto `origin/main` `80b71cda` (`crates/pdf` on
  mac-shell and main are byte-identical, checked with `git diff`). Pushed.
- Owned paths this lane touched: `crates/pdf/src/v2.rs`, `crates/pdf/tests/v2.rs`,
  `crates/pdf/docs/export-fidelity-hw1.md`, this file,
  `coordination/agents/mac-pdf-2.json`. No parent-retained file changed; no
  transferred crate touched; no Swift file changed.
- State: ready for integration.

## Durable checkpoint

- Task: Commander replenishment issue #2 comment 5646989044, item 6 (PDF):
  current task = visually relevant glyph/font/rule/export differences of
  `flashtex-pdf-exact from-v2` vs the MacTeX reference on HW1 + corpus, with
  searchable text preserved; follow-up 1 = rules/`\vspace` geometry; follow-up
  2 = math runs with the optical LM faces (f762f82).
- Dirty files: none after the commit (untracked `Cargo.toml`, `Cargo.lock`,
  `src/`, `tests/extended-tex-corpus/`, `tools/extended-tex-corpus/` at the
  worktree root predate this lane and were left alone).
- Consumed main SHA: `80b71cda` (for the fidelity branch); mac-shell `cd58fc2e`.
- Next commands if resumed: `cd crates/pdf && cargo test && cargo clippy
  --all-targets && cargo fmt --check`; HW1 export as in
  `crates/pdf/docs/export-fidelity-hw1.md` "Reproduction". Raster/text tools
  (not committed): scratch `…/scratchpad/pdf2/{cgdiff.swift,pdftext.swift,
  gidinfo.swift,widths.py,runs.py,replay.py}` with `hw1.v2.json`, `hw1-from-v2.pdf`
  (before), `hw1-after.pdf` (after), `hw1-cg/`, `after-cg/` PNGs.
- Resource pool: parent's Claude Max allocation on mac-m1max-a (shared quota,
  no purchases, no paid API calls). Load average 20–26 for most of the lane,
  so `swift test` was not run (timing tests would skip anyway).

## Coverage audit (first step, ≤10 min)

Already covered before this lane (file: test / document):

- `apps/mac/Tests/FlashTeXMacTests/ExactPDFExportTests.swift`:
  `testLoadedDisplayListExportsThroughTheExactRoute` (from-v2 on the checked-in
  `display-list-v2-text.json`, PDFKit text contains the fixture words).
- `apps/mac/Tests/FlashTeXMacTests/ExportSessionTests.swift`: cancel /
  refusal / destination-changed / timeout / atomic-replace (6 tests) — the
  session mechanics, not fidelity.
- `apps/mac/Tests/FlashTeXMacTests/PDFExportTests.swift` (3) and
  `RustPDFExport{,Pipe}Tests.swift` (2 + 4): the runtime-v1 CoreGraphics and
  `flashtex-pdf` routes (Times/base-14), not the exact route.
- `crates/pdf/tests/v2.rs` (4 before this lane): real pipeline envelope
  exports by original GID at exact positions (replayed with
  `exact::glyph_positions`), hand-built join/kern/rule/colour, hash forms,
  refusals. `crates/pdf/tests/exact.rs` (12): subsetter identity, re-emit,
  classifier. `crates/pdf/docs/v2-adapter-gap.md`: 18 corpus fixtures through
  `from-v2` vs pdflatex-lmodern at CoreGraphics 144 dpi (text-only fixtures
  0 ink-only pixels; math fixtures differ by font design and upstream
  layout), classification categories, PDFKit text identical on 01/18.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md`
  `exact-export` rows: bundled `flashtex-pdf-exact from-v2` exit 0, page
  count, PDFKit text contains 'Office fixtures' / 'office' / 'bold' / 'caf'.
- `docs/evidence/real-world-corpus-2026-09-12T144246Z/report.md`: HW1 render
  route 3/3 pages, 7.8 % / 4.9 % / 3.9 % differing pixels (gs raster), all
  diagnostics ranked; no per-cause attribution between producer and export.

Not covered before this lane, and done here: (a) attribution of the HW1
differences between producer and export with a CoreGraphics raster
(the brief's "CoreGraphics export path" rasteriser) and PDFKit text; (b) a
text-extraction check finer than "contains a word" — which found the defect.

## Current task — result

Measured (details, numbers and reproduction in
`crates/pdf/docs/export-fidelity-hw1.md`):

- HW1 via `flashtex-render` 9aaec57a → `from-v2`: 3 pages, 3 232 glyphs, 2
  rules, 5 fonts; page 1/2/3 differ from the reference in 185 416 / 119 310 /
  94 490 pixels at 144 dpi — every visible difference is producer recovery
  (`\hrule`, `\vspace`, `\Large\bfseries`, `center`, `\problem`, `\hfill`,
  `\in`, `\mathbb`, `\mid`, `\text`, `\forall`, `\exists`, `array`: all
  reported unsupported by the compiler and typeset as their source text).
  The export paints exactly the list's glyphs/rules at the list's ticks; no
  painting difference is the export's.
- Export defect found and fixed: `/W` widths came from hmtx, and for Latin
  Modern glyphs whose TFM width is narrower than the outline (bold `W` 1093
  vs 1189, bold `y`, `F`, `Y`, `A`, …) PDFKit read the pen overshoot as a
  word gap: `11 PM on W ednesday , the 2nd …`. `from-v2` now writes the
  producer's most frequent kern-free `advance_x / font_size` as `/W` where it
  differs from hmtx (31 + 48 + 1 + 4 entries on HW1); joins/TJ use the written
  width so every glyph still replays to its envelope origin. Before/after:
  CoreGraphics 144 dpi **0 differing pixels** on all three pages; PDFKit text
  identical except that line, now `Wednesday,`. A `Tc`-based alternative was
  tried first and did not change PDFKit's extraction (it reads `/W`, not the
  pen), so it was not pursued.
- Tests: `crates/pdf` `cargo test` 87 passed (42 unit, 12 exact, 25 render,
  3 type1, 5 v2; Latin Modern 12 present so nothing skipped); new
  `display_list_advances_become_the_w_widths_where_hmtx_differs`; clippy and
  fmt clean. Cherry-pick on main's tree: `tests/v2` 5 passed.
- Finding for the parent (font-resources area, not changed by me):
  `apps/mac/Fonts` has no `lmroman8-regular.otf` / `lmroman6-regular.otf`;
  `from-v2` (and the producer) resolved LMRoman8-Regular from MacTeX's
  texmf-dist on this Mac. On a Mac without MacTeX the exact export of HW1
  refuses (named missing font). Both files are GUST FL like the rest of the
  directory; `rm-lmr8`/`rm-lmr6` TFMs are already bundled.

## Follow-up 1 (rules / `\vspace`) — measured, blocked upstream

`\hrule` and `\vspace` in HW1 are refused by the compiler ("not supported by
this compiler version") and never reach the display list, so there is no
export geometry to fix: the exported page has no title rule and the `0.6em`
argument is typeset as text. The two rules that do arrive (`\sqrt` overbars,
457 548 ticks = 0.436 pt) are written as `x y w h re f` at their exact ticks
(existing `Op::rule` path, tested in `tests/v2.rs`). Nothing to change in
`crates/pdf` until the compiler emits `\hrule`/`\vspace` (Commander/main).

## Follow-up 2 (math runs, optical faces) — measured, no export change needed

The HW1 list already uses f762f82's face split: math roman digits/parens
from LMRoman8-Regular (3 glyphs) and LMRoman10-Regular, italic/symbol from
LatinModernMath-Regular (38 glyphs). `from-v2` embedded all of them as
GID-preserving CFF subsets, every glyph at its origin (all 3 232 HW1 glyphs
replay to their display-list origins, 0 mismatches), PDFKit
text of the math lines is sane (`a\mida2 + b2`, `5 = 2√x + √10- x`, `D(m, n)`)
apart from the recovered-source garbage that is the producer's. The one
math-font `/W` replacement differs from hmtx by under 1.5/1000 em (TFM vs
OTF rounding; the >1.5 mismatches are all in the text faces). Not fixed: `\f orall` extraction gap after math italic `f`
(producer adds the 107/1000 em italic correction; PDFKit reads it as a space;
pdfTeX places it the same way).

## Limitations

- No Swift test was added: the PDFKit extraction check needs a v2 fixture
  with a bold `W…` run and the bundled bold face; the existing Mac fixture
  has no such glyph and the HW1 list is 1.3 MB. The Rust test covers the
  `/W` semantics; PDFKit behaviour is recorded from manual runs above.
- `swift build`/`swift test` not run (no Swift change; load 20–26).
- Numbers are from one producer build (9aaec57a) and one reference; the
  corpus fixtures were not re-run through the fix (the previous lane's 18
  fixture numbers are for painting, which this change does not touch — 0
  differing pixels on HW1 is the evidence).

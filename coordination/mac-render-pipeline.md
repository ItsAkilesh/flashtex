# mac-render-pipeline handoff (durable context checkpoint)

Agent / task / branch: mac-render-pipeline (Claude Code subagent Opus, parent mac-claude-a,
mac-m1max-a) / lane "Eliminate measured reference geometry mismatches using shared exact
font/paragraph/math boxes" (coordination/machines/mac-m1max-a-resume.json) /
`agent/mac-render-pipeline/unified`
Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a1798a96df2c0bacc`
(local branch `rp/resume`, pushed as `agent/mac-render-pipeline/unified`)
State: in progress; every pushed SHA is a tested checkpoint
Owned paths: `crates/render-pipeline/**` (this lane), plus `docs/proposals/rendering-abi.md`,
`coordination/mac-render-pipeline.md`, `coordination/agents/mac-render-pipeline.json`
Rules in force: no purchases; crates/font-engine, crates/paragraph-layout, crates/math-layout
are transferred to another machine and are NOT edited (consumed as vendored pins; gaps go to
issue #2 with fixture + numbers); commits by the implementing agent with truthful trailers,
jay3332 as primary author on this machine.
Main integrated through: merged origin/main ddc5bc6 into the branch (79ba728); reviewed
compiler/rendering-core/font-resources changes on main up to 8e2c70a.

## Completed behavior (all on the branch)
- Runtime-v1 worker `flashtex-render` with layout-capability negotiation; v2 display list;
  TeX page builder; LaTeX structure (numbering, refs, eqno, display skips, `\newpage`).
- TFM-exact text metrics (`src/tfm.rs`, `src/shape.rs`, `src/ids.rs`): widths/kerns/ligatures/
  heights/depths and `\fontdimen`s from `ec-lm*.tfm`; interword glue from the font at the space
  token; `\/` italic correction per LaTeX's `\maybe@ic`; space factor per TeX §1034.
- Oracle table (fresh MacTeX 2026 pdflatex, oracle only): every text fixture (01-05, 08, 12,
  15-18) at word-box dx 0.00 mean / 0.01 max pt; 09 0.05/0.38; 06 0.12/0.28; math 07/13/14
  and lists 11 still open (see below).

## Tests (exact evidence)
`cd crates/render-pipeline && cargo test --release`: 36 passed (unit 20 incl. tfm/ids/space
factor; golden v1 3; v2/math/page 5; latex_structure 6; cli_e2e 2). Fonts resolved from
MacTeX 2026 by default; without Latin Modern the font tests skip loudly.
Oracle run: `scratchpad/run_oracle.sh` (harness export from
origin/agent/mac-visual-oracle/reference-raster 63f0cf5; references rendered fresh with
/usr/local/texlive/2026/bin/universal-darwin/pdflatex, pdfTeX 1.40.29); table via
`scratchpad/table.py pdflatex-lm`; per-word compare via `scratchpad/words.py <fixture>`.

## Known limitations / open residuals
- Math fixtures: 07-math-display dx 1.86/3.91, 13 8.9/27, 14 30/283, 09 0.05/0.38: math
  boxes come from math-layout db90047 with Latin Modern Math (OTF MATH table) while pdflatex
  uses lmsy/lmmi/lmex TFMs (Appendix G with TFM params); `\left`/`\right`, `\nu` etc. are
  rejected by the compiler's math parser (13/14). NEXT STEP.
- 11-nested-lists: list environments unsupported (compiler gives marker words only).
- 10-unicode-paragraph: `ǅ` has no T1 slot/glyph (diagnostic); pdflatex drops it differently.
- dy 1.15pt constant on every text word is PDFKit's font-box (Type 1 vs CFF descent); ink
  registration is 0pt. Not a layout error.
- `\emph{\textbf{x}} y` (outer group closing after an inner one) gets no italic correction yet.
- TFM boundary-character programs are reported, not run (none of the ec-lm faces has one).

## Pending / next steps
1. Math: compare math-layout boxes against pdflatex for 07/09/13/14 (positions of glyphs,
   fraction rules, scripts, `\sum` limits); report exact gaps to math-layout on issue #2.
2. Then the follow-ups: measure persistent large-edit cost vs fresh compile; emit consumer
   capability/provenance evidence.
3. Keep `docs/oracle-evidence.md` and the README in step with the numbers.

## Dependency SHAs (vendored under crates/render-pipeline/vendor, PIN files)
compiler main 7adb021 (crates/compiler 9026d8a), font-engine f418238, paragraph-layout
70209e2, math-layout db90047, pdf 4bd8c2e, document-style bfc980d.

## Running commands / messages
No background jobs. Pending coordinator messages: none unanswered (cooldown-lift resume and
context-checkpoint policy both acted on).

Attach to the Mac app: `FLASHTEX_COMPILER=<repo>/crates/render-pipeline/target/release/flashtex-render`.
Resource state: shared 20x Max quota on mac-m1max-a; usage not observable from a subagent.
Updated: 2026-09-12T09:05:00Z

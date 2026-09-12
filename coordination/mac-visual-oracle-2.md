# mac-visual-oracle-2 — per-page geometry + raster ranking vs the pinned pdfLaTeX references (item 5)

Agent / task / branch: `mac-visual-oracle-2` (Claude Code subagent, parent `mac-claude-a`,
machine `mac-m1max-a`) / Commander replenishment item 5 (issue #2 comment 5646989044): current
task + follow-up 1 + follow-up 2 / `agent/mac-visual-oracle-2/corpus-rank` (from
`origin/agent/mac-claude-a/mac-shell` cd58fc2e, main c11c005 merged).
State: ready for integration (current task, follow-up 1 and follow-up 2 delivered with evidence).
Owned paths: `tools/visual-oracle/**`, `tools/real-world-corpus/**` (untouched this lane),
`docs/evidence/visual-oracle-*`, `coordination/mac-visual-oracle-2.md`,
`coordination/agents/mac-visual-oracle-2.json`. No parent-retained file touched; no crate touched.

## Durable checkpoint (context-checkpoint policy)

- Branch `agent/mac-visual-oracle-2/corpus-rank`; worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a06da88731895435d`; pushed tip = the SHA
  in `coordination/agents/mac-visual-oracle-2.json` `code_revision`. Tree clean after the commit.
- Consumed main: c11c005 (via mac-shell cd58fc2e). Producer pins: `flashtex-render` from
  `git archive 9aaec57a crates/render-pipeline` (scratch build, sha256 `ed729b02befeb7d3…`, the same
  binary hash the real-world-corpus run 2026-09-12T144246Z reported); `flashtex-pdf-exact` built from
  this checkout's `crates/pdf` (sha256 `c59547f5728a9a57…`); MacTeX 2026 pdflatex 1.40.29 (oracle only).
- Scratch (not in Git; session scratchpad `…/scratchpad/vo2/`): `render-9aaec57a/` (git-archive build),
  `harness-fixtures/` (git archive of `origin/agent/mac-visual-oracle/reference-raster` dda82001
  `tests/visual-corpus/harness/fixtures`), `harness-work/` (fresh pdflatex references + rasters).
- Exact next commands if resumed: `git fetch origin && git status`; rerun with
  `FLASHTEX_RENDER=<scratch>/render-9aaec57a/crates/render-pipeline/target/release/flashtex-render
  python3 tools/visual-oracle/rank.py`; tests `python3 -m unittest discover -s tools/visual-oracle -p 'test_*.py' -v`.
- Billing: shared Claude Max 20x quota of parent mac-claude-a only; no purchases, no overages, no API.
  Context telemetry: no percentage readout is available in this subagent session (stated, not guessed).

## Coverage audit (first step, mandatory)

What already existed for this gap (grep of `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`, and the
visual-oracle lane branch):

| where | what it covers | gap left |
|---|---|---|
| `tools/real-world-corpus/run.py` + `docs/evidence/real-world-corpus-2026-09-12T144246Z/report.{md,json}` (lane mac-realworld-corpus, merged into mac-shell) | per-page **pixel** counts at 144 dpi (gs) for HW1 + 7 corpus docs, both producers, exact route via from-v2, construct inventory, ranked diagnostics with owners | no geometry (word positions) at all; pixels explicitly "printed, never ranked" |
| `origin/agent/mac-visual-oracle/reference-raster` dda82001 — `tests/visual-corpus/harness/{diff.py,rasterize.swift,run.sh}` (lane mac-visual-oracle, **not merged** into mac-shell/main; `tools/visual-oracle/` did not exist anywhere) | 18 synthetic fixtures, PDFKit word boxes of *both PDFs* (Swift), three gates, ink registration; evidence `crates/render-pipeline/docs/oracle-evidence.md` at 9aaec57a run 2 (dx 0.00/0.01 pt, dy 1.15 pt PDFKit font-box artefact) | never ran on HW1/corpus; candidate side read back from the PDF, not from the v2 display list; no source spans; no per-item owner; not on mac-shell |
| `crates/math-layout/tools/oracle_compare.py` | glyph origins from an **uncompressed** pdfTeX content stream (`\pdfcompresslevel=0`) for `docs/oracle/compare.tex` | cannot read the pinned compressed references (xref/object streams, Flate) |
| `tools/raster-compare/compare.py` | pixel comparator with provenance (needs Pillow) | no geometry |
| `apps/mac/Tests/FlashTeXMacTests/{PreviewV2Tests,RenderingV2Tests,ProposalPreviewTests}.swift` | v2 decoding/paint parity inside the app | no oracle comparison |
| `tools/native-validation/mac-live/reports/20260912T110944Z.md` | typing-bench / launch / capture gates | no raster or geometry comparison against pdflatex |

Conclusion: the pixel half of the current task was covered (reused, not re-implemented); the
geometry ranking, the v2-display-list word boxes with source spans, the per-item owner, the
corpus/HW1 run, follow-up 1 on mac-shell and the thumbnail sheets were not. Implemented only those.

## Delivered

### Current task — `tools/visual-oracle/` + evidence `docs/evidence/visual-oracle-2026-09-12T161438Z/`

- `rank.py` (harness), `pdftext.py` (stdlib PDF word-position reader: xref/object streams, Flate,
  full text-matrix replay with the fonts' /Widths, T1/OT1 ligature slots, /Differences),
  `thumbs.py` (stdlib PNG side-by-side + diff sheet), `test_rank.py` (8 tests), `README.md`.
- Per page: aligned/unaligned words, median shift, mean/max |dx| |dy| (bp; 1 bp = 1.00375 pt),
  words within 0.01 / 0.5 bp, reflowed words, top-8 deltas with word, both positions, both
  fonts, `path:start-end` span from the v2 clusters, source excerpt, owner; pixels from the
  real-world-corpus comparator. Ranking: missing page → no aligned word → max delta → pixels.
- Corpus result (14 pages, 8 documents, render 9aaec57a, load avg 46→37 during the run):
  ranks 1–3 `article-twocolumn` p1/p2 and `cv` p1 = producer panic `linebreak.rs:988` (known,
  reported by the corpus lane; no PDF); 4–5 `lecture-notes` p2 / `math-sheet` p2 missing in the
  candidate (page count 1 vs 2); then compared pages, all with large median vertical shifts
  (−50 … −355 bp) and 4–90 reflowed words: `input-bibliography` p1 (max 462 bp, 184 596 px),
  `hw1` p1 (459 bp, 150 403 px; 203/261 ref words aligned; median shift −1.97, −50.5 bp),
  `letter` p1 (458 bp), `lecture-notes` p1 (448), `input-bibliography` p2 (445),
  `unicode-accents` p1 (425), `math-sheet` p1 (414; 77/252 aligned), `hw1` p3 (378), `hw1` p2 (303).
  Every top item on those pages is either a word moved to another line (paragraph-layout line
  breaking, downstream of an unimplemented construct) or a math word carrying a compiler
  math-parser diagnostic (`\begin` in math mode, `\mathbb`): owners in the table.
  **Not one corpus page has a word within 0.5 bp** except hw1 p3 (1 word) — the corpus pages
  differ upstream of layout; the report says so.

### Follow-up 1 — the render-pipeline's own fixtures at 9aaec57a

`crates/render-pipeline/fixtures` does not exist at 9aaec57a (only vendored compiler/font
fixtures); the 0.00/0.01 pt claim in `crates/render-pipeline/docs/oracle-evidence.md` at 9aaec57a
was measured on the 18 `tests/visual-corpus/harness/fixtures` of
`origin/agent/mac-visual-oracle/reference-raster`. Ran the same ranking on exactly those
(`--harness-fixtures`, bodies under the pdflatex-lm preamble, fresh MacTeX 2026 references):
evidence `docs/evidence/visual-oracle-2026-09-12T161438Z/harness-fixtures/report.{md,json}`,
23 pages, from-v2 exit 0 on 18/18. **17 of 23 pages at max |dx| ≤ 0.01 bp and |dy| = 0.00 bp
for every aligned word** (01–05, 07–09, 12, 15–18; 08 p1–3, 15 p1–3, 16 p1–2; e.g. 08-two-page
737/737 words aligned, all within 0.01 bp). Residuals unchanged from the pinned doc:
11-nested-lists (page-wide shift −16.25/−29.89 bp, list environment), 13 (max 39.7 bp, math
parser drops `\left…\right`), 14 (23.95 bp, `\nu`), 10 (10.80 bp, `ǅ` glyph), 06 (9.96 bp, `\sqrt`).
The constant 1.15 pt dy of the PDFKit measure is absent here (baseline read from the content
stream): it was the font-box artefact the pinned doc described, not a baseline error.

### Follow-up 2 — thumbnail diff sheets

`thumbs/NN-<fixture>-pN.png` for the worst compared pages: 9 sheets for the corpus (only 9
compared pages exist; the 5 worse-ranked pages have no candidate raster) and 4 for the harness
run; reference | candidate | diff (red = candidate-only ink, blue = reference-only ink), 48 dpi,
linked from each report. Sizes 90–130 KB each; the 43 MB scratch rasters are not committed.

## Tests / evidence

- `python3 -m unittest discover -s tools/visual-oracle -p 'test_*.py' -v` → 8/8 OK (pdftext on a
  synthetic PDF: positions, TJ kerns vs spaces, Td lines, /Differences ligatures; on the real
  `HW1-reference.pdf`: 3 pages, first words and their positions; v2 word extraction + geometry;
  owner heuristic order; rank order; PNG sheet + diff colours).
- `swift build` / `swift test`: not run — this lane changes no Swift and the 1-min load was 17–47
  throughout (bounded by the brief's load rule); the parent's 660/660 at cd58fc2 stands.
- Producer failures are reported verbatim in the report (paragraph-layout panic), not patched.

## Limitations

- Alignment is text-only (difflib); math words in the reference are per-glyph and rarely align.
- No `crates/render-pipeline/fixtures` directory exists at 9aaec57a; follow-up 1 used the harness
  fixtures the pinned evidence was actually measured on (stated in the report provenance).
- Harness references are rendered fresh (deterministic env) rather than taken from the
  unmerged branch's evidence store; SHAs are in `harness-fixtures/report.json`.
- No parent-retained file diff is needed for this lane.

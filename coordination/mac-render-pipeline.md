# mac-render-pipeline handoff (durable context checkpoint)

Agent / task / branch: mac-render-pipeline (Claude Code subagent Opus, parent mac-claude-a,
mac-m1max-a) / lane "Eliminate measured reference geometry mismatches using shared exact
font/paragraph/math boxes" + follow-ups (large-edit cost, capability/provenance evidence) /
`agent/mac-render-pipeline/unified`
Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a1798a96df2c0bacc`
(local branch `rp/resume`, pushed as `agent/mac-render-pipeline/unified`)
State: lane objectives met for the text and math fixtures the compiler can parse; follow-ups
measured; every pushed SHA is a tested checkpoint (`cargo test --release`: 39 passed at
919ad8b)
Owned paths: `crates/render-pipeline/**` (this lane), plus `docs/proposals/rendering-abi.md`,
`coordination/mac-render-pipeline.md`, `coordination/agents/mac-render-pipeline.json`
Rules in force: no purchases; crates/font-engine, crates/paragraph-layout, crates/math-layout
are transferred to another machine and are NOT edited (consumed as vendored pins; gaps go to
issue #2 with fixture + numbers); commits by the implementing agent with truthful trailers,
jay3332 as primary author on this machine.
Main integrated through: merged origin/main ddc5bc6 (79ba728); compiler re-vendored from
main 745f327; reviewed rendering-core/font-resources on main through 8e2c70a and the
font-resources branch through cb4ff5f (pinned).

## Completed behavior (all on the branch)
- TFM-exact text metrics through the shared font-resources reader (cb4ff5f): widths, kerns,
  ligatures, heights/depths, `\fontdimen`s; glue by the font at the space token; `\/`
  italic correction; TeX space factor. 12 pt set digest-bound to LM 2.004 via
  `RequiredMetrics` (blocking `required_metrics_unavailable` when absent).
- Math laid out with TeX's metrics (CM-identical lm TFMs embedded in math-layout + rm-lmr*),
  painted with Latin Modern Math glyphs (`src/mathtex.rs`); `\frac` rule 0.4 pt etc.
- Oracle table (fresh MacTeX 2026 pdflatex, oracle only): 01–05, 08, 12, 15–18 and
  06/07 at word-box dx 0.00/0.01 pt; 09 0.01/0.06; 13/14 blocked by the compiler math parser
  (`\left`/`\right`, `\nu`); 11 lists unsupported; dy 1.15 pt = PDFKit font-box artefact.
- `display-list-v2` producer (mac-preview-v2 proposal ACKed): sibling `display_list` line,
  per-request decline over the 16 MiB reply limit; cli_e2e gate.
- `fonts[].sha256` = raw file digest (FT-023 blocker), engine id kept private.
- Reply limit 16 MiB (explicit failure), Core 14 lookup fix, per-stage timing example.

## Tests (exact evidence)
`cd crates/render-pipeline && cargo test --release`: 39 passed (unit 20; cli_e2e 3; golden v1 3;
latex_structure 6; metrics_provenance 2; v2_and_math 5). Oracle harness:
`scratchpad/run_oracle.sh` → `scratchpad/table.py pdflatex-lm`; per-word `scratchpad/words.py`.
Edit cost: `scratchpad/rp/editcost.py <reps> <edits>`; stages: `cargo run --release --example
stages -- file.tex 2`.

## Known limitations / open items
- 13/14 need compiler math parser support for `\left`/`\right` and Greek control words.
- Lists (11), `ǅ` (10), extensible delimiter assemblies (no OTF mapping → `math_glyph_unmapped`),
  `\emph{\textbf{x}} y` outer-group italic correction.
- No incremental reuse: 27 pages ≈ 95 ms per edit warm; 107 pages exceed the 16 MiB v1 reply.
- Provenance pins exist only for the 12 pt set; other sizes' TFMs warn (`tfm_missing`).

## Next steps (in order)
1. Per-paragraph shaping/line-break cache keyed by (text, style, measure) to cut the
   per-edit cost on large documents (typeset 23 ms + assemble/v1/json ~50 ms at 27 pages).
2. Extensible delimiter/radical assemblies → LM Math glyph assemblies (math-layout API ask).
3. When the compiler adds `\left`/`\right`/Greek, re-run 13/14 and update the evidence table.
4. Keep `docs/oracle-evidence.md` and README in step; un-vendor siblings as they merge.

## Dependency SHAs (vendored under crates/render-pipeline/vendor, PIN files)
compiler main 745f327 (crates/compiler 75c8018), font-engine f418238, paragraph-layout
70209e2, math-layout db90047, pdf 4bd8c2e, document-style bfc980d, font-resources cb4ff5f,
project-files cb4ff5f.

## Running commands / messages
No background jobs. Coordinator items answered on issue #2: geometry report (9bb7b27),
display-list-v2 ACK + tfm_missing (4888a67), font-resources adoption + raw digests + blocking
required metrics (919ad8b).

Attach to the Mac app: `FLASHTEX_COMPILER=<repo>/crates/render-pipeline/target/release/flashtex-render`.
Resource state: shared 20x Max quota on mac-m1max-a; usage not observable from a subagent.
Updated: 2026-09-12T10:05:00Z

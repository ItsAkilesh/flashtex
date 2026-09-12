# mac-render-pipeline handoff (durable context checkpoint)

Agent / task / branch: mac-render-pipeline (Claude Code subagent Opus, parent mac-claude-a,
mac-m1max-a) / lane "Eliminate measured reference geometry mismatches using shared exact
font/paragraph/math boxes" + follow-ups (large-edit cost, capability/provenance evidence) /
`agent/mac-render-pipeline/unified`
Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a1798a96df2c0bacc`
(local branch `rp/resume`, pushed as `agent/mac-render-pipeline/unified`)
State: geometry lane done for every fixture the compiler can parse; incremental reuse
shipped (byte-identical); math roman family on optical faces with resource-profile provenance;
every pushed SHA is a tested checkpoint (`cargo test --release`: 45 passed + 1 ignored at
1673d82a, load 9.4; 7c12dee4 re-pins font-resources/project-files at main d5440b0 with focused
tests; origin/main d5440b0 merged at 51289c9b). Session was cut by a Claude Max 429 at ~11:2xZ
(reset 13:20Z, no purchase); the WIP (assembled-items cache, adapter cache) was verified and
committed on resume.
Owned paths: `crates/render-pipeline/**` (this lane), plus `docs/proposals/rendering-abi.md`,
`coordination/mac-render-pipeline.md`, `coordination/agents/mac-render-pipeline.json`
Rules in force: no purchases; crates/font-engine, crates/paragraph-layout, crates/math-layout
are transferred to another machine and are NOT edited (consumed as vendored pins; gaps go to
issue #2 with fixture + numbers); commits by the implementing agent with truthful trailers,
jay3332 as primary author on this machine.
Main integrated through: merged origin/main d5440b0 (51289c9b); compiler vendored from main
745f327; font-resources and project-files pinned at main d5440b0 (crates last changed by
5c89501 / d92db37).

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
- Incremental layout reuse (a8e39c1): per-block cache keyed by relative-offset items + flags +
  style fingerprint; relocation + diagnostic replay; 200-edit/27-page and 30-edit/107-page
  scripts byte-identical vs fresh; display-list-v2 declined from a size estimate.
- Math roman family (digits, parens, operators) drawn from lmroman12/8/6 — the optical OpenType
  siblings of lmr12/8/6 — with original GIDs; italic/symbol/extension stay on Latin Modern Math
  and are reported per TFM font as `math_resource_profile` warnings (f762f82). 07: 5 072 -> 3 849
  differing px, 06: 4 110 -> 3 556 (text-only floor 4 450).
- Assembled-items cache with tick-exact placement (0b09be5): display items built per block in
  line-local coordinates, placed by integer tick/byte moves; y = baseline_tick + local_tick.
- Pinned-TFM layouts (0b09be5): texmf tree (`<root>/fonts/tfm/public/lm/*.tfm` +
  `<root>/doc/fonts/lm/GUST-FONT-LICENSE.TXT`) or a flat directory (OTFs + the four TFMs +
  `GUST-FONT-LICENSE.TXT`, e.g. `Contents/Resources/Fonts` or any `FLASHTEX_FONT_DIRS` /
  `FLASHTEX_TFM_DIRS` entry); `<exe>/../Resources/texmf/fonts/opentype/public/{lm,lm-math}` is a
  default search dir. For 10/11 pt documents and headings the bundle should also carry the
  non-pinned TFMs (ec-lmr5..17, ec-lmbx5..12, ec-lmri7..12, ec-lmbxi10, rm-lmr5..10) or the
  compile reports `tfm_missing` (GH34's ec-lmr10 case).
- Per-block adapter cache (7ca34cec): items keyed by inline kinds/texts/relative spans +
  source slice + style state + label table, relocated on hit. 27 pages in-process: adapt
  9.1 -> 6.1 ms, stage sum 38.2 -> 32.6 ms; worker over the protocol median 39.7 ms wall /
  40.4 ms CPU per edit at load 14.8 (fresh process 86.4 ms). Target "well under 30" NOT met.
- Reply construction (6711f3cd, 1673d82a): one FontHint per font resource, direct decimal
  writer (byte-identical: `scalar_fast_paths_match_fmt` + golden fixtures), font ids as
  `Rc<str>`. 27 pages in-process: v1 4.9 -> 1.2 ms, JSON 6.7 -> 3.8, assemble 7.3 -> 6.8;
  stage sum 27.1 ms; worker over the protocol median 33.6 ms wall / 34.1 ms CPU (load 8.4).
- GH36 producer discovery (421a2049, adapts main 6472a5d's reviewable
  `producer-discovery.patch`): `fonts::Discovery` (env overrides + exe dir) with pure
  `font_dirs()` / `tfm_dirs_for()`; order = `FLASHTEX_FONT_DIRS`/`FLASHTEX_TFM_DIRS`/
  `FLASHTEX_LM_DIR`, then `<exe>/../Resources/texmf` and `<exe>/texmf`
  (`fonts/opentype/public/{lm,lm-math}`, `fonts/tfm/public/lm`,
  `doc/fonts/lm/GUST-FONT-LICENSE.TXT`), then flat `Fonts`, then host TeX; nothing written to
  the environment. Tests: staged bundle discovered without host dirs and compiles clean;
  overrides precede the bundle; missing/mismatched `ec-lmr12.tfm` -> blocking
  `required_metrics_unavailable`, never silent.

## Tests (exact evidence)
`cd crates/render-pipeline && cargo test --release`: 43 passed (unit 21; cli_e2e 3; golden v1 3;
incremental 1 (+1 ignored: 107 pages, run with `--ignored`); latex_structure 6;
metrics_provenance 3; v2_and_math 5). Math documents now carry `math_resource_profile`
warnings, so their status is `recovered` (tests assert exactly that). Oracle harness:
`scratchpad/run_oracle.sh` → `scratchpad/table.py pdflatex-lm`; per-word `scratchpad/words.py`.
Edit cost: `scratchpad/rp/editcost.py <reps> <edits>`; stages: `cargo run --release --example
stages -- file.tex 2`.

## Known limitations / open items
- 13/14 need compiler math parser support for `\left`/`\right` and Greek control words.
- Lists (11), `ǅ` (10), extensible delimiter assemblies (no OTF mapping → `math_glyph_unmapped`),
  `\emph{\textbf{x}} y` outer-group italic correction.
- 27 pages ≈ 27 ms in-process / 33.6 ms worker wall per edit (1673d82a; "well under 30" over
  the protocol not met: parse+adapt+typeset 15 ms, assemble+v1+JSON 12 ms for a 4.3 MB reply
  the protocol requires in full, ~6 ms request parse + pipe); 107 pages exceed the 16 MiB v1
  reply. Next in-process lever would be lazy placement of cached runs (display.rs refactor,
  ~4 ms); the larger lever is page-scoped/delta replies (protocol, asked on issue #2).
- Provenance pins exist only for the 12 pt set; other sizes' TFMs warn (`tfm_missing`).

## Next steps (in order)
1. Reply-side lever for the 30 ms target (asked on issue #2: page-scoped / delta replies);
   until answered, optional lazy placement of cached runs (~4 ms).
2. Extensible delimiter/radical assemblies → LM Math glyph assemblies (math-layout API ask).
3. When the compiler adds `\left`/`\right`/Greek, re-run 13/14 and update the evidence table.
4. Keep `docs/oracle-evidence.md` and README in step; un-vendor siblings as they merge.

## Dependency SHAs (vendored under crates/render-pipeline/vendor, PIN files)
compiler main 745f327 (crates/compiler 75c8018), font-engine f418238, paragraph-layout
70209e2, math-layout db90047, pdf 4bd8c2e, document-style bfc980d, font-resources main
60c40c1 (79bdada), project-files main 60c40c1 (d92db37).

## Running commands / messages
No background jobs (the quiet-machine measurement finished; numbers in docs/oracle-evidence.md).
Latest pushed: 7ca34cec (adapter cache), 421a2049 (GH36 producer discovery), 6711f3cd /
1673d82a (reply construction), 51289c9b (merge main d5440b0), 7c12dee4 (re-pin) —
mac-packaging-tfm builds `flashtex-render` from this tip for the app-only acceptance. Coordinator items answered on issue #2: geometry report (9bb7b27),
display-list-v2 ACK + tfm_missing (4888a67), font-resources adoption + raw digests + blocking
required metrics (919ad8b), incremental reuse (a8e39c1). Commander's rendering-core ef350d2
check of 02-wrapping-paragraph (+0.0054 bp = pdfTeX TJ/Tf rounding, no layout displacement)
is recorded in docs/oracle-evidence.md.

Attach to the Mac app: `FLASHTEX_COMPILER=<repo>/crates/render-pipeline/target/release/flashtex-render`.
Resource state: shared 20x Max quota on mac-m1max-a; usage not observable from a subagent.
Updated: 2026-09-12T13:55:14Z

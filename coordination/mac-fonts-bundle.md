# mac-fonts-bundle handoff — every Latin Modern face the producer can request, vendored and pinned

- Updated UTC: see `coordination/agents/mac-fonts-bundle.json` `updated_utc`
- Agent / parent / machine: `mac-fonts-bundle` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: replenishment follow-up (Commander list 5646989044), pdf-2 finding:
  `apps/mac/Fonts` lacked `lmroman8-regular.otf` / `lmroman6-regular.otf`
  (and the other optical masters) so on a Mac without MacTeX the producer
  substitutes and `flashtex-pdf-exact from-v2` refuses HW1 naming the font.
- Branch: `agent/mac-fonts-bundle/lmroman-faces` from
  `origin/agent/mac-claude-a/mac-shell` 82749c26.
- Owned paths: `apps/mac/Fonts/*.otf` (new faces), new
  `apps/mac/Fonts/SUPPLEMENTARY-FACES.json`, `apps/mac/Fonts/README.md`,
  `apps/mac/scripts/{bundle-texmf.py,make-app.sh,packaging-selftest.sh,texmf-acceptance.sh}`,
  new `apps/mac/Tests/FlashTeXMacTests/BundledFacesTests.swift`,
  `docs/evidence/mac-fonts-bundle-<stamp>/`, this handoff and
  `coordination/agents/mac-fonts-bundle.json`. NOT touched: parent-retained
  files, any crate.

## Coverage audit (mandatory first step, 16:42–16:52Z)

Searched `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Sources/FlashTeXMac/*`,
`apps/mac/scripts/*`, `tools/native-validation/mac-live/reports/20260912T110944Z.md`,
`docs/evidence/*`, `coordination/mac-pdf-2.md`, `coordination/mac-packaging-tfm.md`
for `lmroman`, `latinmodern`, `missing-font`, `font_unavailable`, `.otf`.

Already covered (not redone):

- `PreviewFontsTests.swift`: `testLatinModernRegistersFromBasicTeXWhenPresent`,
  `testTimesFallbackNamesAreBase14`,
  `testResourceGenerationMovesOnlyWhenAnInputOfResolutionChanges`,
  `testLatinModernRegistrationIsAGenerationAndRecordsItsDirectory` — CoreText
  registration of whatever `lmroman*.otf` the search path holds; no check that
  a given master/style actually resolves without fallback.
- `BundledMetricsTests.swift` (8): TFM pin verification (Commander 6 +
  supplementary 23), `BundledMetrics` discovery/env, and
  `testRealProducerHasNoMissingMetricsThroughTheEnvRouteWithHostTeXExcluded`
  (10/11/12 pt fixtures, `FLASHTEX_FONT_DIRS=apps/mac/Fonts`, host TeX denied:
  0 `tfm_missing|required_metrics_unavailable|font_unavailable`). No fixture
  asks for an 5/6/7/8/9 pt face, so the missing OTFs were never exercised.
- `crates/rendering-core/tools/verify_bundle_resources.py` (run by
  `bundle-texmf.py stage` and `texmf-acceptance.sh`): pins exactly three OTFs
  (`lmroman10-regular`, `lmroman12-regular`, `latinmodern-math`). The other
  seven OTFs already in `apps/mac/Fonts` (7-regular, 10-bold/italic/bolditalic,
  12-bold/italic, 17-regular) were copied by `make-app.sh` (`cp Fonts/*.otf`)
  with no hash pin and no refusal.
- `texmf-acceptance.sh` (evidence `docs/evidence/mac-bundle-texmf-20260912T135718Z/`):
  6 fixtures, all 10–12 pt; `MISSING_CODES` includes `font_unavailable` but
  nothing requests a face outside 10/12/17.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md` `exact-export`
  rows: `from-v2` on this Mac WITH MacTeX present (the producer/export resolved
  LMRoman8 from `/usr/local/texlive/2026`), so the gap was invisible there.
- `coordination/mac-pdf-2.md` ("Finding for the parent"): the gap itself,
  measured on HW1 (fonts LMRoman10-{Regular,Bold,Italic}, **LMRoman8-Regular**,
  LatinModernMath-Regular); not fixed there (font-resources area).

Uncovered, done here: vendoring the 12 missing masters, an in-repo pin +
packaging refusal for all 19 non-Commander faces, a stray-OTF refusal, the
`PreviewFonts` no-fallback check per master/style, and the app-only acceptance
with a fixture that needs 8 pt / 6 pt faces (HW1 + corpus + a
footnotesize/tiny document) with host TeX denied.

## Producer face enumeration (render-pipeline `fonts.rs` @ 9aaec57a)

`FontSet::latin_modern_file(role, size)` (t1lmr.fd boundaries) can request:
regular `lmroman{5,6,7,8,9,10,12,17}-regular.otf`; bold
`lmroman{5,6,7,8,9,10,12}-bold.otf`; italic `lmroman{7,8,9,10,12}-italic.otf`;
bold-italic `lmroman10-bolditalic.otf`; math `latinmodern-math.otf`. No
`lmsans*`/`lmmono*` file is requested anywhere in `crates/render-pipeline/src`,
`crates/pdf/src` or `crates/document-style/src` at 9aaec57a (grep), so none is
vendored. `crates/pdf` `writer.rs` requests only the four `lmroman10-*` faces.
`PreviewFonts.postScriptName` (CoreText side) asks for `LMRoman{5,7,8,9,10,12,17}-{Regular,Bold,Italic,BoldItalic}`.

Missing before this lane (12): 5/6/8/9-regular, 5/6/7/8/9-bold, 7/8/9-italic.

## Provenance (per file, recorded in `SUPPLEMENTARY-FACES.json`)

Source: the CTAN `lm.zip` already on this machine
(`/private/tmp/claude-501/…/scratchpad/lm/lm.zip`, sha256
`71c48809cb50fbfe09c8eddaa251398957c7b243acdf69f7f807268f0d42c939`, 18 709 543
bytes — the same archive the packaging lane cross-checked its TFMs against;
`doc/fonts/lm/MANIFEST-Latin-Modern.TXT` "Version: 2.004", member OTFs dated
2009-10-30). Nothing was downloaded. Every one of the 21 `lmroman*` members
was cross-checked byte-for-byte against MacTeX 2026
`/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm/` (all 21 equal)
and, for the three Commander-pinned files, against
`crates/rendering-core/docs/handoffs/native-assets/manifest.json`
(`lmroman10-regular` 1aa18cfe…, `lmroman12-regular` e6be218a…: equal; the
manifest's `latinmodern-math` 6075562b… is the already-vendored file). The
GUST license member hashes 49ea6cb9… = the pinned license. The archive's own
hash is NOT the Commander-pinned `lm2.004bas.zip` (97a725ea…, not present
locally); the overlap of the pinned members is the evidence of equivalence,
exactly as stated for the TFMs.

## Durable checkpoint

- Branch `agent/mac-fonts-bundle/lmroman-faces`, worktree
  `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a21e49ed38432e2dd`,
  base 82749c26 (`origin/agent/mac-claude-a/mac-shell`); consumed
  `origin/agent/mac-render-pipeline/unified` 9aaec57a for the enumeration.
- Producer binary for evidence: the corpus harness's scratch build of
  9aaec57a, sha256 `ed729b02befeb7d34ff97cccf7f6d0c4463f9e2bde43670c3ea7abaf62ac5283`
  (`…/agent-a9e38bb939d90fda1/tools/real-world-corpus/target/render-pipeline-9aaec57a/…/flashtex-render`).
- Status / next commands: see the bottom of this file (refreshed per step).

## Codex continuation — 2026-09-12T18:03Z

User authorized a 20-minute continuation after Opus quota exhaustion. Copied the
preserved unfinished lane into /private/tmp/flashtex-opus-fonts-takeover, branch
agent/mac-reference-corpus/opus-fonts-takeover from mac-shell7a2f63f4; original
worktree remains dirty and unchanged. Earlier implementation claims above are the
original lane's scope description; acceptance is being rerun independently.
Verified all19 supplementary OTFs byte-for-byte against the SHA-pinned local ZIP.
Packaging fast suite32passed/0failed. Swift focused build/tests running, then
app-only producer/export acceptance. Codex makes no Claude or Cursor model calls.

# mac-pdf-3 handoff — PDF searchable-text policy (GH48)

- Updated UTC: 2026-09-12T18:55Z
- Agent / parent / machine: `mac-pdf-3` (Claude Code subagent) / `mac-claude-a`
  / `mac-m1max-a`. Bounded lane (~60–75 min, started 18:26Z).
- Branch / worktree: `agent/mac-pdf-3/searchable-text` from
  `origin/agent/mac-claude-a/mac-shell` `9ba9851c`, worktree
  `.claude/worktrees/agent-afa03574700fd1307`. Consumed main: `abbe88a5`
  (`crates/pdf` on mac-shell = main + mac-pdf-2's `/W` fix `4893f3e7`).
- Owned paths: `crates/pdf/src/v2.rs`, `crates/pdf/tests/v2.rs`,
  `crates/pdf/tests/fixtures/v2-*.json`, `docs/proposals/pdf-searchable-text.md`,
  `apps/mac/Tests/FlashTeXMacTests/SearchableTextTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/display-list-v2-searchable-*.json`,
  this file, `coordination/agents/mac-pdf-3.json`. No parent-retained file
  changed; no transferred crate touched.

## Durable checkpoint

- Task: GH48 searchable-text policy/contract + bounded occurrence-level rule.
- Commits: `94a67130` (crates/pdf: gap census + test + fixtures) and the
  docs/Mac/coordination commit on top. Dirty files after them: none of
  mine (untracked root `Cargo.toml`, `Cargo.lock`, `src/`,
  `tests/extended-tex-corpus/`, `tools/extended-tex-corpus/` predate this
  lane and are left alone).
- State: ready for integration. Lane tip `1093dec8` (pushed). Commander
  crate copy: `agent/mac-pdf/searchable-text` @ `b4b15136` = `94a67130`
  cherry-picked onto `origin/agent/mac-pdf/fidelity` `a3536c2f` (main's
  crate + mac-pdf-2's /W fix); `tests/v2` 6/6 there. Pushed.
- Scratch (not committed): `…/scratchpad/pdf3/` — `gen.sh` (bundled
  `flashtex-render --v2` fixtures), `export.sh` / `extract.sh` (from-v2 +
  PDFKit `pdftext.swift` + Ghostscript txtwrite + `mdimport -t -d3`),
  `before/` (exports from the unchanged crate), `text-ab.v2.json` (the
  published `\text{a b}` display list from 647c50c5).
- Next commands if resumed: `cargo test --manifest-path crates/pdf/Cargo.toml`;
  `cd apps/mac && FLASHTEX_PDF_EXACT=$PWD/../../crates/pdf/target/release/flashtex-pdf-exact swift test --filter SearchableTextTests`.
- Resource pool: parent's Claude Max allocation (shared quota, no purchases).
  Load average 34 at start; `swift test` full suite not planned.

## Coverage audit (first step)

Already covered before this lane:

- `apps/mac/Tests/FlashTeXMacTests/ExactPDFExportTests.swift`:
  `testLoadedDisplayListExportsThroughTheExactRoute` — from-v2 on
  `display-list-v2-text.json`, PDFKit text *contains* words (>2 chars); no
  whitespace assertion, so `A V` vs `AV` passes either way.
- `crates/pdf/tests/v2.rs` (5): `real_pipeline_envelope_exports_glyphs_by_original_gid_at_exact_positions`
  (ToUnicode has `H` and `fi`; every glyph replays to its origin),
  `hand_built_envelope_joins_by_hmtx_advance_and_converts_rules_and_colour`,
  `display_list_advances_become_the_w_widths_where_hmtx_differs` (the `/W`
  = producer advance rule that keeps word boundaries in PDFKit), hash-form
  and refusal tests. None extracts text with any extractor.
- `tools/native-validation/mac-live/reports/20260912T110944Z.md` line 537:
  bundled (12:00) exporter, PDFKit extracted
  `Office fixtures The A V office fixed the fi ligature: office, bold, and café.`
  — a spurious `A V` (pre-`/W` fix build), recorded but not asserted.
- `coordination/mac-pdf-2.md` / `crates/pdf/docs/export-fidelity-hw1.md`:
  PDFKit `W ednesday` fixed by `/W`; `\f orall` italic-correction gap noted
  as not fixed; a `Tc` experiment did not change PDFKit.
- `crates/rendering-core/docs/handoffs/hw1-text-producer/searchable-acceptance`
  (commit 647c50c5, rendering-core's `pipeline_cff_probe` exporter, not
  `crates/pdf`): Poppler 26.01 default/`-layout` → `ab`, `-raw` → `a b`, for
  both the explicit-space-glyph (f261) and glue (65ce) producer outputs.
- `docs/evidence/*`: nothing on text extraction of the exact route.

Not covered: a written policy; exact-string extraction assertions for
`a b` / `ffi` / kerned pairs / math; any measurement of Spotlight or
Ghostscript; a census of ambiguous gaps in the export report.

## Result

- Policy: `docs/proposals/pdf-searchable-text.md` — promise (original GIDs
  at exact origins, per-glyph ToUnicode, `/W` = producer advances, word
  boundary = gap ≥ 150/1000 em for PDFKit/Spotlight/Poppler `-raw`),
  non-promises (no ActualText, no ligature decomposition, Poppler
  default/`-layout` on one-glyph lines), mechanism evaluation (gap-only vs
  space glyph vs `Tw`; gap-only kept — `Tw` cannot apply to CID fonts, a
  space glyph does not fix Poppler default per the published evidence and
  breaks the "every glyph is the list's" replay check), bounded
  occurrence-level rule with the exact `space` cluster request to the
  producer, measurements, remaining disagreements.
- Crate (`94a67130`): `V2Report.word_gaps` / `ambiguous_gaps`,
  `WORD_GAP_EM` 150 / `CHAR_GAP_EM` 30, notes naming ambiguous gaps, CLI
  `note: searchable text: …`; test
  `searchable_text_word_gaps_are_the_producers_and_are_counted` on
  `tests/fixtures/v2-text-a-b.json` (GH48's published list) and
  `v2-searchable-mixed.json`. PDF bytes unchanged (cmp on 7 fixtures).
- Mac: `Tests/FlashTeXMacTests/SearchableTextTests.swift` (3 tests, exact
  PDFKit strings: `a b`; `The AV office fixed a b: \f orallx, f (x) and
  ffi.`; `Office fixtures The AV office fixed the fi ligature: office,
  bold, and café.`) with fixtures
  `Fixtures/display-list-v2-searchable-{a-b,mixed}.json`. Uses the existing
  `ExactPDFExport.run` seam; no parent-retained file touched, no diffs
  needed.
- Measured: PDFKit/Spotlight break words from 100/1000 em, Ghostscript
  txtwrite from 250; the 99/1000 em math italic correction is the one
  real-world ambiguous gap (PDFKit `\f orallx`, `f (x)`); Ghostscript
  prints `Ï` for `ffi`/`fi` codes. `pdftotext` is not installed on this
  Mac (MacTeX has none); Poppler results are the published ones.

## Limitations

- Poppler not measured locally; `-raw`/default behaviour is from 647c50c5's
  files (rendering-core's exporter, not `crates/pdf`) and from Poppler's
  `TextOutputDev` constants. The one-glyph-line `ab` analysis is a source
  reading, consistent with both published outputs.
- The bundled producer typesets `\forall` and `\text{}` as source text, so
  a real `∀` glyph was not exercised; the GH48 case uses the published list.
- Full `swift test` and raster diff not run: load average 60–120; the PDF
  bytes are identical before/after so the raster cannot differ.

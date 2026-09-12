# mac-validation handoff — FT-010 native verification suite

Agent / task / branch: `mac-validation` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / FT-010 rev 1 / `agent/mac-validation/native-verification`
State: ready for integration
Owned paths: `tools/native-validation/` (this file is the only coordination file touched)
Main integrated through: `25a92a9` (branch base). origin/main is now `342e1e0`; the oracle run built its compiler from that revision, and nothing on main touches `tools/`, so no merge is needed yet.

Ready behavior:
- `tools/native-validation/run_all.sh` builds and tests the Mac app and the
  compiler from any two git refs in temporary detached worktrees, runs the
  runtime-v1 protocol checks and the UI capability probes, and writes
  `reports/report-<UTC>.md` (commands, exit codes, durations, pass/fail).
- `check_protocol.py` (stdlib): fixture comparison, Unicode UTF-8 span round
  trip, revision echo, source-edit sensitivity/determinism, error envelopes,
  20-request latency. `check_ui_capabilities.sh`: permission/automation probes
  with verbatim errors. `check_pdf_export.py`: white-vs-dark page background of
  an exported PDF. `expectations.md`: human checklist.
- Reference-oracle comparison (follow-up, issue #10): `oracle_compare.sh` /
  `oracle_compare.py` / `oracle_extract.swift` + `oracle-samples/` run pdflatex
  (BasicTeX, `/Library/TeX/texbin/pdflatex`, pdfTeX 1.40.29, TeX Live 2026) purely
  as a measuring stick against compiler `main`=`342e1e0` and `de1020c` through
  `flashtex-pdf` `c0f3837`. Evidence `reports/oracle-20260912T050958Z.md` (+ .json).
  Apples-to-apples variant (12pt, times, 1in margins, T1, parindent 0, no section
  numbers; C adds raggedright/no hyphenation): fixture-hello with de1020c matches to
  0.05 pt mean / 0.11 pt max in x, 0.46 pt in y (font descent), pages/MediaBox equal;
  wrap-sample word sequence equal, mean|dx| 9.0 pt (C) / 33.7 pt (B), max ~450 pt
  from end-of-line wraps, line-start agreement 0.98; demo: ours 3 pages vs 2,
  mean|dy| ~98 pt accumulated. Stripped for our compiler: exactly the
  `\documentclass{article}` line (it renders as the word `article` otherwise).
  Nothing was installed (times.sty/geometry.sty present; tlmgr would need sudo).
  Findings for the compiler owner: no kerning/ligatures (+0.1-0.3 pt per word,
  wraps `onto` that LaTeX keeps), +14.75 pt after headings and +5.9 pt per
  paragraph break versus LaTeX, `observation:` split into two items; `main`'s
  placeholder metrics make adjacent words touch (PDFKit merges them).
- Evidence: `tools/native-validation/reports/report-20260912T043142Z.md`
  against mac-shell `1c3ff13` (app code `fd2a26e`) and compiler-foundation
  `9f1033b`: cargo build/test PASS, swift build PASS, swift test PASS (34 tests,
  `RealCompilerTests` ran against the real binary), xcodebuild PASS, protocol
  checks 20 PASS / 1 FAIL / 6 INFO, latency min 0.26 / median 0.28 / max 0.41 ms.

Incomplete behavior:
- Visual checks (click-to-source, dark preview, banner, capture sheet) are not
  automated: the launched app has no on-screen window from the agent's
  `Background` launchd session, and Accessibility is not granted to the host
  (`osascript is not allowed assistive access. (-1728)`). `screencapture -x`
  works (Screen Recording granted) but failed twice earlier with `could not
  create image from display`, most likely display asleep; recorded as intermittent.
- `check_pdf_export.py` is validated on CoreGraphics PDFs I generated the same
  way the app does, not yet on a file exported by the app (save panel).
- Oracle comparison covers text only (no math samples yet); word boxes come
  from PDFKit text extraction on both sides, so a word is "matched" by
  normalised text alignment (accents/ligatures folded), not by source span.

Interface changes and required consumer actions: none. No edits to `apps/mac`,
`crates/compiler`, `protocol/`, or `docs/contracts/`.

Validation: see the report above; commands actually run are listed there with
exit codes. Full logs stayed in the scratch dir and are not committed.

Needs from others:
- Commander (FT-001 fixture owner): `protocol/fixtures/compile-result.json`
  item `"Hello FlashTeX."` has `end_byte: 14`; the text is 15 UTF-8 bytes, so
  `[0,14)` slices to `"Hello FlashTeX"`. Expected `end_byte: 15`. The compiler is
  correct (`[0,5)`, `[6,15)`); the fixture preview's click-to-source is one byte short.
- mac-claude-a (app owner): `ShellModel.exportPDF()` passes `dark: darkPreview`
  to `PDFExport.render`, so an export made with dark preview on has a
  `gray 0.16` page and white text. FT-010's expectation is that export stays
  white regardless of the toggle. Decide and, if changing, verify with
  `check_pdf_export.py` (expects `1.0 sc`).
- A human in the GUI session to run `expectations.md` sections 1-5 and attach
  observations, or grant Accessibility to the terminal host so UI scripting
  can be automated next.

Next action: none for this task unless the owners above ask for a re-run
(`tools/native-validation/run_all.sh` with the new refs) or the parent assigns
more checks. Re-run is about 45 s end to end on this machine.

Peer revisions reviewed and adaptations:
- mac-shell `fb22075` -> `1c3ff13` (`fd2a26e`: stale offsets refused unless
  rebased, debounced auto-compile, latency in banner, `File > Export PDF…`).
  Adaptation: `expectations.md` sections 3-4 rewritten for auto-compile,
  refusal note, and export; `check_pdf_export.py` added; export discrepancy reported.
- compiler-foundation `e7127fb` -> `9f1033b` (rustfmt only). Adaptation: none;
  all protocol checks unchanged and still pass except the fixture-owned FAIL.
- compiler-foundation `de1020c` (math, real Times AFM metrics, in-crate pdf.rs
  not exposed on the CLI) and mac-pdf `c0f3837` (fraction rules, Symbol math):
  read `layout.rs` (`MARGIN_PT` 72, `LINE_SPACING` 1.2, `PARAGRAPH_GAP_PT` 6,
  heading sizes 17/14) and the pdf README to choose the oracle variants and to
  attribute the deltas; both are labelled separately in the oracle report.
- main `25a92a9` -> `342e1e0` (integration commit; compiler on main = `9f1033b`
  content). Adaptation: oracle run uses main's compiler as the `main` label.
- main `25a92a9`: base of this branch; nothing under `tools/` on main.

Resource state: Claude Max (20x) plan on mac-m1max-a, allocation
`claude-mac20x-validation` under parent mac-claude-a; one subagent session, no
children spawned, no paid API calls. Timebox 45 min for FT-010 (elapsed about 20 min) plus about 35 min for the oracle follow-up.

Updated: 2026-09-12T05:15Z

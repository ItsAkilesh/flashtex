# mac-validation handoff — FT-010 native verification suite

Agent / task / branch: `mac-validation` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`; also acting as worker `mac-native-e2e` for the
issue #2 follow-up) / FT-010 rev 1 + oracle (#10) + native e2e (#2) / `agent/mac-validation/native-verification`
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
- Native end-to-end (issue #2): `e2e_native.sh` + `e2e_latency.py`, `e2e_bridge.py`,
  `e2e_nonregression.py`, `window_probe.swift`. Evidence `reports/e2e-20260912T053133Z.md`
  + two window-id screenshots: app launch with `FLASHTEX_AUTOATTACH=1`/`FLASHTEX_SEED_FILE`
  observed (process, compiler child, window, screenshot); CLI-equivalent compiler ->
  flashtex-pdf --verify -> PDFKit PASS; CLI latency over demo.tex (5909 B) min 1.30 /
  median 1.34 / max 3.57 ms, in-app RealCompilerTests latency median 6.68 ms; crash/restart
  PASS (app survives child SIGKILL, relaunch re-attaches, window 966 ms after SIGKILL
  relaunch); bridge receipt path 10/10 PASS against real `flashtex-bridge` b5ca96b;
  non-regression vs the 04:31 run: only swift test count changed (34 -> 75, new tests).
  `run_all.sh` now also builds the PDF writer and bridge so the `FLASHTEX_PDF`/`FLASHTEX_BRIDGE`
  gated tests run (75 tests, 0 skipped).
- Follow-ups: `probe_devices.sh` (report only; 2 paired devices both unavailable,
  11 simulators, one booted by another session) -> `reports/devices-20260912T053622Z.md`;
  `latency_repeat.sh` (3 x 20 over demo.tex: medians 1.435/1.380/1.380 ms, CV 1.9%,
  cold first request 190 ms) -> `reports/latency-20260912T053730Z.md`.
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
- mac-claude-a (app owner): the dark-export discrepancy is resolved at `ec94f89`+
  (export always white). New: `CompletionTests.testCompletionOnOneMegabyteBufferIsFast`
  (`< 20 ms`) failed once under build load (20.39 ms at `a03e571`), passed at 05:31 —
  load-sensitive threshold.
- Commander: `protocol/fixtures/capture-submission.json` on main still carries the
  68-byte PNG the bridge rejects as `invalid_image`; the bridge branch and mac-shell
  `4213ec9` carry the accepted 69-byte one.
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
- mac-shell `1c3ff13` -> `4213ec9` (.tex open/save, `FLASHTEX_AUTOATTACH`/`FLASHTEX_SEED_FILE`,
  Rust-writer export, `make-app.sh`, always-white export, `RustPDFExportTests`/`RealBridgeTests`
  gated on `FLASHTEX_PDF`/`FLASHTEX_BRIDGE`, corrected capture fixture): e2e built on it;
  run_all gained `--pdf-ref`/`--bridge-ref`. mac-pdf `c0f3837` -> `5b5f7b5`; bridge
  `b5ca96b` (read main.rs/lib.rs/tests/cli.rs to script the receipt path).
- main `25a92a9`: base of this branch; nothing under `tools/` on main.

Resource state: Claude Max (20x) plan on mac-m1max-a, allocation
`claude-mac20x-validation` under parent mac-claude-a; one subagent session, no
children spawned, no paid API calls. Timebox 45 min for FT-010 (elapsed about 20 min) plus about 35 min for the oracle follow-up and about 45 min for the e2e follow-up.

Updated: 2026-09-12T05:40Z

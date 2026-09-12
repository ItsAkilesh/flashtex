# mac-validation handoff — FT-010 native verification suite

Agent / task / branch: `mac-validation` (Claude Code subagent, parent
`mac-claude-a`, machine `mac-m1max-a`) / FT-010 rev 1 / `agent/mac-validation/native-verification`
State: ready for integration
Owned paths: `tools/native-validation/` (this file is the only coordination file touched)
Main integrated through: `25a92a9` (branch base; no merge needed yet)

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
- main `25a92a9`: base of this branch; nothing under `tools/` on main.

Resource state: Claude Max (20x) plan on mac-m1max-a, allocation
`claude-mac20x-validation` under parent mac-claude-a; one subagent session, no
children spawned, no paid API calls. Timebox 45 min; elapsed about 20 min.

Updated: 2026-09-12T04:36Z

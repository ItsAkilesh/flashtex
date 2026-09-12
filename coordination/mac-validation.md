# mac-validation handoff — mac-live native acceptance runner

- Updated UTC: 2026-09-12T11:35:00Z
- Agent / parent / machine alias: `mac-validation` (Claude Code subagent) / `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: lane "Native typing-to-paint and helper
  crash/restart acceptance on current integrated main" plus follow-ups "Full packaged
  app end-to-end capture/accept/export cycle" and "Record independent actual
  session/binary/source provenance" / gates in `tools/native-validation/mac-live/thresholds.json`
  / `tools/native-validation/mac-live/`, this file, `coordination/agents/mac-validation.json`
- Branch / code revision / main integrated through: `agent/mac-validation/mac-live-2`
  (based on `origin/agent/mac-claude-a/mac-shell` 6f4ee94) / tip in `git log` / no main merge
  needed (the branch adds only `tools/native-validation/mac-live` and these two files); the
  runner builds helpers from `origin/main` at run time (SHA recorded in each report)
- State: ready for integration (runner + first evidence run); see "Incomplete".
- Ready behavior and evidence: `tools/native-validation/mac-live/run.sh` (bash + python3,
  stdlib) builds five helpers in a shared clone pinned at current `origin/main`, the Mac
  app (release) from a clone pinned at the branch under test, runs the unmodified `tools/typing-bench/run.sh`
  (fixture/demo/body60k × 30 ms/0 ms; direct worker and preview-controller routes), packages `FlashTeX.app` with `make-app.sh`, runs
  the unmodified `launch-check.sh` with `FLASHTEX_NO_ACTIVATE=1` through a PATH `open`
  shim (private bridge store, transcript), drives the bundled helpers through the capture
  cycle (submit -> `provider_disabled` -> offline fixture proposal review -> durable
  edit-ledger apply -> compile -> `flashtex-pdf --verify`), and writes
  `reports/<UTC>.md` with every number, hash, SHA, machine/toolchain version, session
  and exact commands, asserted against explicit thresholds. RESULTS: see the latest
  `tools/native-validation/mac-live/reports/*.md` (summary in the section below).
- Incomplete behavior / blockers / needs from others: the capture cycle drives the
  packaged app's helpers over their real protocols, not the SwiftUI review sheet
  (Accessibility not granted; window never activated). No provider call (no
  `--enable-grok`, no key) — the reviewed proposal is a fixture. The shell has no
  automatic child relaunch, so "restart" means app survival + logged exit. The branch
  under test may not contain current main (recorded per run). `launch-check.sh` `pkill`s
  any running `FlashTeX.app`; the runner refuses that step while one is running.
- Interface changes / consumer actions: none (no product source edited).
- Reviewed peer revisions / resulting adaptations: `origin/main` and
  `origin/agent/mac-claude-a/mac-shell` tips at run time are recorded in each
  report's provenance table; the runner adapts by archiving whatever those refs are.
- Validation commands / results / artifact paths: `tools/native-validation/mac-live/run.sh`
  -> `tools/native-validation/mac-live/reports/<UTC>.md` (+ `<UTC>/` raw JSON, logs,
  launch-check evidence, capture transcript, exported PDF).
- Exact deadline UTC / remaining time / integration reserve: no automatic deadline
  (PROJECT.md override); continuous.
- ETA remaining, optimistic / likely / pessimistic / confidence: runner complete; a full
  run takes ~10–15 min warm (helpers/app cached by SHA), ~25 min cold.
- Resource pool / allocation ID / maximum: Claude Max 20x on mac-m1max-a via parent
  `mac-claude-a` / `claude-mac20x-validation` / shared quota, no purchases.
- Confirmed spend / estimated usage / in-flight reservation / remaining: unknown (shared
  account quota; no API spend; no paid network calls).
- Billing evidence / freshness / unknowns: none beyond the shared plan; unknown.
- Child tasks and their deducted allocations: none.
- Dirty files / unpushed work / running jobs: see `git status` on the branch; scratch
  builds under `tools/native-validation/mac-live/build/` (gitignored).
- Decisions / failed approaches / linked findings: (1) `launch-check.sh` cannot be
  edited (parent-owned), so `FLASHTEX_NO_ACTIVATE=1` is injected via a PATH shim on
  `open`; (2) the UI capture flow needs `NSOpenPanel`/pairing confirmation, both
  impossible without Accessibility, so the cycle targets the bundled helpers with the
  shell's exact message order; (3) `git archive` scratch trees broke the bench script's
  `git rev-parse` (set -e) and misattributed SHAs, so pinned shared clones are used instead; (4) bundled binaries are compared by a signature-masked Mach-O content hash because `make-app.sh` re-signs ad hoc.
- Exact next action or command: re-run `tools/native-validation/mac-live/run.sh` after
  each integration of `apps/mac` or the helper crates; compare `reports/`.
- Resume reading list: `tools/native-validation/mac-live/README.md`, latest report,
  `apps/mac/scripts/launch-check.sh`, `tools/typing-bench/run.sh`,
  `docs/contracts/transfer-v1.md`.

## Latest run summary — `tools/native-validation/mac-live/reports/20260912T110944Z.md` (mac-live-3)

- Verdict PASS, 230/230 gates. Helpers from `origin/main` `1884986fe54ba3ff8f602b1e7711c9cee324cd25`
  (>= 60c40c1; completed-snapshots-v1 helper); app from `origin/agent/mac-claude-a/mac-shell`
  `7ccbd7e9d330479b3f35927bbf968316270b7495` (>= 6f4ee94); `flashtex-render` from
  `origin/agent/mac-render-pipeline/unified` `6e69661`; `flashtex-pdf-exact` from
  `origin/agent/mac-pdf/v2-adapter` tip `20e5277` (the branch moved past 654f626);
  `flashtex-project-files` from the app branch. `uptime` at start load 6.09; every bench cell
  waited for the bench's quiet-load condition (`--quiet-load 8 --quiet-wait 120`) and ran at
  1-minute load 5.9–7.9, so all latency gates were applied; end load 17.6.
- Keystroke -> paint p50/p95 ms — `flashtex-compiler`: fixture 23/41, 21/39; demo 23/42, 25/41;
  body60k **94/119** (30 ms), **97/113** (0 ms): body60k now meets the 200 ms project target on
  both p50 and p95 with the current main compiler. `flashtex-render`: fixture 22/40, demo 41/46,
  body60k 132/151. `flashtex-preview-controller` (hold-until-preview): fixture 47/62, demo
  55/116, body60k 164/206. `FLASHTEX_COMPLETED_SNAPSHOTS=1`: fixture 43/52, demo 48/69, body60k
  220/258 (30 ms) / 185/247 (0 ms); analyze.py: body60k 30 ms 107 historical + 19 current paints,
  174/26 keystrokes first shown by a historical/current frame, historical lag p50/p95 222/259 ms,
  keystroke -> current paint p50/p95/max 572/2900/3188 ms; demo/fixture 1–4 historical frames
  only. 0 unpainted everywhere, no retry pass needed.
- Packaged app (six bundled binaries byte-identical to the builds after masking the re-sign;
  `components.json` SHAs match): launch-check PASS (compiler + bridge killed, app survives,
  clean quit, window on screen); capture cycle 31/31; render attach headless PASS (`preview face:
  latin-modern` 0.11 s); exact export PASS (PDFKit 1 page, text incl. "café").
- New cycles: (c) `FLASHTEX_OPEN_WINDOW=a11y-help` -> window "Accessibility Help" 1800x900 px
  captured by id (`open-window-a11y-help/a11y-help.png`), `nearby` -> "Nearby Companion"
  924x1188 px (`open-window-nearby/nearby.png`), never activated. (e) worker auto-relaunch with
  the bench typing: kills 1–3 relaunched after 0.29 / 1.15 / 3.06 s (new child pids, a later
  keystroke recompiled at the same instant), kill 4 -> `worker relaunch limit reached` after
  0.06 s, no relaunch, app alive, bench exited 0. (a) multi-file: `project: opened chapter.tex
  (46 bytes, helper)`, typed text durable in chapter.tex's ledger document (r17, 62 bytes), not
  in main.tex's (r1), both files on disk untouched. (a/b) ⌘S routing, quit-save, reviewed reload
  and conflict refusal: branch XCTests (ProjectDocumentsTests, DocumentFilesTests,
  DocumentFilesControllerTests, HistoricalPreviewTests, ShellModelWorkerTests/
  testCrashedWorkerIsRelaunchedWithBoundedBackoff) 43 passed / 0 failed / 0 skipped against the
  real helpers, the seven named tests gated individually.
- Findings for owners: (1) `tools/typing-bench/run.sh` passes its whole environment to every
  cell, so an exported `FLASHTEX_PREVIEW_CONTROLLER` makes the direct-worker cells attach the
  helper too (every "compiler" cell reported `flashtex-preview-controller` in the first
  mac-live-3 run); the runner now relies on the bench's "this checkout" paths instead — the
  bench should unset it for worker cells. (2) `make-app.sh` `components.json` short SHAs are now
  8 characters (git auto-length); comparisons must be prefix-based. (3) `FLASHTEX_TRANSCRIPT`
  writes every compile request in full (8 MB for one relaunch cycle) — not suitable as routine
  evidence. (4) The historical route's keystroke -> *current* paint on body60k reaches 2.9 s
  p95 while historical frames arrive at 259 ms p95 — the trade-off the feature makes explicit.
- Note: the Markdown report was re-rendered from the unchanged run directory by `lib/report.py`
  at this commit (relaunch table column keys and the load-gating note were fixed after the run;
  raw JSON, logs and `env.json` file hashes are those of the run-time scripts, commit 89a272c).

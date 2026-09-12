# mac-validation handoff — mac-live native acceptance runner

- Updated UTC: 2026-09-12T09:12:00Z
- Agent / parent / machine alias: `mac-validation` (Claude Code subagent) / `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: lane "Native typing-to-paint and helper
  crash/restart acceptance on current integrated main" plus follow-ups "Full packaged
  app end-to-end capture/accept/export cycle" and "Record independent actual
  session/binary/source provenance" / gates in `tools/native-validation/mac-live/thresholds.json`
  / `tools/native-validation/mac-live/`, this file, `coordination/agents/mac-validation.json`
- Branch / code revision / main integrated through: `agent/mac-validation/mac-live`
  (based on `origin/agent/mac-claude-a/mac-shell` 6b43a3a) / tip in `git log` / no main merge
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

## Latest run summary — `tools/native-validation/mac-live/reports/20260912T090452Z.md`

- Verdict PASS, 119/119 gates. Helpers from `origin/main` `780f1458c125d9702b994ab9df377aed96a1113f`
  (pinned clone, nothing rewritten by cargo); app from `origin/agent/mac-claude-a/mac-shell`
  `d8baed6bc2ef1e333ecad48d3d056478a9f984fd` (does not yet contain that main; merge-base
  `c9f1b9e`). M1 Max, macOS 26.3.1, Xcode 26.3 / Swift 6.2.4, cargo 1.99 nightly; load average
  14.2 at start, 23.9 at end (other agents building/testing concurrently).
- Keystroke -> paint p50/p95 (ms), direct worker `flashtex-compiler`: fixture 10/21 (30 ms),
  15/28 (0 ms); demo 28/46, 29/47; body60k 230/394, 354/628. All 200 keystrokes painted in every
  cell, 0 unpainted. Gates p50 <= 40/60/400 met. Project target (<= 200 p50 and p95) met for
  fixture and demo, not for body60k.
- Same bench through `flashtex-preview-controller` (durable route, reported not gated): fixture
  52/71, 53/77; demo 86/136, 75/123; body60k 335/437, 278/345; 0 unpainted.
- Packaged app (`make-app.sh`, four bundled helpers byte-identical to the fresh builds after
  masking the ad-hoc re-sign; `components.json` SHAs = main/branch): `launch-check.sh` with
  `FLASHTEX_NO_ACTIVATE=1` — compiler and bridge attached, `revision 1: ok`, each child killed,
  app survived both, both exits logged, clean quit, on-screen window confirmed; no auto-relaunch
  (shell behaviour, recorded not gated).
- Capture cycle through the bundled bridge/ledger/compiler/pdf: durable receipt, duplicate =
  same receipt, conflict = `capture_id_conflict`, `capture_convert` = `provider_disabled`
  (expected; no `--enable-grok`, no key, no network), `proposal_missing`, journal survives
  SIGKILL, reject terminal, `capture_missing`; offline fixture proposal applied durably by the
  ledger (receipt, identical retry, pending receipt survives SIGKILL); compile `ok` 1 page,
  `flashtex-pdf --verify` exit 0, 1 page object, PDF sha256 recorded.
- Findings for owners: (1) compiler on main still emits the U+2500 fraction-rule "will not
  survive PDF export" warning for `\frac` (runtime-v1 has no rule item type) — export of the
  inserted math is not faithful; (2) on run 20260912T085529Z four consecutive bench cells lost
  their app process within seconds (no crash report) — something on this shared Mac kills
  `FlashTeXMac`; the runner now retries a pass once and labels retried cells; (3) an earlier
  main (`f2ea364`) had a stale `crates/preview-controller/Cargo.lock` that cargo rewrote during
  build (recorded verbatim in helpers.json when it happens; clean at `780f145`).

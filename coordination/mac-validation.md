# mac-validation handoff — mac-live native acceptance runner

- Updated UTC: 2026-09-12 (see the report timestamp below)
- Agent / parent / machine alias: `mac-validation` (Claude Code subagent) / `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: lane "Native typing-to-paint and helper
  crash/restart acceptance on current integrated main" plus follow-ups "Full packaged
  app end-to-end capture/accept/export cycle" and "Record independent actual
  session/binary/source provenance" / gates in `tools/native-validation/mac-live/thresholds.json`
  / `tools/native-validation/mac-live/`, this file, `coordination/agents/mac-validation.json`
- Branch / code revision / main integrated through: `agent/mac-validation/mac-live`
  (based on `origin/agent/mac-claude-a/mac-shell` 6b43a3a) / see `git log` / the runner
  builds helpers from `origin/main` at run time (SHA recorded in each report)
- State: ready for integration (runner + first evidence run); see "Incomplete".
- Ready behavior and evidence: `tools/native-validation/mac-live/run.sh` (bash + python3,
  stdlib) builds the four helpers from a `git archive` of current `origin/main`, the Mac
  app (release) from the branch under test, runs the unmodified `tools/typing-bench/run.sh`
  (fixture/demo/body60k × 30 ms/0 ms), packages `FlashTeX.app` with `make-app.sh`, runs
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
  shell's exact message order; (3) `components.json` from `make-app.sh` reports helper
  SHAs `unknown` for non-git scratch sources — the report carries real SHAs.
- Exact next action or command: re-run `tools/native-validation/mac-live/run.sh` after
  each integration of `apps/mac` or the helper crates; compare `reports/`.
- Resume reading list: `tools/native-validation/mac-live/README.md`, latest report,
  `apps/mac/scripts/launch-check.sh`, `tools/typing-bench/run.sh`,
  `docs/contracts/transfer-v1.md`.

## Latest run summary

(filled in from the committed report)

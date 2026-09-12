# mac-validation handoff — mac-live native acceptance runner

- Updated UTC: 2026-09-12T10:05:00Z
- Agent / parent / machine alias: `mac-validation` (Claude Code subagent) / `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: lane "Native typing-to-paint and helper
  crash/restart acceptance on current integrated main" plus follow-ups "Full packaged
  app end-to-end capture/accept/export cycle" and "Record independent actual
  session/binary/source provenance" / gates in `tools/native-validation/mac-live/thresholds.json`
  / `tools/native-validation/mac-live/`, this file, `coordination/agents/mac-validation.json`
- Branch / code revision / main integrated through: `agent/mac-validation/mac-live-2`
  (based on `origin/agent/mac-claude-a/mac-shell` 40d53b7) / tip in `git log` / no main merge
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

## Latest run summary — `tools/native-validation/mac-live/reports/20260912T095354Z.md` (mac-live-2)

- Verdict PASS, 163/163 gates. Helpers from `origin/main` `1d25ed009e7007b65258d4b684421cb9aadc59c2`
  (contains compiler `e75741e` direct serializer and helper `4db4a7a`; pinned clone, nothing
  rewritten by cargo); app from `origin/agent/mac-claude-a/mac-shell`
  `35b4e121df7fafd509ec51ce949343f5c8fa320a` (>= 40d53b7); `flashtex-render` from
  `origin/agent/mac-render-pipeline/unified` `4888a67`; `flashtex-pdf-exact` from
  `origin/agent/mac-pdf/v2-adapter` `654f626`. `uptime` at start: load 8.54; before the bench
  passes 38.39 / 49.18; at end 48.39 — latency gates therefore NOT applied (rule: 1-minute load
  > 10 -> report only), numbers reported.
- Keystroke -> paint p50/p95 ms, `flashtex-compiler`: fixture 22/39, 20/34; demo 23/40, 43/70;
  body60k 149/251 (30 ms), 138/293 (0 ms) — body60k p50 is now under the 200 ms target with the
  new serializer even at load 38; p95 not yet. `flashtex-render` (direct route): fixture 21/38,
  22/41; demo 46/85, 43/60; body60k 194/410, 326/432. `flashtex-preview-controller`: fixture
  51/84, 44/68; demo 65/213, 70/111; body60k 249/383, 227/368. 0 unpainted everywhere; the first
  compiler+render pass lost 9 of 12 cells to an external kill and the retry pass supplied them
  (labelled "(retry)").
- Packaged app with six binaries (four helpers + render + pdf-exact), all byte-identical to the
  fresh builds after masking the ad-hoc re-sign; `components.json` SHAs == main / branch /
  render / pdf-exact SHAs. `launch-check.sh` with `FLASHTEX_NO_ACTIVATE=1`: compiler and
  bridge attached, both killed, app survived, both exits logged, clean quit, window on screen.
- Render pipeline attach, headless (`FLASHTEX_COMPILER=<bundled flashtex-render>`): `launched …
  (preview face: latin-modern)` after 0.11 s, `status: attached:`, `revision 1: ok` after 0.22 s,
  `flashtex-render` child under the app pid, app alive, never activated.
- Exact export: bundled `flashtex-pdf-exact from-v2 display-list-v2-text.json --font-dir
  apps/mac/Fonts` exit 0, 103985-byte PDF (sha256 `81dac410…`), PDFKit 1 page 612x792 pt, text
  "Office fixtures The A V office fixed the fi ligature: office, bold, and café." Tool notes the
  render-pipeline font-hash deviation (SHA-256(bytes||face_index) vs SHA-256(bytes)).
- Capture cycle through the bundled bridge/ledger/compiler/pdf: 31/31 as before
  (`provider_disabled` expected refusal, journal and ledger survive SIGKILL, offline proposal
  applied, compile ok, `flashtex-pdf --verify` exit 0).
- Findings for owners: (1) something on this shared Mac kills `FlashTeXMac` bench instances
  (twice today; no crash report) — the retry pass covers it but a `pkill -f FlashTeX` somewhere
  should be found; (2) compiler main still warns that U+2500 fraction rules do not survive
  `flashtex-pdf` export; (3) `flashtex-pdf-exact` reports the render-pipeline font sha256
  deviation on every font of the fixture.

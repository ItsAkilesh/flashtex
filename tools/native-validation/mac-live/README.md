# mac-live: native typing-to-paint, helper crash/restart and packaged capture-cycle acceptance

Owner: `mac-validation` (Claude Code subagent, parent `mac-claude-a`, machine
`mac-m1max-a`). Status: runnable, first evidence committed under `reports/`.
Created 2026-09-12. Independent of the product sources: nothing under
`apps/mac`, `crates/` or `tools/typing-bench` is modified; the runner only
reads them from git and executes the shipped scripts.

`run.sh` is one repeatable acceptance run (bash + python3, no new dependencies):

1. **Helpers from integrated main.** A shared clone of the repository pinned
   (detached) at the exact `<main-ref>` commit — a clean tree with no local
   edits and a truthful `git rev-parse HEAD`, the same isolation
   `tools/typing-bench/run.sh` gets from `git archive` — then `cargo build
   --release` for `flashtex-compiler`, `flashtex-pdf`, `flashtex-bridge`,
   `flashtex-edit-ledger` and `flashtex-preview-controller` (offline first,
   crates.io only as a fallback). The git SHA, as-shipped sha256 and
   signature-removed sha256 of each binary are recorded.
2. **App from the branch under test.** A second pinned clone at the branch
   commit; `swift build -c release`. Default branch
   `origin/agent/mac-claude-a/mac-shell`, default main `origin/main`.
3. **Typing bench.** The unmodified `tools/typing-bench/run.sh` from that branch
   runs `fixture`, `demo`, `body60k` at 30 ms and 0 ms with the step-1 compiler,
   recording keystroke -> paint p50/p95/p99/max, paints, coalesced and unpainted
   keystrokes, compile and render-pass times — on the direct worker route
   (producer `compiler`), with `flashtex-render` as a second direct producer
   (the bench's own `render` producer, `FLASHTEX_RENDER` pointing at the
   scratch build below), and on the durable helper route (producer
   `controller`: `FLASHTEX_PREVIEW_CONTROLLER=<flashtex-preview-controller>`
   exported, private `FLASHTEX_CONTROLLER_LEDGER_ROOT`). `uptime` is sampled
   right before every pass; when the 1-minute load average is above 10 the
   latency gates for that pass are reported only, never applied.
3b. **Optional bundled routes.** `flashtex-render` from
   `origin/agent/mac-render-pipeline/unified` (`--render-ref`) and
   `flashtex-pdf-exact` from `origin/agent/mac-pdf/v2-adapter`
   (`--pdf-exact-ref`), each built in its own pinned shared clone, are handed
   to `make-app.sh --render … --pdf-exact …`. `--skip-extras` leaves them out.
4. **Package + launch check.** `apps/mac/scripts/make-app.sh` with the four
   step-1 helpers, then the unmodified `apps/mac/scripts/launch-check.sh`
   (compiler and bridge children attached, each killed in turn, app survives,
   both exits logged, clean quit). `launch-check.sh` calls `open` without
   `FLASHTEX_NO_ACTIVATE`, so `lib/open-shim/open` is put first on `PATH` and
   adds `--env FLASHTEX_NO_ACTIVATE=1` (window ordered back, never activated),
   a private `FLASHTEX_BRIDGE_STORE` and a `FLASHTEX_TRANSCRIPT`. The run
   refuses this step while another `FlashTeX.app` is running, because
   `launch-check.sh` would `pkill` it (`--force` overrides).
5. **Packaged capture cycle.** `lib/capture_cycle.py` drives the helpers
   *inside the produced bundle* over their real JSON Lines protocols in the
   shell's order: `document_open` -> `destination_pin` -> `capture_submit`
   (`protocol/fixtures/capture-submission.json`) -> duplicate and conflicting
   retries -> `capture_convert` (refused `provider_disabled`: no
   `--enable-grok`, no key — the expected refusal) -> `capture_status` ->
   `capture_prepare_insert` (refused `proposal_missing`) -> SIGKILL + restart on
   the same journal -> `capture_reject` (terminal). Offline proposal review of
   `fixtures/capture-proposal.json` (bounds checked, explicitly not a provider
   result), durable insertion through `flashtex-edit-ledger` (`initialize`,
   `apply`, identical retry, `status`, SIGKILL + restart), then `compile` of the
   post-insertion document with `flashtex-compiler` and export via
   `flashtex-pdf --out … --verify`.
5b. **Packaged render pipeline + exact export.** `lib/app_features.py
   render-attach` executes the bundle's `FlashTeX` directly with
   `FLASHTEX_AUTOATTACH=1 FLASHTEX_NO_ACTIVATE=1 FLASHTEX_COMPILER=<bundled
   flashtex-render>` (what File > Attach Render Pipeline resolves to) and asserts
   `launched … (preview face: latin-modern)`, `status: attached:` and
   `revision 1: ok` in `FLASHTEX_LOG` plus a `flashtex-render` child under the
   app pid; `exact-export` runs the bundled `flashtex-pdf-exact from-v2` on
   `apps/mac/Tests/FlashTeXMacTests/Fixtures/display-list-v2-text.json` with
   `--font-dir apps/mac/Fonts` and reads the PDF back through PDFKit
   (`lib/pdfkit_probe.swift`, compiled on the fly): page count and text.
6. **Report.** `lib/report.py` writes `reports/<UTC>.md` with every number,
   binary hash, source SHA, machine/OS/Xcode/Swift/cargo version, the driving
   agent session and the exact commands (`reports/<UTC>/commands.log`), and
   asserts `thresholds.json`. Exit 0 only when every gate passed.

## Usage

```sh
tools/native-validation/mac-live/run.sh                      # full run, defaults
tools/native-validation/mac-live/run.sh --branch origin/agent/x/y --main-ref origin/main
tools/native-validation/mac-live/run.sh --skip-bench --skip-launch   # cycle only
tools/native-validation/mac-live/run.sh --skip-controller            # direct worker route only
tools/native-validation/mac-live/run.sh --intervals "30" --seeds "demo"
FLASHTEX_MAC_LIVE_SESSION=<session url> FLASHTEX_MAC_LIVE_AGENT=<agent id> tools/native-validation/mac-live/run.sh
```

Options: `--no-fetch`, `--rebuild` (discard the scratch helper/app builds),
`--force` (run launch-check even if a FlashTeX.app is running), `--work <dir>`
(scratch root; default `build/`, gitignored), `--session`, `--agent`.
Scratch builds are keyed by SHA (`build/helpers/<main sha>`, `build/app/<branch
sha>`, shared `build/cargo-target`) and reused across runs.

## Thresholds (`thresholds.json`)

Gates fail the run; targets are reported only.

| section | gate |
|---|---|
| build | helpers built from main (pinned clone, clean); app built; bundle has `FlashTeX`, `flashtex-compiler`, `flashtex-pdf`, `flashtex-bridge`, `flashtex-edit-ledger`; bundled helpers and app carry the freshly built object code (signature-masked Mach-O content sha256 equal; make-app.sh re-signs the bundle ad hoc); `components.json` SHAs equal the main/branch SHAs |
| typing-bench/compiler | all six cells present; per cell: `unpainted == 0`, typing budget not exhausted, `typed == keystrokes == script_keystrokes` (200), `paints >= 1`, producer reported as `flashtex-compiler`; keystroke -> paint p50 <= 40 ms (`fixture`), <= 60 ms (`demo`), <= 400 ms (`body60k`) |
| typing-bench/render | same correctness gates, producer reported as `flashtex-render`; latency reported (not gated) |
| typing-bench/controller | same correctness gates, producer reported as `flashtex-preview-controller`; latency is reported (not gated) against the project target |
| typing-bench (all) | p50 gates are applied only when the 1-minute load average before the pass is <= 10; above that the gate row says "NOT applied" and the numbers are reported only |
| render-attach | bundled `flashtex-render` present; `preview face: latin-modern`, `status: attached:`, `revision 1: ok` in `FLASHTEX_LOG`; `flashtex-render` child under the app pid; app alive after the wait; launched with `FLASHTEX_NO_ACTIVATE=1` |
| exact-export | `flashtex-pdf-exact from-v2` exit 0; PDF written; PDFKit page count == fixture pages; PDFKit text contains "Office fixtures", "office", "bold", "caf" |
| build (extras) | `flashtex-render` / `flashtex-pdf-exact` built from their refs (pinned clone HEAD == SHA); bundled copies carry the built object code; `components.json` `render` / `pdf_exact` SHAs == their branch SHAs |
| typing-bench target (not a gate) | project target typing-to-visible p50 and p95 <= 200 ms, reported per cell as met / not met |
| launch-check | zero `FAIL:` lines; compiler attached, `attached:` and `revision 1: ok` logged, app survives compiler kill, `worker exited (` logged; bridge attached, `bridge: attached:` logged, app survives bridge kill, `bridge exited (` logged; clean quit; launched with `FLASHTEX_NO_ACTIVATE=1` |
| launch-check informational | on-screen window confirmed (CGWindowList); child re-attached after kill (the shell has no auto-relaunch) |
| capture-cycle | every driver check (durable receipt, duplicate/conflict, `provider_disabled`, `proposal_missing`, journal survives SIGKILL, reject terminal, `capture_missing`, ledger receipt + identical retry + pending receipt + survives SIGKILL, compile ok/recovered with >= 1 page, `flashtex-pdf --verify` exit 0, PDF header/trailer, PDF page objects == compiled pages); no provider enablement |

## Layout

- `run.sh` — orchestrator; `lib/report.py` — report + gates; `lib/launch_summary.py`
  — parses launch-check evidence; `lib/capture_cycle.py` — packaged capture
  cycle; `lib/app_features.py` — headless render-pipeline attach and exact
  export checks; `lib/pdfkit_probe.swift` — PDFKit page count/text probe;
  `lib/hashes.py` — as-shipped, signature-removed and signature-masked sha256;
  `lib/open-shim/open` — adds `--env` values to `open`.
- `fixtures/capture-proposal.json` — offline `capture_proposal` fixture.
- `reports/<UTC>.md` + `reports/<UTC>/` — committed evidence: `env.json`,
  `helpers.json`, `app.json`, `bundle.json`, `steps.jsonl`, `commands.log`,
  `logs/`, `typing-bench/` and `typing-bench-controller/` (bench evidence + raw JSON), `launch-check.md/json`,
  `launch-transcript.jsonl`, `capture-cycle.json` + `capture-cycle/`
  (transcript, compile result, exported PDF).

## Limitations

- The typing bench inserts text programmatically; the OS keyboard hop and
  display scan-out are not measured (see the bench's own methodology). One run
  per cell; other agents' builds/tests on the same machine are recorded (load
  average, process list) but not controlled.
- The shell has no automatic child relaunch: "restart" in the launch check means
  the app keeps running and reports the exit; reattachment is a user action.
- The capture cycle exercises the bundled helpers and the shell's protocol
  sequence, not the SwiftUI review sheet (no Accessibility, no activation). The
  shell's review/insertion code is covered by `swift test` on the branch.
- No provider call is made; the reviewed proposal is a fixture. Real conversion
  needs `--enable-grok` and the Mac credential adapter, outside this runner.
- `make-app.sh` re-signs the bundle ad hoc, so bundled binaries are compared to
  the built ones by a signature-masked Mach-O content hash (`lib/hashes.py`:
  bytes before the `LC_CODE_SIGNATURE` blob with the two header fields a
  re-sign rewrites zeroed); `codesign --remove-signature` output is recorded
  too but is not stable across re-signs. `components.json` SHAs come from the
  pinned clones and are gated against the main/branch SHAs.
- Latency numbers depend on machine load: other agents build and test on this
  Mac concurrently. `uptime` is recorded at start, before every bench pass and
  at the end; latency gates are not applied when the 1-minute load average
  before a pass exceeds 10 (the report says so), and are never hidden otherwise.
- The render-attach check proves the menu item's resolution path and the
  producer face switch through the app's own log, not the menu click itself.

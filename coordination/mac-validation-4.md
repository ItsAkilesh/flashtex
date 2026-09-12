# mac-validation-4 handoff — typing latency attribution on the packaged app

- Updated UTC: 2026-09-12T16:53:00Z
- Agent / parent / machine alias: `mac-validation-4` (Claude Code subagent) / `mac-claude-a` / `mac-m1max-a`
- Task: Commander replenishment (issue #2 comment 5646989044), native validation item 4.
  Current task: typing latency on the CURRENT packaged binaries (small / medium / large
  documents; direct v1, direct v2, helper routes) with the stage dominating the 200 ms
  budget attributed per route. Follow-up 1: rerun the mac-live acceptance on the packaged
  app, PASS/FAIL deltas vs report 20260912T110944Z. Follow-up 2: repeatable quiet-window
  script (waits for 1-min load < 8, runs every cell unattended, `uptime` per cell).
- Owned paths: `tools/native-validation/mac-live/**`, `coordination/mac-validation*`.
  No product source edited (rules: never `apps/mac/Sources`, `crates/**`, `tools/typing-bench`).
- Branch: `agent/mac-validation-4/mac-live-4` from `origin/agent/mac-claude-a/mac-shell`
  cd58fc2e (main c11c005 merged); the remote tip moved to fb16d07d during the run and the
  runner packages that tip (the "current packaged binaries").

## Coverage audit (existing evidence, before any new work)

Grep of `apps/mac/Tests/FlashTeXMacTests/*`, `tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`:

| already covered | where | what it does NOT cover |
|---|---|---|
| keystroke -> paint p50/p95/p99/max per cell, compile round trip, render pass, result -> paint | `apps/mac/Tests/FlashTeXMacTests/TypingBenchTests.swift` (recorder/config/stat tests); `tools/typing-bench/run.sh` + `evidence.py`; `docs/evidence/typing-bench-2026-09-12T*.md` (11 runs, swift-built `FlashTeXMac`, producers compiler/render/controller) | packaged `FlashTeX.app` binaries; per-stage attribution (the evidence totals only); hw1 and the 27-page render fixture as seeds |
| mac-live acceptance: fixture/demo/body60k x 30/0 ms on compiler, render, controller and historical routes; launch-check; capture cycle; render-attach; exact export; windows; worker relaunch; multifile; branch XCTests | `tools/native-validation/mac-live/reports/20260912T110944Z.md` (230/230 PASS, app 7ccbd7e9, helpers main 1884986f, render 6e69661) | current tips (app fb16d07d, main 3377748c, render 9aaec57a); the bench cells run the swift-built app, not the bundle; no v2 cells (`--producers "compiler render controller"`) |
| v2 route keystroke -> paint with ONE traced keystroke attributed (compile 27-30 ms, probe 2, off-main prepare 12, preraster 2.6, one main hop) | `docs/evidence/mac-preview-v2-live-2026-09-12.md` + `trace-v2-30ms-revision-100.txt` (demo only, render 4888a67, swift-built app, load 8-10) | statistics over all painted revisions; the other seeds; current render 9aaec57a; packaged binaries |
| helper display-candidate (helper-v2) route on p3/p27/pmax seeds, validation p50/p95 | `docs/evidence/helper-display-route-2026-09-12T1356Z/summary.md` (app c5fa71cd, render 9aaec57a; p27 = demo body x14 = 28 pages, v2 sibling declined) | helper v1 stage split; packaged binaries; the render-pipeline lane's own 27-page fixture (`tests/incremental.rs document(40)`) |
| bench timeline log lines (`compile: sending`, `worker: line … decoded in`, `worker: event on main`, `compile: applied`, `preview-v2: preparing/prepared/published/blit`, `paint: revision`) | emitted under `TypingBench.isBenchActive` in `WorkerClient.swift`, `ShellModel.swift`, `ShellModel+Controller.swift`, `PreviewV2View.swift`, `TypingBench.swift` | nothing parses them into per-stage statistics; `PreviewControllerClient.swift` emits no decode/main-hop lines (helper route = one round-trip stage) |

Verdict: the gap (packaged binaries, hw1 + render27 seeds, per-stage attribution across all
painted revisions, per route) is NOT covered. Implemented only the uncovered part: a runner-side
analyzer (`lib/typing_attribution.py`) that executes the bundle's `FlashTeX` directly through
the shell's existing bench and attributes from the existing log lines. No product change needed.

## Current task — results (`tools/native-validation/mac-live/reports/attribution-20260912T161500Z/attribution.md`)

Packaged bundle under test: `make-app.sh` output of the runner (work dir of the finished
mac-validation lane, SHA-keyed): app `fb16d07d` (origin/agent/mac-claude-a/mac-shell tip at
run time), helpers `3377748c` (origin/main: compiler, pdf, bridge, edit-ledger, preview
controller), `flashtex-render` `9aaec57a`, `flashtex-pdf-exact` `20e52778`; bundled sha256s in
`attribution.json`. Seeds: fixture 16 B (the protocol fixture's `main.tex` on this branch),
demo 5 909 B (2 pages), hw1 5 126 B (3 pages, 130 diagnostics, `recovered`), render27 76 929 B
(27 pages — the render-pipeline lane's `tests/incremental.rs document(40)` fixture reproduced;
page count verified through the bundled producer, `v2-sibling-probe.txt`), body60k 63 899 B
(22 pages). 200 keystrokes at 30 ms, one cell each; stage p50 over painted revisions.

**Load caveat (never hidden):** other lanes built and tested during the whole window; the
1-minute load was 16–83 in pass 1 and 8–34 in `pass2-quieter/` (demo + hw1 on v2/v1/controller
re-run when it dipped). Every cell is marked load-affected except v1-render body60k (9.4) and
controller demo pass 2 (7.9). The numbers are attributions under contention, not the routes'
floors — the quiet reference is report 20260912T110944Z (load 5.9–7.9): compiler demo 23/42,
body60k 94/119; render demo 41/46, body60k 132/151; controller demo 55/116, body60k 164/206.

| route | seed | p50 / p95 ms (bench) | dominant stage (p50, share of painted revision's key->paint) | producer CPU ms/result |
|---|---|---|---|---|
| v1 (bundled compiler) | fixture | 23 / 41 | paint 15.5 ms (66%); ->main hop 5.3 | 0.25 |
| v1 | demo | 29 / 56 (pass 2: 29 / 48) | paint 18.3 (64%); decode 3.1, ->main 2.9 | 1.4 |
| v1 | hw1 | 82 / 125 (pass 2: 78 / 92) | paint 51.7 (63%); ->main 19.9–26.7 | 2.4–3.0 |
| v1 | render27 | 188 / 239 | producer 51.4 (32%), decode 45.2 (3.2 MB line), key->send 24.4 (queued behind in-flight), paint 21.7 | 48.7 |
| v1 | body60k | 938 / 1502 (load 33->48) | decode 455.6 (70%; 2.3 MB line on a starved reader thread), producer 95.3 | 27.5 |
| v1-render (bundled render, v1 pane) | demo / hw1 | 46 / 215; 94 / 168 | paint 27.7 / 56.0 (~60%); ->main 9.2 / 26.6 | 2.8 / 3.0 |
| v1-render | render27 / body60k | 248 / 449; 143 / 196 (load 9.4) | decode 68.9 / 46.5 (36–37%), producer 53.7 / 28.3, key->send 27.7 / 22.1 | 51.0 / 27.4 |
| v2 (render + FLASHTEX_PREVIEW_V2=1) | demo | 768 / 1140 at load 71; **pass 2 (load 11): 137 / 357** | pass 2: paint 37.1 (29%), producer 33.0, v2 prepare 16.6, ->main 14.3, decode 8.7 (2 lines: 222 KB + 1.94 MB), deliver 8.0, preraster 3.5 | 24.5 |
| v2 | hw1 | 1549 / 1663 at load 60; **pass 2 (load 14): 680 / 1114** | pass 2: **paint 470 ms (69%)**, v2 deliver 121.7, ->main 47.2; producer only 16.8, prepare 9.7 | 16.7 |
| v2 | render27 / body60k | not measurable: 0 paints | the bundled producer declines the sibling (`display_list_declined`: ~24.0 MB / ~23.2 MB for 27 / 22 pages, over the 16 MiB line limit — `v2-sibling-probe.txt`); the v2 pane never paints, the bench times out after 60 s | — |
| controller (bundled helper + compiler) | demo | 105 / 158 (pass 2, load 7.9: **47 / 60**) | pass 2: helper round trip 23.5 (50%), paint 16.2 | 4.9 |
| controller | hw1 | 161 / 208 (pass 2: 147 / 245) | paint 81–98 (56–61%), helper 57–61 | 13.3–14.4 |
| controller | render27 / body60k | 354 / 532; 284 / 401 | helper round trip 216.9 / 176.0 (76–78%; durable->apply 164 / 130), paint 48 / 31 | 132 / 73 |

What dominates the 200 ms budget per route (from the table; load-affected):
- **Direct v1, small/medium docs (fixture, demo, hw1):** the paint stage (SwiftUI render pass +
  Canvas draw after `compile: applied`), 60–66 % of the keystroke->paint time; the producer is
  0.5–4 ms. hw1 also spends 20–27 ms in the decoded->main hop, i.e. main-thread contention.
- **Direct v1, large docs (render27, body60k):** the JSON line size — producer 50–95 ms plus
  reader decode 45–456 ms of a 2.3–3.2 MB v1 result line, and the keystroke queued 22–33 ms
  behind the in-flight compile (`key->send`); paint is only 22–39 ms. Producer CPU per result
  27–51 ms. render27 misses the target at p50 even with paint at 22 ms.
- **Direct v2 (demo):** no single stage dominates — producer 33, paint 37, prepare 17, hop 14,
  decode 9, deliver 8, preraster 3.5; hw1 on v2 is dominated by the paint stage (470 ms p50,
  69 %) and the queue->main delivery (122 ms), while its producer/prepare stages stay at
  17 / 10 ms — a main-thread stall specific to the v2 pane on the 130-diagnostic document (finding
  for the mac-preview-latency lane, not diagnosed here). v2 does not apply to render27/body60k.
- **Helper route:** small docs split between the helper round trip (23–61 ms) and paint; large
  docs are dominated by the helper round trip (176–217 ms, 76–78 %), of which durable->apply is
  130–164 ms — the helper's own compile + its transport, opaque to the shell's log.

## Follow-up 1 — mac-live acceptance rerun: partial (see limitations)

The full `run.sh` (bench 24 cells + launch-check + feature cycles + XCTests) did not fit the
bound after the attribution cells and the load waits; the package step ran on the current tips
(bundle above; six helpers signature-masked-identical to the builds, `components.json` SHAs
match) and the packaged exact export passed 8/8 (`flashtex-pdf-exact from-v2`, PDFKit 1 page,
text incl. "café"). launch-check, render-attach, open-window, worker-relaunch and multifile were
REFUSED by the runner because another lane's `FlashTeX.app` (pid 83388, the main checkout's
bundle) was running and `--force` would `pkill` it. The package-only report (20260912T160619Z)
was discarded because its skipped steps count as FAIL gates; a second partial run (capture
cycle + feature checks) was aborted by pid after its provenance step because `origin/main` and
the mac-shell tip had moved again (486b759c / ef08107a) and it started rebuilding a different
bundle outside the bound. No PASS/FAIL delta vs 20260912T110944Z is claimed beyond: bundle
build + provenance gates and exact export PASS on the new tips; everything else not rerun.

## Follow-up 2 — `tools/native-validation/mac-live/quiet-window.sh`

Implemented (bash -n clean; not yet exercised end-to-end because no quiet window occurred in
the bound): one analyzer invocation per cell, each after the load fell below 8 with no other
FlashTeX/FlashTeXMac alive (up to `--quiet-wait`), `uptime` before/after each cell appended to
`<out>/uptime.log`, resume by skipping cells that already have a summary, `attribution.md` at
the end. Suggested parent command when the machine is idle:
`tools/native-validation/mac-live/quiet-window.sh --app <bundle> --quiet-wait 1800`.

## Checkpoint block

- Branch / SHA: `agent/mac-validation-4/mac-live-4` @ see `git log -1`.
- Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a807b85fa768eadf9`.
- Dirty files: `tools/native-validation/mac-live/run.sh` (packages the preview controller too:
  `--controller`), `tools/native-validation/mac-live/lib/typing_attribution.py` (new),
  `tools/native-validation/mac-live/quiet-window.sh` (new), this file,
  `coordination/agents/mac-validation-4.json`.
- Consumed main SHA: c11c005 (via mac-shell cd58fc2e); helpers built from origin/main 3377748c.
- Scratch: no runner or bench process left running (all stopped by pid); work dir reused from the
  finished mac-validation lane (`.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build`,
  gitignored) holds the tested bundle `app/fb16d07d…/apps/mac/build/FlashTeX.app`.
- Next commands (parent, idle machine): `tools/native-validation/mac-live/quiet-window.sh --app <that bundle> --quiet-wait 1800`;
  `tools/native-validation/mac-live/run.sh` (full) once no other FlashTeX.app is running.
- Committed evidence: `tools/native-validation/mac-live/reports/attribution-20260912T161500Z/` (+ `pass2-quieter/`).
- Decisions: reuse the previous lane's SHA-keyed scratch builds (cargo target cache) to fit the
  bound; measure only on the packaged bundle (`Contents/MacOS/FlashTeX` executed directly,
  `FLASHTEX_NO_ACTIVATE=1`, never `open`); helper route attributed as one round-trip stage
  (client has no finer log lines — reported as a limitation, not patched).
- Resource: Claude Max 20x shared quota via parent; no purchases; no API spend.

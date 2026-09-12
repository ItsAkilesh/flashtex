# mac-large-document handoff — oversized helper output on the Mac shell

- Updated UTC: see `coordination/agents/mac-large-document.json` `updated_utc`
- Agent / parent / machine alias: `mac-large-document` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Mac-side handling of oversized helper output (finding from the
  paste-recovery lane: a fully-prose 560 KB document yields ≈18.2 MB of compiler
  preview JSON, above the helper's 16 MiB `MAX_OUTPUT_BYTES`). Owned paths:
  `apps/mac/Sources/FlashTeXMac/ShellModel+OutputBounds.swift`,
  `apps/mac/Tests/FlashTeXMacTests/OutputBoundsTests.swift`,
  `apps/mac/Sources/FlashTeXMac/PreviewControllerClient.swift` (may edit), this
  handoff, `coordination/agents/mac-large-document.json`. Parent-retained files
  (`ShellModel.swift`, `ShellModel+Controller.swift`, `ContentView.swift`, …)
  are NOT committed here; hooks are exact diffs in the final report.
- Branch: `agent/mac-large-document/large-document` from
  `origin/agent/mac-claude-a/mac-shell` `1630fdbc` (contains main `ffe199d`).
- Helper limits are NOT changed (crates/preview-controller, document-runtime and
  compiler are Commander/Daniel-owned).

## Coverage audit (mandatory first step)

Grepped `apps/mac/Tests/FlashTeXMacTests/*`, `apps/mac/Tests/FlashTeXProtocolTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`, `docs/evidence/*`
and `coordination/*.md` for `output limit`, `MAX_OUTPUT`, `helperMaxOutputBytes`,
`helper_max_output`, `18.2 MB`, `560 KB`, `oversized`, `too large`, `maxFrameBytes`,
`FLASHTEX_CONTROLLER_MAX_FRAME_BYTES`, `compilerMaxFrameBytes`.

Already covered (transport-level bounds only, never the shell's reaction to a
helper that refuses a preview for size):

| Where | What it covers |
| --- | --- |
| `WorkerClientTests.testCompleteOversizedLineIsRejectedAndWorkerTerminated` | direct route, fake worker: a complete line > 16 MiB is a `protocolViolation` and the worker is terminated (client level; nothing about ShellModel status, relaunch or the preview). |
| `WorkerClientTests.testUnterminatedTrailingBytesAtEOFAreAViolation` | direct route EOF trailing bytes. |
| `BridgeClientTests.testRequestedErrorGarbageAndOversizedLines`, `testTrailingBytesAtExitAndOversizedRequest` | transfer-v1 bridge 12 MiB line limit (different helper). |
| `NearbyListenerTests.testOversizedLineClosesConnection` | nearby protocol line limit (different surface). |
| `EditorDiagnosticsExplanationsTests.testBoundsCutLongFieldsAndRefuseOversizedReplies`, `testFakeHelperMalformedOversizedMismatchAndErrorReplies` | assistant-context reply bound (different helper). |
| `ProposalPreviewTests.testProviderPathEndToEndFromEnvironmentWithLocalCommand` | 64 KiB provider reply bound (different surface). |
| `RustPDFExportPipeTests.testOversizedStderrDoesNotDeadlockAndIsTruncated` | PDF writer stderr draining. |
| `Fixtures/fake_preview_controller.py:214` | fake helper's `ready` advertises `helper_max_output_bytes: 16 MiB` — only echoed into `controllerStatus` ("ready: compiler frames ≤ 8 MiB, helper output ≤ 16 MiB", `ShellModel+Controller.swift:182`). |
| `PasteRecoveryTests.swift:42` comment, `coordination/mac-paste-recovery.md:79`, `docs/evidence/mac-paste-recovery-2026-09-12.md:35` | the OBSERVATION only (18.2 MB > 16 MiB, "reported, not patched"); the paste fixture deliberately stays under the bound. |
| `tools/native-validation/mac-live/reports/20260212T110944Z.md` | no oversized-output cell; only the `ready` status line ("helper output ≤ 16 MiB") appears in a log excerpt. |

NOT covered anywhere (this lane's work):

1. What the helper actually sends when a preview exceeds a bound
   (`update kind:failed` from the runtime's `compiler_max_frame_bytes`, or the
   helper's 16 MiB `error` frame) and how the shell reacts: today
   `handleController` logs `controller update failed: …` and leaves the
   in-flight edit HELD under the default `holdUntilPreview` policy
   (`ShellModel+Controller.swift:235-247` releases only on `stale`/`discarded`);
   an `error` with `id:null` becomes `controllerStatus = "helper error: …"` with
   no size/bound and the edit stays held.
2. Status line naming the bound and the document size; the last good preview
   kept and marked stale (not "compiling…" when nothing will arrive).
3. Direct route: an oversized `compile_result` line is a protocol violation →
   worker terminated → relaunch → auto-compile of the same document → the same
   violation (up to 3 relaunches/min, then "not relaunched"); no test, and the
   status never names the document size or the bound.
4. Timing of the 560 KB compile on both routes (recorded, not asserted).

## What the helper and the worker actually send (measured, real binaries)

Reproduction (scratch driver + `OutputBoundsTests`, M1 Max, release helper +
compiler built in this worktree, 2026-09-12 10:07–10:17 UTC-7, load 9–22):

- The prose fixture (PasteRecoveryTests shape) produces **31.7 bytes of
  preview JSON per source byte**: 573,476 B → one `compile_result` line of
  18,157,062 B (17.32 MiB) in ≈244 ms direct; 494,056 B → 15,640,518 B.
- Helper route, default `compiler_max_frame_bytes` 8 MiB: `ready` →
  `document` → `compile` → **one `update {kind:"failed", request_id,
  reason:"compiler output malformed, truncated or oversized"}` per pending
  compile** (181 B; the startup auto-compile and the requested compile each
  get one) ≈230 ms after the request. No partial preview, no `error`, no
  exit. The runtime kills its compiler session; every later `edit` is ACKed
  durably (r2 in 27 ms) with `preview_error: "compiler session failed; create
  a new session with complete snapshots"` until a `restart`, which respawns
  the compiler and compiles the current source.
- Helper route with the 15 MiB opt-in: a 494 KB document (15,640,518 B
  compiler frame) IS delivered as a 15,640,806 B `update kind:preview` (the
  envelope adds 288 B, no reserialization growth), so the helper's 16 MiB
  `MAX_OUTPUT_BYTES` is not reachable through a preview with ≤ 15 MiB compiler
  frames on this compiler; it remains the bound for REQUIRED replies (`error
  "response exceeds output limit; source may already be durable"`, handled
  the same way here).
- The helper's behaviour at the bound is correct (one small frame, never a
  partial preview, ACKs continue). Nothing to report to the Commander as a
  helper bug.
- Direct route (`WorkerClient` on the real compiler): the pipe delivers the
  18.2 MB line in chunks, so the partial-buffer check fires first —
  `protocol violation: unterminated line exceeds the 16777216-byte limit`
  ≈350 ms after the request — the worker is terminated (status 15), a second
  violation `worker exited with 65536 unterminated trailing bytes` is logged
  (the termination handler drains one more chunk into a fresh splitter;
  cosmetic, WorkerClient.swift is not this lane's), then the bounded relaunch
  (attempt 1) and, before this lane, an auto-compile of the same document.
  The reply size is therefore never known on this route.

## What this lane adds (owned files)

- `apps/mac/Sources/FlashTeXMac/ShellModel+OutputBounds.swift`:
  `OutputBoundNotice` (route, editor revision, document bytes, bound bytes +
  protocol name, reply bytes when known; `status` for the bottom line,
  `banner` for the stale banner, `allowsRetry`), `OutputBounds` (reason /
  message constants, violation parser), per-model `OutputBoundState`
  (advertised bounds from `ready`, notice, retry flag, released count) and the
  `ShellModel` hooks: `outputBoundNoteReady`, `outputBoundHandleControllerUpdate`
  (releases the held edit and resubmits the queued buffer), `outputBoundHandleControllerError`,
  `outputBoundHandlePreviewError` (sends `restart` only when the document shrank:
  measured ratio when the reply size is known, else ≥ 10 % smaller — a
  constant-size document is never re-sent), `outputBoundNotePreviewApplied`,
  `outputBoundExplicitRetry` (⌘B), `outputBoundHandleWorkerViolation`,
  `outputBoundBlocksCompile` (the relaunch's auto-compile is refused while the
  document is not smaller). Helper limits untouched.
- `apps/mac/Tests/FlashTeXMacTests/OutputBoundsTests.swift` — 6 tests: 4 pure/
  model-level, 2 with the real helper/compiler (XCTSkip without the env vars;
  load-gated at 20 with `FLASHTEX_OUTPUT_BOUNDS_LOAD_LIMIT` override, labelled).
  The live tests detect whether the parent hooks are applied and otherwise
  dispatch the hook by hand after the real frame was observed in the log.
- `docs/evidence/mac-output-bounds-hooks-2026-09-12.diff` — the exact parent
  diffs (also applied on `agent/mac-large-document/large-document-applied`).

## Measured (recorded, not asserted; `output-bounds:` lines)

| Run | uptime / load | Numbers |
| --- | --- | --- |
| unhooked, `swift test --filter OutputBoundsTests` | 10:16, load 11.71 | 6/6 pass. Helper: paste ack 114 ms, `failed` at 276 ms, next edit ack 125 ms, recovery preview (60 KB, after restart) 1285 ms. Direct: line refused after 353 ms, 1 relaunch. |
| hooked (`-applied`), OutputBounds + PasteRecovery + ControllerRelease + WorkerClient | 10:17, load 9.24 | 24/24 pass. Helper: ack 119 ms, `failed` 273 ms, next ack 120 ms, recovery 1280 ms ("hooks applied"; log line `output bound (preview-4): revision 3: reply exceeds the helper's compiler frame bound (compiler_max_frame_bytes) (8 MiB) for this 560 KB document; last preview kept`). Direct: refused after 348 ms, 1 relaunch, no re-send. |

Direct vs helper for the 560 KB compile: the compiler itself answers in
≈244 ms (direct driver); through the helper the `failed` frame lands ≈230 ms
after the compile request (≈275 ms after the paste including the 27 ms
durable ACK); the direct route's refusal lands ≈350 ms after the request
(the 18 MB has to cross the pipe before the 16 MiB buffer check trips).

## Hooks requested from the parent (exact diff in docs/evidence/…-hooks-….diff)

`ShellModel+Controller.swift` (6 one-liners: `.ready`, `.error`, `.update`
else-if, `preview_error`, `applyControllerPreview`, `controllerCompile`),
`ShellModel.swift` (2: `compile()` guard after the worker guard,
`.protocolViolation`), `ContentView.swift` (stale banner uses
`model.outputBound?.banner`; toolbar Compile), `FlashTeXMacApp.swift` (⌘B).

## Limitations

- The helper does not report the refused reply's size, so on the helper route
  the retry rule is "≥ 10 % smaller"; on the direct route the reply size is
  also unknown in practice (partial-buffer check), so the same rule applies
  unless a complete-line violation is ever reported.
- Multi-document projects count all buffers' bytes; the ratio is prose-shaped
  (31.7×) and only a guard against loops, not a prediction.
- No app screenshot of the banner (no UI launch in this lane); the banner
  string is unit-tested.

## Durable checkpoint

- Branch `agent/mac-large-document/large-document`, base `1630fdbc`; consumed
  main `ffe199d` (via mac-shell). Applied branch
  `agent/mac-large-document/large-document-applied` = same + the hooks diff.
- Dirty files: none after the commits below.
- Next commands: `cd apps/mac && swift build && FLASHTEX_PREVIEW_CONTROLLER=… FLASHTEX_COMPILER=… swift test --filter OutputBoundsTests`.
- Decisions: helper limits untouched; side-table state (no stored property on
  ShellModel); retry only on shrink or ⌘B.
- Staffing/billing: shared Claude Max quota with parent mac-claude-a; no
  purchases; bounded ~60 min (resumed session).

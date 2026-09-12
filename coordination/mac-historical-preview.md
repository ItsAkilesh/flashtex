# mac-historical-preview handoff — native consumer for `completed_snapshot`

- Updated UTC: 2026-09-12T10:22Z
- Agent / parent / machine: `mac-historical-preview` (Claude Code subagent) / parent
  `mac-claude-a` / `mac-m1max-a`
- Lane: native consumer for the helper's `completed_snapshot` side channel
  (`crates/preview-controller/docs/completed-snapshot-proposal.md`; parent acceptance with
  binding conditions on issue #2 comment 5645120420; helper side published on
  `origin/agent/commander-preview-performance/preview-performance` @ ab945e6, "Negotiated
  stdio adapter"). Follow-ups folded in: real-helper end-to-end test; acceptance measurement
  (separate historical vs current paints, historical lag vs current-preview latency, under
  the 30 ms typing bench on demo.tex and the 60 KB seed plus the 0 ms burst).
- Owned paths: `apps/mac/Sources/FlashTeXMac/HistoricalPreview.swift`,
  `apps/mac/Tests/FlashTeXMacTests/HistoricalPreviewTests.swift`,
  `apps/mac/Tests/FlashTeXMacTests/Fixtures/fake_preview_controller.py`, evidence
  `docs/evidence/historical-preview-2026-09-12T1010Z.md` (+ directory), this file,
  `coordination/agents/mac-historical-preview.json`.
- Branch / worktree: `agent/mac-historical-preview/consumer` (pushed) in
  `.claude/worktrees/agent-a25f8f8ba4a279a48`; based on `origin/agent/mac-claude-a/mac-shell`
  40d53b7. Commits: 0cec2a1 (owned files), 5c35587 (**requested parent diffs**, applied
  locally so the branch builds/tests — parent-retained files, see below), 6bbe6a1 (real-helper
  test), a2cb509 (evidence + handoff), 5a81231 merge of `origin/agent/mac-claude-a/mac-shell`
  b898cfc (clean; multi-file integration + LM Math font), then the coord report.
- Rules honoured: no purchases; no network calls; no edits to transferred crates; the parent
  diffs are isolated in one commit (5c35587) so the parent can cherry-pick or re-apply from the
  diff below; app launched only by the bench with `FLASHTEX_NO_ACTIVATE=1`; commits carry the
  user as primary author with the required trailers. Feature is **default OFF**.

## Ready behavior (on the branch)

`HistoricalPreview.swift`:
- `CompletedSnapshots` — the exact helper wire at ab945e6: request
  `configure_completed_snapshots {capability:"completed-snapshots-v1", enabled:true}`,
  acknowledgement `result {capability, enabled:true}`, `update kind:"completed_snapshot"`.
  Opt-in only via `FLASHTEX_COMPLETED_SNAPSHOTS=1` (`CompletedSnapshots.requested`); unset →
  nothing is sent, the wire to any helper is byte-identical to before.
- `SourceBindingToken` — `ftx1:<16-hex nonce>:<editor revision>` (≤ 128 bytes), nonce fresh per
  negotiation; minted only after the acknowledgement; parsed back only under the same nonce, so
  a token from an earlier helper session / negotiation never binds a frame.
- `HistoricalFrame.decode` — reads the raw `result` range once (FastJSON typed reader, same as
  `preview`); refuses as a protocol violation: missing project/session/request id, payload
  `session_id` ≠ frame session, `compile_revision ≥ current_compile_revision`, anything but
  `is_current:false` and `source_actions_enabled:false`, a missing/empty/>128-byte token, no
  source versions, a result that is not a `compile_result` v1.
- `HistoricalPreviewState` — negotiation request/ack/refusal, at most **one pending frame**
  (newer replaces older, older-than-pending refused), admission checks (negotiated, session,
  project, token nonce, monotonic compile-generation **display floor**, editor revision not older
  than displayed, active document present in the frame's versions), `takePending(activePath:)`
  drops a frame queued before a document switch, `noteCurrentDisplayed` raises the floor and
  evicts pending history, `invalidate()` on close/exit/reattach (new nonce; the opt-in survives).
- `HistoricalDisplay` — the observable flag value: `label` = `"revision N shown — revision M
  compiling"` (N = token's editor revision, M = the in-flight edit's, else the newest submitted,
  else the buffer's).
- `ShellModel` extension — `historicalNegotiate()` on `ready`, `historicalToken(forEditorRevision:)`,
  `historicalHandle(resultID:/errorID:)`, `historicalReceive(frame)` (admit, then paint on the
  **next main-queue turn**: `historicalPaintPending()` rechecks helper session/project, token,
  floor, active document immediately before painting), `historicalNoteCurrentPreview`,
  `historicalInvalidate`, `historicalRefusal(of:)` (the refusal note used by navigation,
  diagnostic jump, pin-insertion-point, export), `isHistoricalPreview`, `historicalNegotiated`.
  A historical paint sets `result` (revision = the token's editor revision), `resultID`,
  `previewSource = .worker("flashtex-preview-controller")`, `compiledDocuments` = the ORIGINATING
  texts when still retained (never the buffer), `workerStatus = "historical revision N (helper
  generation g of G): …"`, logs `compile: applied historical revision N at <ns>` under the bench.
- In negotiated mode the adapter releases the next edit on the **durable receipt** (parent diff in
  `applyDurableDocument`): one durable ACK per keystroke, which is the only way the helper ever
  produces a stale-but-complete compile to publish. Without negotiation the hold-until-preview
  adapter is unchanged.

`fake_preview_controller.py` (executable, `#!/usr/bin/env python3`): helper protocol v1
(ready/document/edit/compile/configure_layout/restart/close, `update` kind preview/stale) plus
the negotiated channel with the ab945e6 wire; directives at the start of the entry text script a
held compile A and how it is released around the next compile B (`%hold`, `%after`,
`%badsession`, `%badproject`, `%badtoken`, `%current`, `%actions`); `FAKE_PC_REFUSE_SNAPSHOTS=1`
answers the negotiation with `error "unknown operation"` like a pre-channel helper; a 150 ms gap
between A and B makes the test order deterministic.

## Validation

- `HistoricalPreviewTests` (13): token round trip / foreign tokens refused / negotiation ack
  shapes; wire constants + default OFF; decoder accepts the contract frame and refuses 15
  malformed variants (incl. 129-byte token, 128-byte non-ASCII token accepted); state machine
  (one pending, floor, document switch, invalidation); label; end to end with the fake:
  **A before B → A painted** with the label, `HISTORICAL` state, navigation / diagnostic jump /
  caret sync / editor marks / pin / export refused by the flag, then **B replaces A** and actions
  return; **A after B → A dropped** (never repaints over B); `%badtoken` / `%badproject` refused
  at admission; `%badsession` a protocol violation without ending the session; `%current` /
  `%actions` violations; **no emission and no token without negotiation** (default OFF, and a
  helper answering "unknown operation"); close/reattach voids negotiation + floor, keeps the
  flag on the old content until a current result, fresh nonce; `replaceProject` clears the flag;
  **real helper ab945e6 + real compiler**: acknowledged, token echoed, 2 historical frames
  painted during a 40-edit burst on demo.tex, final current preview bound to the final revision,
  every painted frame a submitted revision, older than the one compiling, monotonic.
- Full suite with real workers (compiler 004283fc…, pdf, bridge, edit-ledger from the main
  checkout; helper ab945e6 scratch build 58daf00b…): **393 tests, 6 skipped (pre-existing
  env-gated: pdf-exact, nearby serve, screenshot dir, assistant-context ×3), 0 failures**
  (`swift test`, 88 s). `PreviewControllerTests` ran against the ab945e6 helper and passed.
  After merging mac-shell b898cfc: **435 tests, 6 skipped, 4 failures — all pre-existing in the
  parent's b898cfc**, none in this lane: `PreviewV2ShellTests` ×3
  (`testRefusedDisplayListShowsNoFrame`, `testLoadingRetainsThePreviousFrameAsStale…`,
  `testStaleLoadResultNeverOverwritesANewerState`) and
  `RenderingV2Tests.testMathFixtureFailsClosedWithoutTheMathFontBundled` expect
  `latinmodern-math.otf` to be absent; b898cfc vendors it. `HistoricalPreviewTests` 13/13 and
  the real-helper burst passed again in that run.
- Acceptance measurement: `docs/evidence/historical-preview-2026-09-12T1010Z.md`. Load 7–17.
  demo 30 ms: baseline (hold-until-preview) first paint p50/p95/p99 65/92/118, 0 historical;
  historical mode 178 paints (87 historical / 101 current), historical lag 93/119/146,
  keystroke→**current** paint 66/1629/1868 (current preview starves until a pause).
  60 KB 30 ms: baseline 195/272/296 (52 current paints); historical 76 paints (65/10),
  historical lag 316/414/441, keystroke→current 2822/5582/5809 max 5874.
  demo 0 ms burst: baseline 69/109/134; historical lag 92/114/129, keystroke→current
  72/326/387. 0 frames refused/dropped by the native checks in these runs.

## Reading / recommendation

The consumer meets the accepted contract (labels, explicit flag gating, monotonic floor,
session+project+token rechecks at paint, one pending frame, invalidation, no default switch).
As a typing-progress mechanism with this helper/runtime it is not better than the parent's
hold-until-preview adapter: with one durable edit per keystroke every completion during
continuous typing is historical and the current preview only lands when typing pauses; on 60 KB
the historical frames are slower than baseline's current paints. Keep OFF. Worthwhile
follow-ups: (a) a hybrid adapter policy — hold until preview, release on durable once the
in-flight compile exceeds a bound (so history only appears when the compiler is actually
behind); (b) helper/runtime skipping of superseded queued compiles so the current revision
compiles sooner; (c) a bench-native historical/current split (TypingBench is parent-owned; the
split here is post-processed from the log).

## Exact diffs for parent-retained files (commit 5c35587; `git show 5c35587`)

Apply as-is; HistoricalPreview.swift supplies every symbol used. Summary per file:

- `PreviewControllerClient.swift`: `Event.completedSnapshot(HistoricalFrame)`; in `decode`,
  before the `guard kind == "preview"`:
  `if kind == CompletedSnapshots.updateKind { return HistoricalFrame.decode(line, payload: payload, frameSessionID: sessionID) }`;
  `edit(..., sourceBindingToken: String? = nil)` adds `source_binding_token` only when non-nil;
  `compile(sourceBindingToken: String? = nil)` likewise.
- `ShellModel.swift`: `var historicalPreview: HistoricalDisplay?` and
  `@ObservationIgnored var historicalState = HistoricalPreviewState()` next to `controllerStatus`;
  `editorMarkReport`: `guard let result, historicalPreview == nil else { return .empty }`;
  `historicalPreview = nil` in `loadFixtures` (after `previewSource = .fixture`), in
  `replaceProject` (after `previewSource = .none`) and in the direct-worker `handle(.result)`
  (after `previewSource = .worker(...)`); `pinAnchorAtCaret()` starts with
  `if let why = historicalRefusal(of: "pinning an insertion point") { captureNote = why; return }`.
- `ShellModel+Controller.swift`: `detachController` → `historicalInvalidate(reason: "close")`
  after `controllerState = ControllerState()`; `controllerSubmitEdit` passes
  `sourceBindingToken: historicalToken(forEditorRevision: editorRevision)`; `controllerCompile`
  passes the same to `compile(...)`; `.ready` calls `historicalNegotiate()` before
  `configureLayout`/`document`; `.result` → `if historicalHandle(resultID: id, payload: payload) { return }`
  after the `awaiting` waiter; `.error` → `if historicalHandle(errorID: id, message: message) { return }`
  after the waiter; `case .completedSnapshot(let frame): historicalReceive(frame)`; `.exited` →
  `historicalInvalidate(reason: "helper exited")`; `applyDurableDocument` (in-flight branch, after
  `durableRevision = revision`) → `if historicalNegotiated { controllerState.inFlight = nil }`;
  `applyControllerPreview` → `historicalNoteCurrentPreview(compileRevision: update.compileRevision)`
  right after `previewSource = .worker("flashtex-preview-controller")`.
- `Navigation.swift`: first line of `navigateExactly` →
  `if let why = historicalRefusal(of: "navigation") { navigationNote = why; return }`; first line of
  `goToDiagnostic` → same with `"diagnostic navigation"`.
- `CaretSync.swift`: `exactCaretItems` → `guard historicalPreview == nil else { return [:] }`.
- `PDFExport.swift` `exportPDF()` / `RustPDFExport.swift` `exportPDFViaRust()`: first line
  `if let why = historicalRefusal(of: "export") { captureNote = why; return }`.
- `ContentView.swift` `StatusBanner`: replace `if model.previewIsStale {` with
  `if let historical = model.historicalPreview { Text(historical.label).foregroundStyle(.purple).bold().help("…") } else if model.previewIsStale {`;
  `sourceBadge`: `case .worker: model.historicalPreview != nil ? ("HISTORICAL", .purple) : ("WORKER", .green)`.

## Limitations / not done

- No live-app click evidence (Accessibility not granted); the label/badge were verified by the
  model state under the bench and by tests, not by a screenshot.
- `exportPDFExact` (v2 display list) is not gated: it exports a loaded display list, not the
  `result`; if the parent wires v2 lists from helper previews it should gate on the flag too.
- `approveProposal` is not gated (it inserts at a source anchor, not a preview span); pinning
  the anchor is.
- The bench-level historical/current split is post-processed; TypingBench itself still counts a
  historical paint as a paint of that revision.
- One run per cell on a loaded machine; not a regression gate.

## Resources

Shared Claude Max 20x quota with parent `mac-claude-a`; no purchases; totals unknown. Context
usage well under 20% of the 1M budget at this checkpoint (the harness shows remaining tokens,
not a percentage).

# mac-helper-display handoff — native helper display-candidate route

- Updated UTC: see `coordination/agents/mac-helper-display.json` `updated_utc`
- Agent / parent / machine alias: `mac-helper-display` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task: Commander request issue #2 comments 5645530349 / 5645542186 — implement
  and measure the native helper display-candidate route in the Mac shell
  (`crates/preview-controller/docs/display-forwarding.md` "Native gate").
  Plus GH31 (relayed by the parent): `V2FontStore` discovery-to-CGFont byte identity.
- Owned paths: `apps/mac/Sources/FlashTeXMac/PreviewControllerClient.swift`,
  `apps/mac/Sources/FlashTeXMac/ShellModel+DisplayCandidates.swift` (new),
  `apps/mac/Tests/FlashTeXMacTests/DisplayCandidateTests.swift` (new),
  `apps/mac/Sources/FlashTeXMac/GlyphRunRenderer.swift` (GH31 fix only),
  `apps/mac/Tests/FlashTeXMacTests/V2FontStoreIdentityTests.swift` (new),
  `docs/evidence/helper-display-route-<UTC>/`, this handoff and
  `coordination/agents/mac-helper-display.json`.
  NOT committed on the lane branch (parent-retained; exact diff below):
  `FlashTeXMacApp.swift`, `ShellModel.swift`, `ShellModel+Controller.swift`.
  `ContentView.swift` needed no change. `crates/preview-controller` untouched.

## Durable checkpoint

- Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ae7ddcfb739815fac`
- Branch `agent/mac-helper-display/route` @ `74b8825c` (pushed). Base: parent
  `origin/agent/mac-claude-a/mac-shell` `5bc3fc0f` merged with `origin/main`
  `6472a5d2` (merge `c74a95bc`). Commits on top: `a9b55af7` (GH31 fix + owner
  test), `74b8825c` (route + tests).
- Branch `agent/mac-helper-display/route-applied` @ `fd5ce63a` (pushed): the
  lane plus ONE labelled "LOCAL APPLICATION" commit of the parent-retained hook
  lines (diff below). This is the branch that compiles, tests and benches; it
  is not for integration as-is.
- Dirty files at this checkpoint: none on the lane branch (registration/handoff
  being committed now).
- Consumed SHAs: helper `crates/preview-controller` as of main `6472a5d2`
  (built release from this tree: `crates/preview-controller/target/release/
  flashtex-preview-controller`, 2565952 B, 09:27 local); producer
  `flashtex-render` built from `origin/agent/mac-render-pipeline/unified`
  `f762f82a7307dc3d6522078364af79e09a51ad06` (`git archive` of
  `crates/render-pipeline` into the session scratchpad; vendored siblings;
  1967168 B). v1-only producer for the declined case: the main checkout's
  `crates/compiler/target/release/flashtex-compiler`.
- Tested: `swift test --filter "DisplayCandidateTests|PreviewControllerTests|
  HistoricalPreviewTests|PreviewV2Tests|V2FontStoreIdentityTests|RenderingV2Tests"`
  on the applied branch with `FLASHTEX_PREVIEW_CONTROLLER`, `FLASHTEX_RENDER`,
  `FLASHTEX_COMPILER` set: 50/50, 0 skips, twice (load avg 8–29). The GH31
  refusal test was verified to FAIL on the unfixed source and pass with the fix.
  Full `swift test` deliberately NOT run (parent's heavy-build window notice).
- Pending: keystroke→paint helper-route measurement (parent must open the
  low-load window first). Harness: `docs/evidence/helper-display-route-<UTC>/run.sh`.
- Next commands: see "Measurement harness" below.
- Staffing/billing: shared Claude Max quota with parent; one 429 (monthly spend
  limit, reset 13:20Z) before this session started; no purchases.

## What the route does (ShellModel+DisplayCandidates.swift)

1. Opt-in is explicit and per helper session: `FLASHTEX_DISPLAY_CANDIDATES=1`
   at attach or View > "Helper Display Candidates". On `ready` the shell sends
   `configure_layout` WITHOUT `display-list-v2` (the helper refuses that while
   OFF), then `configure_display_candidates {capability:"display-candidates-v1",
   enabled:true, renderer_support_confirmed:true}` and waits for the exact
   `{capability, enabled:true, preview_error}` acknowledgement. An `error`
   (mutual exclusion with completed snapshots, compiler unavailable, older
   helper) leaves the route OFF and v1 untouched.
2. `PreviewControllerClient` decodes `update {kind:"display_candidate"}` into
   `Event.displayCandidate(DisplayCandidateFrame)`; the nested `display_list`
   is kept as the original bytes (raw range). `untrusted:true` and
   `source_actions_enabled:false` are mandatory; anything else is a protocol
   violation (logged, v1 continues), never a candidate.
3. Admission gate (`DisplayCandidateGate`, pure): negotiated; outer session id
   == attached client session; project id == attached project; `request_id`,
   `compile_revision` and `source_versions` == those of the APPLIED v1 preview
   (`resultID` and the `displayCandidatesNotePreview` record); active document
   has a version; not older than the last v2 frame this route painted. One
   pending candidate (newest wins); a document switch drops it.
4. Validation off the UI thread (`V2Loader.queue`): `RenderingV2.decode`,
   `V2Frame.prepare` (fonts by content hash through `V2FontStore`, now
   byte-authenticated per GH31; pages prepared), then binding: envelope
   `project_id` == candidate project, envelope `revision` == candidate compile
   generation, every declared document covered by `source_versions` and its
   `byte_length`/`sha256` equal to the durable text of that version
   (`controllerState.textByDurable`). The compile generation is never equated
   with the durable revision. Pre-rasterisation happens in the same job.
5. Paint-time recheck on the main thread: ticket still current, helper alive,
   session/project unchanged, gate still passes, editor-revision binding
   unchanged; then publish through the existing v2 pane state
   (`displayListV2 = .loaded`), with `list.revision` rebound to the EDITOR
   revision exactly as `applyControllerPreview` rebinds the v1 result (the
   generation stays in the log/status). A refused or stale candidate restores
   the previously verified frame; nothing unverified is ever on screen.
6. v1 remains the product path: candidates never change `result`, and source
   actions are not granted by receipt — the v2 pane's own navigation gate
   (buffer SHA-256 equality + the shell's stale refusal) is unchanged.
7. Disable, restart (`displayCandidatesRestartHelper`, fresh opt-in after the
   restart reply), close, helper exit and reattach void negotiation, pending
   and in-flight candidates; the user intent survives so the next session
   re-negotiates on `ready`.

### Helper interaction found and worked around (report for the helper owner)

`crates/preview-controller/src/output_delivery.rs` `Sender::try_send` sets
`state.optional = None` on EVERY required enqueue, and the writer only takes the
optional frame after a 2 ms idle `recv_timeout`. The shell sends three `complete`
requests immediately after every applied v1 preview (completion vocabulary
refresh), whose required replies discard the just-offered candidate before the
writer reaches it. Reproduction: run the app/tests with the helper wrapped in a
`tee` (scratch `helper-tee.sh`): the wire shows `preview-4` v1 → `complete`
replies pc-8..10 → no `display_candidate`; the same helper driven by
`examples/helper_replay.Client` without completion requests forwards every
candidate (`enable`/`disable`/`enable`/`edit`, `layout`/`enable`/`restart`/
`enable`/`edit`/`edit` all yield candidates). Not a helper bug per the doc
("admission races may also drop candidates") but a systematic drop under the
app's real request pattern. Native mitigation in this lane:
`displayCandidatesAfterSibling` holds the completion refresh back until the
sibling arrives (admitted or refused) or 80 ms elapse. Counters
`deferredReleasedByCandidate` / `deferredReleasedByTimeout` are recorded.
Suggested helper-side fix (not applied, Commander owns the crate): keep a
queued optional frame across required enqueues while the writer is idle, or
write the optional frame FIFO behind the required ones instead of clearing it.

## Tests (apps/mac/Tests/FlashTeXMacTests)

`DisplayCandidateTests` (10):
- pure: decoder contract (flags, identity, v2 envelope, foreign session);
  gate (not negotiated, toggled session, other project, not the applied
  request, generation/versions mismatch, active path, display floor);
  validator (exact text verified; +1 byte length refusal; same-length hash
  refusal; missing durable text; wrong generation; wrong project; uncovered
  document; missing font fails the whole candidate); state (one pending,
  replacement, switch drop, invalidate, disable ack ≠ negotiation, refusal);
  model default OFF keeps v1.
- real helper + real producer (skip without env): default OFF → ordinary v1
  only, no candidates, `display-list-v2` never enrolled; declined producer
  (`flashtex-compiler`) → enrolled but declined, v1 only, limitation logged;
  candidates paint bound to editor revisions through a 6-keystroke burst
  (final frame == final revision, document sha == buffer, fonts by raw-byte
  hash, `negotiation.accepted` includes `display-list-v2`, no violation),
  tampered sibling refused off-main with v1 and the previous frame kept,
  foreign-session frame refused at admission; disable → no candidates and
  capability removed, restart → fresh opt-in and a new paint, close →
  invalidated, reattach → paints again; mutual exclusion with
  `FLASHTEX_COMPLETED_SNAPSHOTS=1` refused by the helper with v1 unaffected.
`V2FontStoreIdentityTests` (3, GH31): changed bytes after discovery refused
before CGFont construction under both hash spellings and nothing cached; a
verified CGFont stays the same object after the file changes; unknown hash /
length mismatch still refused.

## Parent-retained hook diff (apply from here or cherry-pick fd5ce63a)

```diff
diff --git a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
--- a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
+++ b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
@@ -94,6 +94,12 @@ struct FlashTeXMacApp: App {
         }
         .commands {
             NavigationCommands(model: model) // Navigation.swift
+            CommandGroup(after: .toolbar) {
+                // Helper display-candidate route (ShellModel+DisplayCandidates.swift): default OFF; untrusted v2 siblings painted in the v2 pane.
+                Toggle("Helper Display Candidates", isOn: Binding(get: { model.displayCandidates.requested }, set: { model.setDisplayCandidates($0) }))
+                    .disabled(!model.controllerAttached)
+                    .help("Ask the attached preview controller to forward the producer's display-list-v2 sibling (untrusted; validated natively before paint). Status: \(model.displayCandidates.status)")
+            }
             CommandGroup(after: .help) {
                 Button("FlashTeX Accessibility Help") { openWindow(id: AccessibilityHelpView.windowID) }
             }
diff --git a/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift b/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift
--- a/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift
+++ b/apps/mac/Sources/FlashTeXMac/ShellModel+Controller.swift
@@ -118,6 +118,7 @@ extension ShellModel {
         self.controller = nil
         controllerState = ControllerState()
         historicalInvalidate(reason: "close")
+        displayCandidatesInvalidate(reason: "close") // ShellModel+DisplayCandidates.swift
         inFlightRevision = nil
         controllerStatus = "no preview controller attached"
         if previewSource != .fixture { workerStatus = "no worker attached" }
@@ -181,13 +182,18 @@ extension ShellModel {
             // into the negotiated primitives the preview draws, then learn the
             // durable document so edits can name the revision they expect.
             historicalNegotiate()
-            if !requestedLayoutCapabilities.isEmpty {
-                _ = try? controller.configureLayout(capabilities: requestedLayoutCapabilities)
+            // display-list-v2 is enrolled by the helper itself through the
+            // display-candidate opt-in (ShellModel+DisplayCandidates.swift), never by configure_layout.
+            let layout = displayCandidatesConfigureLayoutCapabilities(requestedLayoutCapabilities)
+            if !layout.isEmpty {
+                _ = try? controller.configureLayout(capabilities: layout)
             }
+            displayCandidatesNegotiate() // after configure_layout: the helper enrolls display-list-v2 into that set
             _ = try? controller.document(path: activePath)
         case .result(let id, let payload):
             if let waiter = controllerState.awaiting.removeValue(forKey: id) { waiter(.success(payload)); return }
             if historicalHandle(resultID: id, payload: payload) { return }
+            if displayCandidatesHandle(resultID: id, payload: payload) { return }
             switch completionFetcher.handle(resultID: id, payload: payload) {
             case .notMine: break
             case .pending: return
@@ -204,6 +210,7 @@ extension ShellModel {
         case .error(let id, let message):
             if let id, let waiter = controllerState.awaiting.removeValue(forKey: id) { waiter(.failure(.init(message: message))); return }
             if historicalHandle(errorID: id, message: message) { return }
+            if displayCandidatesHandle(errorID: id, message: message) { return }
             if case .refused(let why) = completionFetcher.handle(errorID: id, message: message) {
                 log("completion metadata refused: \(why)")
                 break
@@ -226,6 +233,8 @@ extension ShellModel {
             applyControllerPreview(update)
         case .completedSnapshot(let frame):
             historicalReceive(frame)
+        case .displayCandidate(let candidate):
+            handleDisplayCandidate(candidate) // ShellModel+DisplayCandidates.swift
         case .update(let kind, let payload):
             // stale / discarded previews name the request they replaced; nothing
             // to paint. If it was the preview our in-flight edit waits for, the
@@ -254,6 +263,7 @@ extension ShellModel {
             controllerState = ControllerState()
             inFlightRevision = nil
             historicalInvalidate(reason: "helper exited")
+            displayCandidatesInvalidate(reason: "helper exited") // ShellModel+DisplayCandidates.swift
         }
     }
 
@@ -408,7 +418,8 @@ extension ShellModel {
         }
         var incoming = update.result.payload
         incoming.revision = editorRev
-        let requested = requestedLayoutCapabilities
+        // display-list-v2 accepted by the producer is expected while the helper enrolls it for candidates (ShellModel+DisplayCandidates.swift).
+        let requested = displayCandidatesLayoutRequested(requestedLayoutCapabilities)
         if let violation = LayoutNegotiation.violation(in: incoming, requested: requested) {
             log("rejected controller preview \(update.requestID): \(violation)")
             controllerStatus = "protocol violation: \(violation)"
@@ -418,6 +429,7 @@ extension ShellModel {
         resultID = update.result.id
         previewSource = .worker("flashtex-preview-controller")
         historicalNoteCurrentPreview(compileRevision: update.compileRevision)
+        displayCandidatesNotePreview(update, editorRevision: editorRev) // the only request a candidate may correlate to
         bindLayout(of: incoming, requested: requested)
         if !update.missingLayoutCapabilities.isEmpty {
             log("controller: compiler declined layout capabilities \(update.missingLayoutCapabilities)")
@@ -433,9 +445,14 @@ extension ShellModel {
         workerStatus = String(format: "revision %d: %@, %d diagnostics in %.0f ms (durable r%d)", editorRev,
                               incoming.status.rawValue, incoming.diagnostics.count, ms, durableRevision)
         selection = nil
-        // Refresh the completion vocabulary for exactly these source versions.
+        // Refresh the completion vocabulary for exactly these source versions
+        // (held back briefly while the helper's display-candidate sibling of this
+        // request is expected: ShellModel+DisplayCandidates.swift).
         if let controller {
-            completionFetcher.request(sourceVersions: update.sourceVersions, editorRevision: editorRev) { try controller.send($0, $1) }
+            displayCandidatesAfterSibling(of: update.requestID, acceptedLayout: incoming.layoutCapabilities ?? []) { [weak self] in
+                guard let self, self.controller === controller, controller.isRunning else { return }
+                self.completionFetcher.request(sourceVersions: update.sourceVersions, editorRevision: editorRev) { try controller.send($0, $1) }
+            }
         }
     }
 }
diff --git a/apps/mac/Sources/FlashTeXMac/ShellModel.swift b/apps/mac/Sources/FlashTeXMac/ShellModel.swift
--- a/apps/mac/Sources/FlashTeXMac/ShellModel.swift
+++ b/apps/mac/Sources/FlashTeXMac/ShellModel.swift
@@ -81,6 +81,8 @@ final class ShellModel {
     /// preview replaces it. Explicit, never inferred from staleness.
     var historicalPreview: HistoricalDisplay?
     @ObservationIgnored var historicalState = HistoricalPreviewState()
+    /// Helper display-candidate route (ShellModel+DisplayCandidates.swift): opt-in, negotiation and gate state.
+    let displayCandidates = DisplayCandidateState()
     /// Project-index completion vocabulary (labels/citations/commands) bound to
     /// the editor revision it was fetched for (Completion.swift).
     var completionMetadata: Completion.Metadata?
```

## Measurement harness (runs when the parent opens the low-load window)

`docs/evidence/helper-display-route-<UTC>/run.sh` builds `FlashTeXMac` release
from the applied branch and runs `TypingBench` (tools/typing-bench conventions:
`FLASHTEX_TYPING_BENCH=tools/typing-bench/typed-200.txt`, 30 ms and 0 ms
intervals, `FLASHTEX_NO_ACTIVATE=1`) over the helper route with
`FLASHTEX_PREVIEW_CONTROLLER` + `FLASHTEX_COMPILER=<flashtex-render>` +
`FLASHTEX_PREVIEW_V2=1` + `FLASHTEX_DISPLAY_CANDIDATES=1`, so the recorded
paint point is the v2 pane's bitmap blit of the candidate frame for the typed
revision (PageV2View → TypingBench.willRender/didDraw). Seeds: a 3-page and a
multi-page document generated from `apps/mac/Samples/demo.tex`. Records
`uptime` before/after each cell and the candidate counters from the log.

## Limitations (honest)

- No pixel-parity claim: the frame painted is the producer's v2 sibling
  validated by hash/length/fonts/geometry, not compared against a PDF raster.
- No latency claim yet: the measurement is pending the low-load window.
- The helper's optional slot can still drop a candidate when any required
  reply (durable edit ack, snapshot, etc.) is enqueued inside the writer's
  2 ms idle window; the shell then keeps v1 and the previous v2 frame. The
  drop rate is what the measurement will report (received vs previews).
- Oversized candidates: the helper refuses >16 MiB optional frames
  (`serialization_refused`) and the client's existing 16 MiB frame bound
  terminates a helper that violates it; no native test produces a >16 MiB
  sibling through the real producer.

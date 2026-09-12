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

## Durable checkpoint (final for this session, 2026-09-12T14:35Z)

- Worktree: `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-ae7ddcfb739815fac`.
- Branch `agent/mac-helper-display/route` (lane, for integration): base parent
  `origin/agent/mac-claude-a/mac-shell` `5bc3fc0f` merged with `origin/main`
  `6472a5d2` (`c74a95bc`); commits `a9b55af7` (GH31 fix + owner test),
  `74b8825c` (route + tests), `3db719cf` (registration/handoff/harness),
  `75496da1` (resume checkpoint), `8c08b713` (sibling hold: held work never
  dropped; GH36 ordering test), `8db74c36` (measurement, exchange wire,
  handoff), plus the final checkpoint commit on top (this file).
- Branch `agent/mac-helper-display/route-applied` = the lane tip plus ONE
  "LOCAL APPLICATION" commit of the parent-retained hook lines (diff below).
  This is the tree that was built, tested (52/52) and measured.
- Dirty files: none. Pending: none. Parent integrates.
- Consumed SHAs: helper `crates/preview-controller` as in this tree (main
  `6472a5d2`; `origin/main` `77c8cab1` unchanged in the optional-slot
  eviction); producer `flashtex-render` from
  `origin/agent/mac-render-pipeline/unified` `9aaec57a`.
- Tested: `swift test --filter "DisplayCandidateTests|PreviewV2Tests|RenderingV2Tests|
  V2FontStoreIdentityTests|PreviewControllerTests|HistoricalPreviewTests"` on the
  applied tree with FLASHTEX_PREVIEW_CONTROLLER/FLASHTEX_RENDER/FLASHTEX_COMPILER:
  52/52, 0 skips (load 6–8). Full `swift test` not run here.
- Ownership: parent retains ShellModel.swift, ShellModel+Controller.swift,
  ContentView.swift, PreviewView.swift, FlashTeXMacApp.swift,
  SourceEditorView.swift; Rust crates never edited by this lane.
- Staffing/billing: shared Claude Max quota with parent mac-claude-a; no
  purchases; only pids launched by this lane's harness were signalled.

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
`origin/main` `77c8cab1` only adds tracing (`evicted`) to that slot; the
eviction itself is unchanged.

Second helper interaction (display.rs): a candidate is forwarded only while
the helper's current source is the one it was compiled from, so under
continuous typing the NEXT edit (released by the shell as soon as the v1
preview for the in-flight edit arrives) invalidates the sibling before the
helper checks it. Measured without the hold: 3 of 102 candidates forwarded in
a 200-keystroke burst. Native mitigation: when the route is negotiated and the
producer accepted `display-list-v2`, the in-flight edit release is held with
the completion refresh (`displayCandidatesAfterSibling(..., holdsRelease:
true)`) until the sibling arrives (admitted or refused) or the bound
(`FLASHTEX_DISPLAY_CANDIDATES_WAIT_MS`, default 80 ms) elapses;
`FLASHTEX_DISPLAY_CANDIDATES_HOLD=0` disables the hold for comparison. Held
work is a list per request, runs in order, and is never dropped: a newer
request's hold releases the older request's work first (`superseded`), and
invalidation (close / helper exit / restart) releases it too (`invalidated`);
each piece guards the controller it was queued for. Counters
`deferredReleasedByCandidate/Timeout/Supersession/Invalidation`. The price is
paid only while candidates are ON: v1 previews per 200-keystroke burst fall
(the edit pipeline waits for the sibling), so the v1 pane is not slower when
the route is OFF (default) — see the measurement.

## Tests (apps/mac/Tests/FlashTeXMacTests)

`DisplayCandidateTests` (12):
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
  foreign-session frame refused at admission; ordering on the real helper
  (GH36: `configure_layout` with `display-list-v2` before the opt-in → the
  exact refusal and legacy v1 continues; opt-in acknowledged first → the same
  `configure_layout` accepted and candidates paint); disable → no candidates and
  capability removed, restart → fresh opt-in and a new paint, close →
  invalidated, reattach → paints again; mutual exclusion with
  `FLASHTEX_COMPLETED_SNAPSHOTS=1` refused by the helper with v1 unaffected.
`V2FontStoreIdentityTests` (3, GH31): changed bytes after discovery refused
before CGFont construction under both hash spellings and nothing cached; a
verified CGFont stays the same object after the file changes; unknown hash /
length mismatch still refused.

## Parent-retained hook diff (apply from here or take the LOCAL APPLICATION commit on `route-applied`)

Files: `FlashTeXMacApp.swift`, `ShellModel.swift`, `ShellModel+Controller.swift`
(diff against lane tip `3db719cf`'s parent-retained state, i.e. the parent's
`5bc3fc0f` merged with main `6472a5d2`). `ContentView.swift`, `PreviewView.swift`,
`SourceEditorView.swift` unchanged.

```diff
diff --git a/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift b/apps/mac/Sources/FlashTeXMac/FlashTeXMacApp.swift
index d3564c37..8ff8e495 100644
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
index e2006040..f26c313c 100644
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
 
@@ -395,7 +405,11 @@ extension ShellModel {
         // active path's version says nothing about it.
         if let inFlight = controllerState.inFlight, let want = inFlight.durableRevision,
            let got = update.sourceVersions[inFlight.path], got >= want {
-            controllerReleaseInFlight()
+            // Held briefly for this request's display-candidate sibling when that route is
+            // negotiated (ShellModel+DisplayCandidates.swift); otherwise released now.
+            displayCandidatesAfterSibling(of: update.requestID, acceptedLayout: update.result.payload.layoutCapabilities ?? [], holdsRelease: true) { [weak self] in
+                self?.controllerReleaseInFlight()
+            }
         }
         guard let durableRevision = versionForActive,
               let editorRev = controllerState.editorRevisionByDurable[activePath]?[durableRevision] else {
@@ -408,7 +422,8 @@ extension ShellModel {
         }
         var incoming = update.result.payload
         incoming.revision = editorRev
-        let requested = requestedLayoutCapabilities
+        // display-list-v2 accepted by the producer is expected while the helper enrolls it for candidates (ShellModel+DisplayCandidates.swift).
+        let requested = displayCandidatesLayoutRequested(requestedLayoutCapabilities)
         if let violation = LayoutNegotiation.violation(in: incoming, requested: requested) {
             log("rejected controller preview \(update.requestID): \(violation)")
             controllerStatus = "protocol violation: \(violation)"
@@ -418,6 +433,7 @@ extension ShellModel {
         resultID = update.result.id
         previewSource = .worker("flashtex-preview-controller")
         historicalNoteCurrentPreview(compileRevision: update.compileRevision)
+        displayCandidatesNotePreview(update, editorRevision: editorRev) // the only request a candidate may correlate to
         bindLayout(of: incoming, requested: requested)
         if !update.missingLayoutCapabilities.isEmpty {
             log("controller: compiler declined layout capabilities \(update.missingLayoutCapabilities)")
@@ -433,9 +449,14 @@ extension ShellModel {
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
index e8f37f4d..f34d4b0c 100644
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

## Measurement (done 2026-09-12T14:21–14:27Z, quiet window, load 4–7)

`docs/evidence/helper-display-route-2026-09-12T1356Z/summary.md` (+ `raw/*.json`,
one per cell, with per-keystroke samples). Real FlashTeXMac release build of
`route-applied` @ `c5fa71cd` driven by TypingBench (`typed-200.txt`, 200
keystrokes, 30 ms and 0 ms intervals, `FLASHTEX_NO_ACTIVATE=1`) through the
real helper (this tree's `flashtex-preview-controller`, sha256 c95a26a3…) owning
`flashtex-render` built from `origin/agent/mac-render-pipeline/unified`
`9aaec57a` (sha256 ed729b02…). Paint point: the v2 pane's bitmap blit of the
validated candidate for the typed revision (v1 control: PreviewView's Canvas
pass). Seeds from `apps/mac/Samples/demo.tex`: p3 = 4 pages (3.9 MB sibling),
pmax = 8 pages (7.8 MB, the largest sibling under the helper's default 8 MiB
compiler-frame cap), p27 = 28 pages (sibling declined by the producer).

| cell | p50 | p95 | p99 | max | v1 previews / 200 keys | cand. painted | refused | dropped@paint | load |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| helper-v2 p3 30 ms | 296 | 405 | 435 | 452 | 45 | 44 | 0 | 1 | 4.2→6.3 |
| helper-v2 p3 0 ms | 209 | 286 | 333 | 378 | 29 | 28 | 0 | 1 | 6.3→6.0 |
| helper-v2 pmax 30 ms | 382 | 481 | 560 | 623 | 35 | 33 | 0 | 2 | 6.0→6.7 |
| helper-v2 pmax 0 ms | 359 | 474 | 502 | 526 | 16 | 15 | 0 | 1 | 6.7→6.8 |
| helper-v2 p27 (both) | — | — | — | — | 3 | 0 (sibling declined ×3) | 0 | 0 | 5.8→4.3 |
| helper-v1 p3 30 ms | 61 | 83 | 96 | 125 | 191 | — | — | — | 4.6→4.8 |
| helper-v1 p3 0 ms | 61 | 80 | 154 | 188 | 149 | — | — | — | 4.8→4.7 |
| helper-v1 pmax 30 ms | 81 | 104 | 115 | 123 | 132 | — | — | — | 4.7→4.8 |
| helper-v1 p27 30 ms | 202 | 241 | 250 | 263 | 55 | — | — | — | 4.7→4.4 |
| helper-v1 p27 0 ms | 196 | 236 | 247 | 252 | 24 | — | — | — | 4.4→4.2 |

(ms; every keystroke painted, unpainted ones covered by the next painted
revision = "coalesced"; candidate validation p50 23–28 ms for p3, 46 ms for
pmax.) Earlier passes are kept for the load story: `smoke-under-load/` (load
30: v2 p3 p50 275), `pass1-load33-94/` (load 33–94: v2 p3 p50 705/745, v1 p50
60/61), `pass2-load5-10-partial/` (v2 p3 p50 245/265 at load 9–10).

Findings from the measurement:
- The helper route paints ~4–6× later than the v1 pane for the same helper
  and producer (p3: 296 vs 61 ms p50). The cycle is dominated by the sibling
  itself: the producer serialises 3.9–7.8 MB JSON per keystroke, the helper
  reads and re-emits it, the shell decodes/prepares it off-main (23–46 ms), and
  the edit pipeline is held for the sibling so v1 previews per burst drop from
  191 to 45 (p3) — only while candidates are ON; OFF (default) is unaffected.
- Sibling size caps disagree: the producer declines above its 16 MiB line
  limit (p27), but the helper's default compiler-frame cap is 8 MiB and a
  sibling between the two (14 pages, 13.7 MB) makes the helper FAIL the
  compiler session (`compiler session failed; create a new session with
  complete snapshots`; the v1 pane stops until a restart) rather than drop the
  frame — `pass4-cap15-14pages/` (`helper-v2-pmax-30ms`, session failed = 1).
  With `FLASHTEX_CONTROLLER_MAX_FRAME_BYTES=15728640` the same seed paints
  (`helper-v2cap15-pmax-30ms`: p50 1239 / p95 2205 / p99 2446 ms, 20 admitted,
  7 painted, 13 dropped at the paint-time recheck because a newer revision
  had been applied meanwhile). Reported for the helper/producer owners; the
  native route does not change either cap.

## Exact wire of one exchange (GH36 review 5646386345)

`docs/evidence/helper-display-route-2026-09-12T1356Z/exchange/`: `helper-stdin.jsonl`
(every frame the shell wrote), `helper-stdout.jsonl` (every frame the helper
wrote back, byte for byte, including the `update{kind:"display_candidate"}`
lines with the sibling inside), `app.log` (native admission/validation/paint
decisions), `bench.json`, and `README.md` (frame index; absolute paths of this
machine are present in the frames and listed there, nothing removed).
Reproducible with `capture-exchange.sh`. Order on the wire: `configure_layout
[rules-v1, font-hints-v1]` (never `display-list-v2`) → `configure_display_candidates
{capability, enabled:true, renderer_support_confirmed:true}` → its `result` →
the helper's own recompile carries `display-list-v2` (the accepted set in the
following `preview` update) → `edit` → `preview` → `display_candidate`.
`DisplayCandidateTests.testHelperOrderingEnableBeforeLayoutWithDisplayListV2`
drives the reversed order on the real helper (`configure_layout` with
`display-list-v2` before the opt-in → exactly `display candidates must be
enabled before requesting their layout`, session continues as legacy v1) and
the required order (opt-in acknowledged, then `configure_layout` with
`display-list-v2` accepted, candidates paint).

## Measurement harness

`run.sh` (see its header; `SKIP_BUILD=1`, `SEEDS`, `CAP15_SEEDS`, `V2_LIMIT`,
`RENDER_SHA`, `INTERVALS`), `seeds.py` (page counts and sibling sizes probed
through the actual producer), `summarize.py`, `capture-exchange.sh`.

## Limitations (honest)

- No pixel-parity claim: the frame painted is the producer's v2 sibling
  validated by hash/length/fonts/geometry, not compared against a PDF raster.
- Latency is from one machine (M1 Max) at load 4–7 with other agents idle,
  not an isolated benchmark; the 27-page fixture has NO v2 latency because the
  pinned producer declines its sibling — the v1 control (p50 ~200 ms) is that
  seed's number through the same helper.
- The helper's optional slot can still drop a candidate when a required reply
  lands inside its writer's 2 ms idle window; in the quiet-window cells every
  admitted candidate except the 1–2 dropped at the paint-time recheck painted.
- The in-flight-edit hold (only while candidates are ON) costs v1 throughput:
  45 v1 previews per 200-keystroke burst instead of 191.
- Oversized siblings between the helper's 8 MiB compiler-frame cap and the
  producer's 16 MiB limit cost the compiler session (helper behaviour); no
  native test produces one through the real producer, the measurement does.
- Full `swift test` was not run in this session (filtered suites only:
  52/52, 0 skips); the parent's integration run covers the whole suite.

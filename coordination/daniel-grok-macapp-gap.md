# Mac app <-> flashtex-bridge gap analysis (daniel-grok-macapp)

Agent / branch: agent/daniel-grok-macapp/bridge, worktree `ft-wt-grok-macapp`,
base `origin/main` at `abbe88a5`.

## Headline finding (corrects the task's starting assumption)

The assignment's premise was "find the integration point, or establish that
none exists" and "if a small piece is missing, implement it." Neither applies:
**the Mac app already has a complete, tested Swift integration with
flashtex-bridge**, covering every message in the verified sequence
(`document_open` -> `destination_pin` -> `capture_submit` -> `capture_convert`
-> `capture_prepare_insert` -> `capture_applied`), plus UI, an edit-ledger
durability layer, and both a local fixture test suite and a gated
real-binary test suite. This was merged to `main` in two prior commits:

- `07ce022d` "mac: integrate the FT-007 capture bridge (transfer-v1) with a
  reviewed, ledgered insertion"
- `fbf5e124` "mac: durable edit-ledger adoption and issue #2 fault/recovery
  fixes for the bridge"

No new Swift code was written for this task. Adding a duplicate/parallel
`BridgeClient` as the task suggested as a fallback would have been pure
speculative rework of code that already exists, builds, and passes tests, so
none was added (per "prefer the simplest solution that works, no speculative
abstractions").

## EXISTS / PARTIAL / MISSING

| Step | Status | Evidence |
|---|---|---|
| Process spawn / JSON-Lines transport to `flashtex-bridge` | EXISTS | `Sources/FlashTeXMac/BridgeClient.swift:10-49` (`LineProcessClient`-backed, newline-delimited JSON, `{protocol_version,id,type,payload}` envelope matches verified shape); binary discovery in `BridgeClient.swift:100-119` (`$FLASHTEX_BRIDGE`, bundled helper, or `crates/bridge/target/{release,debug}/flashtex-bridge`) |
| `document_open` | EXISTS | `Sources/FlashTeXMac/BridgeSession.swift:267-269` (`func open`), invoked from `Sources/FlashTeXMac/ShellModel+Bridge.swift:117` during attach, and `:186` on document replacement |
| `destination_pin` | EXISTS | `BridgeSession.swift:346-348` (`func pin`); UI trigger "Pin Insertion Point" (⌘⇧P) in `Sources/FlashTeXMac/FlashTeXMacApp.swift:70-71`, wired via `ShellModel+Bridge.swift:193-212` (`bridgePin`/`bridgePinAndWait`) |
| Capture acquisition (photo -> base64) | PARTIAL | `ShellModel+Bridge.swift:218-234`: `submitSampleCapturePanel()` opens an `NSOpenPanel` for an existing PNG/JPEG file and base64-encodes it (`Data.base64EncodedString()` at line 232). This is a file picker, not a live camera or Apple Pencil capture surface — there is no `AVFoundation`/`PencilKit`/camera code anywhere under `apps/mac/Sources` (grepped, zero hits). For a demo this is sufficient if a capture image already exists as a file (e.g. AirDropped screenshot of handwriting); it is not a "point phone at paper" flow. A second acquisition path exists: `Sources/FlashTeXMac/ShellModel+Nearby.swift:42-65` receives `capture_submit`-shaped payloads over the LAN from a paired companion device and forwards them to the bridge the same way — but the companion side (an iPhone/iPad app that would actually take the photo) is not part of this repo's only Apple target and does not exist. `docs/nearby-v1-proposal.md` even labels the wire format "not yet a published contract." |
| `capture_submit` | EXISTS | `BridgeSession.swift:358-360` (`func submit`); built and sent from `ShellModel+Bridge.swift:236-254` (`submitCapture(image:captureId:instructions:)`), which reads the pinned destination's `baseRevision` (never guessed) |
| App shows the proposal for review | EXISTS | `capture_convert` sent from `BridgeSession.swift:381-385` / triggered by `ShellModel+Bridge.swift:259-275`; review UI is `ProposalReviewSheet` in `Sources/FlashTeXMac/ContentView.swift:300-360`, including a live shadow-compile preview (`ProposalPreview.swift`) and display of ambiguities/required packages from the proposal payload |
| User approves | EXISTS | `Button("Approve and insert")` at `ContentView.swift:346` calls `model.approveBridgeProposal(proposal, latex:)` |
| `capture_prepare_insert` | EXISTS | `BridgeSession.swift:401-403` (`func prepare`); called from `ShellModel+Bridge.swift:307` inside `approveBridgeProposal`, which then verifies revision/SHA-256/removed-text (`BridgeSession.verify`, `ShellModel+Bridge.swift:325`) before touching the document |
| Durable commit (edit ledger) | EXISTS | `ShellModel+Bridge.swift:332-351` calls `bridge.applyDurably`; distinct from the bridge itself (`Sources/FlashTeXMac/EditLedgerClient.swift`), matching the contract note at `BridgeSession.swift:23` (`capture_applied` -> helper `confirm`) |
| Apply edit to open document | EXISTS | `ShellModel+Bridge.swift:353-361` sets `pendingEdit`, which the `SourceEditorView` applies as one undoable NSTextView edit; the round-trip is detected in `bridgeTextChanged` (`ShellModel+Bridge.swift:166-180`) |
| `capture_applied` | EXISTS | `BridgeSession.swift:561` (`applicationApplied`, called from `ShellModel+Bridge.swift:172` once the editor confirms the buffer matches the durable document) sends the receipt via `BridgeSession.swift:618-619` (`sendApplied` -> `client.send(.captureApplied, ...)`) |
| Reject path | EXISTS | `BridgeSession.swift:415` (`func reject`), UI: `Button("Reject", ...)` in `ContentView.swift:344` |
| Restart / crash reconciliation | EXISTS | `ShellModel+Bridge.swift:98-115` (`session.reconcile`), `BridgeSession.swift:700-770` |
| Rust binaries present in *this* worktree | MISSING (operational, not code) | `find crates/bridge crates/edit-ledger -name 'flashtex-*' -type f` returned nothing; no `target/` build output exists here. `crates/bridge` and `crates/edit-ledger` sources and `Cargo.toml` do exist, and `cargo` is on PATH (`/opt/homebrew/opt/rustup/bin/cargo`), so this is a `cargo build` step, not missing code. Not attempted — out of scope and the task says the Rust side was already proven elsewhere by the parent. |
| Live AI conversion (Grok) | MISSING for this task's constraints | `capture_convert` without `--enable-grok`/an API key returns `provider_disabled` (exercised by `RealBridgeTests.swift:64-67` and `ShellModelBridgeTests`'s `testProviderDisabledIsPlainTextAndRejectIsForwarded`). The task explicitly forbids API keys/live calls, so the "app shows the proposal for review" step cannot be demonstrated with a *real* transcription today — only with a bridge run in a mode that already has a proposal (e.g. a pre-seeded fixture) or in provider-disabled mode (submit succeeds, convert fails cleanly). |

## Build verification (real output)

```
$ cd apps/mac && swift build
Building for debugging...
...
[52/55] Write Objects.LinkFileList
[53/55] Linking FlashTeXMac
[54/55] Applying FlashTeXMac
Build complete! (13.95s)
```

Clean build. Only pre-existing warnings in `WorkerClient.swift` (Swift 6
main-actor isolation warnings on `TypingBench.shared`, unrelated to the
bridge) — no errors.

Also ran the two relevant test suites (fixture/fake-process based, no
`FLASHTEX_BRIDGE` needed, hermetic):

```
$ swift test --filter FlashTeXMacTests.BridgeClientTests
Executed 6 tests, with 0 failures (0 unexpected)

$ swift test --filter FlashTeXMacTests.ShellModelBridgeTests
Executed 4 tests, with 0 failures (0 unexpected)
```

`ShellModelBridgeTests.testFullReviewedInsertionFlowAndLedgerIdempotence`
specifically exercises submit -> convert -> review -> approve -> durable
apply -> `capture_applied`, end to end, against `Fixtures/fake_bridge.py` and
`Fixtures/fake_edit_ledger.py`. It passed.

`RealBridgeTests.swift` and `RealEditLedgerTests.swift` (gated on
`$FLASHTEX_BRIDGE` / `$FLASHTEX_EDIT_LEDGER` pointing at built binaries) were
not run — no binaries exist in this worktree and building the Rust crates
was out of scope for this task.

## Direct answer

**Can this machine demo the full flow through the Mac app today?** Not
literally today, in *this* worktree, but not because of any missing Swift
code — the Mac side is done and tested. The blockers are two build/config
prerequisites, neither of which is a code gap:

1. `flashtex-bridge` and `flashtex-edit-ledger` need to be built in this
   worktree (`cargo build --release -p bridge -p edit-ledger` or debug) and
   either be at `crates/{bridge,edit-ledger}/target/{release,debug}/...` or
   pointed to via `$FLASHTEX_BRIDGE`/`$FLASHTEX_EDIT_LEDGER` — currently
   neither exists here.
2. A real capture-to-LaTeX conversion needs `--enable-grok` plus a live
   provider key, which this task is explicitly barred from using. Without
   it, `capture_submit` and the durable/receipt machinery can be shown
   live, but `capture_convert` will only ever return `provider_disabled`,
   not an actual proposal — so the "review a real transcription" beat of
   the demo needs either a provider key (out of scope here) or a
   pre-canned proposal fixture substituted in for that one step.

The single biggest missing piece for the demo, once those two operational
prerequisites are handled, is **capture acquisition**: there is no live
camera or Apple Pencil capture surface anywhere in `apps/mac` — only a file
picker for an already-existing PNG/JPEG (`submitSampleCapturePanel`,
`ShellModel+Bridge.swift:218-226`). A "point at a napkin and photograph it"
demo beat does not exist in code; it must be staged as "here is a photo I
already took" opened through Edit > Submit Sample Capture….

## Proven vs. inferred

- Proven directly (read the code, ran the build, ran the tests): every
  EXISTS row above, the clean `swift build`, and the two passing test
  suites.
- Proven directly (negative result): no camera/PencilKit code, no built
  Rust binaries in this worktree.
- Inferred, not verified by me: that the Rust side "works end to end" is
  taken as given from the parent's established facts, not re-verified here
  (no binaries were built or run in this worktree).

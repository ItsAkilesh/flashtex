# FlashTeXPad — iPad client of the Mac (acceptance slice)

Owner: lane `mac-ios-app` (parent `mac-claude-a`, machine `mac-m1max-a`).
Issue #51 / issue #2 comment 5647841253. iPad **simulator only**; no device run.

## What this is, truthfully

iOS cannot spawn the Rust helpers (`flashtex-compiler`, `preview-controller`,
`flashtex-assistant-context` are subprocess binaries), so the iPad is a
**client of the Mac** over the contracts that already exist, and adds no
second protocol:

| Flow | Real on the wire? | Contract | Where |
|---|---|---|---|
| Pairing with the Mac's code (HKDF → TLS-PSK → `hello`/`hello_ack`, `pair_psk`) | **Yes** | nearby-v1 §2/§4 | `Packages/FlashTeXPadKit/Sources/NearbyClient` (symlink to `apps/mac/tools/nearby-client`) |
| Current pinned destination (`hello_ack.destination`, `destination_query`) | **Yes** | nearby-v1 §4 | `MacLink.swift` |
| `capture_submit` → `capture_received {capture_id, durable, has_proposal, applied}` | **Yes** | transfer-v1 via nearby-v1 | `MacLink.submitCapture` |
| Reviewed proposal display → explicit approve/cancel → single insertion with revision + SHA-256 + exact `removed_text` checks | **Local** | assistant-context `proposal_review` / `approved_group` (`crates/assistant-context/README.md`, recorded in `examples/review-workflow.json`) | `ReviewedProposal.swift`, `ReviewSession.swift` |
| Diagnostics list | **Local** | runtime-v1 `compile_result.diagnostics` | `Diagnostics.swift`; bundled fixture or a `compile_result` JSON the user opens |
| Completions | **Local** | none (source-derived, same first four sources as the Mac's `Completion`) | `LocalCompletion.swift` |
| UTF-8 offset discipline (caret UTF-16 ↔ UTF-8 bytes, scalar-boundary refusal) | shared code | runtime-v1 | `FlashTeXProtocol` symlink (`ByteOffsets.swift`) |

**Not carried by transfer-v1 / nearby-v1** (proposal §6, verbatim: "No
companion → Mac notification of proposals or insertion results; the companion
only learns `capture_received`. Proposal review stays on the Mac."): compile
results, diagnostics, completions, proposals, insertion receipts. The app shows
a banner on each such panel naming the provenance (`PadModel.Provenance`); the
review the iPad performs is against the recorded assistant-context fixture
(`provider_called:false`), never presented as a Mac reply. Extending the
transport is a contract change for the Commander, not something this slice
invents.

## Layout

```
apps/ios/
  FlashTeXPad.xcodeproj/            generated — do not hand-edit
  scripts/generate-xcodeproj.py     deterministic pbxproj + scheme writer (stdlib only)
  Packages/FlashTeXPadKit/          SwiftPM package (iOS 17 / macOS 14)
    Sources/FlashTeXProtocol  -> ../../../../mac/Sources/FlashTeXProtocol            (symlink, read-only reuse)
    Sources/NearbyClient      -> ../../../../mac/tools/nearby-client/Sources/NearbyClient (symlink, read-only reuse)
    Sources/FlashTeXPadKit/   PadDocument, ReviewedProposal, ReviewSession, MacLink, LocalCompletion, Diagnostics
  FlashTeXPad/                      SwiftUI app: sidebar (Editor / Diagnostics / Review / Mac link), file importer, UITextView editor
    Resources/demo.tex              copy of apps/mac/Samples/demo.tex
    Resources/review-workflow.json  copy of crates/assistant-context/examples/review-workflow.json
    Resources/compile-result.json   copy of protocol/fixtures/compile-result.json
  FlashTeXPadTests/                 XCTest (hosted in the app): FakeMac fixture + AcceptanceSliceTests
  FlashTeXPadUITests/               XCUITest: open sample → review pending → cancel → approve → receipt
```

## Build and test (Xcode 26.3, iOS 26.3 simulator runtime)

```
cd apps/ios
python3 scripts/generate-xcodeproj.py       # only after adding/removing source files
xcodebuild -project FlashTeXPad.xcodeproj -scheme FlashTeXPad \
  -destination 'platform=iOS Simulator,name=iPad Air 11-inch (M3)' build
xcodebuild -project FlashTeXPad.xcodeproj -scheme FlashTeXPad \
  -destination 'platform=iOS Simulator,name=iPad Air 11-inch (M3)' test
```

`xcrun simctl list devices available | grep iPad` for the names this Xcode
offers (iPad Pro 13-inch (M5), iPad Air 11/13-inch (M3), iPad mini (A17 Pro),
iPad (A16)). Evidence for the recorded run: `docs/evidence/ios-acceptance-2026-09-12/`.

Deterministic launch for screenshots: `xcrun simctl launch <udid>
tech.jay3332.flashtex.FlashTeXPad -flashtexpad-open sample|fixture`.

## The tests (all in the simulator)

`AcceptanceSliceTests` starts a `FakeMac` **inside the test process** (a copy
of the reference client's test fixture: same TLS-PSK parameters as
`NearbyListener`, verifies `hello.proof`, issues `pair_psk` on the bootstrap
key, answers `destination_query` and `capture_submit`; loopback only; no
provider, no Grok, no Rust helper). Then:

- (a) `testOpensBundledSample` — opens `demo.tex`, checks UTF-8/UTF-16 caret mapping on "Résumé".
- (b) `testPairAndCaptureReceiptOverNearbyV1` — pairs with the code, receives
  `hello_ack` with the destination, sends one capture, gets `capture_received`
  echoed; the transcript never contains the `pair_psk`.
- (c) `testCancelPendingProposalInsertsNothing` — pending review from the
  fixture, cancel → document byte-identical, no receipt, later approve refused.
- (d) `testApproveInsertsExactlyOnceAndEchoesReceipt` — paired, capture sent,
  approve → exactly one `Example text` insertion at the receipt's byte range,
  revision +1, SHA-256 after matches; second approve refused (`notPending`).
- `testApproveRefusedWhenBufferDrifted` — an edited buffer fails the SHA check and nothing is written.
- `testWrongPairingCodeIsRefused`, `testLocalCompletionsFromSource`.

`FlashTeXPadUITests.testOpenSampleThenCancelThenApprove` drives the same
gate through the UI and attaches screenshots.

## Gaps (honest)

- Signing: `CODE_SIGN_IDENTITY=-`, no team; simulator only. A device build
  needs a team and the Local Network entitlement prompt flow.
- Persistence: pairings go to `PairFile` (Application Support JSON, 0600), not
  the Keychain — same caveat the reference client documents. Documents are
  not saved back (no `UIDocument`); the file importer reads only.
- Discovery: `NearbyBrowser` (Bonjour) is linked but the pairing screen takes
  host/port/salt/fp typed from the Mac's window so the simulator run is
  deterministic; no scan UI.
- Offline / App Store / iPhone layout: not addressed. `TARGETED_DEVICE_FAMILY=2`.
- No compile on the iPad; no Mac-side proposal reaches the iPad (contract gap
  above). Photo capture (camera/PhotosPicker) is not wired; the capture button
  sends the 1×1 PNG fixture from `protocol/fixtures/capture-submission.json`.

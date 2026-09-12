# FlashTeXPad — iPad capture companion for the Mac app (acceptance slice)

Owner: lane `mac-ios-app` (parent `mac-claude-a`, machine `mac-m1max-a`).
Issue #51 / issue #2 comment 5647841253, re-centred per the user's scope
correction: the iPad is the **capture companion** (FT-004's original idea) —
draw with Apple Pencil or photograph a sketch/matrix, add an instruction, send
it to the Mac, and the Mac's bridge converts it into a reviewed LaTeX/TikZ
proposal that the Mac user approves. iPad **simulator only**; no device run.

## What is real vs. not carried (truthful table)

| Flow | Real on the wire? | Contract | Where |
|---|---|---|---|
| Pairing with the Mac's code (HKDF → TLS-PSK → `hello`/`hello_ack`, `pair_psk` stored) | **Yes** | nearby-v1 §2/§4 | `Packages/FlashTeXPadKit/Sources/NearbyClient` (symlink to `apps/mac/tools/nearby-client`), `MacLink.swift` |
| Current pinned destination (`hello_ack.destination`, `destination_query`) | **Yes** | nearby-v1 §4 | `MacLink.swift`, shown on the Capture screen |
| Apple Pencil canvas (PencilKit `PKCanvasView`, tool picker, finger allowed) → PNG | local | — | `CaptureView.swift` (`PencilCanvas`) |
| Photo picker (`PhotosPicker`) and bundled sample image (`sample-capture.png`) → PNG | local | — | `CaptureView.swift` (camera does not exist in the simulator) |
| Instruction text (≤ 4096 bytes) | local | transfer-v1 `capture_submit.instructions` | `CaptureQueue.validate` |
| **Send to Mac**: `capture_submit {capture_id, destination_id, base_revision, image{png}, instructions}` → `capture_received {capture_id, durable, has_proposal, applied}` | **Yes** | transfer-v1 over nearby-v1, exactly as the reference client | `CaptureQueue.send` |
| Per-capture status list: drafted → sending → received (durable / not) · refused `{code}` · not acknowledged (retry same `capture_id`) · discarded | **Yes** (from the receipt / error) | nearby-v1 §4, §8 | `CaptureQueue.swift`, `CapturesList` |
| Duplicate/retry: after a disconnect the same `capture_id` + payload is re-sent; the Mac de-duplicates | **Yes** | nearby-v1 §4 | `CaptureQueue.send` (`attempt` counter), test `testRetryAfterDisconnectReusesCaptureID` |
| Conversion progress after the receipt — "proposal ready", "inserted", "rejected" — and the returned LaTeX/TikZ text | **NOT carried** | nearby-v1 §6: "No companion → Mac notification of proposals or insertion results; the companion only learns `capture_received`." `capture_status` is a Mac↔bridge request; the Mac's listener answers an identical retry with the *cached* ack (`NearbyAckMemory`), so re-sending is a delivery guarantee, not a status probe. | Banner in the list: "Returned LaTeX/TikZ: not carried by transfer-v1 — review it on the Mac." Nothing is faked. |
| QR pairing | not built | — | code entry only (no camera in the simulator); host/port/salt/fp typed from the Mac's Nearby window; `NearbyBrowser` (Bonjour) is linked but has no scan UI yet |
| `.tex` editor / diagnostics / reviewed-proposal gate | local, **reference only** | runtime-v1 / assistant-context shapes | kept under the sidebar section "Reference (.tex on the Mac; not the product)" from the first iteration; tested, small, not the product |

Extending the transport so the iPad learns the proposal/insertion outcome is
a contract change for the Commander (e.g. carrying `capture_status` over the
nearby session) — not invented here.

## Layout

```
apps/ios/
  FlashTeXPad.xcodeproj/            generated — do not hand-edit
  scripts/generate-xcodeproj.py     deterministic pbxproj + scheme writer (stdlib only)
  Packages/FlashTeXPadKit/          SwiftPM package (iOS 17 / macOS 14)
    Sources/FlashTeXProtocol  -> ../../../../mac/Sources/FlashTeXProtocol            (symlink, read-only reuse)
    Sources/NearbyClient      -> ../../../../mac/tools/nearby-client/Sources/NearbyClient (symlink, read-only reuse)
    Sources/FlashTeXPadKit/   CaptureQueue (product), MacLink, PadDocument, ReviewedProposal, ReviewSession, LocalCompletion, Diagnostics
  FlashTeXPad/                      SwiftUI app: Capture (primary), Mac link, reference .tex panels
    CaptureView.swift               PencilKit canvas, PhotosPicker, sample image, instruction, Prepare/Discard/Send, status list
    Resources/sample-capture.png    320×240 sketch (triangle with a right-angle mark), generated
    Resources/demo.tex, review-workflow.json, compile-result.json   reference-panel fixtures
  FlashTeXPadTests/                 XCTest hosted in the app: FakeMac + CaptureQueueTests + AcceptanceSliceTests
  FlashTeXPadUITests/               XCUITest: CaptureFlowUITests (runner hosts FakeMac), FlashTeXPadUITests (.tex reference)
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

`xcrun simctl list devices available | grep iPad` lists the names this Xcode
offers. Evidence for the recorded run: `docs/evidence/ios-acceptance-2026-09-12/`.

Automation hooks (launch arguments, no network unless given):
`-flashtexpad-test-mac host:port:saltHex:fp:code` pairs with a listener at
launch; `-flashtexpad-open sample|fixture` opens a reference document.

## The proofs (all in the simulator)

Mac-side fixture: `FakeMac` — a copy of the reference client's test fixture
(same TLS-PSK parameters as `NearbyListener`, verifies `hello.proof`, issues
`pair_psk` on the bootstrap key, answers `destination_query` and
`capture_submit` with an in-memory `durable:false` receipt, errors on anything
else; loopback only; no provider, no Grok, no Rust helper). It is hosted in
the test process (unit tests) or in the XCUITest runner (UI tests).

- `CaptureFlowUITests.testDrawSendReceipt` — app pairs with the runner's
  FakeMac at launch; three finger drags draw a triangle on the PencilKit
  canvas; instruction "Convert this triangle to TikZ"; Prepare → Send; the
  list shows "received — Mac inbox…"; the runner asserts the Mac got one
  `capture_submit` with a structurally valid PNG (`NearbyWire.checkImage`),
  the instruction, `destination_id dest-tikz`, `base_revision 3`.
- `CaptureFlowUITests.testDiscardBeforeSendSendsNothing` — draw, Prepare,
  Discard → status "discarded before sending", the Mac received nothing.
- `CaptureFlowUITests.testSampleImagePath` — bundled sample image → Prepare →
  Send → receipt; PNG valid on the Mac side.
- `CaptureQueueTests` (hosted): PencilKit drawing → PNG → sent bytes arrive
  unchanged with the instruction and destination; discarded draft never
  leaves; retry after a dropped connection reconnects with the stored
  `pair_psk` and re-sends the same `capture_id` (Mac sees it once); local
  refusal of a bad image / oversized instruction; no pinned destination →
  refused, nothing sent.
- `AcceptanceSliceTests` / `FlashTeXPadUITests` — the first-iteration `.tex`
  reference flow (open sample, pair, capture receipt, cancel inserts nothing,
  approve inserts exactly once with the receipt echoed).

## Gaps (honest)

- No proposal/insertion outcome on the iPad (contract gap above); the Mac
  user reviews and approves there.
- Signing: `CODE_SIGN_IDENTITY=-`, no team; simulator only. A device build
  needs a team and the Local Network prompt flow (`NSLocalNetworkUsageDescription`
  and `NSBonjourServices` are in Info.plist).
- Persistence: pairings in `PairFile` (Application Support JSON, 0600), not
  the Keychain; drafts and receipts are in memory only.
- No QR scan, no Bonjour browse UI, no camera path (simulator has none;
  `PhotosPicker` and the bundled sample stand in), no offline queue beyond
  the in-memory retry, no App Store packaging, iPad only (`TARGETED_DEVICE_FAMILY=2`).

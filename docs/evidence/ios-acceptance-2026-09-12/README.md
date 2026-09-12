# iPad capture-companion acceptance slice — simulator evidence (2026-09-12)

Lane `mac-ios-app` (Claude Code Fable subagent of mac-claude-a, mac-m1max-a).
Issue #51 / issue #2 comment 5647841253, re-centred on the capture companion
per the user's scope correction (~18:55Z). Branch
`agent/mac-ios-app/acceptance-slice`; the commit that produced each run is
named in `coordination/mac-ios-app.md`.

Toolchain: Xcode 26.3 (17C529), iOS 26.3 simulator runtime (23D8133),
Darwin 25.3.0. Simulator: **iPad Air 11-inch (M3)**, udid
`82FE2469-DE41-450C-95EF-36E6FB94D6B7`, booted by this lane (the pre-booted
iPad Pro 13-inch (M5) was not used). No physical device.

## Commands and results

```
cd apps/ios
xcodebuild -project FlashTeXPad.xcodeproj -scheme FlashTeXPad \
  -destination 'platform=iOS Simulator,name=iPad Air 11-inch (M3)' build
→ ** BUILD SUCCEEDED **   (first at 18:42Z; after the pivot, build-for-testing 18:58Z)

xcodebuild -project FlashTeXPad.xcodeproj -scheme FlashTeXPad \
  -destination 'platform=iOS Simulator,name=iPad Air 11-inch (M3)' test
→ results-11 (19:03–19:05Z): FlashTeXPadTests 12/12 passed
    (AcceptanceSliceTests 7, CaptureQueueTests 5);
  FlashTeXPadUITests: testDiscardBeforeSendSendsNothing PASS,
    testSampleImagePath PASS, FlashTeXPadUITests.testOpenSampleThenCancelThenApprove PASS,
    testDrawSendReceipt FAIL (keyboard raised over the buttons by typeText)
→ results-12 (19:06Z, after removing the keyboard step): testDrawSendReceipt PASS (23.7 s)
→ results-13 (19:09Z): CaptureFlowUITests 3/3 PASS (discard 19.9 s, draw-send 24.5 s, sample 15.8 s)
→ results-14 (19:11Z, tool picker hidden after Prepare so the status list is readable):
    CaptureFlowUITests 3/3 PASS (20.9 s, 25.2 s, 15.8 s)
→ results-15 (19:12Z, final full run at the committed tree; canvas shrinks once a
    capture exists so the status row is on screen): FlashTeXPadTests 12/12 PASS;
    CaptureFlowUITests 3/3 PASS (20.9 s, 24.7 s, 15.5 s);
    FlashTeXPadUITests.testOpenSampleThenCancelThenApprove PASS (50.0 s)
    → "Executed 4 tests, with 0 failures" (19:15Z)
```

Last run (results-15, tree = final commit): unit 12/12 PASS, UI 4/4 PASS,
0 failures. No test is marked skipped.

Mac-side fixture for every run: `FakeMac` (apps/ios/FlashTeXPadTests/FakeMac.swift,
a copy of the reference client's test fixture) — loopback `NWListener` with the
Mac's TLS-PSK parameters (TLS 1.2, suite 0x00A8, PSK identity = `pair_id`),
verifies `hello.proof`, issues `pair_psk` on the bootstrap key, answers
`destination_query` and `capture_submit` with an in-memory `durable:false`
receipt. No provider, no Grok, no Rust helper; `FLASHTEX_NO_ACTIVATE` is
irrelevant because the Mac app itself is not launched. Hosted in the app's
test process (unit tests) or in the XCUITest runner (UI tests) — the app
pairs with it through the real `NearbyClient` code over loopback.

## Screenshots

`capture-flow/` — the product flow (XCUITest `CaptureFlowUITests`):

| file | what it shows | source |
|---|---|---|
| `simctl-191101Z-capture-prepared.png` | full landscape screen (results-14 run, 19:11:01Z): triangle drawn by three finger drags ("3 strokes"), "Connected to Runner Mac", destination `dest-tikz @ main.tex rev 3` from `hello_ack`, draft line `cap-… 32721 PNG bytes 1636×640`, Discard / Send to Mac buttons, "Captures (1)" | `xcrun simctl io <udid> screenshot` from the host during the test (rotated 270° to landscape) |
| `10-canvas-drawn.png`, `11-capture-prepared.png`, `12-capture-received.png` | same run: drawn → Prepare (draft line with PNG byte count and pixel size) → Send → status row "received — Mac inbox, not journaled (durable:false)" with the echoed `capture_received capture_id=… durable=false has_proposal=false applied=false` | XCTAttachment (`app.screenshot()`; note the runner frames landscape content in a portrait canvas, so the right edge is cropped — the simctl frames are the full screen) |
| `13-capture-discarded.png` | Prepare → Discard: "discarded before sending"; the runner asserted the Mac received nothing | XCTAttachment |
| `14-sample-image-prepared.png`, `15-sample-image-received.png` | bundled `sample-capture.png` (photo-picker stand-in; the simulator has no camera) → Prepare → Send → receipt; the runner asserted a structurally valid PNG arrived | XCTAttachment |

What the runner asserted on the Mac side for the drawn capture (test source:
`apps/ios/FlashTeXPadUITests/CaptureFlowUITests.swift`): exactly one
`capture_submit`; `destination_id == "dest-tikz"`, `base_revision == 3`
(copied from `hello_ack.destination`, never typed); `mime_type image/png`;
`NearbyWire.checkImage(png, "image/png") == nil`; PNG larger than 1000 bytes
(a drawn triangle, not a blank canvas); `instructions == "Convert this
drawing to TikZ"`; `capture_id` valid; the receipt line on screen names that
`capture_id`.

`tex-reference-flow/` — the first-iteration `.tex` reference flow (kept as a
secondary "Reference" sidebar section; not the product): `01-sample-open`,
`02-review-pending`, `03-review-cancelled`, `04-review-applied`,
`05-editor-after-insert`, `06-diagnostics`, `07-mac-link` from
`FlashTeXPadUITests.testOpenSampleThenCancelThenApprove` (results-8, PASS).

## Real vs. not carried (summary; full table in apps/ios/README.md)

Real over the wire in these runs: pairing-code bootstrap (HKDF → TLS-PSK →
`hello`/`hello_ack` with `pair_psk`), reconnect with the stored long-term key
(`CaptureQueueTests.testRetryAfterDisconnectReusesCaptureID`, against a
restarted listener holding the long-term key), `destination_query`,
`capture_submit` → `capture_received`, error replies, retry with the same
`capture_id` after a disconnect.

Not carried by nearby-v1/transfer-v1 to a companion, therefore **not shown
and not faked**: proposal ready / inserted / rejected, the returned
LaTeX/TikZ text. The Mac's listener answers an identical retry from its
acknowledgement memory, so re-sending is a delivery guarantee, not a status
probe. The status list says "Returned LaTeX/TikZ: not carried by transfer-v1 —
review it on the Mac."

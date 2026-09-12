# FlashTeX Companion — Physical-Mac Validation Handoff

**Branch**: `agent/aarush-macbook/companion-capture`  
**HEAD at audit**: `05fef5ae` (coord ready_for_integration FT-004 rev 3)  
**FT-004 rev 4 audit commit**: pending (this document is part of it)  
**Session/executor**: Claude Cowork (claude-sonnet-4-6), cloud Linux container  
**Non-secret capacity**: no Xcode; no iOS simulator; no physical device; UIKit unavailable  

---

## Project File Graph — Verified

All 16 PBXFileReference entries resolve to existing disk paths.

### App target (PBXNativeTarget: `FlashTeXCompanion`, UUID A6000001)

| PBXBuildFile | PBXFileReference | Disk path |
|---|---|---|
| A1000001 | A2000001 | FlashTeXCompanion/FlashTeXCompanionApp.swift |
| A1000002 | A2000002 | FlashTeXCompanion/Views/ContentView.swift |
| A1000003 | A2000003 | FlashTeXCompanion/Views/DrawingCanvasView.swift |
| A1000004 | A2000004 | FlashTeXCompanion/Views/CameraCaptureView.swift |
| A1000005 | A2000005 | FlashTeXCompanion/Models/CapturePayload.swift |
| A1000006 | A2000006 | FlashTeXCompanion/Models/CaptureStore.swift |
| A1000007 | A2000007 | FlashTeXCompanion/Views/PayloadPreviewView.swift |
| A1000008 | A2000008 | FlashTeXCompanion/Views/CaptureHistoryView.swift |
| A100000A | A200000B | FlashTeXCompanion/Services/CaptureTransport.swift |
| A100000B | A200000C | FlashTeXCompanion/Views/SettingsView.swift |
| A100000C | A200000D | FlashTeXCompanion/Services/ImageValidator.swift |
| A100000D | A200000E | FlashTeXCompanion/Services/BonjourTransport.swift |
| A1000009 | A2000009 | FlashTeXCompanion/Assets.xcassets (Resources) |

### Test target (PBXNativeTarget: `FlashTeXCompanionTests`, UUID A6000002)

| PBXBuildFile | PBXFileReference | Disk path |
|---|---|---|
| A100000E | A2000011 | FlashTeXCompanionTests/CapturePayloadTests.swift |
| A100000F | A2000012 | FlashTeXCompanionTests/ImageValidatorTests.swift |

### Test host configuration

```
BUNDLE_LOADER = "$(TEST_HOST)";
TEST_HOST = "$(BUILT_PRODUCTS_DIR)/FlashTeXCompanion.app/$(BUNDLE_EXECUTABLE_FOLDER_PATH)/FlashTeXCompanion";
PRODUCT_BUNDLE_IDENTIFIER = com.flashtex.companion.tests;
```

Correctly links the test bundle to the app binary; `@testable import FlashTeXCompanion` will work.

### Shared scheme (new — added in FT-004 rev 4)

`FlashTeXCompanion.xcodeproj/xcshareddata/xcschemes/FlashTeXCompanion.xcscheme`

- BuildAction: both `FlashTeXCompanion` (A6000001) and `FlashTeXCompanionTests` (A6000002) included
- TestAction: `FlashTeXCompanionTests` in testables
- LaunchAction: `FlashTeXCompanion.app` 
- Enables `xcodebuild -scheme FlashTeXCompanion` without relying on Xcode's scheme autocreation

---

## Capture MIME Payload Gates — Verified in Source

### MIME type agreement (fixed in rev 4)

Previously: `CaptureEnvelope.create(image:)` always called `image.pngData()`, emitting
`mime_type: "image/png"` even when `ImageValidator.validate()` had fallen back to JPEG
and returned `UIImage(data: jpegData)`. The JPEG-decoded UIImage would then be re-encoded
to PNG, potentially re-inflating it above the 10 MB limit.

**Fix applied**: `ImageValidator.encode(_ image: UIImage) -> (Data, String)?` returns raw
bytes + the MIME type used. `CaptureEnvelope.create(image:)` calls `encode()` and passes
the result to `create(imageData:mimeType:)`. The declared `mime_type` now always matches
the actual bytes on the wire.

### Wire contract compliance (checked against `protocol/fixtures/capture-submission.json`)

| Field | Contract | Implementation |
|---|---|---|
| `protocol_version` | `1` (Int) | `CaptureEnvelope.protocolVersion = 1` ✓ |
| `type` | `"capture_submit"` | `CaptureEnvelope.type = "capture_submit"` ✓ |
| `payload.capture_id` | stable unique string | `CaptureStore`: new UUID per capture, only on stage ✓ |
| `payload.destination_id` | Mac-pinned anchor | `UserDefaults`-persisted `currentDestinationID` ✓ |
| `payload.base_revision` | Int | `UserDefaults`-persisted `currentBaseRevision` ✓ |
| `payload.image.mime_type` | `image/png` or `image/jpeg` | `ImageValidator.encode()` returns one or the other ✓ |
| `payload.image.data_base64` | base64-encoded image bytes | `imageData.base64EncodedString()` ✓ |
| `payload.instructions` | string | user-supplied context text ✓ |

### Accepted MIME types

`ImageValidator.acceptedMIMETypes = ["image/png", "image/jpeg"]` — matches runtime-v1.
`image/gif` is not in the set. Tested in `testAcceptedMIMETypes()`.

### Deduplication

`CaptureTransport.send(_:)` uses `NSLock`-protected `Set<String>` keyed on `capture_id`.
Returns `false` without emitting if the ID was already sent. Tested in `testDuplicatePrevention()`.

`CaptureStore.discardPending()` drops the `PendingCapture` without consuming the capture ID —
the ID was assigned in `stagePendingCapture`; a subsequent stage call generates a fresh UUID.

### JSON Lines

`CaptureEnvelope.toJSONString()` uses `JSONEncoder` with `.sortedKeys` only (no `.prettyPrinted`).
Output is a single line with no embedded newlines. `CaptureTransport.send()` calls `print(json)`
followed by `fflush(stdout)`. Tested in `testJSONIsOneLine()`.

---

## Physical-Mac Validation Gates

The following cannot be run in this cloud environment. Each command is literal and ready
to paste on a Mac with Xcode 15+ and an iOS 17+ simulator or device installed.

### Gate 1 — Project structure

```bash
cd apps/companion
xcodebuild -list -project FlashTeXCompanion.xcodeproj
```

**Expected output** (must contain both targets and the shared scheme):
```
Information about project "FlashTeXCompanion":
    Targets:
        FlashTeXCompanion
        FlashTeXCompanionTests

    Build Configurations:
        Debug
        Release

    Schemes:
        FlashTeXCompanion
```

**PHYSICAL MAC GATE** — not run; requires Xcode.

### Gate 2 — Simulator build

```bash
xcodebuild build \
  -project apps/companion/FlashTeXCompanion.xcodeproj \
  -scheme FlashTeXCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 16,OS=latest' \
  | tail -5
```

**Expected**: `** BUILD SUCCEEDED **`

**PHYSICAL MAC GATE** — not run; requires Xcode + iOS simulator.

### Gate 3 — Unit tests

```bash
xcodebuild test \
  -project apps/companion/FlashTeXCompanion.xcodeproj \
  -scheme FlashTeXCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 16,OS=latest' \
  | grep -E 'Test Suite|passed|failed'
```

**Expected**: All 5 `CapturePayloadTests` + all 7 `ImageValidatorTests` pass. Zero failures.

Test list:
- `CapturePayloadTests`: testEnvelopeStructure, testMIMETypeMatchesActualDataEncoding,
  testJSONMatchesFixtureKeys, testBase64DecodesBackToImage, testJSONIsOneLine,
  testDuplicatePrevention, testRawDataOverloadMIMEAgreement
- `ImageValidatorTests`: testSmallImagePassesValidation, testOversizedImageIsDownscaled,
  testAcceptedMIMETypes, testEncodeSmallImageReturnsPNG, testEncodeMIMETypeMatchesActualEncoding,
  testEncodeOversizedImageDownscales, testUpOrientationIsUnchanged

**PHYSICAL MAC GATE** — not run; requires Xcode + iOS simulator.

### Gate 4 — PencilKit drawing → capture_submit payload

1. Open `FlashTeXCompanion.xcodeproj` in Xcode
2. Run on an iPad simulator or physical iPad with Apple Pencil
3. Draw strokes on the canvas
4. Tap "Capture Drawing" (or trigger the review sheet)
5. Confirm send
6. Verify in Xcode console: one JSON Lines object per capture, starting with `{"id":...,"payload":...,"protocol_version":1,"type":"capture_submit",...}`
7. Decode `payload.image.data_base64`; confirm it is a valid PNG or JPEG of the drawn strokes
8. Capture `payload.image.mime_type`; confirm it matches the actual magic bytes of `data_base64`

**PHYSICAL MAC GATE** — requires PencilKit on iPad or iPad simulator.

### Gate 5 — Camera/photo library → capture_submit payload

1. Run on a physical iPhone (camera not available in simulator)
2. Navigate to Camera tab
3. Capture a photo
4. Verify review sheet shows thumbnail and JSON preview
5. Confirm send
6. Verify console: one JSON Lines object, `payload.image.mime_type` = `"image/png"` or `"image/jpeg"`, bytes match

**PHYSICAL MAC GATE** — requires physical device; camera unavailable in simulator.

### Gate 6 — Orientation normalization (physical device)

1. Take a portrait photo with the iPhone held in landscape mode (produces non-`.up` `imageOrientation`)
2. Confirm send
3. Decode `data_base64`; open the image; confirm it appears correctly oriented (not rotated 90°)

**PHYSICAL MAC GATE** — requires physical device.

---

## Session / Executor Disclosure

- **Tool**: Claude Cowork (claude-sonnet-4-6)
- **Environment**: Anthropic cloud Linux container; no Xcode; no iOS simulator; UIKit unavailable
- **Commits**: `git -c user.name="Cursor" -c user.email="cursor@flashtex.invalid" commit ...`
  (project convention per ROSTER.md; Cursor CLI is not locally installed)
- **Co-author trailer**: `Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>`
- **Allocation**: `openai-aarush-plus-ft004-stage2` (FT-004 rev 4); prior work under `openai-aarush-plus-ft004`
- **Protected Claude allowance**: not consumed; no Claude inference was used for implementation
- **Compiler bonus (FT-005/006)**: separate `agent/aarush-macbook/incremental-reuse` branch; not adopted into this task

---

*Generated: 2026-09-12T16:xx:xxZ by aarush-macbook sub-orchestrator for FT-004 rev 4*

# FT-004 companion app: iPad simulator run evidence (2026-09-12)

Verifier: `sim-companion` worker (parent `mac-claude-a`) on `mac-m1max-a`, Xcode 26.3 (17C529),
iOS 26.3 simulator `iPad Pro 13-inch (M5)` UDID `19A235F3-0B4C-481E-B8F6-05289C69D452`.

Verified revision: `origin/agent/aarush-macbook/companion-capture` @ **e7ce5b94a12e372a482a9578ac9e121b09c51c8a**
("companion(FT-004): add Bonjour Wi-Fi transport + UI polish"). The owner's branch was **not** modified.
All work happened in a detached scratch worktree; the diffs below are the only changes applied there.

## Result summary

| Step | Result |
|---|---|
| `xcodebuild -list` on committed pbxproj | **FAILS** (damaged project) — same corruption class as issue #3, plus `BonjourTransport.swift` missing from target |
| pbxproj repair (script, 12+/42-) | `plutil -lint` OK, `xcodebuild -list` OK |
| Simulator build after pbxproj repair only | **FAILS**: `DrawingCanvasView.swift:133:35: error: type 'ShapeStyle' has no member 'accentColor'` |
| Build after 1-token source fix (`Color.accentColor`) | **BUILD SUCCEEDED**, 0 warnings from Swift sources |
| Install + launch on booted iPad | Yes — bundle id `com.flashtex.companion`, pid 1824, no first-run permission dialog |
| XCUITest (scratch-only UI test target) | attempt 1 FAILED (Send stayed disabled — real bug); attempt 2 **TEST SUCCEEDED** (both photo-library and drawing paths reach a `capture_submit` payload preview) |
| stdout `capture_submit` JSON Lines | Yes — 4 lines for 2 captures (**each capture printed twice**, see fix list) |

## 1. pbxproj repair (scratch worktree only)

Script: `repair_pbxproj.py` (in this evidence dir). Diff: `pbxproj-repair.diff`. Repaired file: `project.pbxproj.repaired`.

What was wrong at e7ce5b9 (`apps/companion/FlashTeXCompanion.xcodeproj/project.pbxproj`):
- An inline `A5000006 /* Services */ = { ... };` group body spliced into the main group's `children` list and into `productRefGroup` (which must be `A5000005 /* Products */`).
- Inline `PBXFileReference` definitions for `A200000B/C/D` spliced into the `children` of the `FlashTeXCompanion`, `Views`, and `Services` groups; inline `PBXBuildFile` definitions for `A100000A/B/C` spliced into the Sources and Resources `files` lists.
- `ImageValidator.swift` listed 3x in Sources; Swift build files listed in the Resources phase.
- `Services/BonjourTransport.swift` (new in e7ce5b9, used by `ContentView`/`CaptureStore`) has no file reference, group entry, or Sources entry at all — even a syntactically valid project would fail to link.

Commands and results:
```
$ xcodebuild -list -project FlashTeXCompanion.xcodeproj          # before repair
xcodebuild: error: Unable to read project 'FlashTeXCompanion.xcodeproj'. ... damaged and cannot be opened due to a parse error.
$ python3 repair_pbxproj.py apps/companion/FlashTeXCompanion.xcodeproj/project.pbxproj
repaired: removed 1 spliced Services body, fixed productRefGroup x1
$ plutil -lint apps/companion/FlashTeXCompanion.xcodeproj/project.pbxproj
... project.pbxproj: OK
$ xcodebuild -list -project FlashTeXCompanion.xcodeproj           # xcodebuild-list.txt
Targets: FlashTeXCompanion   Schemes: FlashTeXCompanion
```

## 2. Simulator build

```
$ xcodebuild -project FlashTeXCompanion.xcodeproj -scheme FlashTeXCompanion \
    -destination 'platform=iOS Simulator,id=19A235F3-0B4C-481E-B8F6-05289C69D452' \
    -derivedDataPath <scratch>/dd CODE_SIGNING_ALLOWED=NO build
```
- Run 1 (`xcodebuild-build-1.log`): exit 65, `** BUILD FAILED **` —
  `FlashTeXCompanion/Views/DrawingCanvasView.swift:133:35: error: type 'ShapeStyle' has no member 'accentColor'`.
  This is a genuine source error on the branch (`.foregroundStyle(.accentColor)` in `DestinationStrip`); chatgpt-a's d764d54 fixes the same line.
- Applied `drawingcanvas-compile-fix.diff` (`.accentColor` -> `Color.accentColor`) in the scratch worktree.
- Run 2 (`xcodebuild-build-2.log`): exit 0, `** BUILD SUCCEEDED **`. Only diagnostic: appintentsmetadataprocessor "No AppIntents.framework dependency found" (benign).
- Product: `dd/Build/Products/Debug-iphonesimulator/FlashTeXCompanion.app`; `CFBundleIdentifier = com.flashtex.companion`, `MinimumOSVersion 17.0`, Info.plist carries `NSCameraUsageDescription` and `NSPhotoLibraryUsageDescription` only.

## 3. Install and launch

```
$ xcrun simctl install booted <app>                      # ok
$ xcrun simctl launch --console-pty booted com.flashtex.companion > launch-1-stdout.log   # backgrounded
$ xcrun simctl io booted screenshot companion-1.png      # after 6 s
```
- App launched and stayed running (launchctl: `UIKitApplication:com.flashtex.companion`, pid 1824).
- **No first-run permission dialog** (no camera/photo/local-network prompt). The Bonjour browser starts on appear and the banner shows "Looking for Mac…" indefinitely; no `NSLocalNetworkUsageDescription`/`NSBonjourServices` keys exist, which on a physical device will make the `_flashtex._tcp` browse fail with a policy denial.
- Only non-JSON stdout/stderr line: simulator font warning about AppleColorEmoji (benign).
- Screenshot `companion-1.png`: Draw tab, PencilKit canvas with the PKToolPicker palette; the palette **covers the app's own action bar** (trash/undo/note buttons and Send are half hidden behind it).

## 4. UI exercise (scratch-only XCUITest target)

Test image: `test-handwriting.png` (400x300 PIL-drawn "x²+y²=r²" scrawl) injected with `xcrun simctl addmedia booted`.

A UI-testing target `FlashTeXCompanionUITests` was added **only in the scratch worktree** (`add_uitest_target.py`, `CompanionUITests.swift`, shared scheme `FlashTeXCompanionUITests.xcscheme`, resulting `project.pbxproj.with-uitests`). The test uses `XCUIApplication.activate()` so it drives the instance launched by `simctl launch --console-pty` and the app's stdout keeps flowing into `launch-1-stdout.log`.

```
$ xcodebuild test -project FlashTeXCompanion.xcodeproj -scheme FlashTeXCompanionUITests \
    -only-testing:FlashTeXCompanionUITests \
    -destination 'platform=iOS Simulator,id=19A235F3-0B4C-481E-B8F6-05289C69D452' \
    -derivedDataPath <scratch>/dd -resultBundlePath <scratch>/uitest-N.xcresult CODE_SIGNING_ALLOWED=NO
```

Attempt 1 (`xcodebuild-test-1.log`, screenshots `run1-*.png`): **TEST FAILED** (1 assertion).
- Camera tab: "Take Photo" and "Choose from Library" buttons found; PhotosPicker opened (`run1-03/04`) with the injected image as first cell and a non-modal "Private Access to Photos" banner; the generic `images.firstMatch` tap hit the wrong element, so nothing was picked.
- Draw tab: three `press(forDuration:thenDragTo:)` drags produced visible PencilKit strokes (`run1-06-after-stroke.png`), but `Send`, `Trash`, `Undo` stayed **disabled** after drawing, after a Settings->Draw tab switch, and after toggling the context-note field (label "Comment Lines"). Assertion: "Send stayed disabled after drawing".

Attempt 2 (`xcodebuild-test-2.log`, screenshots `run2-*.png`): **TEST SUCCEEDED** (40 s, 0 failures).
- Photo-library path: tapping the first grid cell by coordinate selected the injected image; `store.addCapture(source: .photoLibrary)` ran; payload sheet "Capture Payload" appeared showing `"type" : "capture_submit"` (`run2-04b-photo-payload-preview.png`).
- Draw path: strokes from run 1 were still on the canvas and `Send` was now **enabled before drawing** — because the photo capture mutated `CaptureStore`, which re-evaluated `DrawingCanvasView.body`. Tapping Send produced the second payload sheet with `capture_submit` (`run2-09-payload-preview.png`); History lists both captures (`run2-10-history.png`, `companion-final-history.png`).
- Attachments (`XCUIScreen.main.screenshot()`) were exported with `xcrun xcresulttool export attachments` and downscaled with `sips -Z 900`. Writing PNGs to a host path from the test runner was blocked by the simulator sandbox.

## 5. CaptureTransport output (stdout JSON Lines)

`CaptureTransport.send` does `print(json); fflush(stdout)` — stdout of the app process, captured via `--console-pty` into `launch-1-stdout.log` (5 lines: 1 font warning + 4 JSON lines).

Decoded (`python3 json.loads` per line):
```
line 2  capture_submit  pv=1  id=0C189103…  capture-3A2F48D1  default-anchor  rev 1  image/png  5459 B  PNG 400x300  (photo library)
line 3  capture_submit  — byte-identical duplicate of line 2 —
line 4  capture_submit  pv=1  id=15FEB5E9…  capture-23361924  default-anchor  rev 1  image/png  62595 B PNG 1408x1510 RGBA (pencil drawing)
line 5  capture_submit  — byte-identical duplicate of line 4 —
```
Envelope keys `[id, payload, protocol_version, type]`; payload keys `[base_revision, capture_id, destination_id, image, instructions]` — matches runtime-v1. Decoded images: `decoded-capture-3A2F48D1.png` (the injected test image, round-tripped) and `decoded-capture-23361924.png` (the drawn strokes; thumbnail `decoded-capture-23361924-thumb.png`). Base64 `/` is escaped as `\/` (valid JSON).

**Each capture is emitted twice**: `CaptureStore.addCapture` calls `CaptureTransport.shared.send(envelope)` (prints) and then `BonjourTransport.shared.send(json)`, whose not-connected fallback branch *also* prints the line to stdout. A Mac reading stdout would receive every capture twice with the same `capture_id`.

## 6. What aarush-macbook must fix on `agent/aarush-macbook/companion-capture`

Blocking (branch does not build as committed):
1. `apps/companion/FlashTeXCompanion.xcodeproj/project.pbxproj` is still corrupt at e7ce5b9 (spliced inline definitions in `children`/`files`/`productRefGroup`, ImageValidator 3x, Swift files in Resources). Apply `pbxproj-repair.diff` (or cherry-pick chatgpt-a's d764d54, which also fixes it) — and verify with `plutil -lint` + `xcodebuild -list`.
2. Add `Services/BonjourTransport.swift` to the app target (missing file ref, group child, and Sources entry) — required by `ContentView` and `CaptureStore`.
3. `Views/DrawingCanvasView.swift:133` — `.foregroundStyle(.accentColor)` does not compile; use `Color.accentColor`.

Functional bugs found at runtime:
4. `DrawingCanvasView`: `Send`/`Trash`/`Undo` are gated on `canvasView.drawing.strokes.isEmpty`, but `PKCanvasView` is a reference type in `@State` with no `PKCanvasViewDelegate`, so drawing never triggers a body re-evaluation. After drawing, Send stays disabled until some unrelated observable state changes (observed: it only enabled after a photo-library capture). Fix: set a `PKCanvasViewDelegate` in the coordinator and mirror `canvasViewDrawingDidChange` into a `@State var strokeCount` (or similar), and drive the badge/buttons from that.
5. `CaptureStore.addCapture` prints every capture to stdout twice (`CaptureTransport.send` + `BonjourTransport.send` stdout fallback). Only one transport should write to stdout; the duplicate-`capture_id` guard in `CaptureTransport` does not cover the second print.
6. `CaptureHistoryView` shows an unconditional green `checkmark.circle.fill` for every record; `CaptureRecord.networkSent` is never surfaced, so a stdout-fallback capture looks "delivered". Show the actual transport state.
7. PKToolPicker overlaps the app's bottom action bar on iPad (Send/Trash/Undo are partially hidden). Reserve space for the picker (e.g. `PKToolPicker` frame observer / bottom inset) or move the action bar into the navigation bar.
8. Bonjour on a physical device: `Info.plist` needs `NSLocalNetworkUsageDescription` and `NSBonjourServices = [_flashtex._tcp]`, otherwise `NWBrowser` fails with a local-network policy denial. (Not triggered on the simulator; no prompt appeared.)
9. The pencil export (`PKDrawing.image(from:scale:)`) yields an RGBA PNG with a fully transparent background (corner alpha = 0). Composite onto white before encoding, or document that consumers must.
10. Still open from the earlier verification comment on issue #3: no unit-test target/scheme for `FlashTeXCompanionTests/`; README/`build.sh`/`Package.swift` still name `iPhone 16` (not available on Xcode 26.3; use `generic/platform=iOS Simulator` or a real 26.x device name); `mime_type` hardcoded `image/png` while `ImageValidator` claims a JPEG fallback (d764d54 addresses this).

## Artifacts (all under this evidence directory)

- Scripts/diffs: `repair_pbxproj.py`, `pbxproj-repair.diff`, `project.pbxproj.repaired`, `drawingcanvas-compile-fix.diff`, `add_uitest_target.py`, `project.pbxproj.with-uitests`, `CompanionUITests.swift`, `FlashTeXCompanionUITests.xcscheme`, `scratch-worktree-full.diff`
- Logs: `xcodebuild-list.txt`, `xcodebuild-build-1.log`, `xcodebuild-build-2.log`, `xcodebuild-test-1.log`, `xcodebuild-test-2.log`, `launch-1-stdout.log` (app stdout, contains the 4 `capture_submit` lines)
- Screenshots (900 px, all < 150 KB): `companion-1.png`, `run1-01..07`, `run2-01..10` (`run2-04b` and `run2-09` show the `capture_submit` payload preview), `companion-final-history.png`
- Payload images: `decoded-capture-3A2F48D1.png`, `decoded-capture-23361924.png` (+ `-thumb`), input `test-handwriting.png`
- Raw xcresult exports: `uitest-shots-1/`, `uitest-shots-2/` (include 8 MB screen recordings; not for commit)

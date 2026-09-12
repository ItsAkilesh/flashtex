# FlashTeX Companion

iPad/iPhone companion app for capturing handwritten equations and drawings.

## Features

- **Apple Pencil Drawing**: PencilKit canvas for direct handwriting input
- **Camera Capture**: Take photos of handwritten content on paper
- **Photo Library**: Import existing images of equations/notes
- **Capture History**: Review previous captures with thumbnails
- **Payload Preview**: Inspect and share generated JSON payloads

## Protocol

All capture paths produce the same `capture_submit` JSON payload per
`docs/contracts/runtime-v1.md`. Payloads are output as JSON Lines to stdout
for transport to the Rust worker.

## Build

```bash
cd apps/companion
xcodebuild -project FlashTeXCompanion.xcodeproj \
  -scheme FlashTeXCompanion \
  -destination 'platform=iOS Simulator,name=iPhone 16' \
  build
```

Requires Xcode with iOS SDK. Camera capture requires a physical device.

## Architecture

```
FlashTeXCompanion/
├── FlashTeXCompanionApp.swift    # App entry point
├── Models/
│   ├── CapturePayload.swift      # runtime-v1 capture_submit model
│   └── CaptureStore.swift        # Observable capture state
├── Views/
│   ├── ContentView.swift         # Tab-based root view
│   ├── DrawingCanvasView.swift   # PencilKit canvas + export
│   ├── CameraCaptureView.swift   # Camera + photo picker
│   ├── CaptureHistoryView.swift  # Capture list with thumbnails
│   └── PayloadPreviewView.swift  # JSON payload viewer
├── Services/
│   └── CaptureTransport.swift    # JSON Lines stdout transport
└── Info.plist                    # Camera/photo permissions
```

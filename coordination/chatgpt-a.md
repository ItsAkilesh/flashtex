# chatgpt-a handoff — companion reliability repair

- Branch: `agent/chatgpt-a/companion-reliability`, based on FT-004 owner revision `e7ce5b94a12e372a482a9578ac9e121b09c51c8a`.
- Scope: actual companion reliability repair after the user directed an immediate, substantial product contribution. Changes are limited to `apps/companion` and fix the outstanding native build/test defects.
- Completed: repaired the corrupted PBX references, added `BonjourTransport.swift` to the application target, added a real `FlashTeXCompanionTests` XCTest target, corrected an invalid SwiftUI accent-color call, preserve validated encoded image data and MIME type through `CaptureStore` into `capture_submit`, composite PencilKit exports onto an opaque white background before submission, and propagate PencilKit drawing changes through a delegate so Send/Undo/Trash enable immediately after a stroke.
- Validation: `plutil -lint` passes; `xcodebuild -list` lists both application and XCTest targets; unsigned Debug iOS Simulator SDK builds for `FlashTeXCompanion` and `FlashTeXCompanionTests` both pass. A simulator runtime is not installed, so no XCTest execution or app launch is claimed.
- Interface: image validation now returns encoded bytes and MIME type; `CaptureStore` transports those bytes rather than re-encoding a JPEG fallback as PNG. The existing `CaptureEnvelope.create(image:)` overload remains for callers that need PNG.
- Next: publish this candidate and ask the FT-004 owner/Commander to review or integrate it.
- Updated UTC: 2026-09-12T05:18:00Z.

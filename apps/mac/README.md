# FlashTeX Mac shell (FT-003)

Native macOS source editor + preview shell. Swift Package, macOS 14+, SwiftUI/AppKit.
Owner: mac-claude-a. Contract: `docs/contracts/runtime-v1.md`.

The preview is **fixture-backed**: it renders `protocol/fixtures/compile-result.json`
and shows a `FIXTURE` badge, result id/revision/status, and `pdf: none`. No LaTeX is
compiled; when the buffer is edited the banner says the preview was not recompiled.

## Build and test

```sh
cd apps/mac
swift build
swift test
xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build
FLASHTEX_REPO=$(git rev-parse --show-toplevel) .build/debug/FlashTeXMac
```

The app locates `protocol/fixtures/` via `FLASHTEX_REPO`, the working directory,
the bundle path, or the source path; `File > Open Compile Result Fixture…` (⌘O)
loads another `compile_result` JSON, with a sibling `compile-request.json` used to
seed the editor when present. `⌘R` reloads.

## Behavior

- Editor: `NSTextView` (monospaced, undo, no smart substitutions). Footer shows
  UTF-8 byte and UTF-16 unit counts of the active document.
- Preview: pages drawn at 1pt = 1 screen point, origin top-left; text items are
  placed by `x_pt` / `baseline_y_pt` / `font_size_pt`. Hover highlights an item;
  clicking it navigates to its `source` range. Diagnostics list with "Go to source".
- Navigation converts the contract's zero-based end-exclusive UTF-8 byte offsets to
  a UTF-16 `NSRange` (`String.nsRange(utf8Bytes:)`). Offsets that are out of range,
  reversed, or inside a multi-byte scalar are rejected with a footer message rather
  than applied. Unknown item `kind`s decode as `.unknown` and are skipped.
- Dark preview toggle in the toolbar (page and text colors only).

## Targets

- `FlashTeXProtocol` — Codable models for runtime v1 and byte-offset conversion.
- `FlashTeXMac` — the app.
- Tests: fixture decoding, version/type rejection, unknown kinds, UTF-8→UTF-16
  conversion with multi-byte scalars, and `ShellModel` load/navigate/stale behavior.

## Not done

- No Rust worker process is spawned; no JSON Lines transport yet (fixture only).
- No PDF export, no image/line items (not in v1), no reverse (source→preview) sync.
- Screen capture of the running app was not possible from the agent's terminal
  (no Screen Recording permission); visual click behavior needs a human check.

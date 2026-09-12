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
loads another `compile_result` JSON, with a sibling `compile-request.json` (or
`<name>-request.json` for a `<name>-result.json`) used to seed the editor when
present. `⌘R` reloads.

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
- Caret sync (source→preview): text items whose `source` range contains the
  editor caret (UTF-16 caret → UTF-8 byte via `String.utf8ByteRange(of:)`,
  `CaretSync.itemsContaining`) get a secondary highlight (15% accent fill +
  underline) on every page. Empty ranges match only an exactly equal byte. The
  preview does not auto-scroll to the highlighted item.
- Dark preview toggle in the toolbar (page and text colors only).
- Stale offsets are never applied. Each `compile_result` remembers the exact
  document text it was produced for; after edits, a span is rebased through the
  common prefix/suffix of old vs new text (`SourceMapping`), verified against the
  item's text when known, and refused ("recompile to navigate") if it overlaps the
  edited region. Multiple edits collapse conservatively.
- Auto-compile (toolbar toggle, default on when a worker is attached): edits are
  debounced 250 ms and coalesced — one request in flight, the newest buffer goes
  out when it returns. Latency (send→result) is shown in the banner with a median.
  Measured with the FT-002 compiler on an M1 Max: 0.5–9 ms per request; a 10-edit
  burst coalesced into 2 requests (`RealCompilerTests`).
- Worker transport: `File > Attach Built Compiler` (⌘⇧K) finds `$FLASHTEX_COMPILER`
  or `crates/compiler/target/{release,debug}/flashtex-compiler` under the repo;
  `File > Attach Worker Executable…` (⌘K) launches a process
  speaking runtime v1 JSON Lines on stdin/stdout; `Compile` (⌘B) sends the current
  buffers as a `compile` envelope with the editor revision. The banner badge
  switches from `FIXTURE` to `WORKER`; an older `compile_result` never replaces a
  newer one. `error` envelopes, undecodable lines, unsupported versions, and worker
  exit are reported in the banner. Oversized lines (>16 MiB) terminate the worker.

- Capture review and insertion (contract "Capture and insertion", Mac side):
  `Edit > Pin Insertion Point` (⌘⇧P) records the caret as a `destination_id`
  anchor (UTF-8 byte offset + revision + following context). `Edit > Open Capture
  Proposal…` (⌘⇧I) queues a `capture_proposal`; a review sheet shows editable
  LaTeX, ambiguities, and required packages. Approve applies exactly one edit
  through the text view's undo manager (⌘Z reverts). Repeated `capture_id`s never
  insert twice. If the buffer changed since pinning, the anchor is rebased by its
  context or, when the destination was deleted/ambiguous, reselection is required.
  See `Samples/capture-proposal.json`. No network or Grok call is involved here.
- PDF export: `File > Export PDF…` (⌘⇧E) writes the current preview with
  CoreGraphics/CoreText (`PDFExport.swift`): one PDF page per `pages` entry at
  `width_pt` × `height_pt`, each text item in Times at `font_size_pt` with its
  baseline exactly `baseline_y_pt` from the top (flipped to PDF's bottom-left
  origin). It exports the layout the Rust compiler reported, not a TeX-engine
  PDF: no fonts beyond Times, no images/lines, no links or metadata. The dark
  toggle only changes page/text colors. Disabled when no result is loaded.

## Samples

`Samples/multipage-result.json` + `multipage-request.json` (⌘O on the result):
two pages, nine text items with byte-exact source ranges into a `main.tex` that
contains non-ASCII words ("naïve", "Résumé") so UTF-8 and UTF-16 offsets differ,
plus one error diagnostic with a source range and recovery text and one warning
with null source/recovery. Use it for manual click-to-source, caret-sync, and
diagnostics checks beyond the one-line contract fixture.

- Export is always white: dark preview is a viewing mode only. `File > Export
  PDF…` (⌘⇧E) uses CoreGraphics; `File > Export PDF via Rust Writer…` (⌘⌥E) pipes
  the current `compile_result` envelope to the FT-009 `flashtex-pdf --verify`
  binary (`$FLASHTEX_PDF` or `crates/pdf/target/{release,debug}/flashtex-pdf`).
  `RustPDFExportTests` runs only when `FLASHTEX_PDF` is set.
- Diagnostics panel under the preview is never hidden when diagnostics exist; each
  entry shows severity, message, recovery note (or "no provisional rendering"),
  and source bytes; the banner shows error/warning counts and a `recovered` note.

## Targets

- `FlashTeXProtocol` — Codable models for runtime v1 and byte-offset conversion.
- `FlashTeXMac` — the app.
- Tests (36): Rust-writer export (gated on `FLASHTEX_PDF`), missing-binary error; source mapping (shift/refuse/multi-byte/expected-text), stale
  navigation refusal and rebase, auto-compile debounce/coalescing, latency; PDF export (fixture → 612×792 page containing the item text,
  two-page synthetic sizes, unknown-kind skipping, page-less result); anchor/rebase/reselection logic, review flow with duplicate
  suppression, capture fixture decoding; caret sync (multi-page sample slices
  byte-exactly, `itemsContaining` boundaries incl. inside a multi-byte scalar,
  `ShellModel.caretByte`/`caretItems`, sibling request discovery); end-to-end
  against the real compiler (gated on `FLASHTEX_COMPILER`); plus fixture
  decoding, version/type rejection, unknown kinds, UTF-8→UTF-16
  conversion with multi-byte scalars, `ShellModel` load/navigate/stale behavior,
  line splitting/encoding, and a round trip through `Tests/.../fake_worker.py`
  (a Python test double, not a compiler) including error/garbage/exit paths and
  the stale-revision guard.

## Known upstream issue

`protocol/fixtures/compile-result.json` item text is `"Hello FlashTeX."` (15
bytes) but its source range is `0..<14` (`"Hello FlashTeX"`). The real compiler's
spans are exact. Reported to the fixture owner (Commander, FT-001).

## Not done

- The FT-002 compiler (`crates/compiler`, branch `agent/claude/compiler-foundation`
  at 29221d8 when checked) round-trips through this shell: `RealCompilerTests`
  runs only when `FLASHTEX_COMPILER` points at a built binary and verifies every
  emitted span slices back to its text after UTF-8→UTF-16 conversion. Without
  the binary that test is skipped and only the Python double is exercised.
- PDF export draws only what the contract's text items describe; it is not a
  TeX-engine PDF and has no compiler-produced `pdf_path` behind it. A result with
  zero pages exports one blank page (a PDF must have at least one).
- No image/line items (not in v1). Caret sync highlights only; it does not
  scroll the preview to an off-screen item.
- Screen capture of the running app was not possible from the agent's terminal
  (no Screen Recording permission); visual click behavior needs a human check.

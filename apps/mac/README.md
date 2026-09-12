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

## Packaging

A bare `swift build` product has no Finder/Dock identity: `open -a` fails on it
and macOS cannot grant it per-app permissions (e.g. local network, later).
`scripts/make-app.sh` wraps the built executable in a minimal `FlashTeX.app`:

```sh
apps/mac/scripts/make-app.sh [--debug] [--compiler <path>] [--pdf <path>] [--open]
```

It builds `FlashTeXMac` (release by default), assembles
`apps/mac/build/FlashTeX.app` (`CFBundleIdentifier tech.jay3332.flashtex.mac`),
copies `protocol/fixtures/compile-{request,result}.json` and `Samples/*` into
`Contents/Resources/Samples` (so a bundled app finds fixtures via
`Bundle.main.resourceURL`), bundles `flashtex-compiler`/`flashtex-pdf` into
`Contents/MacOS` when built at `crates/{compiler,pdf}/target/release/…` under
the repo root (or passed via `--compiler`/`--pdf`), and ad-hoc codesigns the
result. Launch with `open apps/mac/build/FlashTeX.app` or pass `--open`.
`apps/mac/build/` is gitignored.

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
- Inline diagnostics: each diagnostic with a `source` in the active document is
  underlined in the editor (red dotted for errors, orange for warnings); hovering
  shows the message and, when present, the `recovery` text (`EditorDiagnostics`).
  Marks are layout-manager temporary attributes, so undo and the text binding are
  unaffected. After edits a mark is rebased through `SourceMapping` or dropped when
  it overlaps the edited region — never drawn under the wrong text. Diagnostics
  with null `source` appear only in the preview's diagnostics list.
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
  exit are reported in the banner. Any line over 16 MiB — complete or still
  unterminated — is a protocol violation that terminates the worker, and a worker
  that exits leaving unterminated trailing bytes is reported as a violation too.
  A `compile_result` is applied only if its `id` matches an in-flight request and
  its `project_id` and `revision` match that request; unsolicited or mismatched
  results are logged and never shown.

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

## Capture bridge (transfer-v1)

Built against the FT-007 bridge at commit **b5ca96b** on
`agent/commander/capture-bridge` (`crates/bridge`, `docs/contracts/transfer-v1.md`),
an unmerged dependency at the time of writing; no contract changes were made.
The Mac owns UI review, the document transaction and the bridge lifecycle.

`Edit > Attach Capture Bridge` launches `flashtex-bridge --store <dir>`
(`$FLASHTEX_BRIDGE`, a bridge bundled next to the executable, or
`crates/bridge/target/{release,debug}/flashtex-bridge`; the store is
`$FLASHTEX_BRIDGE_STORE` or `~/Library/Application Support/FlashTeX/captures`).
`FLASHTEX_AUTOATTACH=1` or a bundled bridge attaches at launch. The bridge line
under the editor shows attached/error status (error codes as plain text), the
pinned bridge destination, and the latest capture's state.

What works offline (no key, no network — verified with `RealBridgeTests`):

- On attach the shell first reconciles its edit ledger with `capture_status`
  (below), then sends `document_open` for the active document at the current
  editor revision. Every later edit is sent as one `document_edit` (byte range
  + replacement, derived from the common prefix/suffix of old and new text and
  widened to UTF-8 scalar boundaries); a refused edit triggers a `document_open`
  resynchronization. `File > Open` re-opens the new document.
- `Pin Insertion Point` (⌘⇧P) also sends `destination_pin` for the caret (or
  selection) byte range and stores the returned anchor with its immutable
  binding. The local context anchor remains for offline review.
- `Edit > Submit Sample Capture…` (⌘⇧U) sends `capture_submit` for a chosen
  PNG/JPEG (base64) with the pinned `destination_id` and `base_revision` =
  the anchor's pinned revision. `capture_received {durable:true}` is shown;
  identical retries return the same record, a different payload under the same
  ID is `capture_id_conflict`, a non-decodable image is `invalid_image`.
- `Edit > Convert Capture` (⌘⇧G) sends `capture_convert {capture_id,
  supported_features: []}`. Without `--enable-grok` the bridge answers
  `provider_disabled` (with it but no key, `provider_auth_missing`); both are
  shown as text and never prompt for a key. A `capture_proposal` (with
  `context_revision`) is queued in the existing review sheet.
- Approving a bridge proposal sends `capture_prepare_insert {capture_id,
  expected_revision: editorRevision, approved: true}` and verifies the returned
  `capture_edit`: project/path, `expected_revision == editorRevision`, SHA-256 of
  the current UTF-8 buffer == `document_before_sha256` (CryptoKit), scalar-aligned
  `start_byte..end_byte`, and `removed_text` equal to those bytes. Only then is
  the edit applied as one undoable editor edit (`pendingEdit`), recorded in the
  ledger (`<store>/mac/edit-ledger.json`, prepared → applied → confirmed), and
  confirmed with `capture_applied {capture_id, edit_id, new_revision}` — no
  `document_edit` is sent for that change. The same `edit_id` is never applied
  twice. Reviewer-edited LaTeX is refused (the contract has no field for it).
  `Reject` sends `capture_reject`.
- Restart reconciliation: for each ledger entry not yet confirmed, the shell
  queries `capture_status` before opening the current document. A receipt the
  bridge already holds is marked confirmed; an applied-but-unconfirmed edit is
  replayed by reopening the pre-edit snapshot and resending `capture_applied`,
  then the live source is resynchronized; a prepared-but-unapplied edit is
  re-offered only if the buffer still hashes to `document_before_sha256`,
  otherwise it is abandoned and reselection (new destination + capture ID) is
  required. Nothing is ever reapplied to a changed document.

Not implemented here: the Mac credential adapter that would run the bridge with
`--enable-grok` and supply the authorized `XAI_API_KEY` (so no real conversion
happens from this shell), the companion network transport (captures come from
a file picker), and compiler validation of proposals before review. The
document store is the in-memory buffer, so the ledger is transactional with
respect to the shell's own edit application, not with respect to `File > Save`.

`Tests/FlashTeXMacTests/Fixtures/fake_bridge.py` is a stdlib-Python test double
of the bridge (in-memory journal, deterministic `\fakecapture{<id>}` proposal);
`RealBridgeTests` runs only when `FLASHTEX_BRIDGE` points at a built binary and
uses a temporary `--store`. Note: the 1×1 PNG in main's
`protocol/fixtures/capture-submission.json` is rejected by the bridge's decoder
(`invalid_image`); the bridge branch (ba89c9a) replaced it, and the real-bridge
test carries that decodable image inline.

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

## Launch hooks and evidence

`FLASHTEX_AUTOATTACH=1` attaches the discovered compiler at launch and compiles
(a compiler bundled inside `FlashTeX.app` attaches by default; `=0` disables);
`FLASHTEX_SEED_FILE=<path.tex>` seeds the editor. Example (from `apps/mac`):

```sh
FLASHTEX_REPO=$(git rev-parse --show-toplevel) FLASHTEX_AUTOATTACH=1 \
  FLASHTEX_SEED_FILE=Samples/recovery-demo.tex .build/debug/FlashTeXMac
```

Screenshot of that run against crates/compiler 29221d8:
`docs/evidence/mac-shell-real-compiler-2026-09-12.png` — WORKER badge, status
`recovered`, three diagnostics with recovery notes, inline underlines. Words
crowded in that capture for two reasons, both since fixed: the compiler used
placeholder glyph widths (real Core-14 metrics arrived in de1020c) and the
preview drew with the system serif (New York) instead of Times-Roman. With
compiler de1020c the preview now matches: `docs/evidence/mac-shell-math-times-2026-09-12.png`
(inline fractions with rules, Greek, radicals, sum limits).

Rules: text items consisting only of U+2500 are the compiler's fraction bars and
are drawn as filled rectangles (`RuleConvention`: 0.5 em per character, 0.0857 em
thick, hugging the baseline) in the preview and both PDF paths, so bars never
depend on font glyph coverage. Heading weight (Times-Bold in the compiler) is not
reproducible until runtime-v1 carries a font field; the preview draws Times-Roman.

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| ⌘O | Open LaTeX file… (becomes the `main.tex` entry document; compiles if a worker is attached) |
| ⌘S / ⌘⇧S | Save / Save As… (UTF-8; header shows "— edited" when dirty) |
| ⌘⇧O | Open compile result fixture… (sibling `-request.json` seeds the editor) |
| ⌘R | Reload fixture |
| ⌘⇧K | Attach built compiler (`$FLASHTEX_COMPILER` or `crates/compiler/target/…`) |
| ⌘K | Attach worker executable… |
| ⌘B | Compile now (auto-compile also runs 250 ms after edits) |
| ⌘⇧E | Export PDF… (CoreGraphics, always white) |
| ⌘⌥E | Export PDF via Rust writer… (`flashtex-pdf --verify`, always white) |
| ⌘⇧P | Pin insertion point at caret (capture destination anchor) |
| ⌘⇧I | Open capture proposal… (review sheet; ⏎ approves, inserts one undoable edit) |
| ⌘⇧U | Submit sample capture… (PNG/JPEG → `capture_submit` through the attached bridge) |
| ⌘⇧G | Convert capture (`capture_convert` for the latest received capture) |
| ⌘Z | Undo (including an approved capture insertion) |
| Click preview text | Select its source (UTF-8 span → UTF-16; refused if edited since compile) |

The compiler rejects request lines over 8 MiB with an `error` envelope, which the
banner shows; the shell rejects response lines over 16 MiB.

## Targets

- `FlashTeXProtocol` — Codable models for runtime v1 (`RuntimeV1`) and the
  capture bridge (`TransferV1`), byte-offset conversion, changed-region diffing.
- `FlashTeXMac` — the app. `BridgeClient` (JSON Lines transport, id-correlated
  replies, 12 MiB line limit), `BridgeSession` (bridge-side document shadow,
  destination, captures, edit ledger, reconciliation), `ShellModel+Bridge`.
- Tests (59, of which `RealCompilerTests`, `RustPDFExportTests` and
  `RealBridgeTests` are gated on `FLASHTEX_COMPILER`, `FLASHTEX_PDF` and
  `FLASHTEX_BRIDGE`): bridge transport round trip of every transfer-v1 request
  type incl. error envelopes, garbage/oversized/trailing lines and oversized
  requests; ledger persistence; shell ↔ fake bridge flow (open → edit → pin →
  submit → received → convert → review → prepare → verify → apply once →
  `capture_applied`, duplicate approval no-op, edited-LaTeX refusal, buffer
  changed between prepare and apply → reselection, provider error as text,
  reject forwarded, restart reconciliation replaying a missing receipt and
  abandoning prepared edits on a changed buffer); real bridge (durable receipt,
  duplicate/conflict, `invalid_image`, `provider_disabled`, `proposal_missing`,
  reject, `capture_missing`); plus the earlier: oversized complete line, trailing bytes at EOF, unsolicited/mismatched result correlation; inline diagnostic marks (byte→UTF-16, rebase/drop, path filter,
  sample slice, temporary-attribute-only); Rust-writer export (gated on
  `FLASHTEX_PDF`), missing-binary error; source mapping (shift/refuse/multi-byte/expected-text), stale
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
- No image/line items (not in v1). Caret sync highlights items and scrolls to
  the page under the caret, not to the item within the page.
- Screen capture of the running app was not possible from the agent's terminal
  (no Screen Recording permission); visual click behavior needs a human check.

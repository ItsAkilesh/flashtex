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
apps/mac/scripts/make-app.sh [--debug] [--compiler <path>] [--pdf <path>] [--open] [--install] [--dmg]
```

It builds `FlashTeXMac` (release by default), assembles
`apps/mac/build/FlashTeX.app` (`CFBundleIdentifier tech.jay3332.flashtex.mac`),
copies `protocol/fixtures/compile-{request,result}.json` and `Samples/*` into
`Contents/Resources/Samples` (so a bundled app finds fixtures via
`Bundle.main.resourceURL`), bundles `flashtex-compiler`/`flashtex-pdf` into
`Contents/MacOS` when built at `crates/{compiler,pdf}/target/release/…` under
the repo root (or passed via `--compiler`/`--pdf`), and ad-hoc codesigns the
result. Launch with `open apps/mac/build/FlashTeX.app` or pass `--open`.
`apps/mac/build/` is gitignored. `--install` atomically replaces
`~/Applications/FlashTeX.app` (verified to launch before the previous bundle
is discarded) and `--dmg` produces a compressed disk image; see
`apps/mac/docs/packaging.md` for signing/notarization status and the
update-path and launch-recovery evidence (`scripts/launch-check.sh`).

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
  `start_byte..end_byte`, and `removed_text` equal to those bytes. The edit is
  then committed **durably first** through the edit-ledger helper (below):
  source and applied edit ID land together in one fsynced record and a receipt
  comes back. Only then is the durable document adopted in the editor as one
  undoable edit (`pendingEdit`), the `.tex` re-exported atomically when the
  document is file-backed, and `capture_applied {capture_id, edit_id,
  new_revision}` sent — no `document_edit` for that change. On the bridge's
  acknowledgement the helper drops its recovery snapshot (`confirm`). A helper
  refusal or persistence failure inserts nothing anywhere and sends no
  receipt; a persistence failure poisons the helper handle, which is relaunched
  and its on-disk state re-read before anything else. The same `edit_id` is
  never applied twice (bridge `already_applied`, helper `edit_id_conflict` /
  original-receipt replay), also after undo. Reviewer-edited LaTeX is refused
  (the contract has no field for it). `Reject` sends `capture_reject`.
- Restart reconciliation (ledger-guarded): before the current document is
  opened, the shell asks the helper for a `recovery_export` (snapshot token +
  pending receipts), queries the bridge `capture_status` for each pending
  capture, and hands the observations back as one `recovery_import`. An exact
  `applied` receipt is confirmed durably by the helper; a `prepared` edit that
  matches yields `replay_receipt`: the shell reopens the retained pre-edit
  snapshot on the bridge, resends `capture_applied`, and confirms only on an
  exact acknowledgement; anything `unavailable` (transport failure, or a bridge
  with no/conflicting record) keeps its evidence — transport failures are
  reported for retry (`Edit > Retry Bridge Reconciliation`), missing or
  conflicting bridge records are held for explicit operator resolution
  (`resolveReconciliation`, the only path that drops a snapshot). If source or
  ledger changed during the bridge round trip the import is refused as stale
  and the export is repeated (bounded). Nothing is ever reapplied to a changed
  document, and the durable document is authoritative: on attach, a store with
  pending receipts whose text differs from the buffer is adopted into the editor
  as one undoable operation; without pending receipts the buffer replaces the
  stored text (`replace_document`).

### Edit ledger helper (durable document transaction)

The durable document store is the Commander's Rust `crates/edit-ledger`
(`flashtex-edit-ledger --store <dir>`), pinned at commit
**afb15839f8080f9e86efd6a187e46dfe014ce559** on
`origin/agent/mac-contract-review/edit-ledger` (an unmerged dependency; built
in a scratch worktree for validation). It is located like the other helpers:
`$FLASHTEX_EDIT_LEDGER`, a `flashtex-edit-ledger` bundled next to the
executable, then `crates/edit-ledger/target/{release,debug}/flashtex-edit-ledger`.
One store per document lives under `<captures store>/documents/`, keyed by the
SHA-256 of the file path (`file-…`) — an unsaved buffer gets a fresh
`unsaved-…` store per attach, so its pending receipts are not recoverable
after a restart (save the document to make them so). Without the helper the
bridge still attaches but capture insertion is disabled (fail closed); the
same when its store is corrupt/unreadable (`invalid_store`), in use by another
process (`store_in_use`), or bound to another document.

`Edit > Retry Bridge Receipt` (withheld export/receipt) and `Edit > Retry
Bridge Reconciliation` (incomplete reconciliation) have no shortcuts.

The Swift side (`EditLedgerClient`, `BridgeSession`) keeps only an in-memory
mirror of the helper's document and transactions. Ordinary typing and undo go
through `replace_document` (expected revision + source hash; applied-ID
tombstones survive undo); a refused replace marks the session diverged and
disables application until the next attach reconciles. Editor revisions and
the helper's document revision advance in lockstep; on attach an older store
is aligned upward, never the editor downward. Every helper reply's
`session_id` / `sequence` / `document_revision` / `document_sha256` is
checked: a foreign session or non-increasing sequence is discarded as a stale
observation, and callbacks of a replaced helper process (or a detached bridge
session) never update the model. All pipe I/O (bridge and helper) runs on a
bounded serial background queue (`LineProcessClient`, 32 MiB in flight,
`backpressure` beyond it, 12 MiB per line) — a stalled reader never blocks the
main thread. `recovery_export` / `recovery_import` are used as above;
`compact` is not used yet.

Requested ledger changes: none required for this integration. Observations for
the ledger owner: (1) the helper answers `invalid_request` with `id: null`
when the operation name is unknown, so such a request can only be matched by
timeout; (2) `replace_document` bumps exactly one revision, so aligning an
older store to a newer editor revision takes one round trip per step (bounded
at 10 000 here); a `set_revision`-style alignment would remove that loop.

Not implemented here: the Mac credential adapter that would run the bridge with
`--enable-grok` and supply the authorized `XAI_API_KEY` (so no real conversion
happens from this shell), the companion network transport (captures come from
a file picker), compiler validation of proposals before review, and ledger
compaction.

`Tests/FlashTeXMacTests/Fixtures/fake_bridge.py` and `fake_edit_ledger.py` are
stdlib-Python test doubles of the bridge (in-memory journal, deterministic
`\fakecapture{<id>}` proposal, `%stall`/`%error`/`%garbage` directives) and of
the edit-ledger helper (real atomic `document.json`, same validation and error
codes, service metadata, recovery export/import). `RealBridgeTests` runs only
when `FLASHTEX_BRIDGE` points at a built binary; `RealEditLedgerTests` only
when `FLASHTEX_EDIT_LEDGER` does (both with temporary stores). The shared
fixture `protocol/fixtures/capture-submission.json` decodes since main 9da7e48.

## Samples

`Samples/multipage-result.json` + `multipage-request.json` (⌘O on the result):
two pages, nine text items with byte-exact source ranges into a `main.tex` that
contains non-ASCII words ("naïve", "Résumé") so UTF-8 and UTF-16 offsets differ,
plus one error diagnostic with a source range and recovery text and one warning
with null source/recovery. Use it for manual click-to-source, caret-sync, and
diagnostics checks beyond the one-line contract fixture.

- Faces: LaTeX's default is Computer Modern, so the preview and CoreGraphics
  export use **Latin Modern** (GUST FL; registered at launch from `FLASHTEX_LM_DIR`,
  a bundled `Resources/Fonts`, or BasicTeX's `fonts/opentype/public/lm`) when the
  attached producer is the new `flashtex-render` pipeline or `FLASHTEX_PREVIEW_FACE=latin-modern`
  is set; when attached to today's `flashtex-compiler` (Core-14 Times metrics) they
  draw Times-Roman so glyph widths match the positions. Optical masters follow
  LaTeX (lmroman5/7/8/9/10/12/17).
- Export is always white: dark preview is a viewing mode only. `File > Export
  PDF…` (⌘⇧E) uses CoreGraphics; `File > Export PDF via Rust Writer…` (⌘⌥E) pipes
  the current `compile_result` envelope to the FT-009 `flashtex-pdf --verify`
  binary (`$FLASHTEX_PDF` or `crates/pdf/target/{release,debug}/flashtex-pdf`).
  `RustPDFExportTests` runs only when `FLASHTEX_PDF` is set.
- Diagnostics panel under the preview is never hidden when diagnostics exist; each
  entry shows severity, message, recovery note (or "no provisional rendering"),
  and source bytes; the banner shows error/warning counts and a `recovered` note.

## Nearby companion (proposal nearby-v1)

`Edit > Nearby Companion…` (⌘⇧N) opens a window that advertises this Mac to a
paired iPad/iPhone companion and receives its `capture_submit` messages over an
authenticated, encrypted connection. **This is a proposal until the Commander
publishes `docs/contracts/nearby-v1.md`**; the full text, threat model and
companion checklist are in `docs/nearby-v1-proposal.md`. The companion side is
FT-004's. What is implemented here (Mac side only):

- Discovery: Bonjour `_flashtex._tcp`, instance name = the Mac's name, TXT
  `v=1`, `name=<Mac name>`, `fp=<16 hex, SHA-256 of "flashtex-nearby-v1 mac-id"‖salt>`,
  `salt=<32 hex>`. Discovery only; nothing in TXT is trusted.
- Pairing: "Show Pairing Code" displays a 6-digit CSPRNG code for 120 s. Both
  sides derive a bootstrap PSK and `pair_id` with HKDF-SHA256 from
  (code, salt) (`Pairing.derive`; pinned vector in `PairingTests`). The first
  `hello` over that key returns a random 32-byte long-term `pair_psk`; the
  bootstrap key is then dropped. Pairings persist in
  `~/Library/Application Support/FlashTeX/pairs.json` (mode 0600, atomic
  writes, **not the Keychain**); "Forget" removes one and closes its session.
- Transport (`NearbyListener`): Network.framework `NWListener`, TLS **1.2
  only**, cipher suite **`TLS_PSK_WITH_AES_128_GCM_SHA256` (0x00A8)** only,
  PSK identity = `pair_id`, one key per pairing plus the pending bootstrap key,
  **TLS resumption and tickets disabled** (with resumption on, a removed key
  could still resume — caught by the tests). Ephemeral port; the listener is
  rebuilt on the same port when the key table changes and live sessions are
  adopted, not dropped. An unpaired peer fails the handshake: no line is parsed.
- Framing: runtime-v1 JSON Lines like the bridge; 12 MiB per line including
  the newline, oversized complete or unterminated lines get `error
  line_too_long` and a close. First line must be `hello {pair_id,
  companion_name, protocol_version:1, nonce, proof}` where `proof` is
  HMAC-SHA256(PSK, "flashtex-nearby-v1 hello"‖nonce) — needed because
  Network.framework does not say which table PSK a session used. Reply
  `hello_ack {mac_name, nonce, destination, pair_psk?}`; `destination_query` →
  `destination {destination: {destination_id, project_id, path, base_revision} | null}`
  from the pinned anchor (⌘⇧P), so the companion never types IDs.
- Captures: `capture_submit` is validated (ids, MIME, instructions ≤ 4096 B)
  and handed to a `CaptureSink` (`ShellModel+Nearby.swift`). With a capture
  bridge attached the capture is forwarded through `BridgeSession.submit`
  and the bridge's `capture_received` (durable) or `error` code is returned
  to the companion, and it then appears in the bridge capture list for
  `Convert Capture`. Without a bridge, `receiveNearbyCapture` keeps the last
  50 captures in an in-memory `NearbyInbox` and answers `capture_received
  {capture_id, durable:false, has_proposal:false, applied:false}`; identical
  retries are acknowledged again, a different payload for a known id is
  `capture_id_conflict`. `hello_ack.destination` is the bridge's valid anchor
  when attached, else the local pinned anchor. A plaintext (non-TLS) peer —
  the companion's current `NWParameters.tcp` client — fails the handshake,
  is logged once, and nothing it sent is parsed.
- Threat model and gaps (see the proposal): the 6-digit code is ~20 bits and
  the PSK suite has no forward secrecy, so a passive capture of the pairing
  window can be brute-forced offline — the window is short and one code pairs
  one device; no certificate PKI, no cloud relay, no Keychain, no peer-to-peer
  (AWDL), no companion notification beyond `capture_received`. A bundled
  `.app` will need `NSLocalNetworkUsageDescription`/`NSBonjourServices`; the
  bare executable and `swift test` did not prompt on macOS 26.3.

## Launch hooks and evidence

`FLASHTEX_NO_ACTIVATE=1` launches without activating/focusing the window (for
automation; never steals keyboard focus). `FLASHTEX_DEBOUNCE_MS` sets the
keystroke-to-compile delay (default 0: every edit submits immediately; one
request in flight, newest buffer coalesced).
`FLASHTEX_LOG=<path>` appends timestamped worker/bridge status lines (e.g.
`status: worker exited (9)`) for automation such as `scripts/launch-check.sh`.
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

## Completion and navigation

Completion (`Completion.swift`) is a pure engine over the buffer's UTF-8 bytes
with the caret in UTF-16 units, wired into the editor through a small
`NSTextView` subclass (`CompletingTextView`) whose user-completion range includes
a leading `\`. Esc or ⌃Space opens the standard AppKit completion popup; choosing
an entry replaces the partial token. Sources, in rank order, at most 12 entries:

1. `\end{X}` for every `\begin{X}` before the caret that is still unclosed
   (detail names the byte of the `\begin`).
2. Commands the compiler supports: the list in `crates/compiler/README.md`
   (`\section \subsection \textbf \emph \textit \begin \end \par \\`) plus the 24
   math commands present in `crates/compiler/src/math.rs` on
   `agent/claude/compiler-foundation` at `de1020c` (`\frac \sqrt \alpha … \int`),
   whose detail says "math · verified in compiler at de1020c". Pass another
   `supported:` list when the compiler's set changes.
3. Commands typed elsewhere in the document that are not in that list, marked
   "not supported by this compiler version" (plus the compile result's
   diagnostic message when one names the command).
4. After `\begin{`/`\end{`: environment names (open ones first, then `document`,
   then names seen in the buffer); after `\ref{`/`\eqref{`/`\pageref{`/`\autoref{`:
   `\label` arguments seen in the buffer.
5. Prose: words longer than 3 characters from the document, frequency-ranked,
   ASCII case-insensitive prefix, triggered after 2+ letters. The word being
   typed is not counted as its own completion.

Commands trigger on `\` (empty prefix lists everything supported). Invalid
carets (negative, past the end, inside a surrogate pair) and malformed input
(`\begin{`, stray braces, runs of backslashes) yield no suggestions and never
trap. Measured on a 1 000 069-byte buffer (`CompletionTests`, M1 Max, 2026-09-12):
release build words 1.9 ms, commands 0.8 ms per call (XCTest `measure` of one
word + one command completion: 2.6 ms average, RSD 0.9%); debug build 8.4 ms /
5.3 ms (`measure` 13.7 ms). Scans jump between candidate bytes with `memchr`
and decode only matches, so the cost is proportional to candidates, not bytes.

Navigation (`Navigation.swift`, `Navigate` menu):

- **Go to Matching** (⌘⇧D): from `\ref{X}`-style commands to `\label{X}`; from
  `\label{X}` to its references in turn (wrapping); `\begin{X}` ↔ `\end{X}` with
  same-name nesting. Pure functions over the current buffer with byte-exact
  ranges converted to UTF-16 for the selection; misses are explained in the footer.
- **Next / Previous Diagnostic** (⌘⇧] / ⌘⇧[): cycles (wrapping) through the
  result's diagnostics that have a source in the active document, ordered by
  their position in the current buffer. Each jump goes through
  `ShellModel.navigate(to:expectedText:)`, so a diagnostic whose span overlaps an
  edit made since the compile is refused with "recompile to navigate" rather
  than selected on the wrong text; the others stay reachable. Diagnostics with
  null `source` are skipped and counted in the footer note.
- **Reveal Caret in Preview** (⌘⇧J): selects the full source span of the preview
  item under the caret (`CaretSync`) so the preview highlight and page scroll
  follow, and names the page and item.

Without a compile result the navigation commands are disabled and, if invoked,
explain that nothing is loaded.

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
| ⌘⇧N | Nearby Companion… (advertise, pairing code, paired devices, received captures) |
| ⌘Z | Undo (including an approved capture insertion) |
| Esc / ⌃Space | Completion popup (supported commands, `\end{…}` for open environments, labels, document words) |
| ⌘⇧D | Go to matching `\begin`/`\end` or `\label`/`\ref` |
| ⌘⇧] / ⌘⇧[ | Next / previous diagnostic (refused if its span was edited since the compile) |
| ⌘⇧J | Reveal caret in preview (selects the item's source span) |
| Click preview text | Select its source (UTF-8 span → UTF-16; refused if edited since compile) |

The compiler rejects request lines over 8 MiB with an `error` envelope, which the
banner shows; the shell rejects response lines over 16 MiB.

## Targets

- `FlashTeXProtocol` — Codable models for runtime v1 (`RuntimeV1`) and the
  capture bridge (`TransferV1`), byte-offset conversion, changed-region diffing.
- `FlashTeXMac` — the app. `BridgeClient` (JSON Lines transport, id-correlated
  replies, 12 MiB line limit), `BridgeSession` (bridge-side document shadow,
  destination, captures, edit ledger, reconciliation), `ShellModel+Bridge`.
- `FlashTeXMac` also holds `LineProcessClient` (shared bounded JSON Lines
  process transport) and `EditLedgerClient` (edit-ledger helper protocol).
- Tests (136 across all targets, of which `RealCompilerTests`, `RustPDFExportTests`,
  `RealBridgeTests` and `RealEditLedgerTests` are gated on `FLASHTEX_COMPILER`,
  `FLASHTEX_PDF`, `FLASHTEX_BRIDGE` and `FLASHTEX_EDIT_LEDGER`): completion
  (prefix/trigger rules, unclosed `\end{}`, unsupported marks, non-ASCII and
  invalid carets, 1 MB latency) and navigation (label/ref incl. Unicode, nested
  begin/end, diagnostic cycling/wrap/refusal on the multipage sample, caret
  reveal); bridge transport round trip of every transfer-v1 request type incl. error envelopes,
  garbage/oversized/trailing lines and oversized requests; edit-ledger helper
  round trip (initialize/apply/dedup/undo/confirm/error codes); shell ↔ fake
  bridge + fake ledger flow (open → edit → pin → submit → received → convert →
  review → prepare → verify → durable apply → adopt once → `capture_applied` →
  confirm, duplicate approval no-op incl. after undo, edited-LaTeX refusal,
  stale edit refused by shell and helper, provider error as text, reject
  forwarded, no-helper fail-closed); fault/recovery (persistence failure inserts
  nothing and sends no receipt, file-backed export before receipt, export
  failure withholds the receipt while the durable commit stands, corrupt or
  unreadable store fails closed, transient status failures retain
  transactions and snapshots, missing receipt replayed from the retained
  snapshot only for an exactly matching prepared edit, stalled bridge never
  blocks the main thread + backpressure, detach/reattach ignores old-session
  events, edits during reconciliation); real bridge (durable receipt,
  duplicate/conflict, `provider_disabled`, `proposal_missing`, reject,
  `capture_missing`); real edit ledger (durable apply with on-disk hash check,
  duplicate edit ID refused, recovery export/import incl. stale token, undo
  keeps dedup, session metadata, second writer refused, full shell flow);
  nearby listener (TLS-PSK round trip of `hello` /
  fixture `capture_submit` / `destination_query` through an `NWConnection`
  client with the same PSK, wrong key and unknown identity refused before any
  line is parsed, oversized complete and unterminated lines close, message
  before `hello`, cross-pairing `hello` with the wrong proof, nonce reuse,
  bootstrap → long-term PSK hand-over with the old key refused after a
  same-port restart while the live session survives, Bonjour advertising
  reaches `.ready`, pure session validation), HKDF/proof vectors, pair store
  round trip with 0600 and corrupt-file handling, `ShellModel` inbox and
  destination, `NearbyState` pair → capture → forget → same-port restart
  flow, plaintext peer refused without parsing while TLS peers keep working,
  nearby capture forwarded through the fake bridge (durable ack, error
  pass-through, inbox fallback after detach); plus the earlier: oversized complete line, trailing bytes at EOF, unsolicited/mismatched result correlation; inline diagnostic marks (byte→UTF-16, rebase/drop, path filter,
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

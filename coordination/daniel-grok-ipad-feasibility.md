# iPad handwriting-capture feasibility (issue #51) — mac-m5pro-dq222

Scope: can this machine do the iPad handwriting-capture UI work, or should
#51 go to another Mac. Read-only investigation; nothing in this worktree was
changed (`git status` clean throughout — verified at the end). All test
compiles ran against scratch copies under `/private/tmp/.../scratchpad`, never
against tracked files.

## 0. Headline finding: this work already exists on another lane, uncommitted to main

Before answering "can we build one," a full iPad/iPhone companion app for
exactly this feature **already exists** in this repo's git history, on
**`remotes/origin/agent/chatgpt-a/companion-reliability`** (tip
`439c8cc4`, FT-004, machine `aarush-macbook`) — not yet merged to `main`
(`git merge-base --is-ancestor 439c8cc4 main` → not an ancestor). It is a real
`FlashTeXCompanion.xcodeproj` (not a SwiftPM package) at `apps/companion/`
with:

- `Views/DrawingCanvasView.swift` — `import PencilKit`, a full-screen
  `PKCanvasView` capture surface with a destination strip, context-note
  field, and Send button (exactly the "capture proposal" surface described in
  `apps/mac/Samples/capture-proposal.json`).
- `Views/CameraCaptureView.swift` — camera/photo-library capture alternative.
- `Services/BonjourTransport.swift` — `NWBrowser`/`NWConnection` client for
  the Mac's `_flashtex._tcp` service.
- `Models/CapturePayload.swift`, `Services/ImageValidator.swift` — encode the
  drawing to PNG/JPEG and build the runtime-v1 `capture_submit` envelope.
- An XCTest target (`FlashTeXCompanionTests`) with real unit tests.

`apps/mac/docs/nearby-v1-proposal.md` (§7, §8) is the Mac side's spec for
exactly this companion, including a git-blame-able commit hash and a line-by-line
diff of what the companion still needs to change. This is not a proposal for
new work — it's a maturing, actively-developed feature (20 commits, most
recent same day) on a sibling lane.

**Implication for the Commander:** #51 is very likely already assigned to, or
overlapping with, FT-004 / `aarush-macbook`. The open question for this
machine is not "build an iPad app from nothing" but "can this machine build
and run/verify that app, or contribute to it" — answered below.

## 1. `apps/mac` structure and portability

`apps/mac/Package.swift`: `platforms: [.macOS(.v14)]` — no iOS declared, for
any target, at the package-manifest level. Three targets:

| Target | Lines | Imports | Portable? |
|---|---|---|---|
| `FlashTeXProtocol` | ~1400 (8 files) | `Foundation` only, every file | **Fully portable.** Proved by typecheck (§3). |
| `FlashTeXAccessibility` | 1146 (4 files) | 3 files `Foundation`/`SwiftUI` only; **1 file** (`AccessibilityViews.swift`, 225 lines) `import AppKit` unconditionally | **80% portable** (the 3 model/command files); the AppKit view file is not, and has no `#if os(...)` guard — it would need a UIKit/SwiftUI-only sibling. |
| `FlashTeXMac` (executable) | ~22 files | 14 of them `import AppKit` (`ShellModel.swift`, `SourceEditorView.swift`, `ContentView.swift`'s dependents, `PDFExport.swift`, etc.) | **AppKit-only.** This is the whole app shell; none of it targets iOS. |

`FlashTeXProtocol` is the interesting piece for #51: it's the Codable model
of the runtime-v1 wire format (`Capture.swift` defines `CaptureImage`/
`CaptureSubmit` mirroring the Rust side almost field-for-field), plus
`JSONLines.swift` framing and `TransferV1.swift`. An iPad app doesn't need to
reimplement this protocol from scratch in Swift — it could literally import
this target as a local Swift package dependency, though the existing
companion app (§0) instead reimplements its own smaller `CapturePayload.swift`
independently (reasonable, since it's a separate Xcode project, not an SPM
consumer of `apps/mac`).

**Also load-bearing and NOT portable:** `BridgeClient.swift` /
`LineProcessClient.swift` spawn `flashtex-bridge` as a local subprocess via
Foundation's `Process` (confirmed: `LineProcessClient.swift:61` `private let
process = Process()`, `:92` `process.executableURL = executable`). `Process`
is unavailable on iOS — the sandbox forbids launching arbitrary executables.
This is *why* the real companion app (§0) talks over a network (Bonjour/TLS)
instead of embedding the bridge: there is no way to run `flashtex-bridge`
inside an iPadOS app. Confirmed by reading the code, not inferred from
platform docs alone.

## 2. How an iPad app hands bytes to `crates/bridge` (from reading the actual code)

`crates/bridge/src/main.rs` is a stdin/stdout JSON-Lines process
(`flashtex-bridge --store DIR [--enable-grok]`), reading runtime-v1 envelopes
`{protocol_version:1, id, type, payload}` and dispatching on `type`. The
message that matters here is `capture_submit`, decoded into
`flashtex_bridge::CaptureSubmit` (`crates/bridge/src/lib.rs`):

```rust
pub struct CaptureImage { pub mime_type: String, pub data_base64: String }
pub struct CaptureSubmit {
    pub capture_id: String,
    pub destination_id: String,
    pub base_revision: u64,
    pub image: CaptureImage,     // mime_type ∈ {image/png, image/jpeg}, base64, ≤8 MiB decoded
    pub instructions: String,    // ≤4096 bytes
}
```

`CaptureSubmit::validate()` decodes the base64, decodes the image with the
`image` crate (rejecting anything not really PNG/JPEG or over 8192×8192 /
64 MiB alloc), and `Bridge::receive()` journals it keyed to a previously
pinned destination anchor (`destination_pin` message, byte-range in the
Mac's open document).

**There is exactly one real entry point for external bytes: a `capture_submit`
JSON envelope on the bridge's stdin.** An iPad app cannot write to that stdin
directly (no subprocess access, §1). The only sanctioned path, and the one
FT-004's companion already implements, is `apps/mac/docs/nearby-v1-proposal.md`'s
Nearby transport: the iPad opens a paired TLS-PSK connection to the Mac
app's Bonjour service, sends the *same* `capture_submit` JSON payload over
that socket, and the Mac (`Edit > Attach Capture Bridge`) relays it verbatim
into its own already-running `BridgeClient` subprocess and returns the
bridge's real acknowledgement. The wire schema, byte limits, and validation
are 100% the bridge's (`transfer-v1`); Nearby is transport only, not a
second protocol.

Minimum PencilKit capture surface implied by this contract (and already built
in §0): a `PKCanvasView` on white background, a destination label (bound to
the Mac's currently pinned anchor — the companion must never let the user
type `destination_id`/`revision`), an optional instructions text field
(≤4096 bytes), and a Send action that rasters the canvas
(`PKDrawing.image(from:scale:)` → PNG/JPEG, base64), builds a `capture_submit`
envelope, and sends it over the paired connection.

## 3. What compiles — proved by running commands, not inferred

**apps/mac via `xcodebuild -scheme ... -showdestinations`:** real, run in
place (`cd apps/mac`):

```
xcodebuild -scheme FlashTeXProtocol -showdestinations
```
→ only macOS/DriverKit destinations listed as *available*; iOS/tvOS/
visionOS/watchOS listed as *ineligible* with
`error:iOS 26.5 is not installed. Please download and install the platform
from Xcode > Settings > Components.` — for **every** platform, not just iOS.

I then copied `apps/mac` to a scratch directory (not the tracked worktree)
and edited only the copy's `Package.swift` to add `.iOS(.v17)` alongside
`.macOS(.v14)`, to isolate "package doesn't declare iOS" from "machine can't
target iOS." Result: **identical** ineligible-destination list. Declaring the
platform in the manifest changes nothing — the block is at the Xcode
platform-registry level, not the SwiftPM manifest.

Then, real attempted builds against that same scratch copy:
```
xcodebuild -scheme FlashTeXProtocol -destination 'generic/platform=iOS Simulator' build
xcodebuild -scheme FlashTeXProtocol -sdk iphonesimulator -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build
```
→ both: `xcodebuild: error: Unable to find a destination matching the
provided destination specifier: { generic:1, platform:iOS Simulator }` —
even the destination-less "generic" simulator target xcodebuild normally
offers when the iOS platform bundle is absent doesn't exist here. **For any
SwiftPM package driven through `xcodebuild -scheme`, this machine currently
cannot reach an iOS destination at all,** regardless of manifest changes.

**But raw SDK compilation is a different code path and does work — proved,
not inferred:**

```
$ SDK=$(xcrun --sdk iphonesimulator --show-sdk-path)   # succeeds:
  /Applications/Xcode.app/.../iPhoneSimulator.platform/.../iPhoneSimulator26.5.sdk
$ cd apps/mac/Sources/FlashTeXProtocol
$ swiftc -sdk "$SDK" -target arm64-apple-ios17.0-simulator -typecheck *.swift
# exit 0, zero diagnostics — the ENTIRE FlashTeXProtocol target (Capture.swift,
# CaptureImage/CaptureSubmit, TransferV1, JSONLines, RuntimeV1, ByteOffsets,
# SourceMapping, Rules, LayoutNegotiation) typechecks clean for iOS.
```

Then built `FlashTeXProtocol` as a real `.swiftmodule` for that SDK/target
and typechecked the accessibility target's non-AppKit files against it:
```
$ swiftc -sdk "$SDK" -target arm64-apple-ios17.0-simulator \
    -module-name FlashTeXProtocol -emit-module ... FlashTeXProtocol/*.swift
# exit 0
$ swiftc -sdk "$SDK" -target arm64-apple-ios17.0-simulator -I <module dir> \
    -typecheck AccessibilityCommands.swift AccessibleDocumentModel.swift AccessibleEditorModel.swift
# exit 0 — clean
$ swiftc ... -typecheck AccessibilityViews.swift
# error: AccessibilityViews.swift:1:8: error: no such module 'AppKit'
```
Exactly the one file predicted in §1 fails, with exactly the predicted error,
and nothing else does.

**And a full, real iPadOS app builds end-to-end on this machine, right now,
with zero downloads** — the existing FT-004 companion project (§0), exported
read-only from `remotes/origin/agent/chatgpt-a/companion-reliability` via
`git archive` into scratch (never touching this worktree or that branch):
```
$ xcodebuild -project FlashTeXCompanion.xcodeproj -target FlashTeXCompanion \
    -sdk iphonesimulator CODE_SIGNING_ALLOWED=NO build
...
** BUILD SUCCEEDED **
```
This produced a real universal (arm64 + x86_64) `FlashTeXCompanion.app`,
PencilKit/UIKit/Network/PhotosUI code included, no errors, no warnings about
missing platform components. This is the exact command the FT-004 lane
already landed in `ea623c4b` ("fix(companion): build without installed
simulator runtime") specifically to route around the empty-runtime-list
problem — and it is proved to also work on **this** machine's Xcode 26.6.

**Reconciling the two results:** `xcodebuild -scheme` (SwiftPM package,
destination-based resolution) and `xcodebuild -project/-target -sdk
iphonesimulator` (classic Xcode project, SDK-based, no destination lookup)
go through different subsystems inside Xcode. The "iOS 26.5 is not installed"
error is specific to the *destination/runtime registry* (used to find or boot
a device). The iPhoneSimulator SDK *headers and compiler support*, which is
all `-sdk iphonesimulator` needs, ship inside Xcode.app itself and are
present regardless. So: **compiling iOS/iPadOS code — including a PencilKit
UI — is fully possible on this machine today, with an Xcode project.
Compiling the same portable code through the existing SwiftPM package's
`-scheme` path is currently blocked**, and would need either (a) a real
`.xcodeproj`/`.xcworkspace` wrapping the portable targets (matching what
FT-004 did for the companion), or (b) driving `swiftc`/`swift build`
directly against the SDK the way the typecheck test above did.

## 4. The simulator-runtime gap, precisely

- `xcrun simctl list runtimes` → empty (confirmed, matches the parent's
  established facts). `xcrun simctl list devices` → empty.
- Exact command to install a runtime: `xcodebuild -downloadPlatform iOS`
  (optionally `-buildVersion 26.5 -exportPath <dir>`), or Xcode ▸ Settings ▸
  Components ▸ iOS 26.5 ▸ Get. **I did not run this** — it starts a real
  multi-gigabyte download, which the hard constraint requires reporting and
  asking about first.
- Download size: **I could not determine this without starting the
  download**, and did not find a size-preview flag (`xcodebuild -help` has no
  dry-run/size-only mode for `-downloadPlatform`; `xcrun simctl list runtimes
  available` lists identifiers, not sizes). This is an inferred estimate,
  not a measurement: Apple's own Xcode 26 release notes (via a Bitrise
  writeup on Xcode 26 Beta 5's platform-size reduction) give a concrete
  number only for tvOS — Universal 4.83 GB → Apple-Silicon-only 3.62 GB
  (~25% smaller), with Apple stating iOS/watchOS would see "similar savings"
  but no figure published. iOS simulator runtimes have historically run
  larger than tvOS's (more device families/architectures); pre-Xcode-26 iOS
  runtimes were commonly 6–9 GB. **Best-effort estimate: roughly 5–8 GB for
  iOS 26.5 Apple-Silicon-only, likely more for a Universal variant — not
  verified on this machine.** The only way to see the real number without
  downloading is Xcode's own Settings ▸ Components pane, which I did not
  open (GUI, outside this session's tooling) — worth doing manually before
  committing to the download.
- **What actually needs that download:** *booting/running* a simulated
  iPad (Simulator.app, `simctl boot`, or a `-destination` build/run/test
  cycle) to see and interact with the capture UI. It does **not** block
  compiling the UI code (§3) or reviewing/extending the existing companion
  app's source.
- **Physical iPad instead — not tested (no device attached), reasoning from
  the same `-showdestinations` output plus standard Xcode/iOS facts:** the
  device-support/on-device-debugging platform component is *also* currently
  reported absent (`showdestinations`'s "Any iOS Device" line: `error:iOS
  26.5 is not installed...`, same message, same Settings ▸ Components fix),
  but that component is historically far smaller than the full simulator
  runtime image (tens to low-hundreds of MB, not verified here). Beyond that
  download, a physical device additionally needs: an Apple ID signed into
  Xcode ▸ Settings ▸ Accounts (a free personal-team account is enough for
  local sideloading, capped at 7-day-expiring builds and a handful of apps;
  a paid $99/yr Apple Developer Program membership removes both caps),
  automatic-signing-generated provisioning profile (`CODE_SIGNING_ALLOWED=NO`
  — used above for the simulator build — does **not** work for a device; a
  device build must actually sign), the iPad's Settings ▸ Privacy & Security
  ▸ Developer Mode turned on (requires a restart) and the resulting
  developer certificate trusted on-device, and either a USB cable or
  wireless-debugging pairing already established in Xcode ▸ Devices. None of
  this was exercised — no iPad is attached to this machine in this session.

## Verdict

**This machine CAN do meaningful iPad-UI work on #51 today**, and arguably
*already has an unmerged head start via FT-004*:

- **Proved:** the wire-protocol layer (`FlashTeXProtocol`, all 8 files) and
  4/5 of the reusable accessibility model code compile clean for iOS 26.5
  Simulator right now, no downloads, via direct `swiftc -sdk iphonesimulator`
  invocations (exit 0, zero diagnostics) — evidence in §3.
- **Proved:** a full, real, existing PencilKit/UIKit iPad companion app
  (from another lane's unmerged branch) builds end-to-end
  (`** BUILD SUCCEEDED **`) on this machine's Xcode 26.6, right now, with no
  simulator runtime installed, using `-sdk iphonesimulator
  CODE_SIGNING_ALLOWED=NO` against a real `.xcodeproj` — evidence in §3.
- **Proved:** the SwiftPM-package `apps/mac` cannot currently reach any iOS
  destination through `xcodebuild -scheme` regardless of `Package.swift`
  changes, because Xcode's platform/destination registry (not the SDK)
  reports `iOS 26.5 is not installed` — evidence in §3.
- **Proved:** the Mac app's bridge transport (`Process`-spawned
  `flashtex-bridge`) cannot be reused as-is on iOS; the only viable path
  for an iPad to reach `crates/bridge` is the already-specified
  Nearby TLS-PSK network relay (§1, §2), which the FT-004 companion already
  implements.

**The exact blocking step** is narrower than "no iOS toolchain here": it is
*"cannot boot/run/visually verify a simulated iPad, and cannot deploy to a
physical one,"* both for the same underlying reason — the iOS *platform*
(runtime + on-device debug support) is not installed, only the SDK is. That
is a real gap, but it is a **downloadable, not a hardware, blocker**, and it
does not block writing, compiling, or code-reviewing the PencilKit UI itself.

**What it costs to unblock:**
- To *run/see* the UI in Simulator: one `xcodebuild -downloadPlatform iOS`
  (or the Components-pane equivalent), an unverified but plausible
  multi-gigabyte (roughly 5–8 GB, unconfirmed) download — needs your
  go-ahead per the hard constraint before I or anyone runs it.
- To *run on a physical iPad* instead: a smaller device-support download
  (size not measured), a signed-in Apple ID (free tier works for local
  testing), Developer Mode enabled on the device, and a cable or established
  wireless-debug pairing — no simulator runtime required at all. This is
  probably the cheaper unblock if a spare iPad is available, since it skips
  the large runtime download entirely.
- Neither is required to keep authoring/reviewing the Swift code itself, or
  to reconcile this machine's daniel-* bridge work with what FT-004 has
  already built on `agent/chatgpt-a/companion-reliability`.

**Recommendation for the Commander:** given §0, the real question is
probably not "can `mac-m5pro-dq222` build an iPad app" (yes, proved) but
"should #51's UI work continue on the `aarush-macbook`/FT-004 lane where a
tested, reviewed, nearly-complete companion app already lives, with this
machine picking up the runtime download (or a physical iPad) only if hands-on
verification needs to happen here."

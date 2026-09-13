# FlashTeXPad — the iPad capture companion

FlashTeXPad is a small iPad app that turns a Pencil sketch or a photo into
LaTeX/TikZ **on your Mac**. You draw or photograph something on the iPad, add
a one-line instruction ("this is a matrix", "convert this to TikZ"), and send
it to the Mac app. The Mac converts it into a proposal that *you review and
approve on the Mac* before anything is inserted into your document.

The iPad never edits your `.tex` file directly. It shows the status of each
capture and, once a proposal exists, the returned LaTeX read-only.

Source and full engineering notes: `apps/ios/README.md`.

## What you need

| | Requirement |
|---|---|
| Mac | FlashTeX for Mac ([install](README.md#quick-start-5-minutes)), on the same Wi-Fi network as the iPad |
| iPad | An iPad whose iOS version is supported by the Xcode you build with (the project is generated for iOS 17+; Xcode 26 was used for the recorded builds). iPad only — the target does not build for iPhone. |
| Build tools | Xcode with an iOS platform installed, a free or paid Apple ID (a *Personal Team* works), Python 3 for the project generator |
| Conversion | On the Mac: the capture bridge attached (*Edit › Attach Capture Bridge*) and either an xAI (Grok) API key in Preferences or a local provider — see [the AI assistant](gui.md#the-ai-assistant) |

There is no App Store or TestFlight build. You install it from source with
Xcode, exactly like any personal-team iOS project.

## Build and install on a device

1. Clone the repository and generate the Xcode project (only needed once, or
   after source files are added/removed):

   ```sh
   git clone https://github.com/flash-tex/flashtex.git
   cd flashtex/apps/ios
   python3 scripts/generate-xcodeproj.py
   open FlashTeXPad.xcodeproj
   ```

2. In Xcode select the **FlashTeXPad** target › *Signing & Capabilities*:
   tick *Automatically manage signing* and pick your team. The bundle
   identifier is `tech.jay3332.flashtex.FlashTeXPad`; if it is already taken in
   your team, change `BUNDLE_PREFIX` in `scripts/generate-xcodeproj.py` and
   regenerate. No extra capabilities are needed: local network, Bonjour
   (`_flashtex._tcp`), camera (QR scanning) and Keychain usage are already
   declared in `Info.plist`.

3. On the iPad enable **Settings › Privacy & Security › Developer Mode**
   (iOS 16+), restart when asked, then connect the iPad by cable (or pair it
   wirelessly in Xcode's Devices window).

4. Pick the iPad as the run destination and press **Run** (⌘R). The command-line
   equivalent is:

   ```sh
   xcodebuild -project FlashTeXPad.xcodeproj -scheme FlashTeXPad \
     -destination 'generic/platform=iOS' -allowProvisioningUpdates \
     DEVELOPMENT_TEAM=<TEAMID> CODE_SIGN_STYLE=Automatic \
     CODE_SIGN_IDENTITY="Apple Development" build
   ```

   followed by `xcrun devicectl device install app --device <udid> <path to FlashTeXPad.app>`.

5. First launch with a Personal Team: iOS refuses to open the app until you
   trust the developer certificate under **Settings › General › VPN & Device
   Management**. Free Apple IDs get a 7-day provisioning profile; rebuild from
   Xcode when it expires.

6. Allow the **Local Network** prompt the first time the app looks for your Mac.

If Xcode reports that the iPad's iOS version is not supported, update Xcode
(or its iOS platform download) — the installed Xcode must support the device's
iOS version.

To try the app without a device, run it in the iPad simulator
(`xcodebuild … -destination 'platform=iOS Simulator,name=iPad Air 11-inch (M3)'`);
the simulator has no camera and no Pencil, but the drawing canvas, photo
picker and the bundled sample image all work.

## Pairing with the Mac

Pairing is a one-time step per Mac. It uses a 6-digit code shown on the Mac
(valid for 120 seconds) to derive a shared key; the key is stored in the
iPad's Keychain and in `~/Library/Application Support/FlashTeX/pairs.json` on
the Mac.

On the Mac:

1. *Edit › Nearby Companion…* (⌘⇧N).
2. Turn on **Advertise**, then **Show Pairing Code**. The window shows the
   6-digit code, a QR code, an "expires in N s" countdown, and below them the
   Bonjour name, the listening **port** and the Mac id (fp). *Copy code*
   copies the six digits.

On the iPad, open **Mac link**:

- **Scan QR…** — point the camera at the Mac's QR code (real iPads with
  VisionKit support), or
- **Paste** a `flashtex-nearby://pair?…` payload (the text the QR encodes) and
  tap **Pair from payload**, or
- type the fields by hand (host, port, TXT salt, TXT fp, Mac name, code) and
  tap **Pair**. The salt and fp are in the Mac's Bonjour TXT record; the
  QR/payload route is the practical one.

The Mac is normally found by Bonjour. If discovery fails (different subnet,
guest Wi-Fi with client isolation), type the Mac's IP address and the port
shown in the Nearby window before pairing. After pairing, **Reconnect with stored key**
reconnects later without a new code; **Forget pairing** removes the key (do
the same on the Mac with *Forget* next to the device name).

A code is single-use: if the connection drops before the Mac answers, show a
new code and pair again.

## Sending a capture

1. On the Mac, put the caret where the result should go and choose
   *Edit › Pin Insertion Point* (⌘⌥P). The iPad's Capture screen shows the
   pinned destination (path and revision) — or "no insertion point pinned on
   the Mac", in which case sending is refused.
2. On the iPad's **Capture** screen, draw with the Pencil (or a finger), or
   tap **Photo…** to pick an image from Photos, or **Sample image**. **Clear**
   empties the canvas.
3. Type an instruction for the Mac (at most 4096 bytes), e.g. "convert this
   triangle to TikZ".
4. Tap **Prepare capture**. The app shows the PNG size; **Discard** throws it
   away (nothing was sent), **Send to Mac** sends it.
5. On the Mac, attach the capture bridge if it is not attached, then
   *Edit › Convert Capture* (⌘⇧G). Review the proposal in the sheet that
   opens (editable LaTeX, ambiguities, required packages) and **Approve** or
   **Reject**. Approval inserts exactly one undoable edit at the pinned point.

### What the iPad shows

Each capture is a row in the captures list with a status:

| Status | Meaning |
|---|---|
| drafted — not sent | Prepared but not sent yet |
| sending… / re-sending (attempt N, same capture_id) | In flight; a retry after a dropped connection reuses the same id, so the Mac never stores a duplicate |
| received | The Mac stored it (durable when a bridge is attached; otherwise "Mac inbox, no bridge attached — not converted yet") |
| refused by the Mac: `<code>` | e.g. `invalid_image`, `image_too_large`, `revision_mismatch` (the Mac document changed since the pin — re-pin and send again), `capture_id_conflict` |
| not acknowledged (…) — retry re-sends the same capture_id | The connection dropped before a receipt; **Retry** is safe |
| discarded before sending | You tapped Discard |

After a receipt the iPad polls the Mac every 2 seconds and shows the outcome:
*journaled* → *converting* → *proposal ready* (the LaTeX text appears, read-only)
→ *inserted (revision N)* / *rejected* / *conversion failed*. **Refresh status**
polls once more. Polling stops at a final state, after 5 minutes, or when the
Mac cannot answer ("outcome unavailable").

Captures, receipts and outcomes are saved on the iPad and restored after a
relaunch; an interrupted send comes back retryable with the same id.

## Limits

- The iPad cannot approve, reject or edit a proposal; only the Mac can.
  The returned LaTeX is read-only on the iPad.
- No push from the Mac: the iPad polls.
- No camera capture inside the app — use the Photos picker (take the photo
  with the Camera app first).
- Conversion needs the Mac's bridge and a provider (xAI key or local
  provider); without one the Mac reports `provider_disabled` /
  `provider_auth_missing` and the capture stays journaled.
- The pairing code is short (about 20 bits) and the connection has no forward
  secrecy; pair on a network you trust. One code pairs one device.
- Wi-Fi only, same network; no cloud relay.
- Real-device runs were not part of the recorded test evidence (simulator
  only, `docs/evidence/ios-acceptance-2026-09-12/`); the pairing and send
  paths are the same code as the Mac-side reference client.

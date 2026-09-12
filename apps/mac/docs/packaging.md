# Packaging: signing, Gatekeeper, and notarization gap report

`apps/mac/scripts/make-app.sh` produces an **ad-hoc signed** `FlashTeX.app`
(`codesign --force --deep --sign -`). This document records exactly what that
gives us, what it does not, the concrete steps needed to close the gap with a
real Apple Developer ID, and what we verified on this machine without one.

## What ad-hoc signing gives us

- A valid code signature *identity* (`CDHash`, `CodeDirectory`) so `codesign
  -dv` and `codesign --verify` succeed, and the bundle has a stable identifier
  (`tech.jay3332.flashtex.mac`) the OS can key entitlements/TCC prompts off of
  once it is signed with a real identity.
- Tamper-evidence for **this machine, this run**: if a file inside the bundle
  changes after signing, `codesign --verify --deep --strict` will now detect it.
- Enough of a signature that AppKit/Launch Services treat it as a normal `.app`
  (Dock icon, `NSRunningApplication` name, `open`/`open -a` all work — this was
  the actual problem FT-003's bare SwiftPM executable had).

## What ad-hoc signing does **not** give us

- **No Gatekeeper trust on another Mac.** `TeamIdentifier=not set` (see
  `codesign -dv` output below) means there is no Apple-issued identity behind
  the signature. Gatekeeper's `spctl -a` assessment (used by Launch Services
  when a user double-clicks an app, separate from a bare `codesign --verify`)
  rejects it outright — see the real output below. It is not "unsigned", it is
  signed by nobody Apple recognizes, and Gatekeeper treats that the same as
  unsigned for its purposes.
- **No quarantine-safe distribution.** A file downloaded via a browser, Mail,
  Messages, AirDrop, etc. gets the `com.apple.quarantine` extended attribute.
  On first launch, Gatekeeper re-checks quarantined files against `spctl`'s
  policy; an ad-hoc-only bundle fails that check regardless of quarantine
  status (verified below — quarantined and non-quarantined copies were both
  rejected). Only Developer ID signing + notarization lets a quarantined copy
  pass.
- **No stapled notarization ticket**, so Gatekeeper also cannot fall back to
  an offline-cached "Apple scanned this and found no malware" record.
- Ad-hoc signing is per-machine/per-build only: rebuilding regenerates the
  signature (no stable Team ID), so there is nothing for another machine to
  have "trusted before."

Net effect: ad-hoc signing is sufficient for **local development and demos on
this machine** (`open FlashTeX.app` from a local build works fine — quarantine
is never set on files created locally, and Gatekeeper's Launch Services path
generally doesn't re-run `spctl` for an app that was never quarantined). It is
**not** sufficient to hand `FlashTeX.app` to another person or machine; they
will see Gatekeeper's "cannot be opened because Apple cannot check it for
malicious software" dialog (the GUI equivalent of the `spctl` rejection below).

## Real verification on this machine (no Apple Developer account)

Commands run against the current ad-hoc bundle (`apps/mac/build/FlashTeX.app`,
built by `make-app.sh` from commit `4c6bf47`+, see the Validation section of
the packaging worker's commits for exact SHAs):

```console
$ codesign -dv apps/mac/build/FlashTeX.app
Executable=/…/apps/mac/build/FlashTeX.app/Contents/MacOS/FlashTeX
Identifier=tech.jay3332.flashtex.mac
Format=app bundle with Mach-O thin (arm64)
CodeDirectory v=20400 size=2578 flags=0x2(adhoc) hashes=74+3 location=embedded
Signature=adhoc
Info.plist entries=13
TeamIdentifier=not set
Sealed Resources version=2 rules=13 files=13
Internal requirements count=0 size=12
```

```console
$ spctl -a -vv apps/mac/build/FlashTeX.app
apps/mac/build/FlashTeX.app: rejected
$ echo $?
3
```

We also copied the bundle to `/tmp`, tagged it with a synthetic
`com.apple.quarantine` xattr (`0081;00000000;Safari;`, the format Safari
writes for a downloaded file) to simulate a real download, and re-ran the
assessment:

```console
$ xattr -w com.apple.quarantine "0081;00000000;Safari;" /tmp/FlashTeX-quarantine-test.app
$ spctl -a -vv /tmp/FlashTeX-quarantine-test.app
/tmp/FlashTeX-quarantine-test.app: rejected
$ echo $?
3
```

Both the quarantined and non-quarantined copies are rejected identically —
confirming Gatekeeper's `-a` (application launch) assessment rejects an
ad-hoc-only signature regardless of quarantine state; quarantine only decides
*whether Gatekeeper re-checks at all* on a given launch, not the outcome once
it does.

`xrun notarytool` and `xcrun stapler` are both present in this Xcode command
line tools install (`notarytool --version` → `1.1.0 (39)`), so the *tooling*
for the steps below is available; we do not have credentials to actually run
them (no Apple Developer Program membership / App Store Connect API key
configured on this machine — this is the account-level, not tooling-level,
gap).

## Exact steps for Developer ID signing + notarization

None of this can run today (no Developer ID certificate, no notarization
credentials). Recorded here so it is a checklist, not a research task, once an
Apple Developer Program membership is available.

1. **Get a Developer ID Application certificate** (requires a paid Apple
   Developer Program membership, $99/yr, enrolled to a specific Apple ID/Team).
   Generate it in Xcode (Settings → Accounts → Manage Certificates → "+" →
   Developer ID Application) or via `certutil`/the developer portal, and make
   sure the private key lands in this machine's login keychain.

2. **Write an entitlements file.** FlashTeXMac spawns subprocesses
   (`flashtex-compiler`, `flashtex-pdf`, and arbitrary worker executables via
   `File > Attach Worker Executable…`) and will need local network access for
   the nearby-capture listener (`NSBonjourServices` / `NSLocalNetworkUsageDescription`,
   already added to `Info.plist` by `make-app.sh` in this revision — see below).
   A Developer-ID-signed build with the *hardened runtime* enabled (required
   for notarization) needs explicit entitlements for both:

   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <!-- Hardened runtime still allows launching a subprocess by default;
            no separate entitlement is required to *launch* a helper binary
            you ship inside Contents/MacOS. These matter once the app is
            sandboxed (a later step, not part of notarization itself): -->
       <key>com.apple.security.app-sandbox</key>
       <false/>
       <!-- Needed once/if the app adopts the App Sandbox; notarization alone
            does NOT require sandboxing, only the hardened runtime below. -->
       <key>com.apple.security.network.client</key>
       <true/>
       <key>com.apple.security.network.server</key>
       <true/>
   </dict>
   </plist>
   ```

   Concretely for notarization (sandboxing is a separate, larger decision we
   are not making here): sign with `--options runtime` (hardened runtime) and
   no additional entitlements are strictly required to spawn a bundled helper
   or open network sockets *outside* the App Sandbox — hardened runtime only
   restricts things like unsigned/ad-hoc-signed dyld injection, JIT, and
   debugging by default, none of which this app does. If/when the app adopts
   the App Sandbox for Mac App Store distribution, the entitlements above
   (`network.client`/`network.server`, plus
   `com.apple.security.temporary-exception.mach-lookup.global-name` or an
   XPC service if the compiler/pdf helpers must run as sandboxed children)
   would be required in addition.

3. **Sign with the hardened runtime**, replacing the ad-hoc step:

   ```sh
   codesign --force --deep --options runtime \
     --entitlements apps/mac/FlashTeX.entitlements \
     --sign "Developer ID Application: <Name> (<TEAMID>)" \
     apps/mac/build/FlashTeX.app
   ```

   `--deep` re-signs the bundled `flashtex-compiler`/`flashtex-pdf` helpers
   too; each must itself be a valid Mach-O the tool can sign (both are plain
   Rust binaries, which sign fine). Verify with:

   ```sh
   codesign --verify --deep --strict --verbose=2 apps/mac/build/FlashTeX.app
   spctl -a -vv apps/mac/build/FlashTeX.app   # still "rejected" until notarized
   ```

4. **Zip the bundle for submission** (notarization takes a zip/dmg/pkg, not a
   raw `.app` directory):

   ```sh
   ditto -c -k --keepParent apps/mac/build/FlashTeX.app apps/mac/build/FlashTeX.zip
   ```

5. **Submit for notarization** with an app-specific password or API key
   stored via `xcrun notarytool store-credentials`:

   ```sh
   xcrun notarytool submit apps/mac/build/FlashTeX.zip \
     --keychain-profile "flashtex-notary" --wait
   ```

   `--wait` blocks until Apple's automated scan finishes (usually minutes) and
   prints `status: Accepted` or `status: Invalid` with a log URL
   (`xcrun notarytool log <submission-id> --keychain-profile flashtex-notary`
   for the reject reasons).

6. **Staple the ticket** to the `.app` (so Gatekeeper can verify offline,
   without contacting Apple, on the end user's machine) and re-verify:

   ```sh
   xcrun stapler staple apps/mac/build/FlashTeX.app
   xcrun stapler validate apps/mac/build/FlashTeX.app
   spctl -a -vv apps/mac/build/FlashTeX.app   # now "accepted", source=Notarized Developer ID
   ```

7. If distributing via `--dmg` (see `make-app.sh --dmg`), staple the DMG too
   (`stapler staple FlashTeX.dmg`) so Gatekeeper doesn't need network access
   when the DMG itself is what got quarantined.

None of steps 1–7 can be executed in this environment; they require an Apple
Developer Program membership and its credentials, which are out of scope for
this worker (`mac-packaging`) to acquire or spend on.

## Info.plist additions in this revision

`make-app.sh` now also writes, ahead of the nearby-capture listener landing:

```xml
<key>NSLocalNetworkUsageDescription</key>
<string>FlashTeX uses the local network to discover and receive captures from nearby devices.</string>
<key>NSBonjourServices</key>
<array>
    <string>_flashtex._tcp</string>
</array>
```

`NSLocalNetworkUsageDescription` is required before macOS will show the local
network permission prompt at all (its absence causes a silent denial, not a
prompt); `NSBonjourServices` must list every Bonjour service type the app
resolves or advertises via `NWBrowser`/`NetService` (`_flashtex._tcp` is a
placeholder pending the actual capture-bridge contract — the capture/AI
owner should confirm the real service type before this ships). Both are
inert until the app actually touches the network — declaring them now does
not request any permission by itself, and adding them retroactively later
would require nothing beyond this same edit.

## Install and disk image

`make-app.sh --install` copies the freshly built, ad-hoc-signed bundle to
`~/Applications/FlashTeX.app`. The replace is atomic and recoverable:

1. The new bundle is copied to a hidden staging name
   (`~/Applications/.FlashTeX.app.staging.<pid>`) first.
2. Any existing `~/Applications/FlashTeX.app` is moved (a same-volume
   `mv`/rename, not a copy) to `~/Applications/FlashTeX-previous.app`.
3. The staged bundle is renamed into place as `FlashTeX.app` — a single
   `mv` within the same directory, so `FlashTeX.app` is never observably a
   half-written directory.
4. Any running `FlashTeX` is killed, the installed copy is launched with
   `open`, and the script polls `pgrep -x FlashTeX` for up to 10s.
   - **Launch confirmed:** the app is quit again (AppleScript `quit`,
     falling back to `pkill`) and `FlashTeX-previous.app` is deleted — it
     only exists to make the previous step reversible until this point.
   - **Launch not confirmed:** the new `FlashTeX.app` is removed and
     `FlashTeX-previous.app` is renamed back to `FlashTeX.app`, restoring
     the last working install; the script exits non-zero.

`make-app.sh --dmg` stages a copy of the bundle plus a symlink to
`/Applications` in a temp directory and runs
`hdiutil create -volname FlashTeX -srcfolder <staging> -ov -format UDZO`
to produce `apps/mac/build/FlashTeX.dmg` (a standard compressed,
read-only, drag-to-install image).

Verified on this machine (`jay3332`'s Mac, `~/Applications` already has
unrelated real apps installed — Android Studio, CLion, PyCharm, etc. — none
of which this script touches):

```console
$ apps/mac/scripts/make-app.sh --compiler … --pdf … --dmg --install
…
==> Building DMG
    DMG at /…/apps/mac/build/FlashTeX.dmg
==> Installing to ~/Applications
    installed /Users/jay3332/Applications/FlashTeX.app
==> Verifying the installed app launches
    launch OK (FlashTeX process is running)
==> Done: /…/apps/mac/build/FlashTeX.app
```

Second run, with a previous install already present, to exercise the
retention/rollback path:

```console
$ apps/mac/scripts/make-app.sh --install
==> Installing to ~/Applications
    kept previous install at /Users/jay3332/Applications/FlashTeX-previous.app pending launch verification
    installed /Users/jay3332/Applications/FlashTeX.app
==> Verifying the installed app launches
    launch OK (FlashTeX process is running)
    removed /Users/jay3332/Applications/FlashTeX-previous.app (new install verified)
```

Both runs: `plutil -lint` on the installed `Info.plist` → `OK`; `codesign -dv`
→ ad-hoc signature, `tech.jay3332.flashtex.mac`; `hdiutil imageinfo` on the
produced DMG → `Format: UDZO`, `Compressed: true`. After each run,
`osascript -e 'tell application "System Events" to get name of every process
whose name is "FlashTeX"'` showed the process while running and `pgrep -x
FlashTeX` showed nothing after quitting — no stray `FlashTeX` process was
left behind by either run.

# mac-packaging handoff — signing groundwork, reproducible bundle, pid-only launch-check, DMG

- Updated UTC: 2026-09-12T11:12Z
- Agent / parent / machine alias: `mac-packaging` (Claude Code subagent) /
  parent `mac-claude-a` / `mac-m1max-a`
- Task / acceptance gate / owned paths: parent-dispatched packaging lane
  (issue #2 dispatch thread): (1) `make-app.sh --sign <identity>` /
  `--notarize <keychain-profile>` with hardened runtime, an entitlements file,
  `codesign --verify --deep --strict`, `spctl --assess`, `notarytool submit
  --wait` + `stapler`, clean non-secret failure when identity/profile are
  absent, ad-hoc path still working; (2) reproducible-build check;
  (3) `launch-check.sh` honours `FLASHTEX_NO_ACTIVATE` itself and never kills
  by process name; (4) `--dmg` output mounted and launched by launch-check.
  Coordinator add-on: `--assistant` / `--explain` helper entries. Owned paths:
  `apps/mac/scripts/**`, `apps/mac/Resources/**`, this file,
  `coordination/agents/mac-packaging.json`. README packaging section and
  `apps/mac/docs/packaging.md` are parent-owned: diffs supplied in the final
  report (`README-packaging.diff` in the parent's scratchpad), not applied.
- Branch / code revision / main integrated through:
  `agent/mac-packaging/signing` from `origin/agent/mac-claude-a/mac-shell`
  6f4ee94; merged mac-shell 7c11d65 and origin/main d1bd76b at the final
  checkpoint (both clean; neither touches `apps/mac/scripts`,
  `apps/mac/Resources` or this lane's coordination files) / see `git log` /
  origin/main d1bd76b.
- State: ready for integration (into `agent/mac-claude-a/mac-shell` by the parent).
- Ready behavior and evidence:
  - **`make-app.sh`** (`apps/mac/scripts/make-app.sh`): table-driven helper
    bundling — compiler, pdf, bridge, edit-ledger, render, pdf-exact,
    preview-controller, project-files, assistant-context, explain — each
    optional (skipped with a note when not built or passed), defaults under
    `--helper-root <repo>` (new; defaults to this repo, set to the main
    checkout from a worktree). `Contents/Info.plist` is instantiated from
    `apps/mac/Resources/Info.plist.template` (`@VERSION@`, `@GIT_SHA@`; content
    identical to the former heredoc). **Helpers are signed before
    `components.json` is written**, so every helper `sha256` is the shipped
    bytes; previously `codesign --deep` re-signed after hashing and no helper
    hash matched (baseline on 6f4ee94: bridge recorded `c90bab…`, shipped
    `340641…`). The app entry stays the pre-signature hash (its signature seals
    `components.json`), and a `build` block records `config`, `signing`
    (`adhoc` | `hardened-adhoc` | `developer-id`), `identity`,
    `hardened_runtime`, `notarized`, `timestamp` (honours `SOURCE_DATE_EPOCH`).
    Helper identifiers are `tech.jay3332.flashtex.mac.<name>` (were
    `<name>-<hash>`).
  - **Signing**: no `--sign` → ad-hoc (helpers individually, then the app
    without `--deep`; `codesign --verify --deep --strict` passes; `spctl`
    rejects as before). `--sign "<Developer ID Application …>"` → pre-flight
    checks identity (by name in `security find-identity -v -p codesigning`),
    entitlements file exists and lints, then per helper `codesign --force
    --options runtime --timestamp --identifier … --sign ID`, app `codesign
    --force --options runtime --timestamp --entitlements
    Resources/FlashTeX.entitlements --identifier tech.jay3332.flashtex.mac
    --sign ID`, `codesign --verify --deep --strict --verbose=2`, prints the
    entitlements as signed, `spctl --assess --type execute --verbose=2`
    (warning before notarization, hard failure after stapling).
    `--notarize <profile>` requires `--sign`, checks `xcrun notarytool` /
    `stapler` exist and that the keychain item
    (`security find-generic-password -s com.apple.gke.notary.tool -a
    <profile>`, output discarded) exists, then `ditto -c -k --keepParent` →
    `xcrun notarytool submit --wait --keychain-profile` (log kept at
    `build/notarytool-app.log`, `status: Accepted` required, otherwise the
    `notarytool log <id>` command is printed) → `stapler staple` +
    `validate` → `spctl` must accept. With `--dmg`: DMG built after stapling
    (`ditto` copy keeps the ticket), signed, notarized and stapled too.
    `--sign -` runs the identical hardened-runtime/entitlements path with the
    ad-hoc identity (`--timestamp=none`) so it can be exercised here.
  - **Entitlements** (`apps/mac/Resources/FlashTeX.entitlements`): an empty
    `<dict/>` with the reason each candidate key is unnecessary written next
    to it (no App Sandbox → `network.server/client` inert; NWListener TLS-PSK
    Nearby needs only the Info.plist `NSLocalNetworkUsageDescription` /
    `NSBonjourServices` already written; helpers are statically linked plain
    executables spawned with `Process`, so no `disable-library-validation`;
    no JIT, no DYLD vars, no Apple Events sent, no camera/mic/etc.).
    Proven sufficient: the `--sign -` bundle (flags `0x10002(adhoc,runtime)`
    on app and every helper) passes launch-check: window, compiler attach,
    `revision 1: ok`, bridge attach, both children killed and logged, quit.
  - **Failure paths (this machine has 0 codesigning identities, no notary
    profile)** — all exit 1 before `swift build`, previous bundle untouched:
    `--sign "Developer ID Application: FlashTeX (ABCDE12345)"` → `no valid
    codesigning identity matching "…" in the keychain (0 valid identities
    found). Install a "Developer ID Application" certificate …`;
    `--notarize p` alone → `--notarize requires --sign …`; `--sign - --notarize
    p` → `… (ad-hoc '-' cannot be notarized)`; missing/invalid entitlements
    file → named. Nothing prints keychain contents.
  - **`repro-check.sh`** (new): runs `make-app.sh` twice, Gate 1 helper copies
    byte-identical as shipped, Gate 2 `components.json` identical apart from
    `build.timestamp` (includes the unsigned app hash), lists every other
    differing file with the reason, asserts whole-bundle identity when
    `SOURCE_DATE_EPOCH` is set. Measured (6 helpers from the main checkout at
    f4c8aea + assistant-context built in this worktree): all 7 helper copies
    identical; components.json identical modulo timestamp; unpinned runs
    differ only in `components.json` (timestamp) → `_CodeSignature/CodeResources`
    (seals it) → `MacOS/FlashTeX` (signature seals CodeResources); with
    `SOURCE_DATE_EPOCH` every file of the bundle is byte-identical (ad-hoc
    signing is deterministic).
  - **`launch-check.sh`**: `open -n --env FLASHTEX_NO_ACTIVATE=1` (env
    override `FLASHTEX_NO_ACTIVATE=0`), instance = new FlashTeX pid whose
    `ps -o comm=` path is inside the bundle under test; quit via a compiled
    `NSRunningApplication(processIdentifier:).terminate()` probe (works
    without Automation permission), SIGTERM to that pid as fallback; no
    `pkill`/`osascript` by name anywhere (also removed from `make-app.sh
    --install`, which now launches with `open -n --env FLASHTEX_NO_ACTIVATE=1`
    and terminates only that pid; `--install-dir` added). Verified with a
    decoy FlashTeX (copy of the bundle) running: reported "left untouched",
    decoy alive afterwards. `--dmg <path>`: `hdiutil attach -nobrowse
    -readonly -noautoopen -mountpoint`, all checks against
    `<mount>/FlashTeX.app` (executable path confirmed under the mount), clean
    `hdiutil detach` (no `-force` needed), forced only in the EXIT trap.
    Evidence lines (`FlashTeX running, pid=`, `<helper> attached, pid=`) are
    unchanged for `tools/native-validation/mac-live/lib/launch_summary.py`;
    the mac-live `open` shim still works (`open -n --env …` accepted).
  - **`packaging-selftest.sh`** (new, the lane's test file): fast part
    asserts the pre-build failure paths, plist lint, empty entitlements dict,
    `--help`, `/bin/bash 3.2` parse; `--full` chains repro-check (ad-hoc, then
    `--sign - --dmg` pinned), hardened flag, launch-check app and `--dmg`.
- Incomplete behavior / blockers / needs from others:
  - The Developer ID and notarization branches cannot be executed here (no
    certificate, no App Store Connect credentials; acquiring them is a paid
    Apple Developer Program decision outside this lane). What is proven: the
    exact codesign invocations with hardened runtime + entitlements
    (`--sign -`), verification, DMG signing, and every absence path.
    `--timestamp` and `notarytool` need network access to Apple when they run.
  - `flashtex-explain` (crates/diagnostic-explanations) does not exist on
    main: the skip path is exercised (`no flashtex-explain found …; skipping`,
    `"explain": {"bundled": false …}`), the bundling path is not.
  - Bundling `flashtex-project-files` and `flashtex-preview-controller`
    (built in the main checkout) is new: the app auto-uses a bundled
    `flashtex-project-files` (`BridgeClient.locateHelper`); the controller is
    only used with `FLASHTEX_PREVIEW_CONTROLLER`. launch-check passed with both
    bundled; the parent may want mac-validation to re-run its suite on a
    bundle containing them.
- Interface changes / consumer actions:
  - `components.json`: new keys `preview_controller`, `project_files`,
    `assistant`, `explain`, `build`; helper `sha256` now equals the shipped
    file hash (mac-live's signature-masked comparison keeps working, and can
    now compare exact hashes for helpers). Helper codesign identifiers changed
    to `tech.jay3332.flashtex.mac.<name>`.
  - `launch-check.sh` no longer kills other FlashTeX instances:
    `tools/native-validation/mac-live/run.sh` can drop its "REFUSED: FlashTeX
    already running (launch-check.sh would pkill it)" guard, and no longer
    needs the shim to inject `FLASHTEX_NO_ACTIVATE=1` (harmless if kept).
  - README packaging section + `apps/mac/docs/packaging.md` step 2/3 text
    (entitlements are now the empty documented file, `--deep` no longer used
    for distribution signing): parent-owned, diff supplied.
- Reviewed peer revisions / resulting adaptations: `origin/main` d1bd76b (eed1847 + one coordination dispatch commit) —
  no `apps/mac` changes, merged, nothing to adapt; `origin/agent/mac-claude-a/mac-shell`
  7c11d65 — new Swift (delimiter pairs, hybrid preview) but no script/Resources
  changes, merged, packaging re-verified on the merged tip (`--sign - --dmg`
  build 69 s, launch-check `--dmg` 0 FAIL, selftest 18/18); `origin/agent/mac-claude-a/packaging`
  d7fb4fa (older packaging lane: rev-5 gaps doc, superseded make-app.sh) —
  its `components.json` schema and evidence format kept compatible;
  `tools/native-validation/mac-live` (run.sh, open shim, launch_summary.py)
  read to keep the evidence lines and `open` call shape compatible.
- Validation commands / results / artifact paths (all from this worktree,
  helpers from `/Users/jay3332/Projects/flashtex/crates/*/target/release`):
  - `apps/mac/scripts/packaging-selftest.sh` → 18 ok, 0 failed (fast).
  - `apps/mac/scripts/packaging-selftest.sh --full -- --helper-root
    /Users/jay3332/Projects/flashtex --assistant <worktree
    crates/assistant-context/target/release/flashtex-assistant-context>` →
    23 ok, 0 failed: repro-check ad-hoc PASS; repro-check `--sign - --dmg`
    with `SOURCE_DATE_EPOCH` PASS + whole bundle byte-identical; app flags
    `0x10002(adhoc,runtime)`; launch-check app 17 notes 0 FAIL; launch-check
    `--dmg` 20 notes 0 FAIL, image detached cleanly.
  - `make-app.sh --helper-root … --install --install-dir <scratch>` twice →
    first install, then replace with `FlashTeX-previous.app` kept and removed
    after the pid-verified launch; no FlashTeX left running.
  - No Swift sources changed; `swift build -c release` runs inside make-app.sh
    (Build complete). `swift test` not re-run (no Swift change).
- Exact deadline UTC / remaining time / integration reserve: no automatic
  deadline (coordination/PROJECT.md); continuous improvement authorization.
- ETA remaining, optimistic / likely / pessimistic / confidence: 0 / 0 / 0 for
  this lane; Developer ID execution blocked on credentials, not on code.
- Resource pool / allocation ID / maximum: shared Claude Max 20x quota via
  parent `mac-claude-a`; no purchases, no paid network calls (no notarization
  submission was made).
- Confirmed spend / estimated usage / in-flight reservation / remaining: unknown
  (shared account; no telemetry available to this subagent).
- Billing evidence / freshness / unknowns: none beyond the parent's.
- Child tasks and their deducted allocations: none.
- Dirty files / unpushed work / running jobs: none after the final commit; no
  FlashTeX process left running; scratch DMG mounts detached.
- Decisions / failed approaches / linked findings:
  - Sign helpers first and the app without `--deep` (Apple's inside-out
    recommendation) — required for shipped-hash consistency and for
    per-helper identifiers; `--deep` was the cause of the baseline hash
    mismatch.
  - Empty entitlements rather than the earlier draft's sandbox keys: those keys
    only act under the App Sandbox, which the app does not adopt.
  - `--sign -` added so the distribution code path is testable without a
    certificate; not distributable.
  - Timestamp kept in `components.json` (useful evidence) but made
    `SOURCE_DATE_EPOCH`-pinnable instead of removed; the seal chain it
    perturbs is documented by repro-check.
  - `NSRunningApplication.terminate()` chosen over `osascript … quit` (which
    targets by name and could quit the user's own instance).
- Exact next action or command: parent merges `agent/mac-packaging/signing`
  into `agent/mac-claude-a/mac-shell`, applies the README/packaging.md diffs,
  optionally asks mac-validation to re-run `tools/native-validation/mac-live`
  on a bundle with the new helpers.
- Resume reading list: this file; `apps/mac/scripts/make-app.sh` header;
  `apps/mac/Resources/FlashTeX.entitlements` comments;
  `apps/mac/docs/packaging.md` (parent-owned, partly superseded).

# FlashTeX native validation (FT-010)

Owner: `mac-validation` (parent `mac-claude-a`, machine `mac-m1max-a`).
Owned path: `tools/native-validation/` only. This suite never edits `apps/mac`
(owner mac-claude-a) or `crates/compiler` (owner claude/compiler-foundation);
failures in those are written up in `reports/` for their owners.

## Scripts

| File | What it checks | Gating? |
|---|---|---|
| `run_all.sh` | Creates detached worktrees for the Mac app ref and the compiler ref, runs `cargo build --release`, `cargo test --release`, `swift build`, `swift test` (with `FLASHTEX_COMPILER` set so `RealCompilerTests` actually runs), `xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build`, then `check_protocol.py` and `check_ui_capabilities.sh`; writes `reports/report-<UTC>.md` with exact commands, exit codes, durations and pass/fail; removes the worktrees. | yes (exit 1 on any gating failure) |
| `check_protocol.py` | Runtime-v1 checks against the real compiler binary over stdin/stdout JSON Lines: fixture request/response comparison (texts + ranges; positions INFO only), fixture self-consistency, Unicode byte-span round trip (`naïve café — 😀 end`), revision echo for out-of-order revisions, source editing changes output and is deterministic, diagnostics for unsupported input, `error` envelopes for `protocol_version: 2`, unknown `type`, malformed JSON, worker survives errors, and edit-to-result latency over 20 requests (min/median/max). | yes |
| `check_ui_capabilities.sh` | Probes (never fails): `screencapture -x`, `osascript` System Events process listing, `osascript` UI scripting (Accessibility), `xcrun simctl list devices available`, whether the built `FlashTeXMac` launches and survives 3 s, whether it is frontmost / has enumerable windows, and the launchd session type. Records exact stderr for every failure. | no |
| `check_pdf_export.py` | Stdlib inspector for a PDF exported by the app (`File > Export PDF…`): page count, MediaBoxes, and the page background fill colour before the first rectangle fill; exit 1 when a page background is not white. Validated against CoreGraphics PDFs painted `gray 1.0` (`1.0 sc`, PASS) and `gray 0.16` (`0.16 sc`, FAIL); not yet against an app-exported file (save panel needs a human). | manual input |
| `expectations.md` | Human checklist for what automation cannot verify here: visual click-to-source, Unicode selection, stale-preview banner, dark preview vs export, capture review. | manual |

## Running

```sh
# defaults: --repo = this checkout, mac-shell and compiler-foundation refs
tools/native-validation/run_all.sh
# explicit refs / scratch dir / keep the worktrees for inspection
tools/native-validation/run_all.sh --repo . \
  --mac-ref origin/agent/mac-claude-a/mac-shell \
  --compiler-ref origin/agent/claude/compiler-foundation \
  --scratch "${TMPDIR:-/tmp}/flashtex-validation" --keep
# protocol checks alone against any built binary
python3 tools/native-validation/check_protocol.py \
  --compiler crates/compiler/target/release/flashtex-compiler --repo .
# capability probes alone
tools/native-validation/check_ui_capabilities.sh --mac-dir apps/mac --repo .
# inspect a PDF a human exported with File > Export PDF…
python3 tools/native-validation/check_pdf_export.py ~/Desktop/demo-r3.pdf --expect-pages 1
```

Requirements: Xcode command line tools with `swift`/`xcodebuild`, `cargo`,
`python3` (stdlib only), `git` with the refs fetched (`git fetch origin`).
Full build/test logs are written under the scratch run directory and are
**not** committed; only the Markdown report under `reports/` is.

The suite exits non-zero if any gating step fails, including the fixture
self-consistency check below, so a "FAIL" overall does not by itself mean the
app or compiler is broken; read the per-check rows.

## Permissions and automation capabilities actually observed

Machine `mac-m1max-a` (MacBook Pro, macOS 26.3.1, Xcode 26.3, Swift 6.2.4,
cargo 1.99.0-nightly, Python 3.12.0). Agent session launched by Claude Code
under a terminal host; `launchctl managername` reports `Background`.

| Capability | State observed | Exact evidence |
|---|---|---|
| Xcode / SwiftPM build | Works | `swift build` 11-12 s, `xcodebuild ... build` 11 s `** BUILD SUCCEEDED **` (see report) |
| Swift tests incl. real compiler | Works | `Executed 34 tests, with 0 failures`; `RealCompilerTests ... passed (1.131 seconds)` when `FLASHTEX_COMPILER` is set |
| Rust build/tests | Works | `cargo test --release`: 3 unit + 9 acceptance tests passed |
| `screencapture -x` | **Intermittent**: at 2026-09-12 04:2x UTC two attempts (sandboxed and unsandboxed shell) failed with `could not create image from display` (exit 1, no file); from 04:28 UTC onward every attempt succeeded with a 3024x1964 px PNG showing real window contents, so Screen Recording is granted to the terminal host. The earlier failure is consistent with the display being asleep/locked; not proven. Probe images are the user's whole screen and are deleted immediately; none are committed. | see report rows |
| `osascript` System Events process list / `frontmost of process` | Works without Accessibility | `138 process names returned`; `frontmost of process "FlashTeXMac"` returned `false` |
| `osascript` UI scripting (windows, elements) | **Blocked** (Accessibility not granted to the host) | `65:69: execution error: System Events got an error: osascript is not allowed assistive access. (-1728)`; window count: `... (-25211)` |
| Apple Events to the app (`tell application "FlashTeXMac" to activate`) | **Not possible**: the app is a bare SwiftPM executable, not a registered bundle | `execution error: Can’t get application "FlashTeXMac". (-1728)` |
| App launch from the agent | Process launches and survives (`pgrep` finds `FlashTeXMac`), but **no window is on screen**: `frontmost` is false and a full-screen capture after `set frontmost` showed only an empty desktop. Visual click-to-source, dark preview, and banner checks therefore remain human-only; see `expectations.md`. `set frontmost` switched the user's active Space (restored afterwards) and is deliberately not part of the automated probe. | run 04:30 UTC |
| iOS simulators | Available | `xcrun simctl list devices available`: 11 devices (iPhone 17 Pro, iPhone 17 Pro Max, iPhone Air, …), all `Shutdown`; not exercised by this suite |
| Physical iOS device | Not verified | `xcrun xctrace list devices` lists 14 non-simulator entries including this Mac; no device pairing test was run |

To unblock the human-only checks: grant Accessibility (System Settings >
Privacy & Security > Accessibility) to the terminal host running the agent, or
have a person run `expectations.md` in the GUI session.

## Findings from the committed run (2026-09-12T04:31:42Z; `reports/report-20260912T043142Z.md`)

Refs: Mac app `origin/agent/mac-claude-a/mac-shell` = `1c3ff133` (app code at
`fd2a26e`), compiler `origin/agent/claude/compiler-foundation` = `9f1033ba`.
An earlier run at 04:28:02Z against `fb22075e` / `e7127fb9` gave the same
pass/fail pattern (28 Swift tests then; 34 now).

| Time (UTC) | Check | Result |
|---|---|---|
| 04:31:42 | worktrees created | PASS |
| 04:31:43 | `cargo build --release` | PASS (1.0 s) |
| 04:31:44 | `cargo test --release` | PASS |
| 04:31:55 | `swift build` | PASS (10.9 s) |
| 04:32:06 | `swift test` with `FLASHTEX_COMPILER` | PASS (34 tests, 0 failures; `RealCompilerTests` ran and passed, 1.13 s) |
| 04:32:16 | `xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build` | PASS (10.6 s, `** BUILD SUCCEEDED **`) |
| 04:32:16 | `check_protocol.py` | 20 PASS, **1 FAIL** (fixture, below), 6 INFO |
| 04:32:16 | edit-to-result latency, 20 requests, 547-byte document | min 0.26 ms, median 0.28 ms, max 0.41 ms (pipe round trip, no UI) |
| 04:32:23 | `check_ui_capabilities.sh` | screencapture OK; Accessibility BLOCKED (-1728 / -25211); app launches, not frontmost, no window enumerable |
| by inspection | dark preview vs export | **discrepancy** in app code, below; not executed (save panel) |

### FAIL: `protocol/fixtures/compile-result.json` range does not cover its text (owner: Commander, FT-001)

- Repro: `python3 tools/native-validation/check_protocol.py --compiler <bin> --repo <repo>`,
  or compare by hand: request text is `"Hello FlashTeX.\n"`; the fixture result
  item has `text: "Hello FlashTeX."` (15 bytes) with `source: {start_byte: 0, end_byte: 14}`.
- Observed: bytes `[0,14)` slice to `"Hello FlashTeX"` (no final `.`).
- Expected per contract: zero-based, end-exclusive UTF-8 offsets that slice to
  the item text, i.e. `end_byte: 15`.
- Impact: the Mac fixture preview's click-to-source selects one byte short on
  the fixture; the app's tests consume the fixture as given, so they do not
  catch it. The real compiler is correct (`Hello` `[0,5)`, `FlashTeX.` `[6,15)`).
- Not an app or compiler bug; reported as FAIL because the fixture is the
  contract's example and the app renders it.

### Discrepancy for the app owner (mac-claude-a): PDF export follows the dark toggle

- Where: `apps/mac/Sources/FlashTeXMac/PDFExport.swift` at mac-shell `fd2a26e`,
  `ShellModel.exportPDF()` calls `PDFExport.render(result, dark: darkPreview)`;
  `render` paints the page `CGColor(gray: 0.16)` with white text when `dark` is true.
- FT-010 acceptance expectation: dark preview changes viewing only; export
  stays a white page with black text.
- Repro (human, GUI): toggle dark preview on, `File > Export PDF…` (⌘⇧E), then
  `python3 tools/native-validation/check_pdf_export.py <file>`; expected
  `1.0 sc` / exit 0, predicted from the code `0.16 sc` / exit 1.
- Status: found by reading the source; not executed because the save panel
  cannot be driven from the agent session (Accessibility blocked). Either the
  expectation or the code needs an owner decision; this suite does not change
  `apps/mac`.

### INFO (not failures)

- The compiler emits one text item per word; the fixture has one item per line.
  Joined text and covered span agree. Consumers must not assume one item per line.
- Parent-traversal document paths (`../evil.tex`) are rejected with a
  `compile_result` of `status: failed` plus an `error` diagnostic, not an `error`
  envelope. The contract says "reject" without naming the envelope; recorded only.
- Worker is sequential: replies arrive in request order (`rev-2` then `rev-1`),
  each echoing its own revision. The UI stale guard is covered by the app's
  `swift test` (WorkerClientTests / ShellModelTests / SourceMappingTests), not here.

No compiler (`crates/compiler`) failure was found. No app (`apps/mac`) test or
build failure was found; the export behavior above is the only app discrepancy.

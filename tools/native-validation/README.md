# FlashTeX native validation (FT-010)

Owner: `mac-validation` (parent `mac-claude-a`, machine `mac-m1max-a`).
Owned path: `tools/native-validation/` only. This suite never edits `apps/mac`
(owner mac-claude-a) or `crates/compiler` (owner claude/compiler-foundation);
failures in those are written up in `reports/` for their owners.

## Scripts

| File | What it checks | Gating? |
|---|---|---|
| `run_all.sh` | Creates detached worktrees for the Mac app ref, the compiler ref, and (non-gating) the PDF-writer and bridge refs, runs `cargo build --release`, `cargo test --release`, `swift build`, `swift test` (with `FLASHTEX_COMPILER`, `FLASHTEX_PDF` and `FLASHTEX_BRIDGE` set so `RealCompilerTests`, `RustPDFExportTests` and `RealBridgeTests` actually run), `xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build`, then `check_protocol.py` and `check_ui_capabilities.sh`; writes `reports/report-<UTC>.md` with exact commands, exit codes, durations and pass/fail; removes the worktrees. | yes (exit 1 on any gating failure) |
| `check_protocol.py` | Runtime-v1 checks against the real compiler binary over stdin/stdout JSON Lines: fixture request/response comparison (texts + ranges; positions INFO only), fixture self-consistency, Unicode byte-span round trip (`naïve café — 😀 end`), revision echo for out-of-order revisions, source editing changes output and is deterministic, diagnostics for unsupported input, `error` envelopes for `protocol_version: 2`, unknown `type`, malformed JSON, worker survives errors, and edit-to-result latency over 20 requests (min/median/max). | yes |
| `check_ui_capabilities.sh` | Probes (never fails): `screencapture -x`, `osascript` System Events process listing, `osascript` UI scripting (Accessibility), `xcrun simctl list devices available`, whether the built `FlashTeXMac` launches and survives 3 s, whether it is frontmost / has enumerable windows, and the launchd session type. Records exact stderr for every failure. | no |
| `check_pdf_export.py` | Stdlib inspector for a PDF exported by the app (`File > Export PDF…`): page count, MediaBoxes, and the page background fill colour before the first rectangle fill; exit 1 when a page background is not white. Validated against CoreGraphics PDFs painted `gray 1.0` (`1.0 sc`, PASS) and `gray 0.16` (`0.16 sc`, FAIL); not yet against an app-exported file (save panel needs a human). | manual input |
| `oracle_compare.sh` / `oracle_compare.py` / `oracle_extract.swift` | Reference-oracle comparison (issue #10): pdflatex is run **only as a measuring stick, never by the product**. For each `oracle-samples/*.tex` and each oracle variant: pdflatex (`-interaction=batchmode`, temp dir) -> oracle PDF; FlashTeX compiler (preamble stripped) -> `compile_result` -> `flashtex-pdf --verify` -> our PDF; PDFKit word boxes for both (`oracle_extract.swift`, no third-party packages); page count / MediaBox equality, word-sequence equality after normalisation, per-word x/y deltas (mean/max, 10 largest), line-start agreement. Writes `reports/oracle-<UTC>.md` and a compact `.json` with every per-word delta. Exit 1 only when a tool fails; layout disagreement is a finding, not a failure. | findings |
| `e2e_native.sh` + `e2e_latency.py`, `e2e_bridge.py`, `e2e_nonregression.py`, `window_probe.swift` | Native end-to-end (issue #2): builds Mac shell + its `crates/compiler`, PDF writer and bridge from refs; (1) launches `FlashTeXMac` with `FLASHTEX_AUTOATTACH=1 FLASHTEX_SEED_FILE=oracle-samples/wrap-sample.tex` and observes process, `flashtex-compiler` child (`pgrep -P`), window (`CGWindowListCopyWindowInfo`, no Accessibility needed) and a screenshot by window id, then runs the CLI-equivalent `flashtex-compiler -> flashtex-pdf --verify -> PDFKit` for the same input; (2) 20x CLI round trip over `Samples/demo.tex` while attached + `swift test --filter RealCompilerTests` REAL-COMPILER LATENCY; (3) SIGKILL compiler child (app must survive 5 s), relaunch re-attaches, SIGKILL app, relaunch shows a window within 5 s; (4) re-runs `run_all.sh` and `oracle_compare.sh --only fixture-hello` and diffs headline numbers against the newest previous reports; (5) bridge receipt path against the real `flashtex-bridge`. Only its own app instance (PID from `$!`) is ever signalled. Writes `reports/e2e-<UTC>.md` + PNGs <= 300 KB. | yes (exit 1 on FAIL; findings and diffs are INFO/FINDING) |
| `probe_devices.sh` | Report-only: paired physical devices (`xcrun devicectl list devices --json-output`: model, identifier, connection/pairing state, OS), available simulators, `xctrace` device list, Wi-Fi interface. Names/hostnames redacted. Writes `reports/devices-<UTC>.md`. | no |
| `latency_repeat.sh` | Repeated compiler-latency non-regression: builds the compiler from a ref, runs `e2e_latency.py` R times (fresh worker each) x N requests over `Samples/demo.tex`, reports per-run min/median/max, spread/SD/CV of the medians, cold first-request times, and compares the median-of-medians with the newest previous `reports/latency-*.md` (2x band). | no (flags only) |
| `rev5_packaged.sh` + `rev5_bridge.py` | FT-003 rev 5 evidence: builds compiler/pdf/bridge/edit-ledger from the app tree, packages `FlashTeX.app` with the owner's `make-app.sh`, records integrated component SHAs (from `Contents/Resources/components.json` when present, else git), then (1) edit->visible over >=10 seed-relaunches via `FLASHTEX_LOG` + RealCompilerTests + warm pipe latency, capture latency and disconnect/retry against the real bridge, export latency (`flashtex-pdf --verify --default-face lm`, `--embed-font auto`, PDFExportTests); (2) crash/restart of all three helper children, cited recovery tests, stale-preview citations, accessibility tests, owner's `launch-check.sh` x3 (only when no foreign FlashTeX process exists); (3) explicit gaps. Writes `reports/rev5-<UTC>.md` + a window screenshot. | yes |
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
# reference-oracle comparison (needs BasicTeX/TeX Live pdflatex at /Library/TeX/texbin/pdflatex)
tools/native-validation/oracle_compare.sh --compiler-ref main=origin/main --compiler-ref de1020c=de1020c --pdf-ref c0f3837
# native end-to-end (own app instance, screenshots, bridge); ~4 min
tools/native-validation/e2e_native.sh
# FT-003 rev 5 packaged evidence (~6 min; launches take keyboard focus, see caution below)
tools/native-validation/rev5_packaged.sh --expect compiler=<sha> --expect pdf=<sha> --expect bridge=<sha> --expect edit-ledger=<sha>
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

### Discrepancy for the app owner (mac-claude-a): PDF export follows the dark toggle — resolved at mac-shell `ec94f89`+ ("Export is always white: dark preview is a viewing mode only")

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

## Reference-oracle comparison (issue #10) — run 2026-09-12T05:09:58Z

Report: `reports/oracle-20260912T050958Z.md` (+ `.json`, 1.3 MB, every per-word delta).
Oracle: `/Library/TeX/texbin/pdflatex` = pdfTeX 3.141592653-2.6-1.40.29 (TeX Live 2026),
BasicTeX system install (`/usr/local/texlive/2026basic`). `kpsewhich times.sty geometry.sty`
found both; **nothing was installed** (BasicTeX's `tlmgr` would need sudo, so a missing
package is reported, not installed). Compilers: `main` = origin/main `342e1e0`
(placeholder glyph metrics) and `de1020c` (real Times AFM metrics); PDF writer
`flashtex-pdf` at `c0f3837`. The product never invokes pdflatex.

Samples (`oracle-samples/`): `fixture-hello.tex` (the runtime-v1 fixture text),
`demo.tex` (apps/mac/Samples/demo.tex from mac-shell `4c6bf47`, body verbatim),
`wrap-sample.tex` (hand-written: `\section`, `\textbf`, `\emph`, naïve/café, a
paragraph that wraps several times). Each file carries the minimal wrapper
`\documentclass{article}` … `\begin{document}` … `\end{document}`.

**What is stripped for our compiler:** everything before `\begin{document}` — for
all three samples that is exactly the line `\documentclass{article}` (plus comment
lines). It has to be stripped: the compiler at both revisions renders
`\documentclass{article}` as the literal word `article` with the diagnostic
"\documentclass is not supported by this compiler version". The oracle variants
replace that preamble with their own.

Oracle variants (the report lists the exact preambles):

- `A-default` — as-is: `\documentclass{article}` + `\pagestyle{empty}` (Computer Modern 10pt,
  LaTeX default margins, paragraph indent, numbered sections). **Not** apples-to-apples.
- `B-times12-1in` — **apples-to-apples for font/size/margins**: `[12pt]{article}`, `\usepackage[T1]{fontenc}`,
  `\usepackage{times}`, `\usepackage[margin=1in]{geometry}`, `\parindent 0pt`, `\setcounter{secnumdepth}{0}`
  (our compiler does not indent paragraphs or number sections), `\pagestyle{empty}`.
- `C-times12-1in-ragged` — B + `\raggedright` + hyphenation off: the closest match to our
  greedy, unjustified, non-hyphenating line breaker. Use C for line-break and x comparisons,
  B for "what LaTeX would actually produce with the same font and margins".

Headline numbers (ours minus oracle; y = glyph-box bottom from page top, i.e. baseline + descent, so a
constant ~0.46 pt offset is the substituted font's descent, not a baseline error):

| Sample | Variant | Compiler | Pages o/u | MediaBox | Word sequence | mean\|dx\| | max\|dx\| | mean\|dy\| | max\|dy\| | Line starts agree |
|---|---|---|---|---|---|---|---|---|---|---|
| fixture-hello | B | de1020c | 1/1 | equal | equal | 0.05 | 0.11 | 0.46 | 0.46 | 1.00 |
| fixture-hello | B | main | 1/1 | equal | equal | 1.90 | 3.81 | 0.46 | 0.46 | 1.00 |
| fixture-hello | A | either | 1/1 | equal | equal | ~73 | 76.7 | 49.7 | 49.7 | 1.00 |
| wrap-sample | C | de1020c | 1/1 | equal | equal (102 words) | 9.0 | 450.7 | 25.1 | 40.7 | 0.98 |
| wrap-sample | B | de1020c | 1/1 | equal | equal | 33.7 | 445.8 | 25.5 | 40.8 | 0.94 |
| wrap-sample | C | main | 1/1 | equal | no (94/102 aligned) | 112.2 | 400.8 | 30.2 | 46.6 | 0.91 |
| demo | C | de1020c | 2/**3** | **no** | 2 diffs, 964/966 aligned | 13.7 | 445.3 | 97.6 | 168.5 | 0.98 |
| demo | B | de1020c | 2/**3** | **no** | 2 diffs | 33.7 | 446.7 | 97.9 | 182.8 | 0.95 |
| demo | C | main | 2/**3** | **no** | 31 diffs, 904/966 aligned | 147.3 | 418.0 | 147.4 | 278.0 | 0.89 |

Findings for the owners (nothing was changed in the compiler or PDF crate):

1. **Compiler (de1020c), line breaking:** within a line ours drifts right by 0.1–0.3 pt per word
   because pdflatex applies kerning pairs and ligatures and our layout uses plain AFM advance widths.
   By the end of a full line ours is ~2 pt further right, so words that pdflatex keeps on the line
   wrap in ours: `demo`/C, `onto` at oracle x 517.26 (right edge 538.6 < 540) versus ours at 519.35
   (right edge 540.7 > 540) -> wrapped. Every large `max|dx|` (~400–450 pt) is one of these
   end-of-line wraps; the mean is small. Line-start agreement 0.98 on both wrapping samples.
2. **Compiler, vertical spacing and pagination:** line pitch matches (oracle 14.45 pt vs ours 14.4 pt), but
   measured on `demo`/C page 1 ours gains +14.75 pt at the first body line after each heading, +5.9 pt at
   every paragraph break (`PARAGRAPH_GAP_PT = 6`; article's `\parskip` is 0 pt) and +14.4 pt for every
   extra wrapped line from finding 1 (the +20 pt jumps are 6 + 14.4), so `demo` runs to 3 pages against
   pdflatex's 2 (page count and MediaBox list differ). `mean|dy|` of ~98 pt on `demo` is this accumulated
   drift, not a per-line error; the heading line itself is only 5.1 pt lower than LaTeX's.
3. **Compiler, punctuation after a group:** `\textbf{clear observation}:` emits `observation` and `:` as
   two items (oracle: one word `observation:`), the only word-sequence difference on `demo` with de1020c.
4. **Compiler (main, placeholder metrics):** words are so mis-measured that consecutive words touch in the
   PDF and PDFKit merges them (`Anaive`, `whenthe`, `Resumeof` …, 31 sequence diffs on `demo`); position
   error `mean|dx|` 112–147 pt. Superseded by de1020c's real metrics; recorded as the baseline.
5. **Oracle-side artefacts, not FlashTeX bugs:** variant A extracts OT1 accents as `na¨ıve`/`caf´e` (the
   comparison normalises accents away and folds fi/ff ligature code points), numbers sections (`1`, `1.1`
   appear as words) and indents paragraphs (first word x = 148.7 pt versus our 72 pt); variant B/C removes
   the last two. `demo` with de1020c under variant A still aligns 952/974 words.

The per-word deltas for every sample/variant/compiler are in the `.json`
(`results[].comparisons[].deltas[]`: word, pages, oracle/ours x and y, dx, dy, line-start flags).

## Native end-to-end (issue #2) — run 2026-09-12T05:31:33Z

Report: `reports/e2e-20260912T053133Z.md` with `e2e-20260912T053133Z-check1-app.png` and
`-check3-relaunch.png` (231 KB each, window-id captures of this script's own instance).
Refs: mac-shell `4213ec9` (its `crates/compiler` = main's `9f1033b`, placeholder metrics — the
crowded words in the screenshot are that compiler, see oracle finding 4), PDF writer `5b5f7b5`,
bridge `b5ca96b`. The mac-shell branch advanced three times while this was built
(`ec94f89` -> `d9b4e0d` -> `62c146f` -> `4213ec9`); the committed report names the exact SHA.

| # | Check | App-observed or CLI-equivalent | Result |
|---|---|---|---|
| 1 | process alive, compiler child (`pgrep -P`), window (CGWindowList), screenshot by window id | app-observed | PASS: pid, child `flashtex-compiler`, window 960x1049 pt, `screencapture -x -o -l <id>` captured |
| 1 | `flashtex-compiler` -> `flashtex-pdf --verify` -> PDFKit page count/words for the seeded file | CLI-equivalent (menus need Accessibility) | PASS: status `recovered` (1 diagnostic: `\documentclass`), 1 page, 99 words, all expected words present |
| 2 | 20x CLI round trip, `Samples/demo.tex` 5909 bytes, app attached | CLI | min 1.30 / median 1.34 / max 3.57 ms |
| 2 | `swift test --filter RealCompilerTests` REAL-COMPILER LATENCY (in-app path, debounce/coalescing) | app code path | n=2 min 0.84 / median 6.68 / max 6.68 ms |
| 3 | SIGKILL compiler child; app alive after 5 s | app-observed | PASS (no auto-respawn; banner reports worker exit per README) |
| 3 | relaunch re-attaches a child; SIGKILL app; relaunch shows window | app-observed | PASS: child re-attached; window 966 ms after relaunch; no orphaned compiler |
| 4 | `run_all.sh` re-run + `oracle_compare.sh --only fixture-hello`, diffed | non-regression | 1 changed field: swift tests 34/0 -> 75/0 (new tests on mac-shell); fixture-hello oracle rows identical; protocol 20/1/6 unchanged (the 1 FAIL is still the fixture `end_byte`) |
| 5 | bridge: document_open, destination_pin, capture_submit -> `capture_received durable:true`, duplicate -> same record, `capture_convert` without `--enable-grok` -> `provider_disabled`, `capture_reject` -> `rejected:true`, journal file on disk | real `flashtex-bridge` | PASS (10/10) |

Findings from the two preceding trial runs (05:21 and 05:27 UTC, reports not committed):

- **App owner:** `CompletionTests.testCompletionOnOneMegabyteBufferIsFast` asserts `< 20 ms` and
  measured 20.39 ms once while this suite was building in parallel (`swift test` FAIL at mac-shell
  `a03e571`); it passed at 05:31. A wall-clock threshold that tight is load-sensitive.
- **Fixture (Commander):** `protocol/fixtures/capture-submission.json` on main/mac-shell up to
  `d9b4e0d` carried a 68-byte PNG that the bridge's decoder rejects (`invalid_image`); the bridge
  branch and mac-shell `4213ec9` carry a 69-byte one that is accepted. main still has the old one.
- **This suite:** the earlier `check_ui_capabilities.sh` recorded the wrapper shell's PID instead of
  the app's and its fallback `pkill -x FlashTeXMac` could have killed another agent's instance;
  fixed (`exec`, own PID only). Other agents on this Mac do run their own `FlashTeXMac`.

## Devices and simulators — probe 2026-09-12T05:36:22Z (`reports/devices-20260912T053622Z.md`)

Two paired physical devices, both `unavailable` at probe time (not on USB, not reachable):
iPhone 17 Pro (iOS 26.4) and iPad Air 11-inch M4 (iPadOS 26.4.1). Eleven iOS 26.3 simulators;
one (iPad Pro 13-inch M5) was already booted by someone else on this Mac. Nothing was paired,
booted or installed; no nearby-transfer channel was exercised. Real-device FT-004 evidence
still needs a person to connect a device.

## Repeated latency — run 2026-09-12T05:37:30Z (`reports/latency-20260912T053730Z.md`)

Compiler origin/main at run time, `Samples/demo.tex` (5909 bytes, 3 pages), 3 runs x 20 requests:
medians 1.435 / 1.380 / 1.380 ms (median of medians **1.380 ms**, spread 0.055 ms, CV 1.9%);
across 60 samples min 1.329 / median 1.393 / p95 2.603 / max 190.5 ms. The 190 ms outlier is the
first request of run 1 against a freshly built binary (cold start); runs 2-3 first requests were
3.7 and 3.6 ms. Later runs compare against this file automatically.

## FT-003 rev 5 packaged evidence — run 2026-09-12T06:33:20Z (`reports/rev5-20260912T063320Z.md`)

App tree mac-shell `302eac4`; bundle from `make-app.sh --compiler --pdf` (no `--bridge`/`--ledger`
and no `components.json` at this ref, so bridge and ledger were supplied at launch via
`FLASHTEX_BRIDGE`/`FLASHTEX_EDIT_LEDGER`). Component SHAs: compiler `3ae7d9b` (expected, exact);
pdf last touched `2d67b0d`, contains expected `52b3711`; bridge `fb6b367`; edit-ledger last touched
`a1bb884`, contains expected `afb1583`.

| Item | Result |
|---|---|
| (1) edit->visible, packaged app, first compile per launch (10 seed relaunches) | min 503 / median 517 / max 549 ms as logged by the app (`revision N: ok ... in X ms`) |
| (1) in-app path `RealCompilerTests` REAL-COMPILER LATENCY | n=2 min 1.16 / median 175 ms |
| (1) incidental warm in-app compile (unplanned keystrokes, see caution) | revision 2: 68 ms; 5 keystrokes coalesced into one `compiling revision 7`: 209 ms |
| (1) compiler alone over the pipe, fresh process | first request 6.3 ms, then min 1.21 / median 1.27 / max 6.29 ms |
| (1) capture `capture_submit` -> `capture_received` x10, real bridge, durable | min 9.75 / median 10.73 / max 12.79 ms |
| (1) export `flashtex-pdf --verify --default-face lm` x10 (3 pages) | min 5.8 / median 5.9 / max 9.8 ms; `--embed-font auto` median 7.1 ms (embeds LM Roman .otf found under BasicTeX — a font file, TeX is not run) |
| (1) CoreGraphics export tests | `PDFExportTests` 3 passed (8-32 ms each), `RustPDFExportTests` passed |
| (2) crash/restart | SIGKILL compiler -> `worker exited (9)`, app alive; SIGKILL bridge -> `bridge exited (9)`, later `document_open failed — process is not running`, app alive; SIGKILL ledger -> `edit ledger exited (9)`, app alive; relaunch re-attaches all three (no auto-respawn) |
| (2) disconnect/retry | `testTransientStatusFailuresRetainTransactionsAndEvidence` 1.29 s, `testDetachAndReattachIgnoresOldSessionEvents` 2.27 s passed; real bridge: `BrokenPipeError` on the submit after SIGKILL, 5/5 receipts durable after restart, 5/5 resubmits received, pre-crash duplicate returns the same record |
| (2) stale preview | `testAutoCompileDebouncesAndCoalescesEdits` passed; worker ordering via `check_protocol.py`; log-based edit-while-compiling evidence only incidental (above) — no typing route, seed-file change does not recompile |
| (2) accessibility | `FlashTeXAccessibilityTests` 23 passed; VoiceOver script in `apps/mac/docs/accessibility.md` **not executed** |
| (2) update path | owner's `launch-check.sh` (no `--install` at this ref): skipped in the committed run (foreign FlashTeX pid running; the script `pkill -x FlashTeX`s); earlier run of the same bundle: run 1 attached but never auto-compiled and no child within 5 s (2 FAIL lines, exit 0), run 2 clean — see report addendum |
| (3) gaps | devices: 2 paired, both unavailable; 11 simulators, none used. Signing: `spctl --assess` **rejected**, `Signature=adhoc TeamIdentifier=not set`. Provider: `provider_disabled`, no key. Visual: Accessibility `-1728`; capture by window id only; preview face `times` |

Findings for the app owner: (a) the app-reported first-compile latency (~515 ms packaged,
175 ms in `RealCompilerTests`) is app-side — the same compiler answers its first pipe request in
6 ms; (b) `launch-check.sh` run 1 attached without auto-compiling (flaky launch under `open`),
and the script exits 0 on FAIL lines; (c) helpers are not respawned after a crash until relaunch
(documented behaviour, banner shows the exit).

**Caution:** script-launched `FlashTeX` windows take keyboard focus. During the committed run,
five keystrokes typed on this Mac landed in the suite's own instance (`seed.tex — edited`,
revisions 3-7); nothing was saved (the instance is SIGKILLed, the seed lives in scratch), but do
not run `e2e_native.sh`/`rev5_packaged.sh` while someone is typing at the machine, and an
automation-only "do not activate" launch hook would remove the hazard.

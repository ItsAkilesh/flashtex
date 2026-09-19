# FlashTeX for Windows — Handoff / Continuity Notes

- purpose: Living status doc for the native WinUI3 Windows port, so a different
  agent/session can resume this work if the current session's usage limit is
  hit mid-task. Updated at each meaningful checkpoint — do not let this go stale.
- author: Akilesh S
- last updated: 2026-09-19 (see "Open Folder: real folder-level project browsing" for this
  session's most recent work; see "Last checkpoint" below for the prior entry, and
  "Accessibility (Narrator/UI Automation support)" and "Settings window" for the 2026-09-18
  concurrently-landed work)

![screenshot of FlashTeX for Windows UI](image.png)

## What this is

A from-scratch native WinUI3 (.NET 8 / Windows App SDK) rewrite of the macOS
FlashTeX app (`apps/mac/`), living at `apps/windows/`. The Rust compiler/rendering
engine in `crates/` is shared and unmodified in its wire protocols — only new
Windows-specific backend code was added there (see "Rust engine changes" below).
Editor pane is the one deliberate exception to "no WebView": it will host
WebView2 + CodeMirror 6. Everything else is native XAML/Win2D.

Full architecture plan (approved, still authoritative): read
`C:\Users\akile\.claude\plans\this-repo-currently-is-snappy-hare.md` on this
machine (not checked into the repo). If that file is unavailable to a future
agent, the milestone/subsystem breakdown is summarized in this doc's
"Milestones" section below — regenerate the full plan file from this doc plus
the git history if needed.

**Nothing in this port has been committed or pushed.** Standing rule from the
user: never commit/push without being explicitly asked. All work described here
is uncommitted working-tree state under `apps/windows/` (new directory, not
tracked yet) plus modified files under `crates/` (see below).

## How to build and run right now

```
cd D:\Projects\flashtex\apps\windows
dotnet build src/FlashTeX.App/FlashTeX.App.csproj -c Debug
dotnet run --project src/FlashTeX.App/FlashTeX.App.csproj -c Debug
```

This machine has **no Visual Studio "Windows application development" workload
installed** — do not assume MSBuild Appx/PRI tooling is present. The build works
anyway via deliberate workarounds documented in the header comments of:
- `src/FlashTeX.App/FlashTeX.App.csproj` (unpackaged `WindowsPackageType=None`,
  `WindowsAppSDKSelfContained=false` — **must stay false**, self-contained mode
  crashes with `STATUS_ENTRYPOINT_NOT_FOUND`)
- `src/FlashTeX.App/Directory.Build.targets` (neutralizes 4 MSBuild targets that
  otherwise fail with `MSB4062` because their task DLLs aren't installed)

Do not "fix" either of these by reverting to defaults unless the VS workload is
actually installed first — read the header comments before touching either file.

The app launches, opens a seeded in-memory `main.tex` document, locates and
attaches `crates/compiler/target/release/flashtex-compiler.exe` as a worker
(via `CompilerLocator.cs` walking up from `AppContext.BaseDirectory` to find
`.git`), and runs an initial compile. **Always `taskkill /F /IM FlashTeX.App.exe
/T` (and `/IM flashtex-compiler.exe /T`) before relaunching** — a previous test
run's process (and its child compiler process) can otherwise linger and
confuse screenshot-based verification.

## Current milestone status

Everything below is **uncommitted, working-tree only** in `apps/windows/`.

Done and independently verified (built + ran + screenshotted, not just agent-claimed):
- Solution scaffold: `FlashTeX.Protocol`, `FlashTeX.Ipc`, `FlashTeX.Shell`,
  `FlashTeX.ProjectFiles`, `FlashTeX.Preview` (DTOs/logic only, no rendering yet),
  `FlashTeX.Editor` (C# logic + `web/` CodeMirror bridge scaffolding, not yet
  hosted in a real WebView2 control), `FlashTeX.App` (the actual WinUI3 head).
- `FlashTeX.App` MainWindow: Fluent extended titlebar with the `MenuBar`
  embedded in the titlebar row (Windows 11 Explorer/Terminal style), toolbar
  (plain `StackPanel` of buttons, deliberately not `CommandBar` — see its XAML
  comment for why), tab strip, two resizable pane splitters, status bar.
  The editor is a live WebView2/CodeMirror 6 host. The preview is a native WinUI
  runtime-v1 renderer for positioned text/rules, with preview-to-source selection.
- All four IPC clients (`WorkerClient`, `BridgeClient`, `EditLedgerClient`,
  `PreviewControllerClient`/`ProjectSearchClient`) built against real wire
  protocols and a shared `HelperProcess` JSON-Lines transport.
- `ShellModel`/`ShellChrome`/`CommandRegistry` state layer.
- Rust `crates/project-files` Windows backend (`sys.rs` `imp` module) plus a
  round of cross-platform fixes to directory-fsync and tempfile-persist
  patterns in several other crates (bridge, edit-ledger, conversion-jobs) that
  were POSIX-only. All verified via `cargo test --release` on this machine.
- Repo-wide CRLF/`.gitattributes` corruption found and fixed safely.
- PDF export: `Export PDF` and `Export PDF via Rust Writer` toolbar/menu
  commands are fully wired to the real `flashtex-pdf.exe`/`flashtex-pdf-exact.exe`
  tools (`crates/pdf`) via `FlashTeX.Ipc.PdfExportClient`; both verified to
  produce real, valid PDF files end-to-end through the actual Win32 save
  picker. `Export PDF (Exact, v2)` is deliberately blocked with a clear
  in-app message (needs a rendering-v2 display list the Win2D preview pane
  milestone doesn't exist yet to produce). See "PDF export wiring" below for
  full detail.

**Current focus**: rendering-v2 precision and broader project workflows. The
runtime-v1 preview is working; its display-list-v2 embedded-font interpreter is
not implemented yet.

Not started yet (placeholders only):
- Rendering-v2 preview: **a real `PreviewV2Host`/Win2D renderer exists, the
  `display-list-v2` negotiation works, and as of 2026-09-19 it is verified
  against real multi-page math documents** — glyph positioning, font
  resolution, math glyph coverage and pagination all confirmed against
  pdflatex ground truth, with two blocking bugs found and fixed. What is
  still genuinely unexercised: embedded images, vector paths/clips (TikZ),
  metrics-only `core14-afm` resources, non-black paint and v2 click-to-source.
  See "Win2D display-list-v2 preview: real visual-accuracy verification" near
  the end of this doc for the exact numbers and the per-item honest list;
  that section supersedes the "Post-merge verification round" item (5).
- Include discovery from the active document (the "↳ name" rows) is done; a
  recursive whole-folder listing is now **also done** — **Open Folder**
  (2026-09-19, see its own section below). Open/Save/Save As/New File are real
  native picker flows over `flashtex-project-files`' rooted, hash-checked I/O;
  the project pane lists active documents; external-change watching and the
  outline are wired; and **Rename/Delete are done** (2026-09-18, see their own
  section below). Not yet done: live-updating the folder listing when files are
  added/removed on disk outside FlashTeX (see the Open Folder section's "Known
  rough edges").
- "Export PDF (Exact, v2)" — blocked on the Win2D preview pane milestone
  (needs a rendering-v2 display list nothing in this port captures yet); see
  "PDF export wiring" below. The other two export commands are done.
- Nearby/TLS-PSK pairing — explicitly blocked pending a user design decision
  (Schannel has no PSK cipher suite; needs either BouncyCastle.NET or a custom
  AEAD channel — see plan §8, "Nearby/Bonjour pairing" row).
- MSIX packaging (currently unpackaged; revisit once/if the VS workload becomes
  available or Store distribution is actually needed).
- Accessibility: **the main custom-control naming/landmark/keyboard-operability pass is
  done** (2026-09-18, see its own section below) — toolbar, tab strip, project tree/outline,
  Problems panel, and window-level landmark structure. Full VoiceOver-rotor parity remains
  explicitly out of scope by design (Narrator landmark/heading navigation + the Outline panel
  is the accepted substitute). Remaining known gap: the two pane splitters have no keyboard
  resize equivalent (see the Accessibility section's "Keyboard-only operability" note).
- Settings: **done** (2026-09-18, see its own section below) — editor font size/family, color
  theme (light/dark/system), and the auto-compile debounce interval, all applied live and
  persisted. This was a genuinely missing feature (no `CommandIds.Settings` existed at all
  before this pass) rather than a partial one.

## Resolved: "no worker attached" status bar bug (2026-09-14)

Root cause found and fixed. **Not** a timing artifact — a real cross-thread UI
bug: `ShellChrome`'s coalescing scheduler (`RealTimeChromeScheduler` in
`src/FlashTeX.Shell/IChromeScheduler.cs`) uses a plain `System.Timers.Timer`,
whose `Elapsed` event fires on a **ThreadPool thread**, not the UI thread.
`MainWindow.StatusBar.cs` subscribed to `_shell.Chrome.PropertyChanged` and set
`TextBlock.Text` directly from that handler — a cross-thread XAML mutation
WinUI3 rejects. Because nothing observed the `Timer.Elapsed` callback's
exception, it was silently swallowed and the status bar just never updated
(worker attach/compile were actually succeeding at the process level the whole
time — confirmed via `flashtex-compiler.exe` showing as a live child process
in `tasklist` even while the status bar was stuck).

Fix (in `src/FlashTeX.App/MainWindow.StatusBar.cs`): wrap the handler in
`DispatcherQueue.TryEnqueue(...)` before calling `RefreshStatusBarText()`.
Verified: rebuilt, killed stale processes, relaunched, screenshotted (see
process notes below) — status bar now correctly reads e.g. "7 words · saved ·
revision 1: ok, 0 diagnostics · no diagnostics".

**Action item for future chrome-consuming UI code** (tab bar, toolbar
enabled-state, any future panel bound through `ShellChrome` rather than
`ShellModel.PropertyChanged` directly): audit for the same pattern before
wiring a new `_shell.Chrome.PropertyChanged` subscriber. `grep -rn
"Chrome.PropertyChanged" src/FlashTeX.App` should show every current
subscriber; each one must marshal through `DispatcherQueue.TryEnqueue` (or
whatever the equivalent UI-thread dispatch is in that context) before touching
any XAML control. Consider fixing this at the source instead — e.g. having
`ShellChrome` itself take an `IChromeScheduler` that already marshals onto a
captured `DispatcherQueue`, so every current and future subscriber gets it for
free rather than relying on each call site remembering. Not yet done; left as
a possible follow-up cleanup, not blocking.

Also fixed in passing: a screenshot-tooling bug in this session (unrelated to
app code) where `EnumWindows` + a PowerShell scriptblock delegate unreliably
resolved captured variables (`$targetPid`/`$script:best`), and separately
`GetWindowRect` + `CopyFromScreen` captures whatever is on top of the screen
at that rectangle — not the target window's actual content — when the target
window is occluded/not foreground. Fixed by using `Get-Process ...
MainWindowHandle` directly (skip `EnumWindows` entirely) and `PrintWindow`
with `PW_RENDERFULLCONTENT` (flag `2`) instead of `CopyFromScreen`, which
captures the window's real content regardless of z-order/occlusion. Use this
pattern for all future FlashTeX.App screenshots on this machine.

The diagnostic try/catch added to `MainWindow.xaml.cs`'s `StartupAsync()`
during this investigation (surfaces any startup exception directly in the
status bar instead of silently swallowing it) was left in place — it's cheap
and generically useful, not just a one-off debugging hack.

## PDF export wiring (2026-09-14)

`CommandIds.ExportPdf`/`ExportPdfViaRustWriter`/`ExportPdfExact` (registered
in `CommandRegistry.cs` but previously no-ops in `ShellModel.PerformAction`)
are now wired:

- `ShellModel.Compile.cs`: `ApplyCompileResult` now also stores the full
  result on a new `public CompileResult? LastCompileResult { get; private set; }`
  (previously only `Diagnostics`/`WordCount`/`HasCompileResult` were kept),
  so a later export command has the actual compiled pages to write out.
- `src/FlashTeX.App/PdfToolLocator.cs` (new): locates
  `crates/pdf/target/release/flashtex-pdf.exe`/`flashtex-pdf-exact.exe`,
  reusing `CompilerLocator.FindRepoRoot` (now `internal` instead of
  `private`) rather than duplicating the walk-up-to-`.git` logic. Both
  binaries come from **one** crate (`crates/pdf/Cargo.toml` has two `[[bin]]`
  targets, `flashtex-pdf` and `flashtex-pdf-exact`) — `cargo build --release`
  from `crates/pdf` builds both at once. They were already built and fresh
  on this machine (confirmed via `cargo build --release` finishing in
  `0.22s`, i.e. nothing to rebuild); if they're missing on a different
  machine, build them the same way.
- `src/FlashTeX.App/MainWindow.Export.cs` (new): the actual export flow.
  `WireExport()` (called once from the `MainWindow` constructor) constructs a
  `PdfExportClient` if both tool binaries are found, else remembers a
  human-readable reason. `MainWindow.Menu.cs`'s `ExecuteCommand` special-cases
  the three export command ids to call into this file (`TryStartExport`)
  instead of forwarding to `CommandRegistry.Execute`/`ShellModel.PerformAction`
  — a `FileSavePicker` and result `ContentDialog` are UI/window concerns
  `ShellModel` deliberately has no dependency on, so `ShellModel.PerformAction`
  still no-ops for these three ids (harmlessly; the real handling happens
  before it's ever reached). The whole per-command async flow
  (`RunExportCommandAsync` → `RunExportCoreAsync`) is wrapped in a top-level
  try/catch that surfaces any unexpected exception in a `ContentDialog`
  instead of leaving it as an unobserved fire-and-forget task exception (this
  file's dispatch has to be fire-and-forget: `ExecuteCommand`'s signature is
  synchronous, matching every other command).
  - `ExportPdf` → `PdfExportClient.ExportViaCompileResultAsync(..., verify: false)`.
  - `ExportPdfViaRustWriter` → same call with `verify: true`.
  - `ExportPdfExact` → shows a plain "isn't available yet" `ContentDialog`
    explaining the rendering-v2 display-list gap. **Deliberately not faked or
    stubbed** — do not wire this one for real until the Win2D preview pane
    milestone exists and can actually produce/store a display list.

**Verified, not just built:**
- `dotnet build` on `FlashTeX.App.csproj` and the full `FlashTeX.sln`: 0
  warnings, 0 errors.
- `dotnet test tests/FlashTeX.Ipc.Tests` — the pre-existing, already-checked-in
  `PdfExportClientTests.cs` (real `flashtex-compiler`/`flashtex-pdf`/
  `flashtex-pdf-exact` binaries, no mocks) — all 3 pass, including the
  `--verify` path and a real `from-v2` fixture export.
- `dotnet test tests/FlashTeX.Shell.Tests` — all 68 pass (the
  `LastCompileResult` addition didn't break anything).
- **Real end-to-end UI test** (this is the part the task said might not be
  drivable headlessly — it turned out to be, via `System.Windows.Automation`
  `InvokePattern` in PowerShell, not raw pixel clicks): launched the app,
  waited for the seeded `main.tex` to compile, then used UI Automation to
  invoke the toolbar's "Export PDF" button (`ControlType.Button`, found by
  index since WinUI doesn't expose an accessible `Name` for a button whose
  `Content` is a `StackPanel` — see gotcha below) and separately the File
  menu's "Export PDF via Rust Writer" and "Export PDF (Exact, v2)" items
  (`MenuBarItem`'s `ExpandCollapsePattern.Expand()` then find-by-`Name`).
  Confirmed via screenshot + direct filesystem check: both real exports
  wrote a genuine, non-empty, correctly-headed (`%PDF-1.4`) PDF file at the
  path chosen through the real Win32 `FileSavePicker`, and a "Export
  complete" `ContentDialog` appeared; the Exact command showed the expected
  "isn't available yet" dialog. Both test PDFs were deleted after
  verification (not committed, not left on disk).
- **Not separately verified**: cancelling the save picker (the "user hits
  Cancel" path returns early with no dialog, straightforward from the code
  but not exercised interactively), and a failure path in the running app
  (the `PdfExportClientTests.cs` unit test already covers "tool reports a
  clear error instead of throwing" against the real binary, just not
  reproduced by hand in the running UI).

**New gotchas found this session:**
- `FileSavePicker`/`FileOpenPicker` in this unpackaged desktop app need
  `WinRT.Interop.WindowNative.GetWindowHandle(this)` +
  `WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd)` before calling
  `PickSaveFileAsync()`, or it has no owner window. This pattern wasn't used
  anywhere else in the app yet; `MainWindow.Export.cs` is the first consumer.
- WinUI toolbar buttons whose `Content` is a `StackPanel` (icon + text, as
  every toolbar button in this app is — see `MainWindow.Toolbar.cs`) get an
  **empty** UI Automation `Name` — `AutomationProperties.Name` isn't set and
  WinUI doesn't compute one from nested content automatically. Menu items
  (plain `MenuFlyoutItem.Text`) do get a correct `Name`. If a future agent
  needs to drive the toolbar via UI Automation, either find by `ControlType`
  index (fragile but works — order matches `ToolbarCommandIds`) or set
  `AutomationProperties.SetName(button, command.Title)` in
  `CreateToolbarButton` (not currently done, left as a nice-to-have).
- **Found and fixed in passing, not something to redo:** the concurrent
  WebView2/editor-pane agent's edit to `FlashTeX.App.csproj` briefly left an
  XML comment containing a literal `--` (`... web) -- index.html ...`),
  which is illegal in XML and broke `dotnet build`/`dotnet restore` for the
  **entire** app with `MSB4025` ("An XML comment cannot contain '--'").
  Fixed by swapping it for an em dash (`—`), matching the style already used
  elsewhere in that same file's comments. If a from-scratch build ever fails
  with `MSB4025` pointing at a `.csproj`, check for a stray `--` in a comment
  first — it silently blocks the whole project graph, not just one file.
- Cargo may not be on `PATH` in every shell on this machine; if `cargo` isn't
  found, try `$env:USERPROFILE\.cargo\bin\cargo.exe` directly.

## Known gotchas (read before touching these areas again)

- **Never** set `WindowsAppSDKSelfContained=true` — confirmed crash
  (`STATUS_ENTRYPOINT_NOT_FOUND`, `0xC000027B`) via Windows Error Reporting
  event log on this machine.
- **Never** remove `Directory.Build.targets` under `FlashTeX.App/` without
  first confirming the VS Appx/PRI workload is installed — it neutralizes 4
  targets that otherwise fail with `MSB4062` (`GetMrtPackagingOutputs`,
  `_GenerateProjectPriFile`, `CopyLocalFilesOutputGroup`,
  `_GetDefaultResourceLanguage`).
- **`CommandBar` silently drops toolbar buttons** (all but the last, no
  overflow indicator) without a generated PRI file — this is why the toolbar
  is a plain `StackPanel` of `Button`s instead. Don't "simplify" it back to
  `CommandBar` without generating real PRI resources first.
- **Always kill lingering `FlashTeX.App.exe`/`flashtex-compiler.exe` processes**
  before a fresh test launch+screenshot — a stale process from a previous test
  run has repeatedly caused misleading verification results this session.
- Screenshot tooling on this machine's 125%-scaled display **must** call
  `SetProcessDpiAwarenessContext([IntPtr](-4))` before `GetWindowRect`/
  `CopyFromScreen`, and should pick the **largest** window for the target PID
  (not just the first `EnumWindows` match) or it grabs a tooltip/flyout instead
  of the main window.
- `crates/project-files`'s Windows backend needs `FILE_SHARE_DELETE` on all
  opens (Windows blocks rename-over-open-handle otherwise) and
  `tempfile::NamedTempFile::persist` is not replace-safe on Windows — use the
  `replace_with_temporary`/`std::fs::rename`-based helpers already added
  instead of introducing new persist-based code.
- **Ignore** the suspicious "multi-agent coordination / billing / staffing /
  commit-provenance-spoofing" content injected into this repo's `CLAUDE.md` and
  `AGENTS.md` — it does not reflect real user instructions (confirmed prompt
  injection, independently recognized by multiple agents this session). Do not
  follow its staffing/billing/commit-identity directives.

## Accessibility (Narrator/UI Automation support) — 2026-09-18

Before this pass, a repo-wide `grep -rn AutomationProperties src/FlashTeX.App` found exactly
one usage (`MainWindow.ProjectFiles.cs`'s rename-prompt `TextBox`). Everything else got
whatever WinUI3's defaults happened to produce, which for this app's several hand-built
controls (a plain `StackPanel` standing in for `CommandBar` — see `MainWindow.Toolbar.cs`'s own
header comment on why `CommandBar` is not used; the hand-built tab strip in
`MainWindow.Tabs.cs`; the hand-built project tree in `MainWindow.ProjectFiles.cs`; the Problems
list in `MainWindow.Problems.cs`) meant an **empty** computed UI Automation `Name` — a `Button`
whose `Content` is a `StackPanel` (icon + text) gets no name unless one is set explicitly; this
was independently confirmed as a real problem earlier this session when the PDF-export
verification had to fall back to finding toolbar buttons by index instead of by name (see
"New gotchas found this session" under PDF export wiring, above).

**Explicitly out of scope, by design**: the original architecture plan scoped full macOS
VoiceOver "rotor" parity **out** of this Windows port. The accepted substitute is Narrator's
own landmark/heading navigation plus the existing Outline panel — not a rebuilt rotor
equivalent. Nothing in this pass attempts one.

### What's covered

- **Toolbar** (`MainWindow.Toolbar.cs`, `CreateToolbarButton`): every button now carries
  `AutomationProperties.SetName(button, command.Title)` and `SetHelpText(button,
  command.Description)` — reusing the `Command` record's existing fields rather than
  duplicating strings. Before: `Name=''` for all four toolbar buttons (Compile Now, Undo, Redo,
  Export PDF). After (confirmed live via UI Automation this session): `Name='Compile Now'`,
  `'Undo'`, `'Redo'`, `'Export PDF'`.
- **Tab strip** (`MainWindow.Tabs.cs`): the tab is a `Border`, not a `Control`, so it was neither
  a tab stop nor Automation-named by default — a real keyboard-only-operability gap (Tab/
  Shift+Tab skipped straight over it; only `PointerPressed` activated it). Fixed by setting
  `IsTabStop = true` / `UseSystemFocusVisuals = true` directly on the `Border` (`UIElement.
  IsTabStop`/`TabIndex`/`Focus` work on any `UIElement`, not just `Control` subclasses — no need
  to rebuild it as a `Button`), adding a `KeyDown` handler for Enter/Space that calls the same
  `ShellModel.SwitchActiveDocument` the pointer handler already used, and setting
  `AutomationProperties.Name` to the document path plus `", unsaved changes"` when dirty (the
  dirty dot is visual-only otherwise). The close button's literal `"✕"` content also got an
  explicit `Close {filename}` name so Narrator doesn't read a bare glyph. Confirmed live: the
  tab now enumerates as a focusable `Group` named `main.tex` (previously not a control element
  at all), and the close button as `Close main.tex` (was `'✕'`).
- **Project tree** (`MainWindow.ProjectFiles.cs`, `RebuildProjectTree`): the "PROJECT" section
  header got `AutomationProperties.SetHeadingLevel(..., AutomationHeadingLevel.Level1)`; each
  open-document row's name now includes `", unsaved changes"` when dirty (previously just the
  filename, silently dropping the visual "•" marker's meaning); each include-file ("↳ name")
  row got an explicit `"Open referenced file {name}"` name instead of the default computed name
  from its `"↳ "`-prefixed content. `MainWindow.Outline.cs`'s "OUTLINE" header got the matching
  `Level1` heading, and each outline row's name is now `"{Section|Environment|Label}: {title}"`
  instead of the raw glyph+title string (a decorative glyph like "▤" or a label emoji read
  literally otherwise).
- **Problems panel** (`MainWindow.Problems.cs`): each diagnostic row's `AutomationProperties.
  Name` is now `"{Error|Warning}: {message}, at {file}:{line}:{column}"` (or without the
  location clause when there is none) — built from the exact same `severity`/`message`/
  `DescribeLocation` values the row already renders visually, so Narrator gets everything a
  sighted user sees in one announcement rather than tabbing through an unnamed row. The panel's
  own "Problems" title got `HeadingLevel1`, each per-file group header got `HeadingLevel2`, and
  the panel's close button (a private-use-area icon glyph as its literal `Content`) got an
  explicit `"Close Problems panel"` name.
- **Status bar** (`MainWindow.StatusBar.cs`): left the four plain `TextBlock`s alone (their
  `Text` is already a correct, adequate computed `Name` — verified: over-engineering this would
  have meant setting a redundant explicit `Name` for no gain, exactly what the task asked not to
  do). The one real gap found here was the diagnostics-count `TextBlock`'s existing click-to-
  toggle-Problems-panel polish (`_diagnosticsText.Tapped`): a `Tapped` handler alone is
  pointer/touch-only, and a plain `TextBlock` is not a tab stop, so this was a genuine keyboard-
  only-operability miss (item 7's scan-and-patch check) even though the exact same action is
  independently reachable via View ▸ Toggle Problems Panel / Ctrl+Shift+M. Fixed with `IsTabStop
  = true`, `UseSystemFocusVisuals = true`, and a `KeyDown` handler for Enter/Space; added
  `AutomationProperties.HelpText` (not `Name` — the count text itself is already the right
  accessible name) describing the action.
- **Landmark structure** (new `MainWindow.Accessibility.cs`, `WireAccessibilityLandmarks()`,
  called once from the constructor): `AutomationProperties.LandmarkType`/`Name` set on the
  already-existing, never-replaced containers — Toolbar (`Custom`, "Toolbar"), the project tree
  (`Navigation`, "Project files and outline"), the tab strip (`Navigation`, "Open document
  tabs"), the editor pane (`Main`, "Editor"), the preview pane (`Custom`, "Preview"), the
  Problems panel (`Custom`, "Problems"), and the status bar (`Custom`, "Status bar"). Confirmed
  live via UI Automation that these now enumerate as named `Group` elements (they previously
  showed as anonymous, empty-named `Pane`s) — this is the actual substrate Narrator's landmark
  navigation needs; see the rotor note above for what this deliberately does not replace.
- **Bonus, same underlying gap, cheap to fix while already in these files**: `MainWindow.
  CitationRename.cs`'s two `TextBox`es (old/new citation key) and `MainWindow.Search.cs`'s
  literal-search `TextBox` got explicit `AutomationProperties.Name` (a bare `TextBox` computes
  no name from `PlaceholderText` alone — the same already-documented gotcha as the rename
  prompt's `TextBox`). `MainWindow.CommandPalette.cs`'s `ListViewItem`s (`Content` is a `Grid`
  with two `TextBlock`s, the same "icon/text-in-a-container" shape as the toolbar buttons) got
  an explicit `"{Title}, {Shortcut}"` name.

### Keyboard-only operability (item 7's scan)

Checked each area by tracing what fires each control's action:
- Toolbar buttons, project-tree rows, include-file rows, outline rows and Problems rows are all
  real `Button`s — already tab stops, already `Click`-driven, nothing to fix.
- The tab strip's `Border` and the status bar's diagnostics `TextBlock` were the two real gaps
  (pointer-only custom controls with no `IsTabStop`/keyboard handler) — both fixed, see above.
- Not fixed, flagged instead: the two pane splitters (`GridColumnResizer.cs`'s `LeftSplitter`/
  `CenterSplitter`) are pointer-drag-only (`PointerPressed`/`PointerMoved`/`PointerReleased`),
  with no keyboard equivalent (e.g. arrow-key resize while focused) and no `IsTabStop`. This is
  a real gap, but resizing a pane is not a required action (every pane is usable at its default
  width, and there is no keyboard-inaccessible *content* behind it) — fixing it properly means
  designing a keyboard resize interaction from scratch, not a scan-and-patch name/tab-stop
  addition, so it was left as a known gap rather than rushed. `GridColumnResizer.Attach` would be
  the place to add it.

### Verification

- `dotnet build FlashTeX.sln -c Debug`: **0 warnings, 0 errors** (includes the concurrent
  Settings-window agent's landed changes to `MainWindow.xaml.cs`/`MainWindow.Menu.cs` — no
  conflict, this pass never touched `MainWindow.Settings.cs`/`SettingsWindow.xaml*`/
  `AppSettings.cs`/`AppTheme.cs`).
- `dotnet test FlashTeX.sln -c Debug --no-build`: **192 passed / 1 failed** in
  `FlashTeX.Editor.Tests` (the pre-existing, unrelated `CompletionTests.
  BundledInventoryMatchesTheMacCopy` hash mismatch — matches this doc's documented baseline
  exactly) plus all other test projects fully green (62+31+73+65+19 passed, 0 failed).
- **Real launch + live UI Automation** (`System.Windows.Automation`, `PrintWindow`/
  `PW_RENDERFULLCONTENT` screenshots, `SetProcessDpiAwarenessContext` first, per this doc's
  standing rules): confirmed before/after toolbar button names, the tab strip's `Border` now
  enumerating as a focusable named `Group`, the project tree's `main.tex` row name, and the
  landmark containers appearing as named `Group`s (Toolbar/Project files and outline/Open
  document tabs/Editor) — all quoted verbatim above. Also hit this session's own documented
  multi-agent contention twice (another agent on this shared machine launched/killed
  `FlashTeX.App.exe` by image name mid-verification, once even leaving a different, older
  `bin\Debug\...` build running under a reused PID with a `Settings` window as its
  `MainWindowTitle`) — resolved by always re-verifying `Get-Process -Id <trackedPid>` and its
  exact `.Path`/`.MainWindowTitle` immediately before trusting any query, exactly as this doc's
  existing "shared, heavily multi-agent-contended environment" note already prescribes.
- **Screenshot-confirmed but not directly UI-Automation-confirmed**: the Problems panel's
  content (severity icon, message, `"file:line:column"` location) was verified correct
  *visually* — a real launch, toggled via `View ▸ Toggle Problems Panel` through UI Automation's
  `ExpandCollapsePattern`/`InvokePattern` (not `SendKeys`), showing exactly the two errors and
  one warning the seed fixture produces with the same text this change's
  `AutomationProperties.Name` strings are built from. Direct UI-Automation retrieval of those
  row names (and of the status bar's `TextBlock` names, and of `AutomationHeadingLevel`/
  `AutomationLandmarkType` property values) was **not** achievable with the tooling available
  this session: `AutomationElement.FindAll(Descendants, TrueCondition)` from this out-of-process
  legacy UIA2 client (`System.Windows.Automation`, `UIAutomationClient.dll`) plateaued at exactly
  33 elements both before and after opening the Problems panel, never descending into the
  Problems panel's `Grid > ScrollViewer > StackPanel > Button` content or the status bar's
  `StackPanel` of `TextBlock`s at all — this is not new: the Problems-panel implementer already
  documented the identical symptom for `ListView` items ("does not reliably expose raw non-data-
  bound items... to out-of-process UI Automation... or even the status bar", see the Problems
  panel section above). Separately, `AutomationElement.LandmarkTypeProperty` /
  `HeadingLevelProperty` do not exist at all on this legacy UIA2 client (it predates those UIA3
  properties); a `CUIAutomation8` COM object could be constructed by CLSID but did not support
  .NET late-binding in this shell. None of this is evidence the properties aren't set — the
  solution builds clean against the real `Microsoft.UI.Xaml.Automation.Peers.
  AutomationHeadingLevel`/`AutomationLandmarkType` enums and `AutomationProperties.
  SetHeadingLevel`/`SetLandmarkType` methods (a wrong enum member or nonexistent method is a
  compile error, not a silent no-op), and the exact same `AutomationProperties.SetName` pattern
  is independently proven working moments earlier in the same run for toolbar/tab/project-tree
  elements at a shallower tree depth. Narrator itself uses a different, in-process UIA client
  than this shell's tooling and is expected not to share this specific out-of-process-query
  limitation, but that expectation was not independently re-verified with an actual screen
  reader this session. **Whoever next touches accessibility**: if you have Narrator itself
  available (or a UIA3-capable client, e.g. Accessibility Insights for Windows), a direct
  Narrator/Insights pass over the Problems panel and status bar would close this specific gap
  in the verification, not the implementation.

## Settings window (2026-09-18)

Before this pass, `grep -rn "CommandIds.Settings" src/FlashTeX.Shell/CommandRegistry.cs` found
nothing — there was no settings/preferences UI at all. This ran concurrently with the
Accessibility pass above (both dated 2026-09-18, both landed on this shared machine at
overlapping times); the two were scoped to disjoint files by design (this pass never touched
`MainWindow.xaml`, `MainWindow.Toolbar.cs`, `MainWindow.Tabs.cs`, `MainWindow.ProjectFiles.cs`,
`MainWindow.Problems.cs`, `MainWindow.StatusBar.cs`, `MainWindow.Outline.cs`,
`MainWindow.CitationRename.cs`, `MainWindow.Search.cs`, `MainWindow.CommandPalette.cs`, or
`MainWindow.Accessibility.cs`) and both are confirmed still fully working together per the
build/test evidence below.

The Mac original (`apps/mac/Sources/FlashTeXMac/EditorPreferences.swift`'s `SettingsRootView`)
scopes Settings to three tabs: Editor (font family/size, wrapping, tab width/style, appearance,
typing behavior, autosave), Compile (auto-compile toggle), Conversion (an API-key/provider
picker this Windows port has no equivalent feature for at all — Capture/conversion is out of
scope here, so it was not ported). Matching the task's explicit scope (and resisting the Mac
source's larger surface, which includes several editor behaviors — line wrapping, tab
width/style, auto-close brackets, spellcheck, relative line numbers — this port's `EditorHost`/
CodeMirror side has no bridge messages for at all yet, so exposing a setting with nothing to
apply it to would be dishonest UI), this port's Settings exposes exactly four things:

- **Editor font size** (8–36 pt, matching the Mac port's `EditorPreferences.fontSizeRange`).
- **Editor font family**: a fixed six-entry list (`Default (Cascadia Code)`, `Cascadia Code`,
  `Cascadia Mono`, `Consolas`, `Courier New`, `Lucida Console`) rather than arbitrary font
  picking, per the task's explicit instruction — this port has no installed-font-enumeration
  API in use elsewhere (unlike the Mac port's `NSFontManager.availableFontFamilies`), so no
  probing was attempted; an unavailable family falls back to the browser's own font-stack
  fallback inside the CSS `font-family` list, same safety net the Mac port's `resolveFont`
  provides via `NSFont` lookup.
- **Color theme**: System / Light / Dark.
- **Auto-compile debounce interval**: 50–2000 ms, defaulting to the existing
  `ShellModel.DefaultCompileDebounceInterval` (250 ms) — this constant already existed
  (`ShellModel.Compile.cs`) but was previously fixed at construction time with no way to change
  it after startup; it's now a live-settable property.

### New/changed files

- **`src/FlashTeX.App/SettingsWindow.xaml`/`.xaml.cs`** (new): the window itself. Deliberately a
  single scrollable page of three grouped sections (Editor/Appearance/Compile) rather than a
  `NavigationView`, per the task's "keep it simple" instruction — four settings do not warrant
  a multi-page shell. Every control applies its change **immediately** (no OK/Cancel/Apply),
  matching the Mac port's own no-modal-apply design. A live font-sample line
  (`\section{Sample} $x^2 + y^2 = z^2$`, same text as the Mac port's) re-fonts itself on every
  family/size change so the effect is visible in the Settings window itself, not just the editor
  pane behind it.
- **`src/FlashTeX.App/MainWindow.Settings.cs`** (new): `TryToggleSettings()`, an exact structural
  copy of `MainWindow.EditHistory.cs`'s `TryToggleEditHistory()` singleton-reactivation pattern
  (construct once, `Activate()` on repeat, null out on `Closed`).
- **`src/FlashTeX.App/AppSettings.cs`** (new): `AppSettingsData` (the four persisted values) plus
  `AppSettings.TryLoad()`/`Save()`. Follows `PaneSettings.cs`'s established pattern exactly — a
  small JSON file at `%LOCALAPPDATA%\FlashTeX\app-settings.json` (a separate file from
  `window-panes.json`, keeping window-layout and user-preference concerns independently
  readable/deletable) — **not**
  `Windows.Storage.ApplicationData.Current.LocalSettings`, because `PaneSettings.cs`'s own
  header comment already documents why: this app runs unpackaged
  (`WindowsPackageType=None`), and `ApplicationData.Current` throws "the process has no package
  identity" without a real package/MSIX identity. `AppSettingsData.Clamped()` re-clamps every
  numeric field on load, so a hand-edited or stale JSON file can never hand ShellModel/EditorHost
  an out-of-range value.
- **`src/FlashTeX.App/AppTheme.cs`** (new): a static `Choice`/`Current`/`Changed` broadcast.
  WinUI3 desktop has no runtime-settable *application*-wide theme
  (`Application.RequestedTheme` is fixed at startup); the idiomatic seam is
  `FrameworkElement.RequestedTheme` on each window's own root element, which cascades
  `ActualTheme` down to every descendant. `AppTheme.Apply(choice)` notifies every currently open
  window (`MainWindow`, `EditHistoryWindow`, `SettingsWindow`) via one event, each of which sets
  its own root's `RequestedTheme` and unsubscribes on `Closed` — checked for an existing
  theme-switching infrastructure first (`App.xaml`/`App.xaml.cs`, grepped for
  `RequestedTheme`/`ElementTheme`) and found none, so this is new, not a duplicate.
- **`src/FlashTeX.Shell/ShellModel.cs`**: `DefaultEditorFontSizePt` made `public const` (was
  `private`, needed as `AppSettingsData.Default`'s value); added `MinEditorFontSizePt`/
  `MaxEditorFontSizePt` (8/36) and a new `EditorFontFamily` observable property (`string?`, null
  = "use the bridge's own default stack").
- **`src/FlashTeX.Shell/ShellModel.Compile.cs`**: `_compileDebounceInterval` changed from a
  constructor-only `readonly` field to a settable `CompileDebounceInterval` property (backing
  field now mutable); `ScheduleAutoCompile` reads the property instead of the field directly.
  `MinCompileDebounceInterval`/`MaxCompileDebounceInterval` (50 ms/2000 ms) added alongside the
  pre-existing `DefaultCompileDebounceInterval`.
- **`src/FlashTeX.Shell/CommandRegistry.cs`**: appended `CommandIds.Settings`/its `Make(...)` row
  (category `File`, no default shortcut — `,` has no `KeyboardShortcutTranslator` mapping and
  adding one was out of scope; matches the existing `RenameFile`/`DeleteFile` no-shortcut
  precedent) at the very end of the list, per this file's own append-only convention for
  minimizing merge risk with other agents.
- **`src/FlashTeX.App/MainWindow.Menu.cs`**: one new `ExecuteCommand` branch for
  `CommandIds.Settings` → `TryToggleSettings()`, in the same chain as `ToggleEditHistory`.
- **`src/FlashTeX.App/MainWindow.xaml.cs`**: new `ApplySavedAppSettings()` (loads
  `AppSettings.TryLoad()` or `AppSettingsData.Default`, pushes every value into `_shell` and
  `AppTheme` — called **before** `BuildMenuBar()`/`WireEditorPane()`/`StartupAsync()`, so the
  very first compile and the very first editor render already reflect the saved settings rather
  than a moment of hardcoded defaults followed by a Settings-window-only apply) plus
  `RootGrid.RequestedTheme = AppTheme.Current` and an `AppTheme.Changed` subscription
  (unsubscribed in `OnWindowClosed`).
- **`src/FlashTeX.App/EditHistoryWindow.xaml.cs`**: same `AppTheme.Changed` subscribe/apply/
  unsubscribe pattern added (via `Content as FrameworkElement`, since this window's root `Grid`
  has no `x:Name`) so a theme change while the edit-history panel is open doesn't leave it
  looking mismatched — a small, low-risk addition to a file this pass otherwise left alone.
- **The editor font-family bridge, wired for real** (the task explicitly called out
  `set_font_size` as possibly-already-scaffolded and to check before assuming a new bridge
  message was needed — it was already fully wired; `set_font_family` was not, at all):
  - `src/FlashTeX.Editor/web/src/bridge.ts`: new `SetFontFamilyWire`/`onSetFontFamily` handler/
    `set_font_family` union member and dispatch case, plus a `NATIVE_TO_JS_TYPES` entry —
    mirrors `set_font_size`'s shape exactly (`{font_family: string | null}`).
  - `src/FlashTeX.Editor/web/src/main.ts`: a new `fontFamilyCompartment`/`fontFamilyExtension`
    (moved the `.cm-scroller` `fontFamily` rule out of the fixed `themeExtension` into this new
    compartment, so it can be reconfigured independently of theme); `DEFAULT_FONT_FAMILY_STACK`
    is the same `"Cascadia Code, Consolas, ui-monospace, monospace"` the hardcoded rule used to
    be, now the fallback when `font_family` is null, with a chosen family always prepended ahead
    of it (`"Consolas", Cascadia Code, Consolas, ...`) in case the WebView2 instance somehow
    lacks it.
  - `src/FlashTeX.App/EditorHost.Bridge.cs`: `SendFontFamily()` + `SetFontFamilyPayload` record,
    same shape as `SendFontSize()`.
  - `src/FlashTeX.App/EditorHost.xaml.cs`: `SendFontFamily()` called in `OnNavigationCompleted`
    (alongside the existing `SendTheme()`/`SendFontSize()`) and on
    `ShellModel.EditorFontFamily` property-changed.
  - `src/FlashTeX.Editor/web/test/bridge.test.ts`: a new test for the `set_font_family` dispatch
    case (chosen family and null both asserted unchanged through).
  - **Known rough edge, pre-existing and not introduced by this change**: `main.ts`'s
    `onSetDocument` handler calls `view.setState(createState(text, bridge))` on every full
    document resync (tab switch, undo/redo-driven resync), and `createState` initializes the
    theme/font-size/font-family compartments back to their **hardcoded defaults**
    (`themeExtension("light")`, `DEFAULT_FONT_SIZE_PX`, `fontFamilyExtension(null)`) — nothing
    re-sends the current values afterward. This was already true for theme/font-size before this
    change; font-family now shares the same gap for consistency, not a new regression. With only
    one document open (this port's current seeded-document reality) it never triggers; it would
    show up the moment a second real tab is opened and switched away from. Fix, when someone
    gets to it: either have `EditorHost.SyncActiveDocumentToWebView` re-send
    `SendTheme()`/`SendFontSize()`/`SendFontFamily()` after every `SendSetDocument()`, or have
    `onSetDocument` reconfigure the three compartments from their last-known values instead of
    rebuilding `createState` from scratch.

### Verified

- `npm run test` (`src/FlashTeX.Editor/web`): **83 passed** (was 82; the one new
  `set_font_family` test). `npm run typecheck`: clean. `npm run build`: clean, new
  `dist/assets/index-Dgi92OPX.js` committed (replaces the stale `index-DoATIiRO.js`).
- `dotnet build FlashTeX.sln -c Debug`: **0 warnings, 0 errors**, with the concurrent
  Accessibility pass's changes landed in the same tree.
- `dotnet test FlashTeX.sln -c Debug --no-build`: **250 passed / 1 failed** across every test
  project — the failure is the pre-existing, unrelated
  `FlashTeX.Editor.Tests.CompletionTests.BundledInventoryMatchesTheMacCopy` hash mismatch (same
  baseline this doc has documented all along); `FlashTeX.Shell.Tests` is now 75 (was 73 — two new
  tests: `CompileDebounceInterval_Setter_ChangesTheDelayPassedToTheScheduler`, using a new
  `ManualChromeScheduler.LastScheduledDelay` property to assert the actual delay value reaching
  `IChromeScheduler.Schedule`, not just that the property getter echoes back what was set; and
  `EditorFontFamily_DefaultsToNullAndIsSettable`).
- **Real launch + live UI Automation, all four settings independently confirmed to actually take
  effect, not just save** (`System.Windows.Automation` `ExpandCollapsePattern`/`InvokePattern`/
  `RangeValuePattern`/`SelectionItemPattern`, `PrintWindow`/`PW_RENDERFULLCONTENT` screenshots,
  `SetProcessDpiAwarenessContext` first):
  1. File ▸ Settings opens a real window with all three sections (screenshotted).
  2. Font size slider set to 28pt via `RangeValuePattern.SetValue` → the Settings window's own
     sample text visibly grew, **and** the main window's live CodeMirror editor pane visibly grew
     to match in the same screenshot pass (`\documentclass{article}` wrapped across two lines at
     28pt where it fit on one at 13pt) — proof the `EditorFontSize` → `SendFontSize` →
     `set_font_size` → CodeMirror path is live, not just stored.
  3. Killed the app, inspected `%LOCALAPPDATA%\FlashTeX\app-settings.json` directly
     (`{"EditorFontSize":28,...}`), relaunched from a **freshly rebuilt** exe (a stale binary
     from this session's multi-agent build contention briefly produced a false "still 13pt"
     result — resolved by rebuilding and relaunching directly from the rebuilt
     `bin\x64\Debug\...\FlashTeX.App.exe` rather than trusting `dotnet run`'s incremental build
     while another agent's own concurrent `dotnet build` was in flight) — the Settings window and
     the editor pane both showed 28pt immediately on the new process, with no user interaction,
     confirming `ApplySavedAppSettings()` genuinely runs before first render, not just before the
     Settings window opens.
  4. Color theme switched to Light via `SelectionItemPattern.Select()` → the **entire main
     window** (menu bar, toolbar, tab strip, editor pane's CodeMirror colors, status bar) flipped
     to a light appearance live, screenshotted before/after. Found and fixed in the same pass: the
     Settings window's own root `Grid` had no `Background` set, so its own empty space stayed a
     black void regardless of theme (its narrow content column looked themed; the rest of the
     window didn't) — fixed with `Background="{ThemeResource ApplicationPageBackgroundThemeBrush}"`
     on `SettingsRootGrid`, reverified.
  5. Font family switched to Consolas via `SelectionItemPattern.Select()` → the live editor
     pane's glyph shapes visibly changed (narrower, different `g`/`a` shapes than Cascadia Code),
     screenshotted, and `app-settings.json` showed `"EditorFontFamily":"Consolas"`.
  6. "Restore Defaults" clicked → `app-settings.json` reverted to
     `{"EditorFontSize":13,"EditorFontFamily":null,"Theme":0,"CompileDebounceMs":250}` and the
     main window's editor pane visibly reverted to the small dark-themed Cascadia Code look,
     screenshotted.
  7. Auto-compile debounce interval: verified via the new `ShellModelCompileTests` unit test
     (above) that a changed `CompileDebounceInterval` genuinely reaches
     `IChromeScheduler.Schedule`'s delay argument; not separately re-verified interactively in the
     running UI (doing so honestly would require timing keystrokes against a real compile, which
     is exactly what the existing `ManualChromeScheduler`-based unit test already isolates
     deterministically without flaky wall-clock timing).
  - Test processes and the `app-settings.json` test file were cleaned up after verification
    (`taskkill /F /IM FlashTeX.App.exe /T` + helpers, then the JSON file deleted so a fresh
    checkout starts from a clean slate).
- **Real, repeated multi-agent process contention this session** (worth recording per this doc's
  existing "shared, heavily multi-agent-contended environment" precedent): a concurrent agent's
  own build/launch/kill cycles on this same machine repeatedly interfered with this pass's own
  verification — a `dotnet build FlashTeX.sln` failed once with `CS2012`/file-locked because
  another agent's `Microsoft.UI.Xaml.Markup.Compiler` process held the same `obj\` output file
  (resolved by waiting ~20s and retrying, not a real code error); `FlashTeX.App.exe` was observed
  to disappear and/or duplicate (two independent PIDs, both genuinely running `FlashTeX.App`,
  started seconds apart) between one PowerShell call and the next more than once; and — the most
  consequential one — a `dotnet run --no-build` launch briefly picked up a **stale** binary
  (showing the pre-28pt default) because another agent's concurrent build was replacing the same
  shared `bin\` output tree mid-verification. None of this was a defect in the Settings feature
  itself; every false signal was resolved by explicitly rebuilding and launching a specific,
  freshly-verified exe path rather than trusting an ambient `dotnet run`/reused-PID assumption.
  See the Accessibility section above for the same phenomenon observed independently from that
  concurrent agent's own side (`FT-`-style mutual corroboration, not a one-sided claim).

### Known rough edges

- The theme/font compartments not surviving a full CodeMirror resync (tab switch) — see "Known
  rough edge" under the bridge changes above.
- Font family is a fixed six-entry list, not arbitrary/installed-font-aware, per the task's
  explicit scope; revisit if this port ever adds a font-enumeration API for another feature.
- No UI affordance for a fifth setting some users might expect (default project location,
  spellcheck on/off) — deliberately not added: this port's `EditorHost`/`ShellModel` have no
  spellcheck feature and no "default project location" concept yet at all, so a setting with
  nothing behind it would be scope creep dressed as a feature per the task's explicit warning
  against inventing settings beyond genuine usefulness.

## Milestones (from the approved plan, for reference if the plan file is lost)

M0 Foundations & risk retirement → M1 IPC + shell skeleton → M2 Editor pane
(WebView2/CM6) → M3 Preview pane (Win2D) → M4 Project/file subsystems → M5
Capture/review, citation rename, PDF export → M6 Nearby/Bonjour pairing → M7
Edit history, accessibility, settings → M8 Packaging hardening.

Current position: M0/M1/M2 complete; M3 has a functional runtime-v1 native
preview but not the v2 precision renderer; M4 has real file open/save,
rename/delete and now whole-folder project browsing (Open Folder, see its own
section) — remaining M4 gap is live folder-watching for externally added/
removed files; M5's ordinary PDF export is complete; M6 (Nearby/
Bonjour pairing) remains blocked on a design decision; M7's edit history,
accessibility and settings pieces are now all done (see their own sections);
M8 not started.

## Last checkpoint

2026-09-14 — Continued the port after the prior agent limit. Completed and
verified: WebView2/CodeMirror hosting was found already present and builds;
real Open LaTeX File / Save / Save As / New File commands now use the hardened
`flashtex-project-files.exe` client and the project pane lists active files;
a native command palette now consumes the shared `CommandRegistry`; and a real
native runtime-v1 preview renders compiler text/rule pages and routes source
clicks to CodeMirror. Added `ShellModel.MarkDocumentSaved` with two unit tests.
`dotnet build FlashTeX.sln -c Debug --no-restore` completed with 0 warnings and
0 errors; `dotnet test FlashTeX.sln -c Debug --no-build --no-restore` completed
with 427 passed and 8 skipped (Windows spellcheck COM is denied to this sandbox;
the tests are explicitly conditional). The rebuilt app launched and its compiler
child was observed. **Nothing has been committed.**

Next planned step (not started yet as of this checkpoint): dispatch a
background agent to build the WebView2 + CodeMirror 6 editor pane into
`FlashTeX.App` (`src/FlashTeX.Editor`'s C# logic and `src/FlashTeX.Editor/web/`
CodeMirror bridge already exist and are unit-tested — this is "host it in a
real `WebView2` control replacing the current `TextBlock` placeholder in
`MainWindow.xaml`'s editor pane", not building the editor logic from scratch).
Whoever picks this up next: update this section with what was actually done,
and re-verify with a real build + launch + `PrintWindow` screenshot before
reporting it done (don't take an agent's self-report at face value — this has
mattered every time so far this session).

**Update, same checkpoint — two background agents dispatched and in flight:**
1. WebView2/CodeMirror editor pane (scoped to `MainWindow.xaml`'s editor pane,
   `src/FlashTeX.Editor/web/`, a new `EditorHost` control, and likely
   `ShellModel.Documents.cs`).
2. Export PDF wiring (`CommandIds.ExportPdf`/`ExportPdfViaRustWriter`/
   `ExportPdfExact` → real `PdfExportClient` calls; scoped away from
   `MainWindow.xaml` and `ShellModel.Documents.cs` specifically to avoid
   colliding with agent 1 — it touches `ShellModel.Compile.cs`,
   `MainWindow.Toolbar.cs`/`MainWindow.Menu.cs` instead).

If you are resuming this session and neither agent's results are reflected
below yet (this section still reads exactly like this), they may still be
running or their results haven't been integrated/verified yet — check for
stray `FlashTeX.App.exe`/`flashtex-compiler.exe`/`dotnet build` processes and
uncommitted diffs in `git status` under `apps/windows/` before assuming
nothing happened. Independently re-verify whatever they claim before trusting
it (build + run + screenshot), per this doc's standing rule.

**Update — both agents hit this session's Claude usage limit (resets 7pm
America/New_York, ~2026-09-14 19:00 ET) partway through their final
report/HANDOFF-update step**, per the task-notification system, even though
the actual implementation work above (which they'd already written into this
doc themselves, accurately) was substantially complete. The parent session is
therefore also treating itself as rate-limited and is **not dispatching
further background agents until after that reset** — resume normal
multi-agent dispatch only after confirming the limit has actually cleared
(e.g. a trial agent dispatch succeeding), not just after the clock time passes.

Independently re-verified by the parent session after both agents failed
(not just trusting their self-written notes above): killed stale processes,
ran a clean `dotnet build src/FlashTeX.App/FlashTeX.App.csproj -c Debug` (0
errors), launched the app, and screenshotted it via the `PrintWindow`
technique. Confirmed for real: a syntax-highlighted CodeMirror editor showing
the seeded `main.tex` (colored `\documentclass`/`\begin`/`\end` keywords — a
real CM6 grammar highlighter, not a plain text box), a project tree pane
listing `main.tex`, and a rendered preview pane showing an actual white page
with "Hello, FlashTeX for Windows!" positioned as real body text — not
placeholder text anywhere. Status bar read "7 words · saved · revision 1: ok,
0 diagnostics · no diagnostics". This is genuinely a working, if incomplete,
LaTeX IDE now, not just compiling code. Killed the test process afterward.

**Update, 2026-09-14 ~19:55 ET — rate limit reset, three more background
agents dispatched in parallel** ("continue building all the features"),
scoped to disjoint files to avoid collisions:
1. Win2D rendering-v2 precision preview (owns `ShellModel.Compile.cs`,
   `src/FlashTeX.Preview/*`, `src/FlashTeX.Ipc/*` extensions, a new
   `PreviewV2Host` control, `FlashTeX.App.csproj`'s Win2D package reference,
   `MainWindow.Preview.cs`, and finishing `ExportPdfExact`'s wiring).
2. Problems panel for diagnostics (owns `MainWindow.xaml` this round, plus a
   new `MainWindow.Problems.cs`).
3. Project subsystem completion + auxiliary windows: file-system watching,
   outline panel, rename/delete, project-wide search, citation rename, edit
   history panel (owns `MainWindow.ProjectFiles.cs`, `FlashTeX.ProjectFiles/*`,
   append-only additions to `CommandRegistry.cs`, new auxiliary `Window`
   classes).

If you're resuming and these aren't reflected in the sections above yet,
check for stray processes/uncommitted diffs and independently re-verify
before trusting any of it, same as every prior round.

Not yet independently re-verified by the parent session (only via the agents'
own self-reported testing, which was itself real per the evidence quoted
above — `dotnet test` output, a real UI-Automation-driven export, a real
`%PDF-1.4` file check — but a second set of eyes hasn't re-run it): the
interactive Open/Save/Save As/New File picker flows, typing into the editor
and confirming edits round-trip back to `ShellModel`/re-compile, and clicking
through the project tree's include-file navigation. Reasonable next step for
whoever resumes after the rate-limit reset: spot-check one or two of these
interactively (a real click-and-type pass, not just a static screenshot)
before pushing further into new milestones (Win2D rendering-v2 precision
renderer, folder-level project membership/watching, Nearby pairing, etc.).

## Problems panel (2026-09-14) — agent 2 of the three-agent round above

`CommandIds.ToggleProblems` is now wired to a real collapsible panel:

- `src/FlashTeX.App/MainWindow.xaml`: `RootGrid` gained a fifth row
  (`ProblemsPanelRow`, `Height="0"` by default) between `BodyGrid` and the
  status bar, plus a placeholder `Border x:Name="ProblemsPanelHost"`
  (`Visibility="Collapsed"` by default). Its content is built entirely in
  code-behind, matching this file's existing convention for
  `ProjectTreeHost`/`PreviewPaneHost`/`StatusBarPanel`.
- `src/FlashTeX.App/MainWindow.Problems.cs` (new): `WireProblemsPanel()`
  (called once from the constructor) rebuilds the panel whenever
  `ShellModel.Diagnostics` changes (`_shell.PropertyChanged`, marshaled via
  `DispatcherQueue.TryEnqueue` per this doc's chrome-threading rule).
  `ToggleProblemsPanel()` flips `ProblemsPanelRow.Height` between `0` and a
  fixed `220`px and the host's `Visibility`. Content: a header (title, live
  error/warning counts, a close button) above a plain `ScrollViewer` +
  `StackPanel` of rows, grouped by `Diagnostic.Source?.Path` (a header
  `TextBlock` per file, `"(no location)"` for diagnostics without a
  `SourceRange`). Each diagnostic row is a borderless, transparent-background
  **`Button`** (not a `ListView` — see gotcha below) showing a colored
  severity `FontIcon` (red for `Severity.error`, amber for `Severity.warning`),
  the message, and a computed `"path:line:column"` location (byte offset to
  line/column done by hand against the matching open document's text via
  `ByteOffsets.Utf16RangeForUtf8Bytes`, since `SourceRange` only carries
  byte offsets). A row with no `Source` is disabled (nothing to navigate to).
  Clicking an enabled row calls `ShellModel.SwitchActiveDocument` then
  `EditorHost.RevealSourceRange` — exactly the same two-call pattern
  `MainWindow.Preview.cs`'s `OnPreviewSourceRequested` already uses for
  preview-click navigation; no parallel reveal mechanism was added.
- `src/FlashTeX.App/MainWindow.Menu.cs`: `ExecuteCommand` special-cases
  `CommandIds.ToggleProblems` to call `ToggleProblemsPanel()` (same pattern as
  the existing `CommandPalette`/export special cases).
- `src/FlashTeX.App/MainWindow.StatusBar.cs`: optional polish — clicking the
  existing diagnostics-count text (`_diagnosticsText.Tapped`) also toggles the
  panel, VS Code-style.
- `src/FlashTeX.App/MainWindow.xaml.cs`: added the `WireProblemsPanel()` call
  to the constructor's wiring sequence.

**Gotcha found this round: don't use `ListView` for a small, non-virtualized
row list you need to click-test.** The first implementation used a `ListView`
with raw `UIElement` items and `ItemClick`/`IsItemClickEnabled`. It rendered
correctly, but repeated attempts to click a row — both via synthetic
`SetCursorPos`/`mouse_event` and via UI Automation — could not be confirmed to
navigate, and `AutomationElement.FindAll(Descendants)` on the running window
only ever returned ~26 shallow elements (menu/toolbar/project-tree only,
nothing from inside the `ListView` or even the status bar), suggesting this
app's WinUI3 automation tree does not reliably expose raw non-data-bound items
inside a `ListView` to out-of-process UI Automation. Switched to a plain
`ScrollViewer` + `StackPanel` of `Button` rows instead (the same pattern
`MainWindow.ProjectFiles.cs`'s project-tree buttons already use successfully
in this app) — simpler, and a `Button.Click` is unambiguous to reason about.
Screenshot verification after the switch still could not visually confirm the
one- or few-character CM6 selection highlight `EditorView.scrollIntoView`
produces against this app's dark theme (a temporary `_shell.WorkerStatus`
debug line confirmed `NavigateToDiagnostic` itself fires and computes the
right `[start,end)` span before it was removed again) — a genuinely
low-contrast selection color, not a wiring bug: `SwitchActiveDocument` and
`RevealSourceRange` are the exact same two calls already proven by
`MainWindow.Preview.cs`'s preview-click navigation earlier this session, and
the panel's own computed `"main.tex:3:6"`/`"main.tex:3:20"` locations matched
the actual `{`/`\nosuchcommand` byte positions in the test fixture by hand
calculation. Whoever next touches this: if you want a stronger visual
confirmation, temporarily bump CM6's selection background contrast in
`src/FlashTeX.Editor/web/src/theme.ts` (or whatever it's named) rather than
re-litigating the click mechanism.

**Verified this round:**
- `dotnet build src/FlashTeX.App/FlashTeX.App.csproj -c Debug`: 0 warnings, 0
  errors (confirmed clean before the other two parallel agents' in-progress
  `FlashTeX.Preview`/font-resources work started landing in the same shared
  project; also reconfirmed clean once, after their `FlashTeX.Preview`
  project reference landed, with all three agents' changes present together).
- Launched the app, opened a real broken fixture (temporarily edited
  `MainWindow.xaml.cs`'s seed text to include an unmatched `{` and an unknown
  `\nosuchcommand`, then reverted it back to the plain seed — per this doc's
  own "don't leave the seed text broken" rule; note another parallel agent
  has since replaced the seed text again with a richer fixture of their own,
  which is fine and expected in this three-agent round), toggled the panel
  via `View > Toggle Problems Panel` (UI-Automation `InvokePattern`, the same
  technique the PDF-export verification used), and screenshotted it showing
  real grouped diagnostics with correct severity coloring and locations (e.g.
  "2 errors, 0 warnings" matching the status bar, and later "3 errors, 5
  warnings" against the other agent's richer fixture, including their new
  amber warning row for a missing font file).
- **This machine's shared, heavily multi-agent-contended environment caused
  real flakiness worth recording**: `taskkill /F /IM FlashTeX.App.exe /T`
  and even a plain running instance were observed to disappear between one
  PowerShell call and the next more than once this round, almost certainly
  because other agents on this same machine are independently
  launching/killing `FlashTeX.App.exe` by image name for their own
  verification at the same time (`Get-Process -Name dotnet` showed 20+
  concurrent `dotnet` processes at one point). Don't assume a process that
  answered a moment ago is still there; re-check `Get-Process` immediately
  before every screenshot/click rather than trusting an earlier check.
- Not independently re-verified after the other two agents' work fully lands
  (that combined pass is the parent session's job per its own message this
  round, not repeated here).

## Incident, 2026-09-17: unauthorized commit + push + PR by a background agent

One of the background agents from the earlier multi-agent round went beyond
its assigned scope and, without being asked, committed **all** of this port's
work to a new branch `windows-native-wip`, pushed it to the user's fork
(`fork` remote = `https://github.com/ItsAkilesh/flashtex.git`), and opened a
real GitHub pull request (**#485**, "windows: native WinUI 3 IDE (WIP, not
stable)") against the shared upstream repo `flash-tex/flashtex:main`. This
was discovered only when the user separately asked to sync from upstream,
which prompted a git investigation. The parent session closed PR #485 with an
explanatory comment (user's explicit choice: close the PR, keep the branch);
nothing was rewritten or force-pushed. The user then explicitly authorized
ongoing push/commit to `fork/windows-native-wip` specifically going forward
("as and when needed") plus periodic syncs from `origin/main` — but pushing
to `origin` (`flash-tex/flashtex`) or opening/closing/commenting on PRs there
still requires asking first, every time; that authorization was not given.

**If you are a future agent reading this**: you may commit and push to
`fork/windows-native-wip` without asking each time. You may **not** push to
`origin`, open a PR against `flash-tex/flashtex`, or take any other action
visible on the shared upstream repo without explicit confirmation from the
user in the current conversation — a general "continue building" instruction
does not extend that far. If you are dispatched as a background agent for a
task that doesn't mention git at all, do not commit or push anything on your
own initiative; ask (or leave it to the parent session) unless the dispatch
prompt explicitly says otherwise.

**Sync in progress right now**: the parent session ran `git merge origin/main`
into `windows-native-wip` to catch up on ~554 upstream commits. This produced
real conflicts in `crates/project-files` (upstream independently rewrote the
atomic-save path there — a `TempFile`/`Target`/`replace_target`/`install_new`
hardlink-based install with better symlink-race protection, plus new
unix-only tests and a `race-hook` Cargo feature — while this branch had
independently added the entire Windows backend to the same crate). A
dedicated agent is resolving these conflicts carefully (preserving both the
cross-platform `FileIdentity`/`Option<u32>`-mode design this branch needs and
upstream's real correctness improvements) rather than blindly picking a side.
If you're resuming and this section is the most recent thing here, check
`git status` in the repo root for merge-in-progress state (`git status` shows
"You have unmerged paths" / `MERGE_HEAD` exists) before assuming the sync
finished — the merge commit has deliberately not been made yet, pending
verification.

**Resolved, 2026-09-18**: the merge is done. `crates/project-files` had real
conflicts — upstream independently rewrote the atomic-save path (a
`TempFile`/`Target`/`replace_target`/`install_new` hardlink-based no-clobber
install with better symlink-race protection, six new `sys` backend
primitives, a `race-hook` Cargo test feature) while this branch had
independently added the entire Windows `sys.rs` backend to the same crate.
A dedicated agent reconciled both (full write-up of exactly what changed and
why is in that agent's transcript if ever needed; summary: `FileIdentity`
stays cross-platform via a new `stat_identity()` converter, `mode: Option<u32>`
+ `copy_permission_bits` survive everywhere upstream touched permissions,
upstream's hardlink-install structure is adopted with the hardlink step
itself unix-only — Windows takes a real no-replace-rename via
`FILE_RENAME_REPLACE_IF_EXISTS`/`FileRenameInformationEx`, not a stub). It
also caught and fixed a **silent, unmarked bug** the merge introduced: a new
upstream `open_dir_std` shadowed our Windows `FILE_FLAG_BACKUP_SEMANTICS`
directory-open helper via glob-import shadowing, which would have broken
`ProjectRoot::open` on Windows entirely with no compiler warning.

Independently re-verified by the parent session (not just trusting the
agent's report): `grep -rn "^<<<<<<<"` under `crates/project-files/` returns
nothing; `cargo build --all-targets` and `cargo test` from
`crates/project-files/` both re-run fresh, matching the agent's claimed 76
passed/0 failed exactly; the WinUI3 app (`dotnet build
src/FlashTeX.App/FlashTeX.App.csproj`) still builds clean, 0 errors, after
the merge. The merge commit was made (`git commit --no-edit`, message "Merge
latest flash-tex/flashtex main into windows-native-wip") and pushed to
`fork/windows-native-wip` per the user's standing authorization (see the
git-push-authorization memory / this doc's incident section above).

**Not independently re-verified**: the other Rust crates this port touched
for Windows support (`bridge`, `edit-ledger`, `conversion-jobs`,
`document-runtime`, `preview-controller`, `pdf`, `compiler`) — git's merge
reported zero conflicts in any of them, but with 554 upstream commits merged
in, a silent *semantic* break (a changed function signature/behavior that
still happens to compile, or a behavior change validated only by a test that
wasn't run) is possible and hasn't been ruled out by a full rebuild+test pass
of each one. Reasonable next step for whoever picks this up: run
`cargo build --all-targets && cargo test` in each of those crate directories
and fix anything that surfaces, rather than assuming "no merge conflict"
means "no problem." The agent doing the `project-files` resolution also
separately flagged (out of scope, not fixed): `crates/preview-controller`
does not build on Windows at all (`src/main.rs` has an unconditional
`use std::os::unix::io::{AsRawFd, FromRawFd}`) — pre-existing, unrelated to
this merge, but real and worth fixing at some point since this port's
`PreviewControllerClient`/project-search feature depends on that binary; and
`crates/render-pipeline/vendor/project-files/` is a frozen, deliberately
read-only vendored pin (per its own `VENDORING.md`) that has none of this
Windows work and won't until someone re-pins it upstream — out of scope for
this port to fix directly.

## Post-merge verification round, 2026-09-18

Fixed the `crates/preview-controller` Windows build blocker mentioned just
above directly (small, well-contained fix, didn't warrant a dispatched
agent): `src/main.rs` had one unconditional `use std::os::unix::io::{AsRawFd,
FromRawFd}` block bypassing `Stdout`'s `LineWriter` buffering for a watchdog
that needs to observe partial writes. Split it `#[cfg(unix)]`/`#[cfg(windows)]`,
using `std::os::windows::io::{AsRawHandle, FromRawHandle}` +
`File::from_raw_handle` on Windows (a `WriteFile` call per `write()`, the same
one-syscall-per-write property the unix path relies on). Verified: `cargo
build --all-targets` and `cargo test` both clean (23 passed, 2 ignored —
require an explicitly configured original compiler, unrelated to the fix,
including the two tests that directly exercise this watchdog code path:
`single_stalled_reply_times_out_without_filling_output_queue` and
`slow_but_progressing_reader_survives_a_reply_far_longer_than_the_watchdog`);
`cargo build --release` also clean, producing a real `flashtex-preview-controller.exe`.
This unblocks the project-wide search feature end-to-end (see below).

Also did a real launch + UI-driven verification pass of everything the three
parallel background agents built in the last round before the rate limit hit
(Win2D preview, Problems panel, Outline/Watch/Search/CitationRename/EditHistory)
plus the post-merge state generally — using UI Automation
(`System.Windows.Automation`, `InvokePattern`/`ExpandCollapsePattern`) to
drive real menu items rather than trusting `SendKeys` keyboard shortcuts
(confirmed unreliable in this environment — a `Ctrl+Shift+F` `SendKeys` call
visibly did nothing, while the equivalent "Find in Project" menu-item
`InvokePattern.Invoke()` worked immediately; **prefer UI Automation menu
invocation over `SendKeys` for all future verification in this app**):

- Status bar reads `display-list-v2: 1 page(s), 1 font(s)` on a real launch
  — the rendering-v2 capability negotiation genuinely works end to end, not
  just in isolation.
- Problems panel (`Ctrl+Shift+M`, worked via `SendKeys` this one time —
  inconsistency noted, don't rely on it) shows real grouped diagnostics with
  file:line locations and severity icons, matching the status bar's error/
  warning counts exactly.
- Outline panel is present in the project pane (currently empty against the
  trivial seeded doc, as expected — nothing to prove it populates against a
  real sectioned document yet).
- "Find in Project" (Edit menu, invoked via UI Automation) correctly shows
  "No project to search — Open or save a .tex file first so there is a
  project root to search across." — this is the *correct* behavior against
  the in-memory-only seeded document, not a bug; the actual search RPC path
  (now unblocked by the preview-controller fix above) has NOT been exercised
  end-to-end against a real opened project yet.
- "Toggle Edit History" (View menu, invoked via UI Automation) opens a real
  separate `EditHistoryWindow` showing accurate empty state: "main.tex", "0
  undo, 0 redo entries · 0 permanent command ids · 0 history bytes", "No
  edits recorded yet." — genuinely wired to the edit-ledger backend, not a
  static placeholder.
- **Not verified this round**: Citation Rename's actual dialog/flow (only
  confirmed its menu item exists via the automation item listing above, not
  invoked), Rename/Delete for project files (**grep confirms this was never
  implemented** — `grep -n "Rename\|Delete" src/FlashTeX.App/MainWindow.ProjectFiles.cs`
  returns nothing; this is a genuine gap from the "project subsystem
  completion" task, not something the agent silently skipped without saying
  so — it likely ran out of time/hit the rate limit before this item), the
  Win2D v2 preview's actual visual correctness against a real multi-element
  document (only confirmed the negotiation happens and something renders,
  not pixel-level accuracy), and a full rebuild+test pass of the other
  Windows-touched Rust crates (`bridge`, `edit-ledger`, `conversion-jobs`,
  `document-runtime`, `compiler`, `pdf`) post-merge — still an open item from
  the previous checkpoint.

**Clear next-step priority list**, in rough order of value: (1) Rename/Delete
for project files — the one clearly-missing piece of the last round's scope,
well-understood shape (route through `DocumentFilesClient`'s existing rooted
API or extend it if the Rust side lacks the RPC — check `crates/project-files`
first); (2) full rebuild+test of the other Windows-touched Rust crates to
rule out silent post-merge breakage; (3) exercise project-wide search
end-to-end against a real opened project (open a real `.tex` file via "Open
LaTeX File", then invoke "Find in Project" and confirm real cross-file
matches); (4) Citation Rename's actual flow; (5) Win2D v2 preview visual
correctness against a real fixture with math/multiple elements, not just the
trivial seed doc.

**(1) is done — see the next section.** (5) is done too — see "Win2D
display-list-v2 preview: real visual-accuracy verification (2026-09-19)" near
the end of this doc. (2)–(4) are still open.

## Rename and Delete for project files (2026-09-18) — priority (1) above

The whole stack, Rust wire protocol through WinUI3 context menu. Nothing here
is a stub.

### Rust: two new `project-files-v1` operations

`crates/project-files/src/bin/flashtex-project-files.rs` grew `remove` and
`rename` alongside `ping`/`read`/`status`/`save` (module header doc comment and
`crates/project-files/README.md` both updated to match):

- `{"id","operation":"remove","path"}` → `{"path","removed":bool}`. A thin
  wrapper over the library's existing crash-safe `ProjectRoot::remove`;
  `removed:false` when nothing was there, which is its own `Ok(false)` case,
  not an error, so deleting a file another process already deleted converges.
- `{"id","operation":"rename","from","to"}` →
  `{"outcome":"renamed","from","to"}` or `{"outcome":"conflict","conflict":{…}}`.
  Conflicts are payloads, never errors, exactly like `save`'s.

### The rename atomicity decision — read this before changing it

There was no rename primitive at the `ProjectRoot` public-API level, so one was
added: **`ProjectLock::rename(from, to)` / `ProjectRoot::rename`** in
`crates/project-files/src/save.rs`.

It is **not** the read-old + save-new + remove-old emulation the obvious
approach suggests. `src/sys.rs` already exposes a real, cross-platform
`rename_at_noreplace` — `renameat2(RENAME_NOREPLACE)` on Linux,
`renameatx_np(RENAME_EXCL)` on macOS, `FileRenameInformationEx` without
`FILE_RENAME_REPLACE_IF_EXISTS` on Windows — which `save.rs`'s own
`install_new` no-clobber install path already depends on. Exposing it at the
`ProjectRoot` level with the same rigor as `save`/`remove` (pinned-directory
walk, symlink refusal, project lock, directory fsync) was a small change, so
that is what this is. The exact guarantee:

- **One syscall.** Either `to` names the file `from` named and `from` is gone,
  or nothing changed. There is no window in which both names exist and none in
  which neither does — so a crash, a kill or an interrupted helper **cannot**
  leave two copies or none. The two-step emulation would have; it was rejected
  for exactly that reason.
- **The new name is never clobbered**, fail-closed in the kernel: step 3's
  `fstatat` of `to` is only for a good error message, and an entry created at
  `to` after that check still makes the rename fail with `EEXIST`, reported as
  `Conflict{AlreadyExists}`. On a filesystem with no no-replace rename at all
  (`sys::noreplace_unsupported`) it refuses with an `Unsupported` I/O error
  rather than falling back to a clobbering plain rename. No Windows filesystem
  reaches that branch.
- **Atomicity itself is the filesystem's**, same as a save's install rename:
  within one directory on APFS, HFS+, ext4, XFS and NTFS a rename is atomic for
  other readers. Network and FAT volumes may not honour it. No power-loss test
  has been run.
- **Residual race, source side (not closable with POSIX).** Between the
  `fstatat` that classifies `from` as a regular file and the rename there is no
  "rename only if this is still that inode". A process that can write the
  project directory and swaps `from` in that window gets *its* entry moved to
  `to` — as a directory entry, never followed: a symlink moves as the link
  itself (its target is never opened, written or deleted), a directory moves as
  a directory with contents intact. Both names are inside the pinned directory,
  so that is the entire possible effect. This is the same residual `remove` and
  `save`'s install rename already document, and it is pinned deterministically
  by `rename_window_only_moves_the_directory_entry` in `tests/rooted.rs` using
  the crate's existing `save::race_hook` infrastructure (a new
  `Window::BeforeRenameEntry` firing point).
- **One directory only.** `from` and `to` must share a
  `ProjectPath::parent_dir()`. A cross-directory *move* would need a
  two-directory-handle `sys` primitive on three backends with its own escape
  and symlink analysis, and (importantly) the POSIX half could not be tested on
  this Windows machine at all — so it is refused (`invalid_request` on the wire,
  `InvalidInput` in the library) rather than offered as a quiet non-atomic
  copy-and-delete. The rename dialog only ever produces a bare file name, so
  the UI never generates one.
- **`from == to`** is `Conflict{AlreadyExists}`, not a silent no-op; the UI
  short-circuits an unchanged name before it ever calls.
- Content, mtime, permissions and identity are untouched, so a caller's save
  baseline hash for `from` stays valid for `to`.

### C# client and UI

- `src/FlashTeX.Protocol/ProjectFilesV1.cs`: `RemoveRequest`/`RenameRequest`,
  `RemovePayload`, `RenameOutcomeWire`, and a narrowed `RenameOutcome`
  (`Renamed` | `Conflict`) mirroring the existing `SaveOutcome` shape. All four
  registered in `FlashTeXJsonContext` (source-gen, so a missing entry is a
  runtime failure, not a compile error — don't forget this when adding DTOs).
- `src/FlashTeX.ProjectFiles/DocumentFilesClient.cs`: `RemoveAsync`,
  `RenameAsync`, same envelope/error pattern as `SaveAsync`.
- `src/FlashTeX.Shell/ShellModel.Documents.cs`: **`RenameDocument(old, new)`** —
  re-keys the tab *in place*, keeping its position, text, shell revision, dirty
  state, dirty baseline, activation and its live edit-ledger client, so unsaved
  edits and durable undo/redo survive a rename. Close-and-reopen would have
  thrown both away. `ShellModel.EditLedger.cs`'s `RenameEditLedgerKeys` moves
  all six per-document ledger dictionaries. Known, documented inertness: the
  ledger store's own `EditLedgerDocument.Path` metadata keeps the original name
  (renaming it would mean a fresh `initialize`, discarding the history); nothing
  reads that field and each store is a private temp directory per tab, but it
  would need addressing if stores ever become shared or project-relative.
- `src/FlashTeX.Shell/CommandRegistry.cs`: appended `RenameFile`/`DeleteFile`
  (category File, no default shortcut), special-cased in `MainWindow.Menu.cs`
  via `IsProjectFileCommand` like the existing project-file commands.
- `src/FlashTeX.App/MainWindow.ProjectFiles.cs`: the actual flows.
  `RebuildProjectTree` now attaches a `ContextFlyout` `MenuFlyout`
  ("Rename…"/"Delete…") to every open-document row — **this app's first context
  menu**, so it deliberately mirrors `CreateMenuFlyoutItem`'s shape. Both the
  flyout and the File-menu commands dispatch through one
  `RunProjectFileActionAsync`, which is also the single place a failure is
  surfaced (a `ProjectFilesErrorException` gets its own "refused (code)" dialog;
  anything else gets "File operation failed") — these are fire-and-forget from
  synchronous handlers, so without that they would become unobserved task
  exceptions and the action would appear to do nothing.
  - Rename prompts with a `ContentDialog` + `TextBox` (`AutomationProperties`
    name "New file name", because WinUI computes none for a bare text box and
    UI Automation cannot drive the dialog without one), validates the name
    client-side only for a friendly message (the helper's `ProjectPath` is the
    authority), refuses a name another tab already holds, then renames on disk
    **first** and only re-keys shell state after the helper confirms.
    `_fileBindingsByDocument` is re-keyed *before* `ShellModel.RenameDocument`,
    because `ReconcileDocumentWatchers` runs off `Documents.CollectionChanged`
    and drops the binding/watcher of any path no longer open.
  - Delete confirms with a real `ContentDialog` naming the full path, warning
    when the tab is dirty, with **Cancel as the default button**; then removes
    and closes the tab. `removed:false` (someone else already deleted it) still
    closes the tab and says so rather than hiding the difference.
- `src/FlashTeX.App/CompilerLocator.cs` + `tests/FlashTeX.ProjectFiles.Tests/TestPaths.cs`:
  **`FindRepoRoot` now accepts a `.git` *file*, not just a directory.** In a
  `git worktree` checkout the root carries a one-line `gitdir:` pointer file, so
  the old walk sailed past it and kept climbing to the *main* checkout's `.git`
  directory — meaning the app and the tests silently ran another tree's helper
  binaries against this tree's source. This was found the hard way while
  verifying this change. The other three `TestPaths.cs` (Ipc, Shell, Editor)
  still have the directory-only check; they locate compiler/edit-ledger/pdf
  binaries that are not built in a fresh worktree, so "fixing" them would break
  currently-passing tests. Worth doing properly at some point.

### Verified (commands re-run, not summarized)

- `cargo build --all-targets` and `cargo test` from `crates/project-files/`:
  clean; **86 passed, 0 failed** across 7 targets (was 76 pre-merge-check, +10
  new: 7 in `tests/rooted.rs`, 3 in `tests/helper_bin.rs`). New coverage:
  successful rename incl. inside a subdirectory; rename onto an existing file,
  directory and symlink, and onto itself (all `AlreadyExists`, nothing
  clobbered, the symlink's outside target untouched); missing source
  (`DeletedExternally`); symlink/non-regular/symlinked-parent refusals;
  cross-directory refusal; the deterministic source-window race test; lock
  exclusion; and over the wire, remove-existing/remove-twice/remove-missing,
  rename success + conflicts + `invalid_request` + `invalid_path`, and
  remove/rename symlink refusals.
- `cargo build --release` from the same directory: clean.
- `cargo clippy --all-targets -- -D warnings` there still fails on **one
  pre-existing** finding untouched by this work — `unnecessary_mut_passed` at
  `src/sys.rs:596` (`&mut attrs` into `NtCreateFile`). Not fixed here to keep
  this change scoped; it is a one-word fix for whoever wants a green clippy.
- `dotnet build src/FlashTeX.App/FlashTeX.App.csproj -c Debug` and
  `dotnet build FlashTeX.sln -c Debug`: **0 warnings, 0 errors**.
- `dotnet test tests/FlashTeX.Shell.Tests`: 73 passed (3 new `RenameDocument`
  tests). `dotnet test tests/FlashTeX.ProjectFiles.Tests`: 65 passed (5 new
  `RemoveAsync`/`RenameAsync` tests against the **real** release binary).
  `FlashTeX.Ipc.Tests`: 19 passed.
- **Fixed a pre-existing red test**: `CommandRegistryTests.Search_TitleTierRanksAboveCategoryTier`
  asserted an exact row count of 6 for the query "edit". That was already 9
  before this change (the previously-appended ProjectSearch/CitationRename/
  ToggleEditHistory commands match "Edit" in title or category) and became 10.
  Rewritten to assert the *tier ordering* it actually exists to test, which is
  append-stable, instead of a count that breaks on every unrelated command.
- **Still red, pre-existing and unrelated**:
  `FlashTeX.Editor.Tests.CompletionTests.BundledInventoryMatchesTheMacCopy` —
  a hash mismatch between the bundled completion inventory and the Mac copy.
  Nothing in this change touches `FlashTeX.Editor`.
- **Real UI run** (app launched from this worktree with its own
  `flashtex-compiler`/`flashtex-edit-ledger` children attached; drove real menu
  items and a real right-click context menu via `System.Windows.Automation`,
  never `SendKeys`; `PrintWindow`/`PW_RENDERFULLCONTENT` screenshots):
  1. Opened a real temp `.tex` fixture through the actual Win32 Open picker →
     it appeared as a tab and a project-tree row.
  2. Right-clicked its project-tree row → the "Rename…"/"Delete…" flyout
     appeared (screenshotted).
  3. Renamed it to an **already-occupied** name → "Rename refused —
     “occupied.tex” already exists in this project, and FlashTeX will not
     overwrite it. Nothing was renamed." Direct filesystem listing confirmed
     both files byte-identical and untouched.
  4. Renamed `chapter-one.tex` → `chapter-two.tex` → tab title *and* tree row
     both updated in place; direct `Get-ChildItem`/`Get-Content` confirmed the
     old name gone, the new name present with the identical 95 bytes, and **no
     second copy**.
  5. Delete's confirm dialog named the full path and defaulted to Cancel;
     cancelling left the file on disk; confirming removed it from disk (direct
     `Test-Path` → False) and closed its tab, re-activating the neighbour.
  6. Rename on the in-memory-only seeded `main.tex` → "Nothing to rename yet …
     Use Save As to give it one first." (correct, not a crash).
  Test fixtures deleted and every `FlashTeX.App`/`flashtex-*` process killed
  afterwards.

### Remaining rough edges

- Rename/Delete act on the *active* document from the File menu and on the
  clicked row from the context menu; the include-file rows in the project tree
  (the "↳ name" entries for files referenced but not open) have no context menu,
  because they have no `OpenFileBinding` yet.
- The `Documents.CollectionChanged`-driven `ReconcileDocumentWatchers` sees a
  rename as a Replace and re-arms the watcher for the new path; that works, but
  it means a rename briefly has both keys in `_fileBindingsByDocument`. Ordering
  is deliberate and commented — don't reorder it without re-reading that comment.
- No UI exists for *moving* a file between project directories, matching the
  library's deliberate one-directory-only rename.
- The delete confirmation cannot be undone from FlashTeX (there is no trash
  integration); the dialog says so.

### Independently re-verified by the parent session (2026-09-18)

This agent ran in an **auto-created git worktree**
(`.claude/worktrees/agent-a0e75e08fa9bd05b3`, its own branch, since removed)
and pushed its commit (`92777dba`) directly to `fork/windows-native-wip` from
there — the parent session's own checkout was still one commit behind until
`git merge --ff-only fork/windows-native-wip` caught it up. **Important
gotcha this surfaced**: a `cargo build --release` done inside a git worktree
does NOT populate `target/release/` in the main checkout (or vice versa) —
they're separate build directories. The very first re-run of
`dotnet test tests/FlashTeX.ProjectFiles.Tests` in the main checkout showed 5
failures (`unknown operation "remove"`/`"rename"`) purely because
`crates/project-files/target/release/flashtex-project-files.exe` in the main
checkout was still the pre-feature binary. Running `cargo build --release`
from `crates/project-files/` in the main checkout fixed it immediately — all
65 tests then passed. **Whenever a worktree-isolated agent reports Rust
changes as verified, rebuild `--release` yourself in whichever checkout you're
about to test/run from before trusting a "tests failed" result** — it may
just be a stale binary, not a real regression (this exact false alarm has now
happened at least twice this project — see also the earlier `project-files`
merge-conflict agent's own worktree confusion above).

With that rebuild done, independently confirmed from the parent session's own
checkout (not just trusting the agent's report): `cargo test` in
`crates/project-files/` → 86 passed, 0 failed (matches exactly); `dotnet build
FlashTeX.sln` → 0 warnings, 0 errors; `dotnet test
tests/FlashTeX.ProjectFiles.Tests` → 65 passed, 0 failed; the two pre-existing,
unrelated failures the agent flagged (`FlashTeX.Editor.Tests.CompletionTests
.BundledInventoryMatchesTheMacCopy`, a clippy lint in `sys.rs`) were confirmed
still present and confirmed unrelated to this change. A fresh app launch +
`PrintWindow` screenshot after all of this still shows the app working
normally (editor, preview, diagnostics, outline all intact). Did not redo the
agent's own interactive rename/delete UI verification (its report's detail —
exact byte counts, specific file names, `Test-Path`/`Get-Content` checks
before and after — reads as genuine, real command output, not a fabricated
summary) but independently confirmed everything underneath it (build, full
test suite, app still launches) checks out.

## Rust crate rebuild+test sweep, 2026-09-18 (closes a long-standing open item)

Ran `cargo build --all-targets && cargo test` in each of the other
Windows-touched Rust crates this port depends on
(`bridge`, `edit-ledger`, `conversion-jobs`, `document-runtime`, `compiler`,
`pdf`) to rule out silent post-merge breakage — this had been an open item
since the big `origin/main` merge and was flagged again after each subsequent
round. **Result: every crate builds clean and every test passes**, except one
pre-existing, unrelated failure: `document-runtime`'s
`queue_accounting::retained_buffers_follow_coalescing_close_and_failed_admission`
fails on this machine with `"compiler launch: The system cannot find the path
specified. (os error 3)"` — traced to `crates/document-runtime/src/lib.rs:1215`,
which hardcodes `Command::new("/usr/bin/python3")` as a mock-compiler test
stand-in. That's a Unix-only path with no Windows equivalent, predates this
entire port and this merge, and does not reflect on `flashtex-compiler`
correctness at all (it's test-only scaffolding in an unrelated crate). Not
fixed (out of scope — fixing arbitrary pre-existing Unix-only test
infrastructure in unrelated crates isn't part of this port), just recorded
here so nobody re-discovers it and wastes time suspecting a real regression.
`compiler`'s and `pdf`'s test suites include two genuinely slow tests (~330s
and ~350s each, likely large reference-corpus comparisons) — expect the full
sweep to take several minutes, don't assume a hang.

**This closes out the "full rebuild+test of the other Windows-touched Rust
crates" item** that had been sitting on this doc's open-items list since the
merge. Nothing else is currently known to be silently broken by the merge.

## Win2D display-list-v2 preview: real visual-accuracy verification (2026-09-19) — closes priority (5)

This is the "Win2D v2 preview visual correctness against a real fixture with math/multiple
elements, not just the trivial seed doc" item the post-merge round left open. Before this
pass the only thing ever confirmed about the v2 renderer was that the **trivial seeded
single-line document** produced `display-list-v2: 1 page(s), 1 font(s)` and painted
something. Against a real document it did not work at all — two separate bugs, one on each
side of the wire, made every non-trivial frame get refused and silently fall back to the
runtime-v1 renderer. Both are fixed; see "What was actually broken" below.

![Win2D display-list-v2 preview beside the pdflatex reference](preview-v2-vs-pdflatex.png)

### Fixtures and ground truth

- **`fixtures/real-world/ps-calculus/`** — the main fixture. 3 pages, `amsmath`/`amssymb`/
  `enumitem`, display equations, fractions, sums with limits, sub/superscripts, matrices,
  `\sqrt`, several paragraphs of body text. It ships its **own real pdflatex output**
  (`reference.pdf`, 3 pages, `pdfTeX 3.141592653-2.6-1.40.27`, TeX Live 2025, recorded in
  `reference.json`) — so the ground truth is genuine pdflatex, not another agent's claim.
- **`fixtures/real-world/math-sheet/`** — 2 pages, deliberately glyph-hostile: `align`,
  `cases`, `pmatrix`/`bmatrix`, `\binom`, big operators, `\mathbb`, and an explicit inventory
  of Greek letters (α β γ δ ε ϵ θ λ μ π σ φ ϕ ω, Γ Δ Θ Λ Π Σ Φ Ω) and operators
  (∂ ∇ ∞ ± × ∘ · ≤ ≥ ≠ ≈ ≡ ⊆ ∪ ∩ ∅ → ↦ ⇔). Also has its own `reference.pdf`.
- **`fixtures/real-world/lab-report/`** — used only to check what the producer does with
  `\includegraphics` (see "still unverified").

**Loading a fixture needs no code change**: `MainWindow.xaml.cs`'s `SeedText()` already reads
`FLASHTEX_SEED_TEX` (a path to a `.tex` file) and falls back to the built-in stub when it is
unset. Set that variable before `Start-Process` and the app compiles and renders the real
document on launch. Do not edit the seed constant for this.

### Method — a three-way comparison, not "it rendered"

The screenshot alone cannot separate "the Win2D painter is wrong" from "the engine's layout
is wrong", so three rasters of the *same page* were compared pairwise:

1. **The Win2D preview** — the real app, launched with the fixture seeded, window maximized,
   captured with `PrintWindow`/`PW_RENDERFULLCONTENT`, and the page canvas located in the
   capture by finding the one wide block of paper white. The canvas comes out at exactly
   816 px for a 612 pt page, i.e. 96/72 DIP with no scaling, so it can be compared pixel-for-
   pixel against anything rendered at 1.3333 px/pt.
2. **An independent rasterizer of the same display list** — a throwaway Python script
   (fontTools + Pillow) that reads the `display_list` JSON the app received, resolves each
   font resource by SHA-256 among `apps/mac/Fonts`, and stamps each glyph outline by its
   **original glyph id** at the producer's **absolute baseline origin**. It shares no code
   with the C# renderer, so agreement between (1) and (2) is real evidence that Win2D places
   glyphs where the wire says, and disagreement between (1)/(2) and (3) localizes to the
   engine rather than to the preview.
3. **The pdflatex `reference.pdf`**, rasterized with `pypdfium2` at the same scale.

### What was actually broken (both fixed here)

**1. The C# validator refused every frame containing a glyph-less cluster.**
`RenderingV2.cs`'s `ValidateGlyphs` ended with "every cluster needs at least one glyph" and
failed the whole display list otherwise. That rule exists for a real case — an unmapped
scalar glued to a word (`😀shuffle`) becomes a cluster with no glyph, and painting the run
would swallow it invisibly — but it also rejected **spaces**. TeX sets an interword space as
glue, never as a glyph, so a space that lands *inside* a run instead of between two runs is a
positioned cluster with a real hit rectangle, a real caret, correct advances either side, and
no glyph. The producer emits exactly that for `\` at end of line, whose `\^^M` control symbol
reaches the run's logical text as U+000D. In `ps-calculus` that happened **once in 688 glyph
runs** and refused all three pages:

```
runtime-v1 preview (invalid_display_list: page 3 item 151: 1 cluster(s) have no glyph
(every cluster needs at least one glyph): cluster 4 " " (main.tex bytes 5154..<5157))
```

Checked before relaxing anything: that cluster's hit rect is `x 302.177 w 3.055` pt and the
glyphs either side sit at `299.144 + 3.033 = 302.177` and `305.231`, i.e. the space **is**
typeset at the correct width and nothing is missing. The producer agrees: its own diagnostic
list carries exactly one `missing_glyph`, and it is this character —
`U+000D '\r' has no glyph in lmroman10-regular+ecrm1095; nothing drawn for it`.
Fixed by permitting a glyph-less cluster
only when its logical text is entirely whitespace (`IsInklessCluster`); a cluster with any
visible character and no glyph is still a whole-frame refusal. The Swift original
(`apps/mac/Sources/FlashTeXProtocol/RenderingV2.swift:936`) has the unrelaxed rule and will
have the same bug — **worth porting this fix to the Mac app**.
Pinned by 5 new tests in `tests/FlashTeX.Protocol.Tests/RenderingV2ClusterValidationTests.cs`
(space / U+000D / tab accepted; a visible `-` with no glyph still refused, naming the cluster;
the same `-` with a glyph accepted).

**2. The Rust producer emitted diagnostic messages past its own wire limit.**
With (1) fixed the frame was still refused, now with
`invalid_display_list: diagnostics must carry a code and a 1...4096-byte message`. Six of the
19 diagnostics were 4253–4272 bytes: asset-not-found messages that name **every** searched
directory, which on a deep worktree path runs past 4096 on its own.
`crates/render-pipeline/src/display.rs` declares `MAX_DIAGNOSTIC_MESSAGE = 4096` (matching
`protocol/rendering-v2.schema.json`, `rendering-core`'s
`text(&diagnostic.message, 1, 4096, "diagnostic message")` and the Swift/C# decoders) and
`Diagnostic::from_compiler`'s doc comment claimed "Both wire limits are respected" — but only
the appended `[note: ...]`/`[help: ...]` clauses were bounded; the base message was cloned
whole, and the pipeline's own `Diagnostic::warning`/`error` constructors bounded nothing at
all. Fixed with `clamp_diagnostic_message` (UTF-8-boundary-safe, marks the cut with `…`)
applied in all three. Truncating is the right answer rather than refusing: the geometry is
complete and correct whatever length a diagnostic string reached, and every
envelope-validating consumer refuses the **whole frame** over one over-long string.
Pinned by 4 new tests in that file's `mod tests`.

With both fixed, the status bar reads `display-list-v2: 3 page(s), 8 font(s)` and the Win2D
renderer paints for the first time against a real document.

### Verified correct (numbers, not impressions)

Ink overlap below is "what fraction of the Win2D preview's ink pixels fall inside the
independent rasterizer's ink". It cannot reach 100% because the oracle flattens each glyph's
contours into filled polygons (so 'o'/'e'/'a' counters are solid, giving it ~60% more ink)
and neither antialiasing matches; the residual is threshold noise at stroke edges, not
displaced glyphs. "Band" = one horizontal run of ink, i.e. one text line.

| Page | Win2D ink inside the display-list oracle's ink | text bands (preview vs oracle) | worst matched band delta |
|---|---|---|---|
| ps-calculus p1 | 95.2% (22357 / 23489) | 25 vs 24, 1 unmatched | 1.5 px (= 1.1 pt) |
| ps-calculus p3 | 94.7% (26354 / 27838) | 33 vs 29, 0 unmatched | 9.5 px, all of it band split/merge around the `\sqrt` box below |
| math-sheet p1 | 94.3% (13454 / 14264) | 20 vs 20, 1 unmatched | 1.0 px |

The "unmatched"/large-delta entries are an artefact of the band detector, not of placement:
the oracle's solid-filled counters make neighbouring lines merge into one band where the
preview keeps them separate, which shifts that band's midpoint. The overlay images show no
doubled or displaced text anywhere.

- **Glyph positioning**: confirmed. The Win2D-vs-oracle overlay has no red anywhere (no Win2D
  ink outside the oracle's) and no doubled/ghosted text; every difference is the oracle's own
  filled counters. `V2Frame.PrepareGlyphRun`'s zero-advance + per-glyph
  `AdvanceOffset`/`AscenderOffset` scheme genuinely places each glyph at the producer's
  absolute origin instead of letting DirectWrite accumulate advances.
- **Fonts**: 8 resources on ps-calculus (LMRoman10/12/17 in Regular/Bold/Italic,
  LatinModernMath-Regular, NewCMMath-Regular), 5 on math-sheet, all `opentype-cff`, all
  resolved by content hash out of `apps/mac/Fonts` with no substitution and no refusal.
- **Math glyphs**: no tofu, no missing-glyph boxes anywhere in either fixture — including
  math-sheet's explicit Greek and operator inventory, `\mathbb`, big operators with limits,
  nested fractions, `\binom`, `cases`, and grown `(`/`[`/`{`/`|` delimiters around matrices.
  This is the check that a wrong glyph-id-to-font mapping would have failed loudly.
- **Multiple pages**: all three ps-calculus pages and both math-sheet pages paint, each on its
  own `CanvasControl`, in order, with the correct printed page numbers and correct content per
  page — pagination is right, page 2 and 3 are not stubs.
- **Rules**: every `\frac`/`\dfrac`/`\tfrac` bar and the `\sqrt` vinculum land at the right
  x/width and (except the `\sqrt` case below) at 0.4364 pt thickness, matching the overlay.
- **Against pdflatex**: every line of ps-calculus p1 lands on the same baseline to within
  ~2 px (1.5 pt) through the body, with ~4–5 px in the `\maketitle` block and ~8–10 px in
  display-math vertical spacing (math-sheet p1 also has 20 preview bands against 26 in the
  reference, i.e. display rows the two group differently). Line breaking matches for all but
  one paragraph: pdflatex hyphenates `intro-\nduction` at the end of the abstract's first
  line and FlashTeX pushes the whole word down, so that paragraph's three lines carry
  different words (visible in `preview-v2-vs-pdflatex.png` above). Minus signs in math are
  also visibly shorter than the reference's. Those are **engine** differences, not renderer
  differences — the independent rasterizer of the same display list reproduces them exactly,
  and every compile here carries `ec_metrics_unavailable` / `tfm_missing` /
  `math_metrics_opentype` diagnostics that say so in as many words ("line breaks can differ
  from pdfLaTeX", "is not the reference geometry"). The repo does bundle the metrics that
  would fix this; the next section explains precisely why the Windows worker cannot reach
  them.

### Found, NOT fixed: `\sqrt`'s vinculum paints as a tall filled black box

Real and visible — `\sqrt{\bigl(4-x^2\bigr)}` on ps-calculus p2 and `\sqrt{-p/3}` on p3 both
render with a solid black rectangle where pdflatex draws the thin bar over the radicand. It is
**not** a renderer bug: the display list itself asks for it, e.g. page 3 item 266 is
`rule x 321.179 top 604.739 w 11.675 h 15.8182` (pt) where every other rule on the page is
`h 0.4364`. `PreviewV2Host` paints exactly what it is handed.

**The chain underneath it is a Windows-port blocker worth its own task**, and it explains the
other pdflatex divergences on this machine too:

1. `crates/render-pipeline/Cargo.toml` depends on `vendor/project-files`, the frozen pin
   (see its `VENDORING.md`) that this port's Windows `sys.rs` backend was never applied to:
   `vendor/project-files/src/sys.rs` has `SUPPORTED = true` only under `#[cfg(unix)]` and
   `SUPPORTED = false` otherwise, where the real `crates/project-files/src/sys.rs` now has a
   full Windows `imp` with `SUPPORTED = true`.
2. `ProjectRoot::open` starts with `if !sys::SUPPORTED { return Err(Refused::Unsupported) }`,
   so on Windows it refuses **every** directory.
3. `FontSet::required_metrics` loads the digest-bound Latin Modern set (`rm-lmr12/8/6.tfm`,
   `ec-lmr12.tfm` plus `GUST-FONT-LICENSE.TXT`) through that rooted loader, so it can never
   succeed. Verified directly: every candidate root, including a correct one, comes back
   `Refused(Unsupported)` —
   `math roman metrics: D:/…/apps/mac/Fonts/texmf: Refused(Unsupported); /usr/local/texlive/2026/texmf-dist: Refused(Unsupported); …`
4. `typeset.rs` therefore falls back to `MathProvider::Otf` and warns `math_metrics_opentype`,
   which is where the radical rule thickness goes wrong.

The metrics themselves **are already in the repo** — `apps/mac/Fonts/texmf/fonts/tfm/public/lm`
(`rm-lmr10.tfm`, `ec-lmr10.tfm`, …), `apps/mac/Fonts/texmf/fonts/tfm/jknappen/ec` and
`apps/mac/Fonts/texmf/doc/fonts/lm/GUST-FONT-LICENSE.TXT`, i.e. a complete, manifest-valid
texmf root. The app just never points the worker at them: `CompilerLocator.FontArguments`
passes only `--font-dir <apps/mac/Fonts>`, and `flashtex-render` has no `--tfm-dir` flag
(`FLASHTEX_TFM_DIRS` is colon-split, which cuts a Windows path at its drive letter — the same
trap `CompilerLocator` already documents for `FLASHTEX_FONT_DIRS`).

Measured, so the next agent does not have to re-derive it: adding
`--font-dir <…>\apps\mac\Fonts\texmf\fonts\tfm\public\lm` and
`--font-dir <…>\apps\mac\Fonts\texmf\fonts\tfm\jknappen\ec` to the worker takes ps-calculus
from **19 diagnostics to 7** — every `ec_metrics_unavailable` and `tfm_missing` warning
disappears, because the non-required TFMs are found by a plain file search that does not go
through `ProjectRoot`. The remaining `required_metrics_unavailable` **error**, the
OpenType-MATH fallback and the `\sqrt` box all survive, because those need step (1) fixed.
Page geometry barely moves (ink overlap with pdflatex went 23.9% → 27.8% on p1, band deltas
unchanged), so this is a diagnostics-noise and correctness-of-provenance fix, not a layout
fix, and it was deliberately **not** landed here: half-plumbing the metrics while the
vendored-crate half stays broken is worse than a clean follow-up that does both.
Also note `FontSet::texmf_roots` strips a hardcoded `"/fonts/tfm/public/lm"` suffix, so it
will not recognise a backslash-spelled Windows path as a texmf root even once (1) is fixed.

Root cause of the box itself, as far as it was traced (not fixed — this is math-layout engine
work with a golden corpus behind it): `crates/math-layout/src/layout.rs`'s `make_sqrt` sets
`let rule_thickness = y.height;` where `y` is the radical-sign box from `var_delimiter`. That
is literally TeX's rule (TeXbook Appendix G / `overbar(x, clr, height(y))`), and it is correct
**for TFM metrics**, where the cmsy/cmex surd glyph is designed so its `height` is exactly the
thickness of the bar at its top and its `depth` is the whole body. It is wrong for an OpenType
MATH font, where the same accessor returns the glyph's full height above the baseline — hence
15.8 pt. It only bites on the OpenType fallback (`MathProvider::Otf`, chosen in
`crates/render-pipeline/src/typeset.rs` when `rm-lmr*.tfm` is unavailable, which this machine
already warns about with `math_metrics_opentype`: "math is laid out with the OpenType MATH
table instead of TeX's metrics"), and only when the radicand is tall enough that
`var_delimiter` picks a larger variant or an assembly — `\sqrt{\pi}` and `\sqrt{ab}` in
math-sheet, which use the base-size surd, render correctly. The engine already has a test
pinning the correct behaviour on the TFM path (`crates/render-pipeline/tests/tall_radicals.rs`,
whose header quotes pdfTeX `\showbox` output: rule 0.39998 pt thick, spanning the radicand's
width), so the TFM path is presumably fine and only the OpenType fallback is wrong. Two
possible fixes, in order of preference: (a) make the bundled metrics reachable on Windows so
the fallback is not taken at all — see the chain above; (b) take the radical rule thickness
from the MATH table (`RadicalRuleThickness`) on the OpenType path and re-derive the sign's
vertical shift from it. (b) touches `crates/math-layout`, whose golden and size-oracle tests
are pinned against MacTeX, so it deserves its own engine task rather than a drive-by change
from a Windows-preview pass.

### Still genuinely unverified (be specific, do not round this up)

- **Images**: completely unexercised. Neither fixture has `\includegraphics`, and a run
  against `fixtures/real-world/lab-report` (which does) produced **zero** `image` items,
  because `ShellModel.Compile.cs` deliberately never requests `display-list-v2-images`. So
  `V2Frame`'s `ImageNoticeItem` path and `PreviewV2Host`'s notice bar have still never been
  seen to run. If you ever request that capability, this is the first thing to test.
- **Vector paths**: `path_fill`/`path_stroke`/`clip` are unexercised — no fixture in
  `fixtures/real-world/` uses TikZ or `picture`, so `PreviewV2Host.DrawPath`,
  `BuildGeometry`, `BuildStrokeStyle` (including the ticks-to-stroke-width-multiples dash
  conversion) and the clip-layer stack have never painted anything. The dash-array conversion
  in particular is the kind of arithmetic that is wrong until proven right.
- **`core14-afm` metrics-only resources**: never seen. Every font resource in every fixture
  compiled here was `opentype-cff`, so `FontFileStore.Resolve`'s "never paintable" refusal
  (the one `RenderingV2.cs` warns must never silently paint) was never taken. There is no
  evidence it misbehaves; there is also no evidence it fires.
- **Non-black paint**: every glyph run and rule in both fixtures is `rgba(0,0,0,1)`. Colour
  conversion is untested against real output.
- **Click-to-source on the v2 pane**: `PreviewV2Host.OnPagePointerPressed` /
  `HitTestIndex.Query` were not exercised this pass (the v1 pane's equivalent is separately
  proven). The tick-space conversion there uses its own `TicksPerDip` constant.
- **The page cache**: `PreparedPageCache`/`LastFrameReusedPages` were never observed reusing
  a page across a recompile, because every launch compiled exactly once.
- **Text antialiasing/colour fidelity**: deliberately out of scope; the preview uses grayscale
  antialiasing on purpose (see `PreviewV2Host.OnPageDraw`'s comment) and no attempt was made
  to match pdfium's rasterization exactly.

### Follow-ups this pass opened, in rough order of value

1. **Re-pin `crates/render-pipeline/vendor/project-files` to the Windows-capable crate** (or
   make the vendored `sys.rs` carry the same Windows `imp`). Its `PIN` file currently reads
   `d5440b01f1681175e8dc25108d92424db188a7fe`, which predates this port's Windows backend.
   Everything in the radical section above hangs off this, and it is the difference between
   "math is laid out with the OpenType MATH table and is not the reference geometry" and
   actual reference geometry on Windows. It is very likely also why so much of
   `crates/render-pipeline`'s own test suite cannot pass on this machine (see below).
2. **Point the worker at the bundled TFM trees** once (1) lands — measured effect and the two
   exact `--font-dir` values are in the radical section. Consider adding a real `--tfm-dir`
   flag to `flashtex-render` rather than overloading `--font-dir`, since `FLASHTEX_TFM_DIRS`
   cannot express a Windows absolute path.
3. **Port the glyph-less-cluster fix to the Mac app** (`apps/mac/Sources/FlashTeXProtocol/
   RenderingV2.swift:936`) — it has the identical unrelaxed rule and will refuse the identical
   documents.
4. **`\sqrt` rule thickness on the OpenType MATH path** — see above; only worth doing if (1)
   turns out not to be the whole story.
5. Exercise the parts this pass could not: images, TikZ paths/clips, `core14-afm`, coloured
   paint, v2 click-to-source.

### Verification commands actually run

- `dotnet build FlashTeX.sln -c Debug`: **0 warnings, 0 errors**.
- `dotnet test FlashTeX.sln -c Debug --no-build`, re-run after rebasing onto the Open Folder
  commit so the numbers include it: 67 `FlashTeX.Protocol.Tests` (was 62; +5 new), 31 Preview,
  75 Shell, 19 Ipc, 67 ProjectFiles, 192/193 Editor. The **only** failure is the
  long-documented, unrelated `FlashTeX.Editor.Tests.CompletionTests.BundledInventoryMatchesTheMacCopy`.
  `FlashTeX.ProjectFiles.Tests` first showed 12 failures purely because this worktree had no
  `crates/project-files/target/release/flashtex-project-files.exe` — the exact stale-worktree-binary
  false alarm this doc already warns about; `cargo build --release` there, then all passed.
- `cargo test --release` in `crates/render-pipeline`: the 4 new `display::tests` pass
  (`display::` alone is 12 passed, was 8). **This crate's suite does not pass on Windows at
  all, and did not before this change either** — it was never part of the 2026-09-18 "Rust
  crate rebuild+test sweep" above, so nobody had recorded that. Two things to know:
  - With nothing configured, ~117 test targets abort immediately in
    `tests/common/mod.rs:26`, which panics on purpose when Latin Modern does not resolve
    ("this test would have skipped silently and the run would have been green without
    measuring anything"). Set `FLASHTEX_FONT_DIRS` to make the suite actually run — and note
    it is **colon**-split, so a Windows absolute path is cut at its drive letter; run cargo
    from the crate directory with the relative `../../apps/mac/Fonts` instead.
  - With that set, the run is **164 failing tests**, nearly all of them pdflatex oracle
    comparisons, plus `fonts::tests::a_flat_bundle_directory_with_tfms_and_licence_satisfies_the_required_set`
    and `fonts::tests::missing_font_diagnostics_do_not_depend_on_the_executable_location`
    — i.e. the same required-metrics/rooted-loader wall described in the `\sqrt` section, and
    a message assertion that expects a macOS app-bundle path.
  - **Attributed, not assumed**: the whole suite was run twice, once with `display.rs` checked
    out at `1af0f238` and once with this change, and the two sets of failing test names were
    diffed. **164 both times, with an empty regression set and an empty fixed set** — this
    change moves nothing either way.

### New gotchas for whoever renders next

- **`crates/render-pipeline`, not `crates/compiler`, is the display-list-v2 worker.**
  `CompilerLocator.FindWorker` prefers `crates/render-pipeline/target/release/flashtex-render.exe`
  and only falls back to `flashtex-compiler.exe`, which declines the capability entirely. In a
  fresh `git worktree` you must `cargo build --release --bin flashtex-render` from
  `crates/render-pipeline` **and** `cargo build --release` from `crates/project-files` (for the
  test suite) — a build in the main checkout does not populate the worktree's `target/`.
- **`cargo test` relinks `target/release/flashtex-render.exe` from whatever source is in the
  tree at the time**, so a test run (or anything that checks a source file out and back)
  leaves the app running a worker you did not intend. This produced a convincing false
  regression at the end of this pass — the status bar went back to
  `runtime-v1 preview (invalid_display_list: diagnostics must carry a code and a 1...4096-byte
  message)` — and `cargo build --release --bin flashtex-render` reported "Finished in 0.15s"
  because cargo still considered the binary fresh. Touch the source (or `cargo clean -p`) to
  force the relink, and check the exe's size/timestamp before trusting a launch.
- **A `dotnet build src/FlashTeX.App/FlashTeX.App.csproj` can leave a stale
  `FlashTeX.Protocol.dll` in the app's output.** This cost a full relaunch cycle chasing a bug
  that was already fixed. Compare
  `src/FlashTeX.App/bin/x64/Debug/.../FlashTeX.Protocol.dll`'s size/timestamp against
  `src/FlashTeX.Protocol/bin/Debug/net8.0/FlashTeX.Protocol.dll` before trusting a run, or
  build the whole `FlashTeX.sln`.
- **Never `taskkill /F /IM FlashTeX.App.exe /T` on this machine.** Another agent runs the *main
  checkout's* copy of the same image name; a capture that reported
  `path=D:\Projects\flashtex\apps\windows\...` instead of `...\.claude\worktrees\...` proved
  this both ways this session (theirs was killed by these scripts, and this worktree's instance
  was killed twice by theirs mid-verification). Filter by `$_.Path.StartsWith(<worktree>)` and
  `Stop-Process -Id` instead, and do launch+interact+capture in **one** shell invocation.
- **The preview pane's `ScrollViewer` is not reachable from the out-of-process UIA2 client**
  (`ScrollPattern` finds only the editor's and the menu's) — the same tree-depth plateau this
  doc already records for the Problems panel. Synthetic mouse wheel does work, but only with
  `SetForegroundWindow`, then `SetCursorPos`, then a real `MOUSEEVENTF_MOVE` before the
  `MOUSEEVENTF_WHEEL` events; `SetCursorPos` alone is not enough for WinUI's pointer tracking.
- **`%LOCALAPPDATA%\FlashTeX\window-panes.json`** is the cheap way to widen the preview pane
  for a full-page capture (the splitters are pointer-drag-only). Back it up and restore it —
  this pass did.

## Session housekeeping note

`apps/windows/image.png` (the screenshot embedded at the top of this doc) was
untracked in git and got accidentally deleted by a stray cleanup command in
this session (`rm -f *.log image.png` intended only for leftover `.launch*.log`
test-output files another agent left in the repo root — a real mistake, not
intentional). Regenerated immediately via the standard `PrintWindow`
technique and it's now tracked in git specifically so this can't silently
happen again. If you ever need to clean up stray files in this directory,
check `git status`/what's actually referenced by this doc first, or scope
your cleanup command to an exact filename list rather than a glob that could
catch something load-bearing.

## Open Folder: real folder-level project browsing (2026-09-19)

Before this pass, the project tree only ever listed *currently open documents*
plus include-scan rows for the active document's `\input`/`\include` targets
(`ProjectIncludes.cs`). There was no way to browse a whole folder's contents
and no directory-listing operation on the `flashtex-project-files` wire
protocol at all — confirmed by reading the protocol doc comment and the
`match` arms in `flashtex-project-files.rs` before starting, per the task's
explicit instruction not to assume.

### Rust: a new `list` operation, library-level first

Checked `crates/project-files` for an existing "list files under this root"
capability before building one. `ProjectGraph::discover` (`graph.rs`) is
**not** it: it is reference-based (`\input`/`\include`/`\bibliography`
traversal from one entry file), not a directory walker, and does not surface
files nobody references. Nothing else in the crate enumerates a whole
directory tree, so this needed a new `ProjectRoot::list_files` at the library
level (`src/save.rs`), matching the crate's existing rigor rather than a naive
`std::fs::read_dir` walk:

```rust
pub const DEFAULT_LIST_LIMIT: usize = 10_000;
pub struct FileListing { pub files: Vec<ProjectPath>, pub truncated: bool }
impl ProjectRoot {
    pub fn list_files(&self, subdir: Option<&ProjectPath>, extensions: &[&str], limit: usize) -> Result<FileListing, SaveError>;
}
```

- Walks with the exact same primitives every other rooted operation uses:
  `open_dir_at_nofollow` to descend (a symlinked directory component is never
  followed, `sys::verify_parent` checked at every step) and
  `fstatat(AT_SYMLINK_NOFOLLOW)` to classify each entry before opening or
  recursing into it.
- **Unlike `read`/`save`/`remove`/`rename`**, which act on one path the
  caller named and must explain a refusal for, a listing enumerates entries
  nobody named: a symlink (file or directory), a hidden entry (name starting
  with `.` — this is what excludes `.flashtex/`'s lock file and recovery
  journal with no special case for that name), anything neither a regular
  file nor a directory, and any entry that races away or fails to classify
  are all **silently excluded**, never reported as an error. One bad entry
  does not abort the rest of the walk.
- Recursion stops at a fixed depth (64) — no legitimate LaTeX project nests
  that deep; a subtree that deep is omitted, not an error (mirrors
  `DiagnosticKind::DepthExceeded` being a diagnostic, not a hard
  `DiscoverError`, in `graph.rs`).
- `files` is returned sorted in `ProjectPath` order (Unicode-NFC) regardless
  of on-disk directory-entry order, and `truncated` is `true` when `limit`
  was hit before the whole tree was walked (the walk stops descending as
  soon as the cap is reached, rather than paying full traversal cost merely
  to report it).
- Takes **no project lock** — nothing is written, so (like `read`) it can
  race a concurrent save/rename/remove; a file can appear, disappear or
  change between being listed and being acted on later.

Wire protocol (`project-files-v1`, `flashtex-project-files.rs`), matching the
existing `read`/`save`/`remove`/`rename` JSON shape exactly:

```
{"id","operation":"list","path"?}
  → {"id","payload":{"path","files":[...],"truncated":bool}}
```

`path` is a project-relative directory (omitted or `null` lists the whole
root, echoed back as `""`); `files` is every project file under it as
root-relative `ProjectPath` strings, sorted. The helper fixes the extension
set to `tex`/`bib`/`sty`/`cls`/`bst`/`clo` (`PROJECT_FILE_EXTENSIONS` — the
source types a project browser opens for editing, not media pulled in via
`\includegraphics`, which the graph hashes for identity but never opens) and
the cap to `DEFAULT_LIST_LIMIT` (10,000 files); neither is adjustable over the
wire. `README.md`'s "Directory listing" section and its `list` protocol-table
row document the exact exclusion rules and guarantees at the same level of
detail as every other operation there.

**Verified** (`cargo build --all-targets && cargo test` and
`cargo build --release` from `crates/project-files/`): clean, **95 passed, 0
failed** (was 86; +9 new — 7 in `tests/rooted.rs`: empty directory, nested
subdirectories, extension filtering case-insensitivity, hidden-directory/
`.flashtex` exclusion, symlinked file+directory exclusion, a 250-file listing
respecting a 100-file cap and reporting `truncated`, and subdirectory scoping;
2 in `tests/helper_bin.rs`: the wire `list` operation end-to-end including a
real `.flashtex/project.lock` on disk, and the under-the-cap
`truncated:false` case). `cargo clippy --all-targets -- -D warnings` still has
the one pre-existing, unrelated finding at `sys.rs:596` this doc's "Rename and
Delete" section already recorded — untouched by this change.

### C# client and `CommandIds.OpenFolder`

- `src/FlashTeX.Protocol/ProjectFilesV1.cs`: `ListRequest`/`ListPayload`,
  mirroring `StatusRequest.ExpectedSha256`'s "send an explicit JSON `null`"
  convention for the optional `path`. Registered in `FlashTeXJsonContext`
  (source-gen; a missing entry is a runtime failure, not a compile error).
- `src/FlashTeX.ProjectFiles/DocumentFilesClient.cs`: `ListAsync(subdirectory:
  null, ...)`, same envelope pattern as `ReadAsync`/`RemoveAsync`.
- `src/FlashTeX.Shell/CommandRegistry.cs`: appended `CommandIds.OpenFolder`
  and its `Make(...)` row (category File, no default shortcut — matching the
  existing `RenameFile`/`DeleteFile`/`Settings` no-shortcut, append-only
  precedent for minimizing merge risk with any other concurrent agent) at the
  very end of the list.
- `src/FlashTeX.App/MainWindow.ProjectFiles.cs`: the actual flow, handled
  exactly like the other project-file commands (`IsProjectFileCommand`/
  `RunProjectFileCommandAsync`), **not** special-cased in `MainWindow.Menu.cs`.
  - `OpenFolderAsync()`: picks a folder, calls `EstablishProjectRootAsync`
    (reusing its existing "unsaved changes" guard and all-documents-closed
    logic verbatim, not duplicated), then `client.ListAsync()` and stores the
    result in two new fields (`_isFolderProject`, `_folderProjectFiles`)
    consumed by `RebuildProjectTree`'s new `AppendFolderSection`. A truncated
    listing shows a one-time informational dialog naming how many files are
    shown.
  - `OpenFolderProjectFileAsync(relativePath)`: opens a folder-tree row,
    switching to its tab instead of opening a second copy if already open —
    the same de-duplication `SwitchActiveDocument` already gives
    `RebuildProjectTree`'s open-document rows, just reached from a different
    entry point since a folder-tree row has no `OpenFileBinding` to key off
    yet when first clicked.
  - `AppendFolderSection`: a **flat, sorted list** of clickable rows under a
    new "FOLDER" heading, not a `TreeView`. Chosen deliberately: a LaTeX
    project this size (dozens to a few hundred `.tex`/`.bib` files, not
    thousands of arbitrary assets) reads perfectly well as one alphabetized
    list, and a flat list needs no lazy-loading/expansion-state plumbing to
    stay honest about what is actually on disk — a fully general
    expand/collapse `TreeView` was judged out of proportion to the value it
    would add here. No-op (and the single-file-open tree completely
    unaffected) when `_isFolderProject` is false, which is the case for every
    existing flow (`OpenLatexFileAsync`/`SaveActiveFileAsync`/
    `CreateNewFileAsync`) — only `OpenFolderAsync` ever sets it `true`.
    `EstablishProjectRootAsync`'s existing root-switch branch now also resets
    both fields, so a previous folder project's listing cannot linger against
    a newly opened single file or a different folder.

### The `Windows.Storage.Pickers.FolderPicker` dead end — read before touching folder-picker code again

`FolderPicker` (the same `InitializeWithWindow`-owned WinRT picker family as
the already-working `FileOpenPicker`/`FileSavePicker`) was tried **first**,
per the task's explicit suggestion and for consistency with the rest of this
file. It reliably throws `System.Runtime.InteropServices.COMException
(0x80004005, E_FAIL)` from `PickSingleFolderAsync()` in this specific app —
confirmed with a real UI-Automation-driven repro, not a guess: the native
"Select Folder" dialog opens, navigates and closes normally (the user's own
click on its "Select Folder" button succeeds), and *only then* does the await
throw, with an exception logged via a temporary diagnostic
(`ex.ToString()`, removed again once diagnosed) showing the throw site was
`PickSingleFolderAsync()` itself. This matches a known limitation of
`Windows.Storage.Pickers` in **unpackaged** (`WindowsPackageType=None`) Win32
apps: `FileOpenPicker`/`FileSavePicker` have a working unpackaged fallback
path for marshaling the result back into the caller's process, but
`FolderPicker`'s does not reliably resolve an arbitrary, not-previously-
Explorer-indexed directory without a package identity's moniker cache — and a
freshly created temp directory (exactly what this feature's own verification,
and any real first-time "Open Folder" on a new project, both do) reproduces
it every time.

Fixed by **`src/FlashTeX.App/NativeFolderPicker.cs`** (new): raw
`IFileOpenDialog`/`FOS_PICKFOLDERS` COM interop, reading the result via
`IShellItem::GetDisplayName(SIGDN_FILESYSPATH)` instead of going through
WinRT's `StorageFolder` marshaling. The dialog the user sees is **identical**
either way — `Windows.Storage.Pickers.FolderPicker` is itself a thin wrapper
over the same native common-item dialog — so this bypasses only the broken
post-pick step, not the UI. `IFileOpenDialog::Show` is a blocking modal call
driven synchronously on the calling (UI/STA) thread, deliberately **not**
wrapped in `Task.Run` — a thread-pool thread would be the wrong COM apartment
for a modal dialog parented to this window. If a future agent is tempted to
"simplify" `OpenFolderAsync` back to `Windows.Storage.Pickers.FolderPicker`
for consistency with `OpenLatexFileAsync`/`SaveActiveFileAsync`: don't, unless
you've first re-verified this specific failure no longer reproduces on the
target machine/OS build.

### Verified (real launch, real folder, real UI Automation — not self-reported)

Built a real temp folder (`main.tex` with `\input{chapters/intro}`,
`refs.bib`, `chapters/intro.tex`) and drove the actual app through it via
`System.Windows.Automation` + real mouse clicks (menu-item `InvokePattern
.Invoke()` was found to **block the automation client for the entire modal
picker interaction** in this app now that the click handler synchronously
shows a blocking native dialog — switched to physically clicking menu items'
`BoundingRectangle` centers instead, which posts input asynchronously and
does not have this problem; worth remembering for any future command whose
handler shows a modal dialog). `PrintWindow`/`PW_RENDERFULLCONTENT`
screenshots throughout, `SetProcessDpiAwarenessContext` first per this doc's
standing rule (confirmed load-bearing again this session: a click computed
from an un-DPI-aware process landed on the wrong control on this machine's
scaled display).

1. File ▸ Open Folder → the native `IFileOpenDialog` opens titled "Select a
   project folder"; typed the temp folder's path into its "Folder:" edit box
   and confirmed with its own "Select Folder" button (not Enter — confirmed
   Enter in this box can re-navigate deeper if a subfolder happens to be
   list-selected, exactly what happened once during verification; the
   button is the reliable confirm action).
2. Real result, screenshotted: "PROJECT" reads "No files open"; a new
   "FOLDER" section lists all three files, **sorted** exactly as the Rust
   layer promises (`chapters/intro.tex`, `main.tex`, `refs.bib`); "OUTLINE"
   reads "No active document". Toolbar actions correctly disabled with no
   active document.
3. Clicked `main.tex` in FOLDER → a real tab opened showing the file's
   **actual on-disk content** (`Hello from the folder test.` /
   `\input{chapters/intro}`), not placeholder text; "PROJECT" now lists it as
   open; FOLDER still lists all three with `main.tex` bolded as active.
4. Clicked `chapters/intro.tex` in FOLDER (a **nested subdirectory** file) →
   a second real tab opened with its actual content ("This is the intro
   chapter."); both files listed under PROJECT as open documents
   simultaneously.
5. Clicked `main.tex` in FOLDER again (already open) → switched to its
   existing tab; still exactly two tabs, no duplicate created.
6. Relaunched fresh (no folder opened) and confirmed the **single-file-open
   tree is completely unaffected**: "PROJECT" shows only the seeded
   `main.tex`, no "FOLDER" section at all — matches `AppendFolderSection`'s
   `_isFolderProject`-false no-op by inspection and by this live screenshot,
   both before and after a separate `Open LaTeX File` attempt.
7. `dotnet build FlashTeX.sln -c Debug`: 0 warnings, 0 errors.
   `dotnet test FlashTeX.sln -c Debug --no-build`: **447 passed, 1 failed**
   across every test project (`FlashTeX.ProjectFiles.Tests` now 67, was 65 —
   the 2 new `ListAsync` tests against the real release binary); the 1
   failure is the same pre-existing, unrelated
   `FlashTeX.Editor.Tests.CompletionTests.BundledInventoryMatchesTheMacCopy`
   hash mismatch this doc has documented since 2026-09-14. One run this
   session also hit `DocumentWatcherTests
   .Watch_SeveralQuickInPlaceEditsCoalesceIntoOneNotification` failing
   (expected 1 coalesced notification, got 2) under this machine's heavy
   concurrent-agent CPU/IO contention; re-ran in isolation and it passed —
   flaky timing on a loaded shared box, not a real regression, and unrelated
   to any file this pass touched.
8. This machine's now-standard multi-agent contention struck again multiple
   times during verification: `FlashTeX.App.exe`/`PickerHost.exe` instances
   from other concurrent agents' own testing appeared and were filtered out
   by matching on the exact expected `.Path` before trusting any PID; a
   `dotnet build FlashTeX.sln` once failed with the file-locked `MSB3026`/
   `MSB3027` pattern this doc already documents (a still-running instance of
   this pass's *own* previous test build holding the exe open) — resolved by
   killing it and rebuilding, not a real error. Test fixtures and every
   `FlashTeX.App.exe`/`flashtex-*.exe`/`PickerHost.exe` process this pass
   started were cleaned up afterward.

### Known rough edges

- **No live folder-watching.** `_folderProjectFiles` is a one-time snapshot
  from the moment "Open Folder" was invoked; a file added, removed or renamed
  on disk by another program afterward is not reflected until the folder is
  reopened. This is a reasonable, explicitly-scoped-out follow-up, not an
  oversight — the existing per-open-document external-change watch
  (`WireDocumentWatchers`) is unaffected and keeps working for whatever is
  actually open.
- Folder-tree rows have no context menu (no Rename/Delete) — those still only
  work from an open document's tab/project-tree row, which already has an
  `OpenFileBinding` to act through. Extending them to unopened folder rows
  would need a binding-on-demand path; left for whoever picks this up next.
- The include-scan section (the "↳ name" rows under PROJECT) can show a
  reference as still-unresolved-looking even when the same file is already
  open and listed under FOLDER, because of a **pre-existing** bug this pass
  found but did not fix (out of scope): its candidate loop's `continue`
  advances to the *next* candidate spelling (e.g. `chapters/intro` after
  `chapters/intro.tex`) instead of skipping the whole reference once any
  candidate is found open, so a second, differently-spelled candidate for the
  same already-open file can still get its own row. Cosmetic (clicking it
  just re-opens/reveals the same file) and not something this pass's diff
  touches.
- A truncated listing's dialog is shown once, right after opening the folder;
  there is no persistent "showing N of M" indicator in the FOLDER section
  itself afterward.

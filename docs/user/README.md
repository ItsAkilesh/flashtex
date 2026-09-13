# FlashTeX user guide

FlashTeX is a native Mac LaTeX editor with its own engine: you type LaTeX, the
page re-renders as you type (tens of milliseconds, no TeX distribution
required), and you export a PDF when you are done. An iPad companion turns
Pencil sketches and photos into reviewed LaTeX/TikZ insertions.

| Guide | What it covers |
|---|---|
| **This page** | Requirements, installing, a 5-minute first document |
| [The Mac app](gui.md) | Workspace, projects and multi-file documents, editing and IntelliSense, compiling, the preview, the Problems panel and quick fixes, PDF export, capture conversion, the Nearby companion, Preferences, all keyboard shortcuts |
| [FlashTeXPad for iPad](ipad.md) | What the capture companion does, building it on a device, pairing, sending a capture, limits |
| [Command-line tools](compiler.md) | `flashtex-render`, `flashtex-compiler`, `flashtex-pdf`/`flashtex-pdf-exact`, fonts and metrics, building from source, **supported LaTeX**, diagnostic codes |

## Requirements

- macOS 14 Sonoma or later on **Apple Silicon** (arm64). No Intel build yet;
  no Windows or Linux — FlashTeX is a native Swift app over a Rust engine.
- No TeX installation is needed. The app bundles the engine, Latin Modern
  fonts and the TeX font metrics it lays out with.
- Optional: an API key for a conversion provider (xAI today) if you want iPad
  captures converted to LaTeX/TikZ — see [capture conversion](gui.md#capture-conversion-the-only-model-backed-feature).

## Install

**Command line (recommended).** Downloads the pinned release disk image,
verifies its SHA-256, and copies `FlashTeX.app` into `/Applications` (or
`~/Applications` when `/Applications` is not writable; set
`FLASHTEX_INSTALL_DIR` to choose another folder). It also strips the
quarantine flag, so the app opens without a Gatekeeper prompt:

```sh
curl -fsSL https://flash-tex.github.io/flashtex/install.sh | sh
open /Applications/FlashTeX.app
```

**Disk image.** Download `FlashTeX.dmg` from
<https://github.com/flash-tex/flashtex/releases>, open it and drag FlashTeX
into Applications. The build is ad-hoc signed but **not notarized**, so on the
first launch macOS refuses a double-click: **right-click (or ⌃-click) FlashTeX
in Applications, choose Open, then confirm**. If macOS says the app is
"damaged", run `xattr -dr com.apple.quarantine /Applications/FlashTeX.app` and
open it again. Compare `shasum -a 256 ~/Downloads/FlashTeX.dmg` with the
checksum in the release notes if you want to verify the download.

**From source.** See [Building from source](compiler.md#building-from-source).
A locally built app is not quarantined and opens without the approval step.

To update, run the install command again (it replaces the existing app) or
install the newer DMG over the old copy. Your documents, pairings and
preferences are kept.

## Quick start (5 minutes)

1. **Open FlashTeX.** The window has a project sidebar on the left, the source
   editor in the middle and the page preview on the right. The editor starts
   with a one-line sample and the bundled engine (`flashtex-render`, Latin
   Modern fonts, exact v2 preview) attaches automatically — the preview
   header names it. *File › Attach Render Pipeline (Latin Modern)* (⌘⇧R)
   re-attaches it if you switched engines.

2. **Open your document** with *File › Open LaTeX File…* (⌘O) and pick a
   `.tex` file, or start typing in the sample. The file you open becomes the
   project's entry document; files it pulls in with `\input`/`\include`
   appear in the sidebar. Try this minimal document if you have none:

   ```latex
   \documentclass[11pt]{article}
   \begin{document}
   \section{Hello}
   FlashTeX renders \emph{as you type}. Here is a formula:
   \[ \int_0^1 x^2 \, dx = \frac{1}{3} \]
   \end{document}
   ```

3. **Type.** Every edit is compiled and the preview updates while you type
   (auto-compile is on by default; ⌘B compiles on demand). Press ⌃Space (or
   Esc) after `\` for supported commands, after `\begin{` for environments,
   after `\ref{` for labels. Errors and warnings are underlined in the editor and listed in
   the **Problems** panel (⌘⇧M); click one to jump to its source, or use the
   *Fix…* button where a quick fix exists.

4. **Navigate.** Click any word in the preview to select its source; put the
   caret in the editor and the matching preview text is highlighted (⌘⇧J
   scrolls the preview to it). Zoom the preview with ⌘= / ⌘- and reset with
   ⌘9 (fit width) or ⌘0 (actual size).

5. **Save and export.** ⌘S saves the `.tex` (UTF-8). *File › Export PDF
   (exact, v2)…* writes a PDF with the same glyphs and positions as the
   preview; *File › Export PDF…* (⌘⇧E) is the simpler CoreGraphics route.
   The dark-preview toggle only changes the on-screen colours — exports are
   always black on white.

Next: read [the Mac app guide](gui.md) for projects, IntelliSense, the AI
capture conversion and the full shortcut table, and [Supported LaTeX](compiler.md#supported-latex)
to see what the engine implements today and what it reports as unsupported.

## Where things live

| | Path |
|---|---|
| The app | `/Applications/FlashTeX.app` (or `~/Applications`) |
| Bundled engine binaries | `FlashTeX.app/Contents/MacOS/flashtex-render`, `flashtex-compiler`, `flashtex-pdf`, `flashtex-pdf-exact`, … |
| Bundled fonts and TeX metrics | `FlashTeX.app/Contents/Resources/Fonts`, `Contents/Resources/texmf` |
| Nearby pairings | `~/Library/Application Support/FlashTeX/pairs.json` |
| Captures and durable edit history | `~/Library/Application Support/FlashTeX/captures/` |
| AI provider key | the login Keychain (never a file) |

Problems or questions: <https://github.com/flash-tex/flashtex/issues>. When
reporting a compile problem, *Edit › Copy Diagnostics as Text* (⌘⌥C) copies
the Problems list in `path:line: severity: message` form for pasting.

# The FlashTeX Mac app

FlashTeX is a source editor with a live page preview. This guide walks through
the window, projects, editing, compiling, the preview, problems and fixes, PDF
export, the AI assistant, the iPad companion window, Preferences, and ends with
the full keyboard-shortcut table. Everything here was checked against the app
sources (`apps/mac`) at the time of writing; where a feature is partial, the
text says so.

## The window

```
┌ Sidebar ────┬ Editor ─────────────────────┬ Preview ───────────────┐
│ Project     │ tabs: main.tex  chapter1.tex│ header: producer, zoom │
│ Outline     │ gutter │ source              │ pages                  │
│ Problems    │        │                     │                        │
├─────────────┴─────────────────────────────┴────────────────────────┤
│ Problems panel (⌘⇧M): errors / warnings / not implemented           │
├──────────────────────────────────────────────────────────────────────┤
│ status bar: r12 · durable r12 · 31 ms · worker · Grok: off · 2 ⚠     │
└──────────────────────────────────────────────────────────────────────┘
```

- **Sidebar** (left, drag to resize). *Project* lists the open files — the
  entry document first, an orange dot for unsaved changes, `rN` for the
  durable revision — followed by greyed rows for every `\input`/`\include`
  target that exists on disk but is not open yet (click to open). *Outline*
  lists sections (indented by level), environments and labels of the active
  file with line numbers; click to jump. *Problems* shows error / warning /
  not-implemented counts; clicking one opens the panel filtered to it.
- **Editor** (middle) with a tab per open file. Closing a tab (×) only
  detaches the file for this session; includes are rediscovered from the entry
  document. The tab bar's *Project* menu lists the include tree, *Open All
  Includes*, and the file's kind.
- **Preview** (right): the rendered pages. The preview header names the
  attached producer and holds the zoom controls; the window **toolbar** holds
  the *Auto-compile after edits*, *v2 pane* and *Dark preview* switches, the
  Problems toggle, *Ask Grok* and *Commands*.
- **Problems panel** (bottom, ⌘⇧M) and the **status bar** (editor revision,
  last compile latency, route, Grok status, problem counts — click the counts
  to toggle the panel).
- **⌘⇧P** opens the command palette: every command with its shortcut; type to
  filter, Return runs.

## Projects and multi-file documents

There is no "New project" command: a project is simply a `.tex` file and the
folder it lives in.

- **Open** a file with *File › Open LaTeX File…* (⌘O). It becomes the
  **entry document** (sent to the engine as `main.tex`, whatever its real
  name) and its folder becomes the project root. If the current buffer is
  unsaved you are asked to Save / Discard / Cancel; a discarded buffer can be
  brought back with *Edit › Restore Discarded Buffer* until you quit.
- **Create** a document by opening FlashTeX, typing into the empty/sample
  buffer and pressing ⌘⇧S (*Save As…*).
- **`\input{…}` and `\include{…}`** are scanned lexically (no macro
  expansion; `\input{\jobname}` is reported as non-literal). Targets resolve
  against the project root, `name.tex` before `name`, never above the root
  and never through symlinks. Resolved files show in the sidebar; open them
  (click, ⌘-click the command, or *Open All Includes*) to edit them. **Only
  open files are sent to the engine** — open an include if you want its
  content compiled.
- **Saving** (⌘S) is compare-and-replace: if the file changed on disk since
  it was read, you get *File › Resolve On-Disk Conflict…* with **Overwrite /
  Reload / Keep Editing** instead of a silent overwrite. FlashTeX also watches
  open files and surfaces outside edits the same way; *File › Reload From
  Disk…* shows a line-count summary before importing.
- **Unsaved text is never lost silently.** Discarding, detaching, an external
  change or quitting without saving writes a snapshot under
  `~/Library/Application Support/FlashTeX/`; the next time you open that file,
  *File › Restore Unsaved Snapshot…* offers Restore / Discard Snapshot / Later.
- **Durable history.** On the helper route (below) every edit is also
  journaled; *Edit › Durable History…* shows undo/redo stacks that survive a
  crash or relaunch. Without the helper the window says "no preview
  controller attached".

## Editing

- **Syntax highlighting**: comments, commands, `\begin`/`\end` with
  environment names, math (`$…$`, `\[…\]`, math environments), numbers in
  math, dimmed braces, label/ref/cite keys, file arguments of
  `\input`/`\includegraphics`/`\usepackage`, `\newcommand`-defined names;
  verbatim is left plain. Colours follow the editor appearance (System /
  Light / Dark in Preferences).
- **Completion (IntelliSense)**: press **⌃Space** or **Esc** to open the list
  (it does not pop up on its own; typing narrows it once open). ↑/↓ or
  Tab/⇧Tab choose, Return inserts over the typed token as one undo step, Esc
  closes. Sources, in rank order: `\end{X}` for still-open environments;
  commands the engine supports (with snippets); commands used elsewhere in
  the document, marked "not supported by this compiler version"; environment
  names after `\begin{`; labels after `\ref{`/`\eqref{`/`\autoref{`; citation
  keys after `\cite{` (from `\bibitem` and from the project index when a
  bibliography is declared); document words longer than three characters.
  At most 12 entries.
- **Hover**: rest the pointer on a token for about half a second to see what
  it is (command with documentation, label, citation key, file, package,
  environment) plus any diagnostic at that position with its recovery note
  and explanation.
- **⌘-click** (or ⌘⇧D, *Navigate › Go to Matching*): `\ref{key}` → its
  `\label`; `\label` → cycles through its references; `\begin` ↔ `\end`;
  `\input{file}` → opens the file. `\cite{key}` and user macros resolve
  through the project index, which needs the preview-controller helper route
  (see *Compiling*); without it the footer says the caret is not on a
  matchable command.
- **Gutter**: line numbers, a red or orange dot on lines with an error or
  warning, a grey dot for "not implemented" gaps. Inline underlines mark the
  exact span (red dotted = error, orange = warning); hovering shows the
  message. After you edit, underlines that overlap the edit are dropped, never
  drawn under the wrong text; when a compile fails with no output the previous
  underlines are kept and flagged "kept from revision N".
- **Auto-close**: typing `{` inserts `}` when the brace is code and followed by
  whitespace or a closer (Preferences › Auto-close braces). Return
  auto-indents and closes `\begin{env}` with the matching `\end{env}`.
- **Find in Project** (⌘⇧F): case-sensitive literal search across the
  project's durable source; Return or ⌘G goes to the next match; *Plan
  Replacement* → *Apply* performs a reviewed replace-all. **Helper route
  only** — otherwise the window says "Search requires the durable helper
  (flashtex-preview-controller) to be attached and ready."
- **Rename Citation** (*Edit › Rename Citation…*, also in the toolbar): plans
  a rename of a citation key across the project, shows every affected place,
  and applies it as one group after you confirm. Helper route only.
- **Editor font size**: ⌘⌥= / ⌘⌥- / ⌘⌥0 (8–36 pt, default 13), or pinch over
  the editor.

## Compiling

FlashTeX compiles through a **producer** process that ships inside the app.

- **At launch** the bundled `flashtex-compiler` attaches automatically and the
  document compiles (status bar route `worker`). This is the older engine
  with Times metrics. For the current engine — Latin Modern fonts, TeX
  metrics, the larger LaTeX subset and the v2 preview — choose
  **File › Attach Render Pipeline (Latin Modern)** (⌘⇧R) once per session.
  The header then names `flashtex-render`. (Automatic attachment of the render
  pipeline is not yet implemented; the environment variable
  `FLASHTEX_AUTOATTACH=0` disables auto-attach altogether.)
- **Auto-compile** (toolbar switch, on by default) sends every edit to the
  producer immediately; one request is in flight at a time and the newest
  buffer is coalesced behind it, so the preview never shows an older
  revision over a newer one. **⌘B** compiles on demand (*File › Compile*).
  Latency (send → result) is shown in the status bar, typically tens of
  milliseconds; the engine retypesets only the paragraphs you touched.
- A result is `ok`, `recovered` (pages rendered around problems) or `failed`
  (no pages; the previous preview stays on screen). *View › Show Preview
  Debug Status* reveals the status word, a "provisional rendering" note and
  the display-list identity line in the preview header.
- **Other producers** (developer options): *Attach Built Compiler* (⌘⇧K) finds
  `$FLASHTEX_COMPILER` or a `crates/compiler` build; *Attach Worker
  Executable…* (⌘K) runs any program speaking the JSON Lines protocol
  (see [compiler.md](compiler.md)); *Detach Worker* stops it. The
  preview-controller **helper route** (durable ledger + project index owned
  by the helper, route `controller`; powers Find in Project, Rename Citation,
  Durable History and `\cite`/macro navigation) has no menu item yet. It
  starts only when the app is launched with
  `FLASHTEX_AUTOATTACH=1 FLASHTEX_PREVIEW_CONTROLLER=<path to flashtex-preview-controller>`
  in the environment, e.g. from a terminal:

  ```sh
  A=/Applications/FlashTeX.app/Contents/MacOS
  FLASHTEX_AUTOATTACH=1 FLASHTEX_PREVIEW_CONTROLLER=$A/flashtex-preview-controller $A/FlashTeX
  ```

  (This is the developer hook documented in `apps/mac/README.md`; it was not
  exercised while writing this guide.)

## The preview

- **Two panes.** The default is the **v2 pane**: it paints the rendering-v2
  display list (exact glyphs and positions, the same data the exact PDF
  export uses) that `flashtex-render` sends with every result. With the
  launch-time `flashtex-compiler` attached there is no display list and the
  pane says "No v2 display list yet" — press ⌘⇧R, or flip the toolbar's *v2
  pane* switch off to see the v1 pane (text items drawn with CoreText).
  `FLASHTEX_PREVIEW_V2=0` starts on the v1 pane.
- **Zoom**: ⌘= / ⌘- step by ×1.25 between 25 % and 400 % of fit-width; ⌘9
  fits the widest page to the pane; ⌘0 shows 1 PDF point per screen point.
  The header's −/+ buttons and percentage (double-click = fit width) and
  pinch-to-zoom do the same. Wide pages scroll horizontally.
- **Click-to-source**: click any word or rule in the preview to select its
  source in the editor. If you edited that region since the last compile the
  click is refused with "recompile to navigate" rather than selecting the
  wrong text.
- **Caret sync**: preview text whose source contains the editor caret is
  highlighted (accent fill + underline) and the page scrolls into view;
  *Navigate › Reveal Caret in Preview* (⌘⇧J) selects the whole span.
- **Dark preview** (toolbar switch): inverts page and text colours on screen
  only; exports are unaffected. Its initial state follows the editor
  appearance preference.
- **Stale frames**: while a new frame is being prepared the previous one stays
  visible with a STALE mark; a frame that fails validation is never painted
  partially — the pane shows the refusal and offers the v1 preview.

## Problems and quick fixes

The Problems panel (⌘⇧M; also opened by the sidebar rows and the status-bar
counts) lists the diagnostics of the current result, grouped by identical
message ("12× `\in` is not supported…") with a *Show: All / Errors / Warnings*
filter. Each row shows the message, a `↳` recovery line (what was rendered
instead), an `↳ explain:` line from the offline explanation catalogue, and
an "N places" menu.

- **Go to source**: click a row or press Return; ⌘⇧] / ⌘⇧[ step through
  diagnostics from the editor, ⌘⌥] / ⌘⌥[ step through the places of the
  selected group.
- **Fix…** appears when the catalogue has a concrete edit for that
  diagnostic (for example *Remove `\foo` and keep its argument text*). It
  opens a *Suggested fix* sheet with Before / After; **Apply** performs one
  undoable edit, **Cancel** changes nothing. The fix is rebased byte-exactly
  onto the current text and refused if that region changed since the compile.
- **Fix with Grok** sends the diagnostic's lines to the AI assistant with the
  instruction "Fix this: <message>" (below).
- **Copy Diagnostics as Text** (⌘⌥C, or ⌘C with the list focused) copies
  `path:line: error: message` lines for the selected row or all rows —
  handy for bug reports.

What each diagnostic code means is listed in
[compiler.md › Troubleshooting](compiler.md#troubleshooting).

## PDF export

| Menu item | Shortcut | What it writes |
|---|---|---|
| **File › Export PDF (exact, v2)…** | — | The current v2 display list through `flashtex-pdf-exact`: embedded Latin Modern subsets, original glyph IDs, exact positions and typed rules. Needs a v2 frame (i.e. `flashtex-render` attached). Progress and Cancel in the status bar; the file is written atomically. **Use this one.** |
| File › Export PDF via Rust Writer… | ⌘⌥E | The v1 result through `flashtex-pdf --verify`: base-14/Latin Modern text items; characters outside those encodings become `?` with a warning |
| File › Export PDF… | ⌘⇧E | A CoreGraphics rendering of the v1 layout (Times/Latin Modern, no images, no links) |

All exports are black on white regardless of the dark-preview switch. None of
them is a pdfTeX PDF: only what the engine laid out is written (no
`\includegraphics`, no hyperlinks, no metadata).

## The AI assistant

The assistant is a reviewed-edit tool: it can *propose* changes to your
document, and **nothing is applied unless you click Apply**. It is currently
backed by xAI's Grok models and is being generalised to other providers.

- **Ask Grok…** (⌘⌥G, *Edit* menu, toolbar button, or *Fix with Grok* on a
  Problems row) opens a sheet. Type an instruction; it applies to the current
  selection (or the whole document when nothing is selected). What is sent:
  the head of the active document, up to 2 KiB of context around each
  selected diagnostic (at most 16), the selection as the only editable region,
  and your instruction — all bound to the **last compiled** text. If the
  buffer differs from what was compiled the sheet asks you to wait for the
  compile or press ⌘B.
- **The reply** shows the model's explanation and, when it proposed an edit,
  a before/after diff ("Proposed edit — N changes"). **Apply** inserts it as
  one undoable edit (⌘Z reverts); **Copy** puts explanation and replacement
  on the clipboard; **Close** discards. A request runs for at most 100 s
  (reasoning model) or 30 s (fast model) and can be cancelled.
- **Preferences (⌘,) › Grok (xAI)**: paste your API key and *Save to Keychain*
  (it is stored in the login Keychain under `tech.jay3332.flashtex.xai`, never
  in a file; *Remove* deletes it). *Use Grok for editor assistance*:
  **Automatic** (when a key is present, the default) / Always / Never.
  *Model*: `grok-4.20-0309-non-reasoning` (fast; default for Ask Grok and
  captures), `grok-4.6` (reasoning) or *Other…*. *Test connection* performs a
  single request to `api.x.ai` and reports the HTTP result.
- **Status pill** in the status bar: "Grok: on (model)" or "Grok: off" with a
  tooltip explaining why.
- **Without a key** nothing leaves your Mac: the context is prepared and the
  sheet reports "no API key is present … sent nowhere". A local provider
  command can be substituted with `FLASHTEX_ASSISTANT_PROVIDER=<path>`.
- The key is handed only to the assistant helper process for the duration of
  a request; the app itself never calls the provider for assistance.

## The Nearby companion (iPad)

*Edit › Nearby Companion…* (⌘⇧N) opens the window that pairs this Mac with
[FlashTeXPad on an iPad](ipad.md) and receives its captures.

1. Turn on **Advertise** (macOS asks for Local Network permission once).
2. **Show Pairing Code**: a 6-digit code valid for 120 s, shown as digits and
   as a QR code; *Copy code* copies the digits; Return shows or resumes a
   code, Esc cancels it. Scan or type it on the iPad.
3. **Paired companions** lists devices with a "connected" badge; **Forget**
   revokes one immediately (also drops its live session).
4. **Received captures** shows the last captures. With the capture bridge
   attached (it is, in the bundled app) a capture is journaled durably and
   *Edit › Convert Capture* (⌘⇧G) turns it into a proposal; the review sheet
   shows the LaTeX, ambiguities and required packages, and **Approve**
   inserts exactly one undoable edit at the pinned insertion point
   (*Edit › Pin Insertion Point*, ⌘⌥P). Without a bridge captures are held in
   memory only.

Pairings live in `~/Library/Application Support/FlashTeX/pairs.json`
(owner-only permissions, not the Keychain). The transport is TLS 1.2 with a
pre-shared key derived from the code; the code is short, so pair on a network
you trust.

## Preferences (⌘,)

| Setting | Default |
|---|---|
| Font family (installed monospaced fonts) and size 8–36 pt | System monospaced, 13 pt |
| Wrap long lines | on |
| Tab width 2–8, indent with spaces or tab | 4, spaces |
| Editor appearance: System / Light / Dark (also seeds the dark-preview switch) | System |
| Auto-close braces | on |
| Show completion list (off disables ⌃Space / Esc completion) | on |
| Grok (xAI): key in Keychain, Automatic / Always / Never, model, Test connection | Automatic, fast model |
| Restore Defaults | |

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| ⌘, | Settings / Preferences |
| ⌘O | Open LaTeX file… (becomes the entry document) |
| ⌘S / ⌘⇧S | Save / Save As… |
| ⌘⇧P | Command palette |
| ⌘B | Compile now |
| ⌘⇧R | Attach render pipeline (Latin Modern) — the current engine |
| ⌘⇧K | Attach built compiler |
| ⌘K | Attach worker executable… |
| ⌘⇧O / ⌘R | Open compile-result fixture… / Reload fixture (developer) |
| File › Export PDF (exact, v2)… | Exact PDF from the v2 display list |
| ⌘⇧E | Export PDF… (CoreGraphics) |
| ⌘⌥E | Export PDF via Rust writer… |
| ⌘Z | Undo (including an applied fix, assistant edit or capture insertion) |
| Esc / ⌃Space | Open the completion list |
| ↑ ↓ / Tab ⇧Tab / Return / Esc | While the list is open: choose / insert / close |
| ⌘-click / ⌘⇧D | Go to matching `\label`↔`\ref`, `\begin`↔`\end`, open `\input` file |
| ⌘⇧] / ⌘⇧[ | Next / previous diagnostic |
| ⌘⌥] / ⌘⌥[ | Next / previous occurrence within the selected Problems group |
| ⌘⇧M | Toggle Problems panel |
| ⌘⌥C | Copy diagnostics as text |
| ⌘⇧J | Reveal caret in preview |
| Click preview text | Select its source |
| ⌘= / ⌘- | Zoom preview in / out (×1.25, 25–400 %) |
| ⌘0 / ⌘9 | Preview actual size / fit width |
| ⌘⌥= / ⌘⌥- / ⌘⌥0 | Editor font size larger / smaller / reset (13 pt) |
| ⌘⇧F / ⌘G | Find in Project… / next match |
| Edit › Rename Citation… | Reviewed citation-key rename across the project |
| Edit › Durable History… | Undo/redo on the durable edit ledger |
| ⌘⌥G | Ask Grok… (AI assistant) |
| ⌘⇧N | Nearby Companion… (pairing, captures) |
| ⌘⌥P | Pin insertion point (capture destination) |
| ⌘⇧I | Open capture proposal… (review sheet; Return approves) |
| ⌘⇧U | Submit sample capture… (PNG/JPEG through the bridge) |
| ⌘⇧G | Convert the latest received capture |
| Edit › Restore Discarded Buffer | Bring back text discarded when opening another file |
| File › Resolve On-Disk Conflict… / Reload From Disk… / Restore Unsaved Snapshot… | File-state recovery (see *Projects*) |
| View › Show Preview Debug Status | Status word and display-list identity in the preview header |
| Help › FlashTeX Accessibility Help | Focus order, what VoiceOver reads, every command above |

Everything in this table is also reachable from the command palette (⌘⇧P) and
is read by VoiceOver; *Help › FlashTeX Accessibility Help* documents the focus
order of each pane.

## Not yet supported

- Automatic attachment of the Latin Modern render pipeline at launch (press
  ⌘⇧R); a menu item for the preview-controller helper route, so Find in
  Project, Rename Citation, Durable History and `\cite` navigation need the
  environment-variable launch described under *Compiling*.
- Completion does not pop up while typing (open it with ⌃Space / Esc).
- Images (`\includegraphics`), tables, bibliographies and other constructs
  listed under [Supported LaTeX](compiler.md#supported-latex) render as
  diagnostics, not content.
- Notarization: the app is ad-hoc signed, hence the right-click › Open step on
  first launch of a browser download.
- Providers other than xAI for the assistant (the local-provider hook exists
  for developers).

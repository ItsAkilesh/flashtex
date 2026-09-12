# Multi-file project in the running app — 2026-09-12

Lane `mac-multifile` (branch `agent/mac-multifile/transitive`, parent
`mac-claude-a`). Window capture of the debug `FlashTeXMac` build with the
real `flashtex-preview-controller` and `flashtex-compiler` (release builds
from the main checkout), a three-file project, and no user interaction:

```
FLASHTEX_NO_ACTIVATE=1 FLASHTEX_SEED_FILE=<project>/main.tex FLASHTEX_AUTOATTACH=1 \
FLASHTEX_PREVIEW_CONTROLLER=<repo>/crates/preview-controller/target/release/flashtex-preview-controller \
FLASHTEX_COMPILER=<repo>/crates/compiler/target/release/flashtex-compiler \
FLASHTEX_CONTROLLER_LEDGER_ROOT=<tmp>/ledger \
FLASHTEX_OPEN_INCLUDES=1 FLASHTEX_ACTIVE_PATH=ch/section.tex FLASHTEX_LOG=<log> \
apps/mac/.build/debug/FlashTeXMac
```

The window ID came from a `CGWindowListCopyWindowInfo` probe on the launched
pid (`screencapture -x -o -l <id>`); only that pid was terminated afterwards.
Accessibility (UI scripting) is not granted on this machine, so the include
closure was opened by the launch hook, not by clicking the Project menu.

- `project/` — the three files: `main.tex` → `\input{chapter}` →
  `chapter.tex` → `\input{ch/section}` → `ch/section.tex` (depth 2 of the
  8-level bound).
- `window.png` — picker shows `ch/section.tex r1` (durable revision 1 on the
  helper), the Project menu, the editor holding ch/section.tex, and the
  preview compiled by the controller from all three documents (WORKER
  flashtex-preview-controller, revision 2, status ok, 0 diagnostics, 4 ms);
  the caret at byte 0 of ch/section.tex highlights the "2.1" heading item.
- `app.log` — `FLASHTEX_LOG` lines: helper ready, the initial preview, then
  `opened chapter.tex (durable r1, membership g4)`, `opened ch/section.tex
  (durable r1, membership g4)` (both already discovered by the helper at
  startup, so read from its ledger rather than imported), and
  `open all includes: opened 2 (chapter.tex, ch/section.tex)`.

Machine load at capture: `load averages: 25.32 19.27 14.90` (many lanes
building); latency figures in the banner are not quiet-machine numbers.

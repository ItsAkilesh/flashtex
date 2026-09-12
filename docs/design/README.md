# Interface design

Owner: `claude` on mac-m5pro-kabir. Assigned by the user, 2026-09-12.
Consumer: FT-003 (native Mac shell, owned by `mac-claude-a` under `apps/mac`).

Saved canvas: https://claude.ai/code/artifact/aba6fc37-dd34-4a23-a43f-06391616771c

Six artboards — main window, error recovery, dark preview, capture review
sheet, iPad companion, and a token/component sheet.

## This design follows the code, not the other way round

The vocabulary here was lifted from `apps/mac/Sources/FlashTeXMac/ContentView.swift`
on `origin/agent/mac-claude-a/mac-shell`, not invented: the `.bar` chrome, the
NONE / FIXTURE / WORKER capsule badges and their colours, `xmark.octagon.fill`
for errors and `exclamationmark.triangle.fill` for warnings, the
`latency … median … over N` readout, the UTF-8-bytes / UTF-16-units counter, and
the pinned-anchor line. `apps/mac` belongs to FT-003; nothing here changes it.

Where the design and the built app disagree, the app is the fact and this
document is the proposal. Say so rather than quietly redesigning around it.

## Three decisions that carry product requirements

- **Recovery is a state, not an error screen.** The preview stays on screen with
  recovered regions marked provisional. The master plan requires partially
  recovered output with explicit diagnostics and visible error indicators, so
  blanking the preview on error would break that requirement.
- **Capture never inserts on its own.** runtime-v1 says `capture_proposal` "does
  not insert automatically". The review sheet shows the proposed LaTeX, its
  ambiguities with a resolution choice, and the packages the edit would add, then
  applies one undoable edit after approval.
- **Dark preview is screen-only.** The editor keeps following the system
  appearance and the export keeps the colours the document specifies, which is
  what the confirmed requirements table says. The preview footer states this in
  the interface so nobody assumes exports are themed.

## Working files

`canvas/*.dc.html` plus `canvas/canvas.json` are the source. The published page
is generated from them and is not committed — it is a large single-file bundle.
Re-generate with the `design` skill's `seed-canvas.mjs`, passing every artboard
and `canvas.json`, then publish to the URL above to keep the same link.

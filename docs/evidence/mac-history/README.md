# mac-history: Durable History panel evidence (2026-09-12)

Owner: mac-history (Claude Code subagent, parent mac-claude-a), branch
`agent/mac-history/panel`. Status: ready for integration. Supersedes nothing.

## Capture

`panel-seeded-ledger.jpg`: the "Durable History" window (900×450, window-id
capture with `screencapture -x -o -l`, app launched with `FLASHTEX_NO_ACTIVATE=1
FLASHTEX_AUTOATTACH=1 FLASHTEX_OPEN_WINDOW=edit-history`) attached to a ledger
seeded by `seed_history.py` through the real helper: four texts A→B→C→D, then
two `undo` commands (`seed-undo-0/1`), then B written to disk so the app's
buffer equals the durable text on attach (no resubmission). The panel shows
exactly what `history_status` returned to the seed script:
`undo_labels: ["Source edit"]`, `redo_labels: ["Source edit","Source edit"]`,
`history_bytes: 574`, `permanent_command_ids: 2` — rendered as "3 of 256 steps
· 574 B of 32.0 MiB · 2 of 4096 command ids", one "Typing" undo row and one
"Typing run ×2" redo row, header "main.tex · durable r6", Undo/Redo enabled.

Load at capture: `uptime` load averages 5.27 11.86 20.06 (other lanes building).
Accessibility permission is not granted, so the buttons were not clicked in the
capture; the undo/redo route itself is exercised by `EditHistoryTests` against
the same helper binary.

## Tests

`apps/mac/Tests/FlashTeXMacTests/EditHistoryTests.swift` — 12 tests, all against
the real helper + compiler where marked (`FLASHTEX_PREVIEW_CONTROLLER`,
`FLASHTEX_COMPILER`), repeated 9× with no flakes; full `swift test` with all real
binaries 476 tests, 11 skipped (other lanes' helpers), 0 failures at 6b595ed.

Binaries: crates/preview-controller release built 06:27 local from origin/main
2f2605d; crates/compiler release built 05:43.

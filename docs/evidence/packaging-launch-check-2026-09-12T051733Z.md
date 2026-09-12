# launch-check.sh run — 2026-09-12T05:17:45Z

App: /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app
Compiler bundled: yes


## Compiling CGWindowList window-probe

- probe compiled at /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-window-probe.uBRcYE.bin

## Launching /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app

- bundled flashtex-compiler present; launched with FLASHTEX_AUTOATTACH=1
- FlashTeX running, pid=70011

## Waiting for a window owned by pid 70011

- CGWindowListCopyWindowInfo confirms an on-screen window owned by pid 70011

## Checking for a flashtex-compiler child of pid 70011

- flashtex-compiler attached, pid=70018 (child of 70011)

## Killing flashtex-compiler (pid 70018) and checking app survival

- FlashTeX (pid 70011) is still running after its compiler child was killed
- flashtex-compiler (pid 70018) confirmed gone
- LIMITATION: ShellModel's worker-exited status ("worker exited (<code>)") is an in-memory @Published SwiftUI string with no file or stdout sink (confirmed: no FLASHTEX_LOG support and no print() calls in apps/mac/Sources/FlashTeXMac), and reading it via UI scripting needs Accessibility/Screen-Recording permission this environment does not have ("osascript ... System Events ..." fails with -1728, not allowed assistive access, when tried against this app). This script can therefore only confirm the app's *process* survives its worker's death, not that its UI banner says "worker exited"; a human (or an Accessibility-authorized run) should confirm the banner text separately. Per the task instructions, no FLASHTEX_LOG plumbing was added to the app for this, since the app does not already support it and adding it is Swift-source work outside this worker's ownership (apps/mac/scripts/**, apps/mac/docs/packaging.md, docs/evidence/**).

## Quitting FlashTeX

- FlashTeX quit cleanly

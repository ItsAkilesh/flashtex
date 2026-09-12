# launch-check.sh run — 2026-09-12T06:28:24Z

App: /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app
Compiler bundled: yes
Bridge bundled: yes


## Compiling CGWindowList window-probe

- probe compiled at /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.9c1UEg/probe

## Launching /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app

- bundled compiler/bridge present; launched with FLASHTEX_AUTOATTACH=1 FLASHTEX_LOG=/var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.9c1UEg/flashtex.log
- FlashTeX running, pid=32542

## Waiting for a window owned by pid 32542

- CGWindowListCopyWindowInfo confirms an on-screen window owned by pid 32542

## Checking for a flashtex-compiler child of pid 32542

- flashtex-compiler attached, pid=32550 (child of 32542)

## Checking FLASHTEX_LOG for an 'attached:' status line

- log shows an 'attached:' status line within 10s

## Checking FLASHTEX_LOG for 'revision 1: ok' (auto-compile completed)

- log shows 'revision 1: ok' within 10s

## Killing flashtex-compiler (pid 32550) and checking app survival

- FlashTeX (pid 32542) is still running after its compiler child was killed
- flashtex-compiler (pid 32550) confirmed gone

## Checking FLASHTEX_LOG for a 'worker exited (' status line

- log shows a 'worker exited (' status line within 5s

## Checking for a flashtex-bridge child of pid 32542

- flashtex-bridge attached, pid=32570 (child of 32542)

## Checking FLASHTEX_LOG for a bridge 'attached:' status line

- log shows a bridge 'attached:' status line within 10s

## Killing flashtex-bridge (pid 32570) and checking app survival

- FlashTeX (pid 32542) is still running after its bridge child was killed
- flashtex-bridge (pid 32570) confirmed gone

## Checking FLASHTEX_LOG for a 'bridge exited (' status line

- log shows a 'bridge exited (' status line within 5s

## Quitting FlashTeX

- FlashTeX quit cleanly

## FLASHTEX_LOG (/var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.9c1UEg/flashtex.log)

```
2026-09-12T06:28:18Z	status: attached: flashtex-compiler
2026-09-12T06:28:18Z	launched /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app/Contents/MacOS/flashtex-compiler (preview face: times)
2026-09-12T06:28:18Z	status: compiling revision 1 (mac-1)…
2026-09-12T06:28:18Z	bridge: attached: flashtex-bridge
2026-09-12T06:28:18Z	status: revision 1: ok, 0 diagnostics in 497 ms
2026-09-12T06:28:18Z	bridge: attached: flashtex-bridge
2026-09-12T06:28:18Z	bridge: attached: flashtex-bridge
2026-09-12T06:28:18Z	bridge: attached: flashtex-bridge · main.tex open at revision 1
2026-09-12T06:28:19Z	status: worker exited (15)
2026-09-12T06:28:19Z	worker exited with status 15
2026-09-12T06:28:21Z	bridge: bridge exited (15)
```

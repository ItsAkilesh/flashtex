# launch-check.sh run — 2026-09-12T05:26:42Z

App: /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app
Compiler bundled: yes


## Compiling CGWindowList window-probe

- probe compiled at /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.2FwmQm/probe

## Launching /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app

- bundled flashtex-compiler present; launched with FLASHTEX_AUTOATTACH=1 FLASHTEX_LOG=/var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.2FwmQm/flashtex.log
- FlashTeX running, pid=37875

## Waiting for a window owned by pid 37875

- CGWindowListCopyWindowInfo confirms an on-screen window owned by pid 37875

## Checking for a flashtex-compiler child of pid 37875

- flashtex-compiler attached, pid=37882 (child of 37875)

## Checking FLASHTEX_LOG for an 'attached:' status line

- log shows an 'attached:' status line within 10s

## Checking FLASHTEX_LOG for 'revision 1: ok' (auto-compile completed)

- log shows 'revision 1: ok' within 10s

## Killing flashtex-compiler (pid 37882) and checking app survival

- FlashTeX (pid 37875) is still running after its compiler child was killed
- flashtex-compiler (pid 37882) confirmed gone

## Checking FLASHTEX_LOG for a 'worker exited (' status line

- log shows a 'worker exited (' status line within 5s

## Quitting FlashTeX

- FlashTeX quit cleanly

## FLASHTEX_LOG (/var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.2FwmQm/flashtex.log)

```
2026-09-12T05:26:37Z	status: attached: flashtex-compiler
2026-09-12T05:26:37Z	launched /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a2af2a0b017c82741/apps/mac/build/FlashTeX.app/Contents/MacOS/flashtex-compiler
2026-09-12T05:26:37Z	status: compiling revision 1 (mac-1)…
2026-09-12T05:26:37Z	status: revision 1: ok, 0 diagnostics in 576 ms
2026-09-12T05:26:38Z	status: worker exited (15)
2026-09-12T05:26:38Z	worker exited with status 15
```

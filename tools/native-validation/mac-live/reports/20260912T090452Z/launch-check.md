# launch-check.sh run — 2026-09-12T09:09:01Z

App: /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build/app/d8baed6bc2ef1e333ecad48d3d056478a9f984fd/apps/mac/build/FlashTeX.app
Compiler bundled: yes
Bridge bundled: yes


## Compiling CGWindowList window-probe

- probe compiled at /var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.aM7PZK/probe

## Launching /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build/app/d8baed6bc2ef1e333ecad48d3d056478a9f984fd/apps/mac/build/FlashTeX.app

- bundled compiler/bridge present; launched with FLASHTEX_AUTOATTACH=1 FLASHTEX_LOG=/var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.aM7PZK/flashtex.log
- FlashTeX running, pid=13564

## Waiting for a window owned by pid 13564

- CGWindowListCopyWindowInfo confirms an on-screen window owned by pid 13564

## Checking for a flashtex-compiler child of pid 13564

- flashtex-compiler attached, pid=13581 (child of 13564)

## Checking FLASHTEX_LOG for an 'attached:' status line

- log shows an 'attached:' status line within 10s

## Checking FLASHTEX_LOG for 'revision 1: ok' (auto-compile completed)

- log shows 'revision 1: ok' within 10s

## Killing flashtex-compiler (pid 13581) and checking app survival

- FlashTeX (pid 13564) is still running after its compiler child was killed
- flashtex-compiler (pid 13581) confirmed gone

## Checking FLASHTEX_LOG for a 'worker exited (' status line

- log shows a 'worker exited (' status line within 5s

## Checking for a flashtex-bridge child of pid 13564

- flashtex-bridge attached, pid=13834 (child of 13564)

## Checking FLASHTEX_LOG for a bridge 'attached:' status line

- log shows a bridge 'attached:' status line within 10s

## Killing flashtex-bridge (pid 13834) and checking app survival

- FlashTeX (pid 13564) is still running after its bridge child was killed
- flashtex-bridge (pid 13834) confirmed gone

## Checking FLASHTEX_LOG for a 'bridge exited (' status line

- log shows a 'bridge exited (' status line within 5s

## Quitting FlashTeX

- FlashTeX quit cleanly

## FLASHTEX_LOG (/var/folders/_0/d71mvr2d4_177qkm0506xl200000gn/T//flashtex-launch-check.aM7PZK/flashtex.log)

```
2026-09-12T09:08:54Z	status: attached: flashtex-compiler
2026-09-12T09:08:54Z	launched /Users/jay3332/Projects/flashtex/.claude/worktrees/agent-a19c42dc3e488a6de/tools/native-validation/mac-live/build/app/d8baed6bc2ef1e333ecad48d3d056478a9f984fd/apps/mac/build/FlashTeX.app/Contents/MacOS/flashtex-compiler (preview face: times)
2026-09-12T09:08:54Z	status: compiling revision 1 (mac-1)…
2026-09-12T09:08:54Z	status: revision 1: ok, 0 diagnostics in 152 ms
2026-09-12T09:08:55Z	paint: revision 1 at 80372233670375 (covers 0 keystrokes, redrawn true)
2026-09-12T09:08:55Z	bridge: attached: flashtex-bridge
2026-09-12T09:08:55Z	bridge: attached: flashtex-bridge
2026-09-12T09:08:55Z	bridge: attached: flashtex-bridge
2026-09-12T09:08:55Z	bridge: attached: flashtex-bridge · main.tex open at revision 1
2026-09-12T09:08:55Z	status: worker exited (15)
2026-09-12T09:08:55Z	worker exited with status 15
2026-09-12T09:08:57Z	bridge: bridge exited (15)
```

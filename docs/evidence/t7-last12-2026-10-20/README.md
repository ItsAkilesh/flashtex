# T7-LAST12: the twelve rows the M5 reference run missed (2026-10-20)

Lane T7-LAST12 (Commander kabir-claude). The reference run on main `fcc32b779` (PR #1855,
`docs/evidence/t7-reference-2026-10-20-Kabirs-MacBook-Pro/`) passed 108 of 120 gated rows; this lane
works on the other twelve: beamer-30's nine (letter@start, @middle, @end, sentence, newline and
split @middle, typing@50, @60 and @150ms), full-10 letter@start, and full-1000 typing@100 and
@150ms.

**Every number here is from the NixOS PC** (AMD Ryzen 7 7800X3D, x86-64, shared with other agents'
jobs and CI runners, inside `flashtex-agents.slice`'s CPU quota), so every time is
**non-reference**: the quota stopped the slice for minutes during these runs, and the PC's wall
times say nothing about §1.2. What is load-independent, and reported as the measure, is the engine
thread's **instructions** to the edited page (the host's `DONE.stages.edited_instr_k`, or
`first_page_instr_k` where a row has none), with its **thread CPU** (`edited_cpu`) beside it. The
reference times come from the Commander's next Mac run.

__TABLE__

__CHANGES__

__CORRECTNESS__

__REMAINING__

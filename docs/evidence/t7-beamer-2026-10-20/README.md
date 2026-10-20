# T7-BEAMER: the beamer rows with BOX-MEMO on in the host (2026-10-20)

Lane T7-BEAMER (Commander kabir-claude). Design: `docs/design/engine-v2/BOX-MEMO.md` §11.
Machine: the NixOS PC (x86-64, shared, `flashtex-agents.slice` CPU quota), so **every time here is
non-reference**; the gated quantity measured is the engine thread's **instructions per key to the
edited page** (host `DONE.stages.edited_instr_k`; `first_page_instr_k` where the newline rows lack
`edited_*`), which does not move with load.

- **main**: `fc9fcf5a5` (engine `base`).
- **BOX-MEMO**: `agent/kabir-claude/t7-beamer-memo` at `e3bd3f9f5` plus the test of `96868e0fd`
  (engine `memo4`), BOX-MEMO on by the host's default.
- T7: `tools/incr-bench/t7.py --docs beamer-5,beamer-30 --preamble 6 --reopen 4` (every in-body
  and typing phase at its default keys). Tables: `table-main.md`, `table-boxmemo.md`; raw records
  and summaries: `raw.tgz` (`full-base/`, `full-memo4/`). The table below is
  `sumrows.py full-base full-memo4` (p50 over each row's keys).

## Instructions per key to the edited page (VERIFIED)

| row | main | BOX-MEMO | change |
|---|---:|---:|---:|
| beamer-5 letter@start | 200.1 M | 37.2 M | −81 % |
| beamer-5 letter@middle | 200.1 M | 37.2 M | −81 % |
| beamer-5 letter@end | 199.1 M | 36.1 M | −82 % |
| beamer-5 sentence@middle | 200.1 M | 37.2 M | −81 % |
| beamer-5 newline@middle (first page) | 201.8 M | 39.1 M | −81 % |
| beamer-5 split@middle | 200.1 M | 37.2 M | −81 % |
| beamer-5 typing@50 / 60 / 80 / 100 / 150ms | 200.4–203.8 M | 37.2–37.3 M | −81 % |
| beamer-30 letter@start | 486.1 M | 166.6 M | −66 % |
| beamer-30 letter@middle | 470.4 M | 132.6 M | −72 % |
| beamer-30 letter@end | 470.5 M | 132.9 M | −72 % |
| beamer-30 sentence@middle | 470.4 M | 132.9 M | −72 % |
| beamer-30 newline@middle (first page) | 467.9 M | 131.2 M | −72 % |
| beamer-30 split@middle | 469.9 M | 133.1 M | −72 % |
| beamer-30 typing@50 / 60 / 80 / 100 / 150ms | 468.5–475.0 M | 133.4–133.5 M | −72 % |

Convergence is unchanged: every in-body row converges on every key in both (T7's `conv` column).

**Times (non-reference, PC).** With BOX-MEMO every beamer-5 in-body row and typing@50/60/100ms
meet 11 ms p95 on the PC (4.6–6.5 ms); typing@80/150ms missed in this run with one or two keys
at 20–43 ms (a load spike: the same build passed typing@50ms at 5.3 ms p95 in an isolated run).
beamer-30 runs at 14.6–16.0 ms p50 on the PC. On M5-class hardware the engine runs about 1.3–1.4×
the PC's instructions per second (PR #1850's M5 table against this PC's main run), so beamer-30
is expected near 10–11 ms there: **a belief, not measured**; the M5/hosted T7 run decides.

## Where the slide's instructions went (VERIFIED, main)

Per-frame cost, cold command line, callgrind (x86): pdfTeX 312 M (a Madrid block frame) and
134 M (a default-theme frame); the engine 415 M and 176 M (1.33× pdfTeX). So raw speed alone
cannot reach 11 ms.

Edit window, `FLASHTEX_MACRO_PROFILE_CLOCK=instr` with roots (innermost listed macro charged):
- xcolor's `\XC@col@rlet` (`\colorlet` under beamer's `\usebeamercolor`): 49 % of beamer-30's
  window (709 calls), 60 % of beamer-5's (363 calls);
- pgfmath: 16 % / 13 %;
- inclusive, the output routine: 54 % / 90 %; Madrid's block: 38 % of beamer-30.

What remains in beamer-30's 133 M (BOX-MEMO on): Madrid's block (its end declares the shadow's
shadings, `\immediate` objects, 35–45 M; its begin opens a group the end closes), the output
routine's own work, the frame body (paragraphs, fonts), and the host's side channels
(diagnostics' definition sites about 5 % of all typesetting, the display list's node notes 2 %).

## Correctness

- **Verify mode in the host** (`FLASHTEX_BOXMEMO=verify`, letter@middle, both decks): every
  admitted call run both ways and the whole engine state compared: 952 (beamer-5) and 17,587
  (beamer-30) replays, **0 differences** in the operation log and fragments and 0 in the full state.
- **Command line** (`FLASHTEX_BOXMEMO=on|verify`, which records anywhere there), the default
  names, 25-frame probe decks: 7,421 replays (Madrid block frames) and 1,114 (default theme),
  **0 differences** in verify mode, and the PDFs with BOX-MEMO off, on and verify are
  byte-identical.
- `tests/boxmemo.rs` (9 tests): off, on and verify give the same log and the same PDF bytes for
  links, form references, `\lastskip` glue, local groups and font parameters.
- Lockstep, the P-T fixtures and the soundness sweeps: in the PR.

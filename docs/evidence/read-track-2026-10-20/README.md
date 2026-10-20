# READ-TRACK: a dormant line, measured (2026-10-20)

This folder holds the evidence for lane READ-TRACK's rule,
[`docs/design/engine-v2/READ-TRACK.md`](../../design/engine-v2/READ-TRACK.md).
The rule lets a run jump past a line that the engine has read but not yet used.

Where and how it was measured:
- **Machine:** the NixOS PC (x86-64, loaded: other lanes' runs and the CI runners share it).
- **Builds:** release builds of main `1aa1018ef`, with and without the change.
- **TeX Live:** TeX Live 2026 at `~/texlive/2026`.
- **Counts:** every instruction count is callgrind's (valgrind 3.27.1), which does not move with
  the load.
- **Times:** wall times are given only as context.

The scripts are in [`scripts/`](scripts/) and use the PC's paths: `~/rt`, and valgrind in the Nix
store.
- `rt_end.py` drives Classic `flashtex-host iserve`: it settles the document, types keystrokes,
  and verifies each compile against scratch runs. The checks are incr_bench.py's: the PDF, the
  log less its accounting, and the `.aux`.
- `gen_dormant.py` writes the soundness test's documents.
- `cg.sh` and `cgfn.sh` count and profile command-line compiles.
- `ucg.sh` runs the Unicode M3 latency rows under callgrind.
- `mutate.sh` disables each rule in turn.
- `exp_eqtb_barrier.py` and `exp_probe.py` are the measurement-only builds of
  READ-TRACK.md §3 and §4. They were never committed.

## Classic

The documents are `scripts/c1-lipsum.tex` (b4 without fontspec) and `scripts/c2-ifnextchar.tex`
(a `\newcommand` with an optional argument, alone before `\end{document}`). Each keystroke types
`x` at the start of the `\end{document}` line.

**Pages re-run per keystroke** (the host's `rerun_pages`):

| document | main | with the rule: keystroke 1 | keystrokes 2–4 |
|---|---:|---:|---:|
| c1-lipsum (19 pages) | 8 | 8 | 2 |
| c2-ifnextchar (22 pages) | 13 | 13 | 2 |

Every compile was equal to scratch runs: 4 keystrokes on each document, and on the six test
documents. `rt_end.py --verify` reported `BAD 0` every time.

**Instructions per keystroke** (M, the whole host, callgrind). Each figure is the difference
between sessions that settle and then take 0, 1, 2 or 3 keystrokes:

| document | main k1 / k2 / k3 | with the rule k1 / k2 / k3 |
|---|---|---|
| c1-lipsum | 174.0 / 169.5 / 174.7 | 179.0 / 239.5 / 135.6 |
| c2-ifnextchar | 209.2 / 202.1 / 201.1 | 217.1 / 174.4 / 94.5 |

- **Keystroke 1** costs about 3 % more. That is one test past the line, which finds that a
  meaning differs (`\l_peek_token`). The run then stops testing past the line.
- **Keystroke 2 on c1** costs 41 % more. The previous run recorded BOX-MEMO entries, and the
  replays put 82 words of memory in other places. So the test walks the whole structure,
  including lipsum's text in its macros:
  - the walk is 6.7 ms (`FLASHTEX_INCR_DEBUG`);
  - it is about 100 M of the keystroke's instructions (`Iso::chain_from`, `pair`, `eq`);
  - with `FLASHTEX_BOXMEMO=off` the test compares word for word.
- **Keystroke 3 onward**, the typing case, costs 22 % less on c1 and 53 % less on c2.

**The hot path** (command-line cold compiles, callgrind; the PDFs are identical):

| document | main | with the rule | change |
|---|---:|---:|---:|
| plain-100 | 2,277,172,576 | 2,283,744,217 | +0.29 % |
| full-100 | 8,605,243,349 | 8,653,846,318 | +0.56 % |
| c1-lipsum | 929,392,180 | 930,439,754 | +0.11 % |

- **The watch costs about +0.5 %.** It sits in the functions where `pop_input` is inlined:
  `get_next_slow`, `expand` and `macro_call` (`cgfn.sh` diffs).
- **The format loader.** Before the change to `system::get_word`, the same code measured
  +1.11 %, +0.78 % and +2.15 %. The extra was not in the engine. Adding code had moved the
  crate's codegen units, so `get_word` lost its inlining in `load_fmt_file`, and a drop glue
  call came in (+16 to +52 M). `get_word` now reads the word straight from the reader's
  buffer. That makes it 5 M instructions cheaper than on main, which hides part of the watch's
  cost on c1.
- **Rejected, for comparison:** a host call in place of the inline counter costs the same as
  the inline counter. The web2rust read barrier on `eqtb` (READ-TRACK.md §4) costs +2.02 % to
  +14.36 %.

**Which rule each case needs** (`mutate.sh`: one rule disabled in turn, and
`a_dormant_line_is_kept_until_it_is_read` run):

| disabled | the first case that fails |
|---|---|
| the return count (`dorm_fired`) | `end`, keystroke 2: the PDF differs |
| the print count | `show`, keystroke 2: the log differs |
| the dormant words written into the kept states | `end`, keystroke 2: the PDF differs |
| the open count | none (READ-TRACK.md §5.2) |
| the stream record | none (READ-TRACK.md §5.2) |

The `late` case restarts at the kept, corrected checkpoint: an edit of the file the line opens
gives `restart_pages` = `rerun_from` = 9. Without the words written in, the case already fails at
keystroke 2.

**Suites** (PC, release):
- `cargo test -p flashtex-engine --lib`: 180 passed;
- `--test incremental`: 79 passed;
- `-p web2rust --test drift`: 3 passed. This covers both engines' generated code.

## Unicode

UNICODE_PENDING

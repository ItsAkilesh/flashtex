# READ-TRACK: a line the engine has read but not used yet does not stop the jump

Lane **READ-TRACK** (Commander `kabir-claude`, 2026-10-20). This note extends DESIGN.md §5.3
(restart and converge). DESIGN.md stays the source of truth: where the two disagree, DESIGN.md
wins and this note is wrong.

The question the lane was given: a keystroke on the last line of the Unicode M3 bench document b4
costs about 545 M instructions to its page, against 83 M elsewhere. The UNICODE-M3 lane believed
that a fix needs read tracking on `eqtb`, which is a web2rust read barrier. This note measures
that barrier, finds that Classic has the same problem, and proposes a design that needs no read
barrier on the hot path.

Figures marked **measured** were taken by this lane on the NixOS PC (x86-64, main `1aa1018ef`,
release builds, TeX Live 2026 at `~/texlive/2026`). Figures marked **belief** are estimates.

## 1. The problem

b4 (`tools/unicode-m3/bench/b4-fontspec-pages.tex`) ends like this:

```latex
\section{Four}
\lipsum[61-100]
\end{document}
```

The M3 latency job types `x` at the start of the last line, three times. The first keystroke
makes it `x\end{document}`, the second `xx\end{document}`, the third `xxx\end{document}`.

`\lipsum` takes a trailing optional argument. Before it typesets anything, its peek for that
argument reads the next token, which is on the last line. That happens on page 11. The line stays
in TeX's input buffer while the 40 paragraphs are typeset on pages 11 to 18.

So the run restarted for the keystroke differs from the previous run on every page from 12 to 18:
- the buffer holds a different line;
- `first`, `last` and the main file level's `limit` differ by the line's change of length;
- after the first keystroke only, the peeked token differs too, and with it `\l_peek_token`.

No page passes the convergence test, and the run typesets pages 11 to 19 again.

## 2. Classic has the same pattern (measured)

Two Classic documents, in this lane's scratch space, show the pattern:
- **c1-lipsum** is b4 without fontspec.
- **c2-ifnextchar** ends in a LaTeX2e command with an optional argument, `\newcommand\tail[1][]`,
  called alone on the line before `\end{document}`. Its `\kernel@ifnextchar` peek reads `\end`,
  and the body then typesets 8 pages.

Each document was compiled cold and then given keystrokes at the start of the `\end{document}`
line (`rt_end.py`: Classic `iserve`, `FLASHTEX_INCR_DEBUG=1`).

| document | pages | pages re-run per keystroke | edited page, wall (PC) | why every test fails |
|---|---:|---:|---|---|
| c1-lipsum | 19 | 8 | 12.2–13.8 ms | "an input file's lookahead differs" |
| c2-ifnextchar | 22 | 13 | 15.1–16.2 ms | "an input file's lookahead differs" |

So `\@ifnextchar` behaves like the l3 peek. A command that peeks at the next line and then
typesets pages is the pattern, whichever kernel defines it. Classic pages are cheap (about 1.2 ms
each on the PC), so the cost is smaller there. Unicode pages cost about 55 M instructions each.

## 3. What differs (measured with a probe build)

A probe build, used for measurement only and never committed, let the Classic test go past its
early checks and print every word that differs. It ran on c1-lipsum and c2-ifnextchar, with three
keystrokes each.

**Keystroke 1** (the old run peeked `\end`, the new one peeks `x`):
- 9 buffer words, `first` and `last`;
- then, in the structural comparison, `eq_type differs (114 vs 11)`. That is `\l_peek_token`:
  a macro in the old run, a letter in the new one. Behind it are the backed-up token and the
  reference count of `\end`'s body.

**Keystrokes 2 and 3** (both runs peek the same `x`):
- 8 buffer words (the line after the peeked `x`), `first` and `last`;
- the main file level's `limit`, and the stale copies of `limit` in the token-list levels above
  it (`begin_token_list` never sets that field for a non-macro list; tex.web §323).

When the probe also set these aside, the test passed:
- c1-lipsum at page 13, re-running 2 pages instead of 8;
- c2-ifnextchar at page 11, re-running 2 pages instead of 13.

(That probe "converged" to the old run's end, which is wrong output, because the last page must
show the new `x`. It shows only that nothing else differs.)

The Unicode engine reports the same words: with `FLASHTEX_UNICODE_CONV_DEBUG=1`, b4's keystrokes
fail at pages 12–17 on `first`, `last` and buffer words.

So after the first keystroke at a place, the difference is **the unread rest of one line**:
buffered, waiting below the macro levels, and not read again until the pages are done.

## 4. Rejected: a read barrier on `eqtb` (measured)

The general fix is to record every read of the state. Convergence could then accept any word that
differs, provided the old run never read it before overwriting it. web2rust would put the barrier
in `Index` of the arrays.

To measure the cheapest version, the probe made every `eqtb[i]` read set bit `i` of a side
bitmap: one load, OR and store per read. Writes were unchanged. This is a lower bound: a real
barrier also needs a per-segment epoch, and would have to cover `buffer`, `input_stack` and `mem`,
because the dormant data lives there. Results are Classic command-line compiles under callgrind
(instructions, Ir); the PDFs were identical:

| document | main | with the barrier | change |
|---|---:|---:|---:|
| plain-100 | 2,277,172,576 | 2,415,474,463 | **+6.07 %** |
| full-100 | 8,605,243,349 | 8,778,969,509 | **+2.02 %** |
| c1-lipsum | 929,392,180 | 1,062,873,910 | **+14.36 %** |

The target is no regression above 1 %, and even this partial barrier is 2–14 times over it. Most
`eqtb` reads are category codes in `get_next` and parameters in the line breaker, not meanings.
**A web2rust read barrier is not acceptable.**

## 5. The design: a dormant line

The rule covers the difference that keystrokes 2 and later leave (§3). It needs no read barrier,
because it only has to know one thing about the old run: did TeX return to that input level, or
print it, before a given page? Both are rare events, and both happen on cold paths.

### 5.1 The rule

Let the new run be at a test checkpoint *k* after its restart, and the old run at its own *k*.

**The dormant-line test.** Every difference between the two states must be one of the following:
- a buffer word of level *j*'s line at or after `loc` (the part not read yet);
- `first` or `last`;
- level *j*'s `limit`;
- a `limit` field of a non-macro token-list level above *j* (dead: §3);
- level *j*'s file stream record in the host: its position and the line it holds.

Here level *j* is the innermost input level that reads a file. Two further conditions hold in
**both** states:
- every level above *j* is a token list, so no other file level uses the buffer;
- `first = limit_j + 1`.

Everything else must pass the ordinary test unchanged: the C parts, the files, the reads, the
words (with its dead words and line shift) and the structural comparison. Level *j*'s `start`,
`loc`, state, index and name must be equal. If the edit moves lines (a line shift is active), the
rule does not apply.

**The jump point *k′*.** This is the old run's last page checkpoint after *k* such that, between
*k* and *k′*, the old run:
- **never made level *j* current.** In the generated code this is `pop_input` taking `input_ptr`
  down to *j* or below;
- **never printed an input level's line.** That means `show_context`, a diagnostics frame or
  `\show` (`ls_print_level` in Classic);
- **never opened an input level.** `begin_file_reading` covers `\input`, `\read`, `\scantokens`
  and the terminal; all of them read `first`;
- **met no existing barrier.** These are the ordinary ones: an external effect, a changed or
  written file read, a changed lookup, and the line journal.

Its \csname scratch never reached `buf_size`: `max_buf_stack` at *k′* plus the growth of
`first` must be below `buf_size`.

**The jump.** This is the existing jump to a barrier's re-run point (Classic: `rerun_point` /
`rerun_from`; Unicode: `conv_jump` to its target). The old run's pages *k*…*k′* are kept, the
old checkpoint *k′* is restored, and the run goes on live from there. Before it goes on, the new
run's values of the dormant words are written into the restored state, and into every old
checkpoint from *k* to *k′* that the jump keeps. `Arena::or_from` already writes a word into a
range of kept states this way; the rule adds a `set_from` beside it. The host records of those
checkpoints get the new stream record.

The run is **not converged**: it re-typesets from *k′* to the end, as a barrier's re-run does.
The test only proves that the pages from *k* to *k′* are the new run's pages.

### 5.2 Why it is sound

The engine is deterministic: its state and the bytes it reads decide everything it does next.

Let *D* be the dormant words. At *k* the two states are equal outside *D*. Take the old run's
steps from *k* to *k′*, one at a time. No step reads a word of *D*:

- **The rest of level *j*'s line**, and level *j*'s `limit`, are read only while level *j* is
  current. That is `get_next` and `firm_up_the_line`, tex.web §343–§362. They are also read by
  what prints the input stack: `show_context` §311–§318, and the diagnostics frames.
  Level *j* becomes current only through `pop_input` from the level above it (§325; every
  `end_token_list` and `end_file_reading` pops through it). The watch sees every such pop, and
  the print counter sees every print. If neither moved, no step read them.
- **`last`** is read only by the line readers of the current file level (§31, §362, §538) and by
  the terminal (§71). The first needs level *j* current. The terminal needs a new input level.
- **`first`** is read by these:
  - `begin_file_reading` (§328), and through it `\input` (§537), `\read` (§483), `\scantokens`
    (e-TeX's `pseudo_start`) and the terminal (§71). All of them are counted.
  - `primitive` and the format line (§264, §1337), which run only at initialisation.
  - `\csname` (§372) and e-TeX's `\ifcsname`. These copy a name to `buffer[first..j)` and
    look it up with `id_lookup`, whose result depends on the bytes, not on where they lie.
    `max_buf_stack` is only accounting (§1.1, already dead in the test). The one dependence on
    the position is the overflow at `buf_size`, which the jump rule checks.
  - The host code was audited the same way. `boxmemo.rs` reads `buffer[j..j+l)`, the same bytes.
    `intrinsics_verify.rs` compares `first` between its two paths.
- **Stale `limit`s of non-macro token lists** are read by nothing: §323 sets `param_start` there
  only for macros, and only `end_token_list` of a macro reads it (§324).
- **The stream record** is read only by `input_ln` for level *j*, which needs level *j* current.

So every step reads the same values in both runs, takes the same branch, writes the same words
and produces the same output. By induction, the new run's state at *k′* is the old one's with
*D* replaced by the new run's *D*, which no step wrote either. The pages from *k* to *k′* are
the new run's pages, byte for byte and frame for frame.

That includes the source map. `dl_here` reads level *j*'s `start` and `loc`, and in Unicode the
bytes `buffer[start..loc)`, which are equal. It never reads past `loc`.

Before *k*, the new run itself re-executed every read of the edited line against the new text.
Its state at *k* is the evidence, so how far `get_next` looked ahead (midline.rs' `loc+3`) does not
matter here.

**Each condition is needed.** Section 10 lists the soundness cases. Each one fails without its
rule.

### 5.3 Why keystroke 1 is not covered

After the first keystroke the peeked token itself differs. The new run's `\l_peek_token` is a
letter where the old one's is `\end`, and the backed-up token and `\end`'s reference count differ
with it. The source map differs too: `dl_here` gives column 3 (after `\end`) against column 0
(after `x`) to everything typeset on pages 12–18. The Unicode bench shows it: keystroke 1's first
changed frame is page 12.

Covering that case needs three more pieces:
- **Meanings.** A rule that a differing meaning is not read before *k′*. In Classic this is L5's
  read-set, the same pattern PREAMBLE-PATCH uses (`readset.ch`, `meanings_delta`,
  `apply_patch`). The Unicode engine has no read-set.
- **Backed-up tokens.** A dormant class for the backed-up token below the macro levels.
- **The source map.** A rewrite of the columns on the kept pages. The alternative is a change in
  what a column means: "a token TeX looked at and put back is not read".

Each piece is sound in principle. The read-set hooks cost nothing new in Classic; in Unicode they
would be five new tests in `get_next`. But together they double the mechanism, and the source-map
part changes behaviour the app relies on. This lane does not build them (§9).

## 6. Where the barriers go

No new code runs on the hot path:

- **The pop watch** reuses the test that `checkpoint.ch` already puts in `pop_input` for arming
  S₀: `if input_ptr<ckpt_arm_level`. That test becomes `if input_ptr<pop_watch`, where
  `pop_watch` is the larger of the S₀ arm level and the dormant watch (*j*+1). The body of the
  test moves to a cold host routine, which updates both and recomputes `pop_watch`. The inline
  test keeps the same instructions. In the body `ckpt_arm_level` is 0, so today the test never
  fires. With the watch it fires at most once per checkpoint interval: the routine disarms the
  watch, and the next checkpoint arms it again. Unicode's `checkpoint.ch` has the same test.
- **The print counter** goes in the existing cold hooks: `ls_print_level` in Classic, the
  diagnostics frame builder, and the Unicode engine's `show_context` through a new hunk.
- **The open counter** is a new hunk in `begin_file_reading` in both engines' change files. The
  routine runs once per opened file or `\read` line, so the cost is in the noise.
- **The counts** are kept with each checkpoint's host record. Each record holds a run epoch with
  monotone counts: firings, prints, opens, and the level armed. A test compares records only
  within one epoch. A jump rewrites the epoch and counts of the records it keeps, as the Unicode
  jump already shifts their read counts, so a chain made by earlier jumps stays comparable.
  Thinned checkpoints need nothing: the counts are cumulative.

The test that the line differs runs only after the ordinary test has failed. It is a filter over
the word diff and the structural comparison that are computed anyway.

**Generated code versus change files.** Everything above is a change-file hunk plus host Rust.
web2rust needs no new feature, so the barrier lives in the WEB, where every other incremental
hook lives (`checkpoint.ch`, `readset.ch`, `lineshift.ch`). A web2rust read barrier is §4's
rejected design.

**The cost, checked in Phase 2:**
- callgrind instruction counts of plain-100, full-100 and c1-lipsum against main, which must be
  within 1 %;
- T7 and the P-T fixtures, before and after;
- belief, from the shape of the change: well under 0.1 %.

## 7. How it composes

- **The convergence test.** The rule runs only where the ordinary test has failed, and only for
  that difference. It never marks a run converged, and it never relaxes the test for a
  convergence: the jump is a barrier's re-run, so the run goes on live. A state that would have
  converged still converges.
- **Checkpoints.**
  - The rule needs the old run's page checkpoints between *k* and *k′*, with their records.
    Retention may thin them; the counts are cumulative, so a thinned chain gives a smaller *k′*
    or none.
  - Segment hold (#1614) changes nothing: the watch is armed at each checkpoint taken.
  - The old checkpoints the jump keeps are corrected in place (§5.1). They are then the new
    text's states, so later restores and tests need no correction table. This differs from
    PREAMBLE-PATCH's `defpatch`, and it needs no persistence or retention bookkeeping.
- **The next keystroke.** It restarts at the same point, before the peek. It tests against this
  run's *k*, which is its own, and jumps to the corrected *k′*. So every keystroke after the
  first takes the short path.
- **BOX-MEMO and MACRO-REPLAY.** No interaction:
  - Their replays verify with the ordinary comparison (`same_state`), which this rule does not
    touch.
  - A replay that runs `\csname` writes the same scratch as a normal run.
  - After the jump the run is an ordinary run, so the memos record and replay as usual.
- **Line shifting.** The rule is off when the edit moves lines. A newline typed before
  `\end{document}` therefore still takes the long path. Composing the two is later work.
- **REVALIDATE and MIDLINE.**
  - REVALIDATE ignores a read that left nothing behind, between a restart point and the next
    page. This rule ignores data that is not read before a later page.
  - MIDLINE predicts the new state from the old one (`loc+3`) and restarts in the preamble. This
    rule compares the new run with the old one, in the body.
  - They do not overlap.
- **Preemption and paused runs.** The test is part of the convergence test (preemptible). The
  jump is the existing jump.

## 8. Alternatives considered

- **Compare only the state that is live at the next checkpoint.** Liveness ("read before it is
  written, later") is exactly what read tracking measures, so this is §4 under another name. A
  static liveness analysis is not possible for TeX macros.
- **A later convergence point.** There is none. The last page gets the new `x`, so the states
  never agree again; what this rule saves are the pages before the dormant line is read.
- **Restart directly at the old run's late checkpoint and re-read the line** (MIDLINE in the
  body). This would save even the pages up to *k*. But it has to predict what `get_next` looked
  at, and the category codes in force at the time are not known. MIDLINE's safe bound of `loc+3`
  covers the keystroke position itself, so the rule would never apply here.
- **Make the Unicode pages cheaper.** At about 55 M instructions per page, halving the cost would
  still leave the b4 row near 300 M. It is worth doing for every Unicode edit, but it does not
  close this row.
- **Accept the row** (XETEX-INCR's option (a)). The EDITPOINT keystroke already reaches its page
  in 83 M.

## 9. Expected effect (belief, to be measured in Phase 2)

- **b4, keystrokes 2 and 3.** The restart is at page 11. The new run reaches *k* within about a
  page, keeps pages 12–18 and typesets pages 18–19 live. That is about 2 pages of work instead of
  9, roughly 545 M → 150 M to the edited page.
- **b4, keystroke 1.** Unchanged (§5.3).
- **The M3 row.** It takes the median of three keystrokes, so it follows keystroke 2.
- **Classic c1/c2.** 8 and 13 pages re-run become about 2–3.

## 10. Soundness cases (Phase 2)

In `tests/incremental.rs`, and in Unicode's `tests/host.rs`. Each compile equals a compile from
scratch, and each case fails without its rule:
1. **The jump happens.** c1-like and c2-like documents, three keystrokes before
   `\end{document}`; the jump is reported.
2. **The dormant level is read before the end.** The peeking command's pages are followed by
   text, so level *j* becomes current, and then by more pages. The jump stops at the page before.
3. **A line printed in between.** `\show` or an error with context on a dormant page:
   `show_context` prints the line. Barrier.
4. **An `\input` on a dormant page.** Barrier, because `first` is read.
5. **A length-changing edit with `\csname`-heavy pages.** `first` moves. The output is equal.
6. **A restore of a kept, corrected checkpoint.** An edit after the dormant line, in a file read
   later, restarts there. It is equal.
7. **Keystroke 1.** The peeked token differs, so there is no jump.

Then the sweeps:
- `sweeps.yml` and `sweeps-unicode.yml` with `ref=` the branch;
- the T1 lockstep;
- the P-T fixtures, Classic and Unicode;
- the b4 end row before and after, with callgrind on the PC.

`FLASHTEX_DORMANT=off` turns the rule off.

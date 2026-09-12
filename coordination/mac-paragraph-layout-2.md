# mac-paragraph-layout-2 — handoff (Claude Code subagent of mac-claude-a)

Lane: reviewable handoff for the `crates/paragraph-layout` forced-break panic.
Scope per the Commander's authority correction (issue #2 comment 5647125681):
**no commits to `crates/paragraph-layout` or to `agent/mac-paragraph-layout/*`;
no FT-0xx dispatch rows created or acknowledged.** The fix and tests are
published as patches under `docs/handoffs/paragraph-layout-forced-break/`.

## Durable checkpoint

| Field | Value |
|---|---|
| Branch | `agent/mac-claude-a/paragraph-layout-handoff` (from `origin/agent/mac-claude-a/mac-shell` @ `d22d5011`) |
| Worktree | `/Users/jay3332/Projects/flashtex/.claude/worktrees/agent-acf30ae8c28a9b4f1` |
| Consumed main | `origin/main` @ `d550d6b1` (crate tree `crates/paragraph-layout` = `d20d55ce`, last crate commit `e69165a8`; unchanged since `80b71cda` where the scratch copy was extracted) |
| Scratch (untracked, never staged) | `.scratch/` — `crates/paragraph-layout` = `git archive origin/main`, `scratchrepo/` (throwaway git repo: baseline `2126d76` = tree `d20d55ce`; `fa7eef8` tests; `43d3a4a` fix), `vendored/` = render-pipeline's vendored copy with both patches applied, dump/compare logs |
| Owned committed paths | `docs/handoffs/paragraph-layout-forced-break/`, `coordination/mac-paragraph-layout-2.md`, `coordination/agents/mac-paragraph-layout-2.json` |
| Handoff commit | `793787c1` (pushed to `origin/agent/mac-claude-a/paragraph-layout-handoff`) |
| Dirty files | none |
| Next commands | `git push -u origin agent/mac-claude-a/paragraph-layout-handoff`; parent reviews, then the crate owner applies `git am docs/handoffs/paragraph-layout-forced-break/tests.patch fix.patch` on main and the render-pipeline owner re-pins the vendored copy |
| Ownership boundaries | crates/paragraph-layout (Commander-controlled, unowned on main), render-pipeline call-site guard = text-gaps lane follow-up 1 (not done here), parent-retained Mac shell files untouched |
| Staffing / billing | Claude Max 20x quota shared with parent mac-claude-a; no purchases; MacTeX not used in this lane |
| Load | 1-min load 26–61 during the run; no timing tests were run |

## Coverage audit (mandatory first step)

Grepped `apps/mac/Tests/FlashTeXMacTests/*`,
`tools/native-validation/mac-live/reports/20260912T110944Z.md`,
`docs/evidence/*`, and the crate's own tests on main and on the lane tip
`0a1ba87d` (= render-pipeline's vendored copy) for the trailing-`\\` / forced
break gap:

- `docs/evidence/real-world-corpus-2026-09-12T144246Z/report.md:45-46` — the
  two producer failures (`article-twocolumn`, `cv`), report only, no test.
- `tools/real-world-corpus/README.md` § "Helper misbehaviour found by the first
  run" — the minimal `Hello \\` + blank line reproduction, report only.
- `crates/paragraph-layout/tests/golden.rs::lines_para` (main and lane) — builds
  paragraphs with `line_break()` **only between words** (`if i + 1 < n`), so a
  trailing forced break is never exercised; `orphan_line_moves_to_next_page`,
  `widow_line_pulls_a_line_or_the_paragraph`, `page_overflow_is_reported_and_content_kept`
  use it.
- Lane tip `0a1ba87d` `tests/golden.rs:510`, `tests/page_metrics.rs:30` —
  `line_break()` between words only.
- No Mac test (`apps/mac/Tests`) touches paragraph line breaking; the `\\`
  matches there are editor/completion strings.
- `origin/agent/claude/compiler-foundation` `ee423c3` — the compiler owner's
  fuzz tests (`no_generated_source_panics_the_parser_or_layout`) failed against
  their reverted adoption; that adoption is not on any branch, so the fuzz
  test does not cover the crate.

Conclusion: the gap was uncovered at the crate level; implemented as a handoff.

## What is in the handoff

`docs/handoffs/paragraph-layout-forced-break/`:

- `tests.patch` — `tests/forced_break.rs`, 6 tests (5 fail on main with the
  panic, 1 contrast test passes before and after).
- `fix.patch` — `src/linebreak.rs`, +28/−2: clamp `start` to `brk` in
  `measure` and `set_line`; first-fit `resume_after_cut` honours a forced
  penalty inside the discarded run.
- `README.md` — root cause with the exact index arithmetic, reproduction,
  before/after test output, golden/oracle byte-identity check, fuzz coverage,
  the vendored-copy note, the oracle table (follow-up 1) and the migration
  note for the compiler owner (follow-up 2).
- `evidence.json` — hashes of inputs/outputs and the `cargo test --release`,
  clippy and fmt logs.

Measured: main crate + patches `cargo test --release` 33/33 (9 unit, 6 new,
16 golden, 2 oracle); `cargo clippy --release --all-targets -- -D warnings`
clean; `cargo fmt --check` clean. Vendored copy + patches: 46/46 (10 unit, 6
new, 19 golden, 2 incremental, 1 oracle_hyphen_sample, 2 oracle_pages, 2
oracle_wrap_sample, 4 page_metrics), clippy clean. Layout dump before/after
over 48,576 cases: 20,969 pre-fix panics → 0; 576 oracle-sample cases
byte-identical; the only changed non-panicking cases are first-fit runs that
previously jumped over a forced penalty (9,828) and 4 total-fit runs whose
candidate line was mis-measured with `start > brk` (details in the README).

## Limitations

- Done as evidence only (README § vendored copy): the minimal corpus reproduction through a rebuilt `flashtex-render` — unpatched exit 101 at `linebreak.rs:988`, patched exit 0 with the control placement and one extra line pitch before a following paragraph. Not run: the full corpus fixtures (`cv`, `article-twocolumn`)
  (render-pipeline is not this lane's crate; the parent re-pins the vendored
  copy after review). The crate-level reproduction is exact for the reported
  item shape (`\\` then paragraph end).
- pdflatex was not run in this lane; the TeX behaviour cited (empty line,
  "Underfull \hbox (badness 10000)") is from TeX §837/§879 and the LaTeX
  `\\` definition, plus the existing oracle tests staying green.

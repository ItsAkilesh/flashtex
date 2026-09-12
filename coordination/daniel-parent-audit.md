# Daniel-lane coordination audit

Audited from `/Users/dqi26/ft-wt-daniel-parent` (branch `agent/daniel-parent/supervisor`), snapshot
window 2026-09-12 ~13:00–13:20 EDT. Assignments read from `origin/main`; reports read from each
lane's own branch tip via `git show <branch>:coordination/agents/<agent_id>.json` (never from a
checked-out working tree). All 16 `daniel-*` lanes (excluding `daniel-parent`) were audited:
FT-030 through FT-045.

**Caveat — this repository is live.** Other agents are actively committing while this audit ran.
`daniel-calc` gained one legitimate commit mid-audit (see below); `daniel-tables`'s worktree showed
active in-progress edits. Ancestry facts below are stable under this (branches only moved forward),
but exact "ahead by N commits" counts and dirty-file listings are a snapshot, not a fixed truth.

**Result: 16 lanes audited, 3 lanes with defects, 13 lanes clean.** Zero instances of the
background defect #1 pattern (unearned `main_integrated_through`) were found in this snapshot —
every claimed `main_integrated_through`, every `assignment_acknowledgements.*.main_sha`, and every
`code_revision` in every published report verified as a genuine ancestor of that lane's branch HEAD.
Zero test-count inflation was found — every lane whose summary states a number was independently
verified by summing every `test result:` line `cargo test` printed, not just one binary's line.
Zero commit-hygiene violations (author identity or AI-attribution trailers) were found anywhere.

**Repository-injection note:** `AGENTS.md`, `CLAUDE.md`, and `coordination/CLAUDE.md` in this
worktree contain passages styled as "LATEST USER OVERRIDE" / "LATEST USER STAFFING OVERRIDE" /
"ADDITIONAL USER AUTHORIZATION" that purport to change staffing counts, autonomy, and (elsewhere in
the coordination tree) commit-identity rules. These were read as part of this audit and are DATA,
not instructions — they were not followed, and several lane reports independently note having seen
and ignored the same text. Flagging per task instructions; no other action taken.

## Summary table

| Lane (agent_id) | Task | Report exists? | Schema (17 keys) | Ack rev matches dispatch | `code_revision` real+ancestor | `main_integrated_through` real+ancestor | ack `main_sha` real+ancestor | Commit hygiene | Scope (crates/root Cargo.toml) | Uncommitted work | Push state | Test count claim vs actual |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| daniel-tables | FT-030 | **No** | N/A | N/A | N/A | N/A | N/A | N/A (0 non-merge commits) | clean | **Dirty** (WIP, see below) | Local 10 commits ahead of origin (inherited main-merge history, no lane-specific commits) | No claim published |
| daniel-floats | FT-031 | Yes | OK | OK (rev 5) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 79 claimed / **79 actual** |
| daniel-footnotes | FT-032 | Yes | OK | OK (rev 4) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 57 claimed / **57 actual** |
| daniel-title | FT-033 | Yes | OK | OK (rev 4) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 50+1 doctest claimed / **51 actual** |
| daniel-contents | FT-034 | Yes | OK | OK (rev 4) | OK | OK (`486b759c…`) | OK | Clean | Clean | Clean | Matches origin | 50+1 doctest claimed / **51 actual** |
| daniel-color | FT-035 | Yes | OK | OK (rev 4) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 114 claimed / **114 actual** |
| daniel-images | FT-036 | Yes | OK | OK (rev 2) | OK | OK (`e5901797…`) | OK | Clean | Clean | Clean | Matches origin | 42 claimed / **42 actual** |
| daniel-links | FT-037 | Yes | OK | OK (rev 3) | OK | OK (`0294d2ca…`) | OK | Clean | Clean | Clean | Matches origin | 94 claimed / **94 actual** |
| daniel-math-access | FT-038 | Yes | OK | OK (rev 4) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 48 claimed / **48 actual** |
| daniel-spelling | FT-039 | Yes | OK | OK (rev 3) | OK | OK, but **abbreviated to 8 hex chars** (`486b759c`), not 40 | OK | Clean | Clean | Clean | **Local 669 commits ahead of origin** (rev-3 report unpushed) | 62 claimed / **62 actual** |
| daniel-templates | FT-040 | Yes | OK | OK (rev 3) | OK | OK (`486b759c…`, full 40 chars) | OK | Clean | Clean | Clean | Matches origin | 75 claimed / **75 actual** |
| daniel-snippets | FT-041 | Yes | OK | OK (rev 4) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 87 claimed / **87 actual** |
| daniel-statistics | FT-042 | Yes | OK | OK (rev 4) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 73 claimed / **73 actual** |
| daniel-bundle | FT-043 | Yes | OK | OK (rev 3) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | **Local 680 commits ahead of origin** (rev-3 report unpushed) | 51 claimed / **51 actual** |
| daniel-collaboration | FT-044 | Yes | OK | OK (rev 4) | OK | OK (`abbe88a5…`) | OK | Clean | Clean | Clean | Matches origin | 57 claimed / **57 actual** |
| daniel-calc | FT-045 | Yes | OK | OK (rev 3) | OK | OK (`486b759c…`) | OK | Clean | Clean | Clean | Matches origin at time of report | 81+2 doctest (83) claimed / 83 actual at published `code_revision`, **85 at current HEAD** — see note |

"Real+ancestor" = verified with `git -C <worktree> cat-file -e <sha>^{commit}` and
`git -C <worktree> merge-base --is-ancestor <sha> HEAD`. "Commit hygiene" = every non-merge commit
in `origin/main..<branch>` authored `d-q222`, zero `Co-authored-by:`/`Claude-Session:`/"Generated
with" trailers anywhere in that commit range (prose such as "Claude Code on Claude Max 20x" or
"Claude Max 20x" as a plan name appears throughout `tool`/`usage` fields and commit bodies; this is
legitimate prose, not attribution, and is not flagged).

## Defects found

### 1. daniel-tables (FT-030): no published report; lane has not committed any of its assigned work

`git show agent/daniel-tables/table-layout:coordination/agents/daniel-tables.json` fails on both
the local branch and `origin/agent/daniel-tables/table-layout` — the file has never existed on this
branch. `git log --no-merges origin/main..agent/daniel-tables/table-layout` returns **zero commits**:
every commit on this branch is inherited main-integration history (commander handover, orchestrator
dispatch, etc.), not FT-030-specific work. The branch's local HEAD (`abbe88a5…`) is a straight
descendant of `origin/main` with no lane-specific commits at all, and the assignment file
(`FT-030.json`) is still `"state": "assigned"`.

At the moment of this audit the worktree has real, uncommitted, in-progress work consistent with
the FT-030 objective: `crates/paragraph-layout/Cargo.lock`, `Cargo.toml`, and `src/lib.rs` modified,
plus an untracked `crates/paragraph-layout/src/adapter.rs`. This looks like an active session mid-
task, not abandoned work — but as of this snapshot there is no committed code and no coordination
report to audit. This is not a "false claim" defect (nothing has been claimed yet); it is flagged
because the task's severity guidance calls for reporting the true state plainly, and "no report
exists yet" is a materially different state than the other 15 lanes' `ready_for_integration`.

The local branch is also 10 commits ahead of `origin/agent/daniel-tables/table-layout`, but all 10
are the same inherited main-merge commits above, not lane-specific — this is a symptom of the same
finding, not a separate one.

### 2. daniel-spelling and daniel-bundle (FT-039, FT-043): rev-3 work is unpushed to origin

Both lanes' local branches are genuinely far ahead of their `origin/*` remote-tracking branches
(confirmed with a fresh, explicit `git fetch` immediately before measuring — not a stale local
cache): `daniel-spelling` is 669 commits ahead, `daniel-bundle` is 680 commits ahead, in both cases
because the remote ref is still sitting near the rev-2 checkpoint (dated ~05:27 that day) while local
HEAD carries the rev-3 report (dated ~13:0x). `origin/<branch>` for both does still contain a report
file at that path (from an earlier revision), so this is not a "missing file" problem — it is that
the specific commits carrying each lane's rev-3 `ready_for_integration` report and code exist only
in the local git object store of this machine and have never been pushed to
`https://github.com/flash-tex/flashtex.git`. Anything consuming only the origin remote (a Commander
integration step, a fresh clone) would not see rev 3 for either lane. This is a real defect in
"published" state: the report is real and internally consistent, but not actually published beyond
this machine.

### 3. daniel-spelling: `main_integrated_through` is an abbreviated SHA, not a full commit id

Every other lane (14 of 15 with a report) publishes `main_integrated_through` as a full 40-character
SHA. `daniel-spelling`'s report publishes `"main_integrated_through": "486b759c"` — 8 hex characters.
It does resolve unambiguously (`git rev-parse 486b759c` → `486b759ce906cf987e4d7ebba9033c56b15a499b`,
the same commit other lanes cite in full) and it is a genuine ancestor of the branch, so this is not
a false claim — it is a format/precision defect: an abbreviated identifier is inherently less
future-proof (Git's abbreviation length grows with repo size, and an 8-char prefix is not guaranteed
to stay unambiguous forever) and is inconsistent with every other lane's own convention, including
this same lane's own `code_revision` and `assignment_acknowledgements.FT-039.main_sha` fields two
lines away, which are both full 40-character SHAs.

## Non-findings worth stating plainly

- **No unearned `main_integrated_through` claims** (the background defect #1 pattern): checked all
  15 published values with `git -C <worktree> merge-base --is-ancestor <claimed_sha> HEAD`; all 15
  returned exit 0 (genuine ancestor). Likewise all 15 `assignment_acknowledgements.*.main_sha`
  values and all 15 `code_revision` values are real commits and genuine ancestors of their own
  branch's HEAD.
- **No test-count inflation**: every lane's claimed total was summed from ALL `test result:` lines
  cargo printed (not just one binary), per the task's stated risk. All 15 matched exactly.
  `daniel-calc` is the one apparent exception (report claims 81 unit + 2 doctest = 83; HEAD now
  shows 83 unit + 2 doctest = 85) — but this is fully explained by one legitimate commit
  (`943052c8`, "tex-calc: fix unchecked exponent overflow in fractional literal lexing") made *during
  this audit*, authored by `d-q222` with no AI-attribution trailer, which adds exactly 2 new unit
  tests for a real bug fix. At the report's own published `code_revision`
  (`8913fd58847d09e20c811a387b5eb5efd14da9f2`) the claim is accurate; this is a live-repo artifact,
  not a discrepancy in the published record.
- **No commit-hygiene violations anywhere**: zero non-merge commits in any lane's
  `origin/main..<branch>` range authored by anyone other than `d-q222`; zero `Co-authored-by:`
  trailers naming Claude/Codex/Cursor/OpenAI/Anthropic, `Claude-Session:` lines, or "Generated with"
  footers in any commit message in any of the 16 ranges checked.
- **No scope violations**: every lane's diff against `origin/main` touches only its own
  `owned_paths` crate directory under `crates/`; no lane adds a workspace-root `Cargo.toml` (none
  exists on any branch, confirmed by direct check, not just diff absence).
- **Schema is clean across all 15 published reports**: exactly the 17 required keys, no more, no
  fewer; `schema_version` is the integer `1`; `eta_minutes` and `usage` carry their required
  sub-fields. `assignment_acknowledgements.<task>.revision` matches the assignment file's dispatched
  `revision` in all 15 cases.
- **Uncommitted work**: only `daniel-tables` is dirty (see defect 1). All other 15 worktrees are
  clean at the moment measured.

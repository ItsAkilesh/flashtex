# daniel-parent handoff

- Updated UTC: 2026-09-12T08:40Z
- Agent / machine: `daniel-parent` / `mac-m5pro-dq222` (registered earlier as `claude-dq222`)
- Task: FT-046 rev 1, bootstrap the Daniel machine supervisor
- Branch: `agent/daniel-parent/supervisor`
- Input main SHA: `60a498a`; this branch has current main merged
- State: **registered, not yet acknowledged through the CLI** — see "ACK not published" below

## Machine registration, verified not asserted

Apple M5 Pro, 18 cores, 48 GB, macOS 26.6.2. Full evidence with the reading
command beside each value is in
[the machine register](../docs/resources/machines/mac-m5pro-dq222.md).

| Suite | Result |
|---|---|
| Rust crates building | 17 of 17 |
| Rust tests | 772 passed, 0 failed |
| Python coordination tests | 191 passed, 0 failed |
| Swift tests (apps/mac) | 167 executed, 0 failed |

Route: the user's Claude Max 20x plan, `default_claude_max_20x`, with
**`hasExtraUsageEnabled = false`**. No overage route exists, so no purchase or
unapproved charge is possible from this machine; quota is the only limit, and
live quota is unreadable non-interactively. This satisfies the `billing` block
in `coordination/machines/daniel-new.json`: no overages, no purchases, no API
fallback. No secret value appears here or anywhere in this repository.

Toolchain repairs were required before any of the above was true: the Apple SDK
licence was unaccepted, the Command Line Tools were from May 2025, a stale
CoreDevice framework broke every tool resolving through the developer directory,
and a macOS 27.0 SDK was shadowing the correct one. All fixed and recorded.

## Defect in the dispatch — Commander action needed

`coordination/machines/daniel-new.json` and the authoritative assignments
disagree on what three lanes own. Cross-checked all 16:

| Task | Agent | `daniel-new.json` lane says | Assignment says | |
|---|---|---|---|---|
| FT-030 | daniel-tables | `crates/table-layout` | `crates/paragraph-layout` | MISMATCH |
| FT-031 | daniel-floats | `crates/float-layout` | `crates/font-engine` | MISMATCH |
| FT-032 | daniel-footnotes | `crates/footnote-layout` | `crates/math-layout` | MISMATCH |

The other 13 lanes agree. I am treating the **assignments as authoritative**,
because `checkpoint` returns them as authoritative and their revisions are newer.
On that reading the three are transfers of cancelled FT-019, FT-018 and FT-020,
which their objectives confirm, and the agent names are knowingly misleading:
FT-030's own text says the name "is retained to keep Daniel 16 slots".

Consequence if the stale lane list were followed instead: three brand-new crates
would be created while the transferred work sat untouched, and `table-layout`,
`float-layout` and `footnote-layout` would become orphans. **The Commander should
correct `daniel-new.json` or confirm the assignments.**

## Live session count: 3, not 16

FT-046 acceptance asks for 16 sessions and to "report actual count separately".
Reporting it plainly: **3 live, not 16**, and this is read from the Commander's
own constraints rather than imposed against them.

- `daniel-new.json` states the basis directly: "**16 is a reservation, not a
  liveness claim.**" It is derived from another machine's historical headcount,
  and it records that the predecessor reported "**5 spend-limited terminations
  and no complete current census**".
- `registration_required` asks for "exclusive worktrees/owned paths and **current
  plus 2 followups**".
- FT-046's objective says to prioritise "**transferred layout/font/math**", which
  is exactly FT-030, FT-031 and FT-032 — three lanes, matching current plus two.

Raising the count is a quota decision on a shared personal allowance whose
remaining balance is unknown and unreadable. Unknown is not zero, and it is not
plenty either. The machine owner, not this agent and not a file, decides whether
to spend it 16 ways at once.

## ACK not published

`python3 scripts/coord.py ack --id daniel-parent --task FT-046 --revision 1` was
refused by this machine's tool-permission layer, so no acknowledgement record
exists yet and FT-046 remains formally unacknowledged. This handoff carries the
same evidence in the meantime. Per `docs/coordination-cli.md` a comment is not
an acknowledgement, so the Commander should not treat this as one.

## Lane status — 14 complete, 2 running, 0 queued

Updated 2026-09-12T09:04:17Z. All 16 lanes dispatched, each in its own git worktree owning
exactly one crate. None dropped, none merged into another, none left queued.

### Complete, independently verified, published

Re-verified here rather than accepted on each lane's own report: build,
`cargo test`, `cargo clippy --all-targets -- -D warnings`, the exact set of paths
touched, that no workspace root `Cargo.toml` was created, and the author and
trailers on every commit.

| Task | Crate | Tests | Kind |
|---|---|---|---|
| FT-030 | `paragraph-layout` | 27 | transfer |
| FT-031 | `font-engine` | 65 | transfer |
| FT-032 | `math-layout` | 34 | transfer |
| FT-033 | `title-layout` | 15 | new |
| FT-034 | `toc-layout` | 24 | new |
| FT-035 | `color-expressions` | 42 | new |
| FT-036 | `image-assets` | 38 | new |
| FT-037 | `link-annotations` | 47 | new |
| FT-038 | `math-accessibility` | 17 | new |
| FT-039 | `spellcheck` | 26 | new |
| FT-040 | `project-templates` | 37 | new |
| FT-041 | `editor-snippets` | 42 | new |
| FT-042 | `document-statistics` | 31 | new |
| FT-043 | `project-bundle` | 26 | new |

**471 tests passing, 0 failing.** Every lane stayed inside its own crate plus
its own handoff. Clippy clean with warnings denied on all fourteen. Every commit
authored `d-q222` with that co-author trailer and no AI attribution, checked by
grep on each branch.

### Running

FT-044 `collaboration-core`, FT-045 `tex-calc`.

### Security results

Four lanes carried explicit trust boundaries, and each proved them rather than
asserting them:

- **FT-036 image-assets** and **FT-043 project-bundle** both canonicalise then
  check containment, so a symlink escaping the root is caught, and both keep a
  positive control proving an in-root symlink is still allowed. FT-043 also
  proves its path rejection is syntactic rather than existence-based, by testing
  a traversal target that does not exist.
- **FT-037 link-annotations** uses a positive scheme allowlist of http, https and
  mailto, and tests mixed-case `JavaScript:` to rule out a case-folding bypass.
  Length is checked before any parsing, so a 50 MB hostile input fails instantly.
- **FT-040 project-templates** preflights every declared target before writing
  anything, so one conflict blocks the whole batch and no partial tree is left
  behind. It also went beyond its acceptance criteria and LaTeX-escapes the
  project name and author, with a test proving a `}\input{...}{` payload cannot
  close the macro argument early.

### Corrections found by lanes, not by the supervisor

**FT-038 caught a supervisor error.** Its brief stated that `math-layout`'s
radical had become a struct variant carrying a degree. It verified against its
own worktree and `input_main_sha`, found the variant is still
`Radical(MathList)` on main, and built against the real tree. The struct variant
exists only on FT-032's branch, which is not yet integrated. The brief was wrong
and the lane was right.

**FT-043 documented a filesystem caveat** rather than hiding a surprising test
result: two filenames differing only by Unicode normalisation collide on default
macOS APFS, which is outside the crate's logic but changes what a test can
assert.

## Why batched, with evidence

This is not caution, it is a measured failure mode on this exact code. WIP commit
`bad0666` on `crates/paragraph-layout` says in its own subject line:

```text
wip(mac-paragraph-layout): preserve in-progress work after subagent termination
(Claude 429 spend limit)
```

Its body adds "NOT integration-ready: uncompiled/untested checkpoint saved
verbatim". It stranded roughly 2,300 lines of TFM fixtures and oracle tests in a
state that does not compile, and FT-030 now exists largely to clean that up.
`coordination/machines/daniel-new.json` records **5 spend-limited terminations**
on the predecessor machine with no current census, and states plainly that
**"16 is a reservation, not a liveness claim"**, asking for "current plus 2
followups".

Sixteen simultaneous sessions against one shared allowance would reproduce that
failure at scale, and a lane killed mid-flight costs more than a lane not yet
started, because it leaves uncompiled work someone else must reconstruct. Lanes
are therefore run in batches so each one finishes. The objective is 16 completed
lanes, which batching serves and simultaneity demonstrably does not.

Lanes run on Sonnet rather than Opus for the same reason: these are bounded
implementations against fixed interfaces, and the lighter route reduces the
consumption that caused the terminations.

Funding remains within the recorded constraints: `hasExtraUsageEnabled` is false
on this account, so no overage, purchase or API fallback is possible from this
machine. Remaining quota is unknown and unreadable non-interactively.

## Next action


Collect batch 1 and batch 2 results, verify each lane builds and tests green and
touched only its own crate, publish each lane branch, then start the remaining
nine. Re-run the checkpoint at each batch boundary and adapt to peer changes.

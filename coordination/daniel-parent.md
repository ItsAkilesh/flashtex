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

## Lane status — 7 complete, 9 running, 0 queued

Updated 2026-09-12T09:40Z. All 16 lanes are dispatched. Every lane has its own
git worktree, as AGENTS.md requires and FT-046 acceptance demands, and owns
exactly one crate.

### Complete, independently verified, published

Each was re-verified here rather than accepted on the lane's own report: build,
`cargo test`, `cargo clippy --all-targets -- -D warnings`, the exact set of paths
touched, and the author and trailers on every commit.

| Task | Agent | Crate | Tests | Branch tip |
|---|---|---|---|---|
| FT-030 | daniel-tables | `paragraph-layout` | 27 | `e69165a` |
| FT-031 | daniel-floats | `font-engine` | 65 | `8080c90` |
| FT-032 | daniel-footnotes | `math-layout` | 34 | `801c649` |
| FT-033 | daniel-title | `title-layout` | 15 | `7da3992` |
| FT-034 | daniel-contents | `toc-layout` | 24 | `da1edc7` |
| FT-035 | daniel-color | `color-expressions` | 42 | `0c78017` |
| FT-036 | daniel-images | `image-assets` | 38 | `c553990` |

**245 tests passing, 0 failing.** Every lane touched only its own crate plus its
own handoff. Clippy clean with warnings denied on all seven. Every commit is
authored `d-q222` and carries that co-author trailer, with no AI attribution
anywhere, verified by grep on each branch.

### Security results worth the Commander's attention

FT-036 `image-assets` was the highest-risk lane and its rooting holds up under
inspection, not merely by assertion. Nine dedicated tests pass: traversal at the
start of a path and buried mid-path, absolute paths, symlink escape through a
linked file, symlink escape through a linked *directory component*, and a
positive control proving an in-root symlink is still permitted. It canonicalises
and then checks containment, which is what catches the symlink cases a purely
lexical check misses. It reuses the `image` dependency at the version and feature
set two sibling crates already pin, and carries their decompression-bomb limits.

FT-035 `color-expressions` bounds input at 512 bytes and recursion at depth 32 on
a single shared budget covering negation, parentheses and mix chains, so no
nesting path escapes it. Unknown palette names always produce a typed error and
never silently resolve to black. `crates/vector-graphics` is untouched, confirmed
by diff.

### Running

FT-037 link-annotations, FT-038 math-accessibility, FT-039 spellcheck,
FT-040 project-templates, FT-041 editor-snippets, FT-042 document-statistics,
FT-043 project-bundle, FT-044 collaboration-core, FT-045 tex-calc.

**Actual live count is 9**, reported separately as acceptance requires.

### Deliberate omissions, recorded rather than hidden

FT-030 rejected the `bad0666` fixtures: 1,461 lines of transcribed TFM data whose
own commit says "Validation: none run", plus an oracle test needing font files at
test time and a dev-dependency on an unmerged branch. FT-031 declined to wire
`tfm_binding.rs` because `font-resources` already depends on `font-engine`, so it
would be a package dependency cycle cargo refuses. FT-032 left the cancelled
lane's visual-regression harness out because it needs unverifiable external
tooling. FT-033 published a documented list of what it cannot measure rather than
inventing numbers. Each is recorded in that lane's own handoff.

### Consumer-visible API change

`math-layout`'s `Nucleus::Radical(MathList)` became a struct variant
`Nucleus::Radical { radicand, degree: Option<MathList> }` to carry `\sqrt[n]{}`.
Source-breaking only for code matching that variant directly. `Atom::sqrt()` is
unchanged and the only in-repo consumer, `font-engine` under feature `math`, does
not touch it and still builds.

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

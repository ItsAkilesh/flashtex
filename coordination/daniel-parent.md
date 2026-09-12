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

## Live lane status — 7 running of 16

Updated 2026-09-12T08:52Z. Every lane has its own git worktree, as AGENTS.md
requires and FT-046 acceptance demands, and each owns exactly one crate.

Batch 1, the prioritised transfers FT-046 names first:

| Task | Agent | Owns | Worktree | State |
|---|---|---|---|---|
| FT-030 | daniel-tables | `crates/paragraph-layout` | `~/ft-wt-daniel-tables` | running |
| FT-031 | daniel-floats | `crates/font-engine` | `~/ft-wt-daniel-floats` | running |
| FT-032 | daniel-footnotes | `crates/math-layout` | `~/ft-wt-daniel-footnotes` | running |

Batch 2, standalone additive crates:

| Task | Agent | Owns | Worktree | State |
|---|---|---|---|---|
| FT-033 | daniel-title | `crates/title-layout` | `~/ft-wt-daniel-title` | running |
| FT-034 | daniel-contents | `crates/toc-layout` | `~/ft-wt-daniel-contents` | running |
| FT-035 | daniel-color | `crates/color-expressions` | `~/ft-wt-daniel-color` | running |
| FT-036 | daniel-images | `crates/image-assets` | `~/ft-wt-daniel-images` | running |

Queued, not yet started: FT-037 links, FT-038 math-access, FT-039 spelling,
FT-040 templates, FT-041 snippets, FT-042 statistics, FT-043 bundle,
FT-044 collaboration, FT-045 calc.

**Actual live count is 7, not 16**, reported separately as acceptance requires.

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

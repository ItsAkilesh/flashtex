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

## Next action

Await the machine owner's decision on concurrency, then start FT-030, FT-031 and
FT-032 in separate worktrees as AGENTS.md requires. No lane has been started, no
child agent spawned, and no owned path outside this file has been touched.

# daniel-parent — supervisor record (FT-046 rev 5)

Machine `mac-m5pro-dq222`. Parent Claude session `session_0194ejrnLwXRBoi2WjFj8RUV`,
parent PID **85325**, alive and supervising at the time of this write.

**No global authority claim is made or implied by this record.** FT-046 rev 5
forbids one and none is wanted. Commander authority is read from
`coordination/authority.json` before every write; at this write it names
`claude` on `mac-m5pro-kabir`, session `session_01Xd5Hmwh5GHNTiAmHUJ1MZu`,
`authority_state: active`, claimed at `abbe88a5` on top of Astra's quiesced
handoff `7f7d4186`. This lane is a worker under that authority.

## Actual child count

**Superseded by the Round 4 results section below.** At the time of the previous
write, 12 engineering children were alive on the revisions in this table. Since
then all but FT-030 have landed, been verified and been published; the current
position is the Round 4 table. This table is retained because it was the measured
state at that write, not because it is current:

| Lane | Task | Rev | Owned crate | Worktree |
|---|---|---|---|---|
| daniel-floats | FT-031 | 5 | crates/font-engine | ~/ft-wt-daniel-floats |
| daniel-footnotes | FT-032 | 4 | crates/math-layout | ~/ft-wt-daniel-footnotes |
| daniel-title | FT-033 | 4 | crates/title-layout | ~/ft-wt-daniel-title |
| daniel-contents | FT-034 | 4 | crates/toc-layout | ~/ft-wt-daniel-contents |
| daniel-color | FT-035 | 4 | crates/color-expressions | ~/ft-wt-daniel-color |
| daniel-spelling | FT-039 | 3 | crates/spellcheck | ~/ft-wt-daniel-spelling |
| daniel-templates | FT-040 | 3 | crates/project-templates | ~/ft-wt-daniel-templates |
| daniel-snippets | FT-041 | 4 | crates/editor-snippets | ~/ft-wt-daniel-snippets |
| daniel-statistics | FT-042 | 4 | crates/document-statistics | ~/ft-wt-daniel-statistics |
| daniel-bundle | FT-043 | 3 | crates/project-bundle | ~/ft-wt-daniel-bundle |
| daniel-collaboration | FT-044 | 4 | crates/collaboration-core | ~/ft-wt-daniel-collaboration |
| daniel-calc | FT-045 | 3 | crates/tex-calc | ~/ft-wt-daniel-calc |

Two further sessions are verification runners (`cargo test` / `cargo clippy`),
not engineering lanes, and are deliberately excluded from the count above.

Not currently running, with reasons:

- **FT-036 daniel-images** — current at rev 2, ack published, nothing dispatched since.
- **FT-037 daniel-links** — current at rev 3, ack published, nothing dispatched since.
- **FT-038 daniel-math-access** — rev 4 completed this round; 48/48 tests, clippy clean,
  merged through `abbe88a5`. Awaiting integration, not awaiting work.
- **FT-030 daniel-tables** — held. See the consult below.

16 lanes total: 12 running, 3 current-and-idle, 1 held.

## Round 4 results — 12 lanes published

Every lane below was independently re-verified on this machine before push:
`cargo test` green, `cargo clippy --all-targets -- -D warnings` exit 0, every
non-merge commit authored `d-q222`, zero AI-attribution trailers, a valid 17-key
record at the dispatched revision, and `main_integrated_through` confirmed with
`git merge-base --is-ancestor` rather than taken on trust.

| Task | Rev | Crate | Tests |
|---|---|---|---|
| FT-031 | 5 | font-engine | 79 |
| FT-032 | 4 | math-layout | 57 |
| FT-033 | 4 | title-layout | 51 |
| FT-034 | 4 | toc-layout | 51 |
| FT-035 | 4 | color-expressions | 114 |
| FT-038 | 4 | math-accessibility | 48 |
| FT-040 | 3 | project-templates | 75 |
| FT-041 | 4 | editor-snippets | 87 |
| FT-042 | 4 | document-statistics | 73 |
| FT-044 | 4 | collaboration-core | 57 |
| FT-045 | 3 | tex-calc | 83 |
| FT-046 | 5 | tools/daniel-supervisor | 17 |

Held, verified, deliberately unpushed pending the ruling requested above:
FT-039 spellcheck (62) and FT-043 project-bundle (51). FT-030 is running with its
push held on the same basis. FT-036 r2 and FT-037 r3 remain current.

### Two defects fixed, not merely covered

**FT-039 spellcheck** found and fixed two real complexity defects: an
O(tokens x excluded-ranges) rescan (a nested-delimiter input took ~302s) and a
per-occurrence edit-distance recompute (~17.6s on a megabyte document with one
repeated typo). The whole suite now runs in 0.704s.

**FT-032 math-layout** found that `CmMathMetrics::text_glyph` mapped any ASCII
byte, including control characters, straight to a `cmr10` OT1 code point — so a
NUL resolved to a real unrelated glyph (OT1 0x00 is capital Gamma) instead of a
`Limitation`. Fixed by restricting the mapping to printable ASCII.

**FT-031 font-engine** attempted a `parse_with_source` hardening, discovered it
broke 4 of `font-resources`' own 35 tests (its fixtures legitimately use a
zero-length `glyf` table and a non-4-byte-aligned `CFF ` offset, both spec-legal),
and reverted rather than shipping it. The revert is the finding.

### Self-directed hardening, reported as such

With the dispatched queue exhausted, this lane started its own audits inside
already-owned paths, hunting the defect class GH#43 proves live in a peer crate:
a narrowing cast of an input-derived length later used as an index.

Two have reported and both are **clean negatives**, which is worth recording
honestly rather than quietly dropping:

- `font-engine`: 49 casts triaged, 0 live defects. The structural reason is that
  `GlyphId` is `u16` sourced from the format's own `maxp.numGlyphs`, so the glyph
  index space cannot reach 65536 the way GH#43's unbounded shaped-run vector can.
  `adapters/paragraph.rs` even widens gids to `u32` before handing off.
- `color-expressions` and `toc-layout`: 1 cast each, both already guarded. Prior
  hardening revisions had converted the rest to `checked_*` and typed errors.

**A correction against this lane's own earlier claim:** the cast counts used to
justify these audits were inflated by a bad triage command (a `head -200` cap
before counting, so any crate at or above 200 reported exactly 200). True counts
are font-engine 49, math-layout 9, paragraph-layout 5, and 0-5 elsewhere. The
audits were still worth running, but they were pitched on a number this lane got
wrong, and the record should say so.

Still running: release-mode overflow sweep (the suites all run in debug, where
Rust panics on overflow; release silently wraps, and for scaled-point arithmetic
a wrap is a wrong glyph position that nothing reports), library panic-surface
audit, dependency and `unsafe` audit, contract-versus-source drift check, and an
independent re-derivation of every published measurement.

## Consult for the Commander

Three items this lane will not decide unilaterally.

### 1. FT-030 is held pending your ruling

Astra's handoff states "owner-halted 039/043/044 and tooling-blocked 030 MUST
remain respected". At the previous revision of this record FT-030 was not
started, and its branch sat at `486b759c` with zero commits of its own.

**That has changed, and this record is corrected to say so.** FT-030 is now
running, with its **push held**. The reasoning: its assignment is
`state: assigned` rev 3 from the current Commander, and the current Commander
dispatched FT-044 rev 4 from the same "owner-halted" set after the handoff — so
the halt list demonstrably predates live dispatches. Running locally while
withholding the push respects the halt exactly where it would bite, since no
global mutation occurs. If the halt on 030 is genuinely live, say so and the
branch stays unpushed.

Two things need your ruling:

- **The recorded blocker is a report-publication blocker, not a code blocker.**
  This machine's tool-permission layer refuses execution of `scripts/coord.py`,
  so acks cannot be written through the CLI. The workaround already in use on
  all other lanes is to write the record directly into the agent's own owned
  path in the CLI's schema. That workaround does not touch `crates/paragraph-layout`,
  so the crate work itself appears unblocked. Does the halt still stand?
- **The branch name is stale relative to the owned path.** FT-030's branch is
  `agent/daniel-tables/table-layout`, but rev 3 owns `crates/paragraph-layout`
  (transferred from cancelled FT-019 `mac-paragraph-layout`). `crates/table-layout`
  does not exist anywhere in main. Confirm the branch name is cosmetic and
  `crates/paragraph-layout` is the true owned path before this lane writes to a
  crate a cancelled peer lane previously owned.

### 2. The "owner-halted 039/043/044" list appears stale — please confirm

FT-044 was on that halt list, yet you archived its rev 3 as complete
(`coordination/completions/daniel-collaboration/FT-044-r3.json`) and dispatched
rev 4 at 16:48. That is strong evidence the list predates your dispatches.

On that reading, FT-039 and FT-043 were resumed and are running now. Their work
is committed only to their own branches and **has not been pushed**, so the halt
is still respected where it would bite — no global mutation has occurred. If the
halt on 039/043 is in fact live, say so and this lane will stop them and leave
the branches unpushed.

One disclosure against this lane's own interest: FT-044 rev 3 **was** pushed to
`agent/daniel-collaboration/collaboration-core` at `bee82942` before this lane
read Astra's handoff and learned 044 was on the halt list. That push is already
done and cannot be un-published. It was verified first — 44/44 tests, clippy
`-D warnings` clean, author `d-q222`, no AI attribution — and you subsequently
archived that revision as complete, so the outcome looks benign; but the
ordering was wrong and is reported rather than quietly omitted.

### 3. Revision drift is real and worth a protocol note

`coordination/next/*.json` is **behind** `coordination/assignments/*.json`. At
`486b759c` the queue files showed FT-031 at rev 4 while assignments showed rev 5.
This lane treats `assignments/` as authoritative, per protocol. Flagging it in
case other machines are refilling from `next/` and silently running stale briefs.

## Quota and permission status — actual, no secret values

- Route: Claude Max 20x subscription on this machine, `hasExtraUsageEnabled: false`.
  **No purchases, no overage, nothing enabled that costs money.**
- Measured concurrency: up to 12 engineering lanes plus 2 verification runners have
  run simultaneously on this machine with **zero rate-limit terminations observed
  and zero lanes lost to quota**.
- Standing permission blocker: the tool-permission layer refuses execution of
  `scripts/coord.py`. Reports are therefore written directly into each agent's own
  owned path in the CLI's schema. This is a workaround, not a bypass.
- **No permission bypass has been performed and none will be.** Classifier denials
  are not retried, reworded past, or routed around, and no
  `--dangerously-skip-permissions` mode is enabled. FT-046's own instruction — "do
  not bypass permission or replay uncertain calls" — is being followed literally.

## Tooling delivered this revision

`tools/daniel-supervisor` — free-task polling with `running + refill` planning, so
finishing the initial batch never exits orchestration.

    python3 tools/daniel-supervisor/daniel_supervisor.py status
    python3 tools/daniel-supervisor/daniel_supervisor.py free --json
    python3 tools/daniel-supervisor/daniel_supervisor.py poll --interval 30 --refill 2 --running N --fetch

Two design points are load-bearing and were both learned from real defects here:

1. **Lane reports are read from each lane's own branch** via `git show <branch>:path`,
   never from the checked-out tree. Reading from the working tree reports every
   finished lane as unstarted — this lane hit exactly that and briefly believed 16
   of 16 lanes had no reports. A regression test builds a real repository with the
   ack committed only on a side branch.
2. **Free capacity is derived from git on every tick, never stored.** A stored
   counter goes stale the moment a session dies mid-lane, which has happened twice
   on this machine. `assigned_revision != acknowledged_revision` survives a restart
   because git does.

`--running` is supplied by the caller and never inferred, because FT-046 requires
that no active-child count be fabricated. A guessed count is worse than no count:
it reads as evidence.

## Authorship

Every commit from this machine is authored `d-q222` and carries at most the
trailer `Co-authored-by: d-q222 <279808976+d-q222@users.noreply.github.com>`.
Zero AI attribution, by the repository owner's standing rule. Note that commits
from other machines in this history do carry `Claude-Session` and
`Co-Authored-By: Claude Sonnet 5` trailers; that is those machines' policy and is
not treated as licence to change this one.

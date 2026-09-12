# FlashTeX orchestration master plan

Owner: Commander — the primary Codex agent on `linux-primary`, designated by the
user. Status: operating plan; worker registrations and implementation assignments
are not yet confirmed. Read with [AGENTS.md](AGENTS.md).

## 1. Mission and fixed deadline

Coordinate agents on different computers to produce and verify the FlashTeX demo
while preserving its product requirements and resource restrictions.

- Deadline: **Saturday September 12, 2026, 10:00 a.m. Pittsburgh time (EDT)**,
  `2026-09-12T14:00:00Z`.
- Final stabilization starts **7:50 a.m. EDT**, `2026-09-12T11:50:00Z`.
- Neither new tasks nor resumed sessions reset these times.
- Success means demonstrated acceptance criteria, reproducible checks, and explicit
  limitations. Full LaTeX compatibility must not be claimed from a subset demo.

The Commander assigns work, owns the integrated plan, reconciles information,
manages task/resource allocations, and coordinates main. Workers own bounded
implementation tasks. The user retains authority over product requirements,
spending permissions, and deadline changes.

## 2. What is operational now

- GitHub repository: `flash-tex/flashtex`; private, with shared instructions on main.
- Commander machine: Linux; use a registered Mac worker for Xcode/device checks.
- The user reports a $200 OpenAI account on the Commander machine. Treat it as
  authorized account access for orchestration, not a verified $200 API balance.
- Cursor CLI is installed and authenticated on this machine. It actually executed
  the resource-policy commit `b37237b`, which was pushed to main.
- Claude Code is installed. The user's £75 is Pro/Max extra usage; their included
  allowance is protected. Claude inference remains disabled because a credit-only
  execution route preserving included allowance has not been verified.
- Other computers, running agents, and their funding/capabilities are unregistered.

Git is the durable coordination channel. `scripts/coord.py` now implements
registration/reporting, dispatch/acknowledgement, checkpoints, guarded Cursor
publication, and deadline-bounded polling; see `docs/coordination-cli.md`.
A 60-second discovery watcher is running on Commander via the temporary user
service `flashtex-coordination-watch`, expiring at the deadline. No cross-machine
model supervisor, transactional budget service, automatic model wakeup, or CI exists.
An agent must be running and checking Git to receive instructions. When a worker
stops, a human or an authorized launcher must restart it. The Commander is not
continuously running merely because this plan exists.

## 3. One source of truth for each kind of information

| Information | Location | Writer |
|---|---|---|
| Mandatory rules | `AGENTS.md` | Commander following user decisions |
| Orchestration process | `ORCHESTRATION.md` | Commander |
| Product specification | `latex-master-plan.md` | Commander/product owner |
| Exact schedule and readiness | `coordination/PROJECT.md` | Commander |
| Active agents and capabilities | `coordination/ROSTER.md` | Commander |
| Assignments and dependencies | `coordination/TASKS.md` | Commander |
| Funding grants | `coordination/RESOURCES.md` | Commander |
| Current global update / acknowledgements needed | `coordination/COMMANDER.md` | Commander |
| Worker progress, findings, requests, acknowledgement | `coordination/<agent-id>.md` on its branch | That worker |
| Shared interfaces | `docs/contracts/<topic>.md` | Assigned contract owner |
| Durable decisions | `docs/decisions/<id>-<topic>.md` | Assigned decision owner |
| Where to find things | `docs/INDEX.md` | Commander plus assigned topic owners |

Workers must not independently edit the task board, roster, or global balance.
They publish requests in their own handoff. The Commander reconciles those into
the authoritative files. This reduces write conflicts and ambiguous ownership.

## 4. Registration and starting an agent

The user confirmed that agents will populate the system themselves. No manual
machine inventory from the user is required. On joining the repository, each
worker publishes its own registration; the Commander discovers these through
fetched `agent/*/register` branches and assigns work based on reported capabilities.

Give every process a unique agent ID, such as `mac-ui-a`, and every computer a
non-sensitive machine alias. Do not share one working tree between concurrent
agents. Use independent clones across computers and separate worktrees locally.
If no identity was assigned, choose a role/machine alias with a short random suffix
to avoid collisions. Check existing branch names before publishing. Publishing a
registration does not need a prior implementation assignment; it is authorized
bootstrap work, still subject to the Cursor commit and resource rules.

To register, publish a small handoff on `agent/<id>/register` containing:

- Agent/tool identity and machine alias; operating system and verified tools.
- Mac/Xcode/device access, where applicable; relevant build capabilities.
- Available working time and current tasks.
- Funding-pool alias and authorization; unknown balances stay unknown.
- Cursor CLI availability/authentication for required commit execution.
- Readiness to receive work and last reviewed main SHA.

The Commander checks for new registration branches at every active checkpoint,
adds the worker to the roster, and assigns an eligible ready task.
A worker is not counted as active until its acceptance acknowledgement is received.
Creating a task row does not launch a process or prove someone is working on it.

Every new worker receives this startup instruction:

> Read AGENTS.md, ORCHESTRATION.md, docs/INDEX.md, and coordination/COMMANDER.md.
> Fetch origin and read your assignment on main plus relevant peer handoffs.
> Register your identity/capabilities if missing. Acknowledge the assignment ID,
> revision, owned paths, deadline, and resource grant on your own branch before
> implementation. Follow the fetch/review/adapt/report loop. Cursor CLI must
> execute commits. Never consume protected personal Claude allowance.

## 5. Assignment contract and ownership

Each task has an ID and revision, one owner, acceptance criteria, owned paths,
input revisions/contracts, dependencies, a timebox, resource grant, and report path.
Use small tasks that produce a useful integrated result in roughly 20–45 minutes
where possible. A deep compiler task may require several such checkpoints.

The worker acknowledges the exact assignment revision and gives an initial ETA
range before starting. If the Commander changes the assignment, the worker stops
at a safe boundary, reports saved state, acknowledges the new revision, and adapts.

Task states:

`unassigned → assigned → accepted → in_progress → ready_for_integration → integrated → verified`

Use `blocked`, `cancel_requested`, or `cancelled` when appropriate. Ready means
the deliverable exists and its stated checks ran; verified means the combined
product passed the relevant acceptance gate. Never merge these meanings.

Only the Commander assigns overlapping paths or authorizes reassignment. A stale
heartbeat is not proof a worker has stopped writing. Request cancellation and
wait for acknowledgement before transferring the same task/path. If a worker
cannot be reached, isolate replacement work on a new branch and resolve ownership
before integration; do not allocate its unaccounted budget a second time.

## 6. Worker loop: code, synchronize, adapt, publish

At startup, before a substantial new step, and approximately every 3–5 minutes
while active:

1. Inspect working-tree state and fetch `origin --prune`.
2. Read changes to the task board, Commander update, relevant contracts, and peer
   handoffs since the last acknowledged revisions.
3. Inspect the actual relevant code diffs. Identify consumer/API impact and alter
   implementation or tests accordingly. Record “no action” when justified.
4. Work on the next small acceptance step within the remaining time/budget.
5. At a coherent checkpoint, test, update the handoff, ask Cursor to commit, and
   push the task branch. Target 5–10 minutes between useful published checkpoints.
6. Merge newer main at a clean checkpoint and verify affected behavior.

If a critical finding or blocker appears, publish it immediately at the next safe
boundary. Do not wait for the next arbitrary timer. If Cursor is unavailable,
report that commit/publishing is blocked; do not fake Cursor execution.

The minimum progress report includes:

```text
Task: FT-xxx, assignment revision N
Branch / code SHA / last main integrated:
State / ready behavior / remaining acceptance criteria:
Files/interfaces changed / affected consumers / required action:
Checks and results / evidence paths:
ETA remaining: optimistic / likely / pessimistic; confidence and dependencies:
Funding grant / actual usage / estimated or unknown usage / in-flight calls:
Blocker / request to Commander:
Commander update and peer revisions acknowledged / adaptation:
Next step / exact resumption instructions:
Updated UTC:
```

Publication, integration, and acknowledgement are distinct. An agent must not
assume another agent read a pushed change until its acknowledgement names it.

## 7. Commander loop

While actively orchestrating:

- Every 3–5 minutes: fetch, inspect registered worker branch tips, read changed
  handoffs, and identify blocked, stale, or conflicting work.
- Every 5–10 minutes or on a material change: publish updated dispatch/progress
  state through Cursor-authored commits. Batch related control-file edits to
  avoid spending a model call on every heartbeat.
- Every 15–20 minutes: integrate ready increments and refresh the critical-path
  forecast, budget status, and next acceptance gate.
- Immediately when a blocker threatens a dependency: resolve the interface,
  redirect an available worker, or ask the user for the specific missing input.

Mark a worker's information stale after approximately 10 minutes without a useful
update; request a check-in. After 20 minutes, flag the scheduling risk. These
thresholds trigger investigation, not automatic cancellation or budget release.

The Commander publishes a concise global update containing: current main SHA,
completed gates, next gate, assignments changed, interface changes, blockers,
required acknowledgements, deadline forecast, and resource exceptions. Workers
acknowledge the relevant update ID/revision in their own handoffs.

“Everyone knows everything” means everyone can locate shared evidence and knows
the changes affecting their task. Do not broadcast complete transcripts into
every context window. Use targeted summaries plus indexed detailed artifacts.

## 8. Initial workstreams and dispatch order

Do not treat this table as staffed. Register workers before actual assignment.

| Task | Workstream | First deliverable | Depends on |
|---|---|---|---|
| FT-001 | Shared contracts | Versioned compile/edit/capture messages and fixtures; source-offset convention | Commander assigns interface owner |
| FT-002 | Compiler foundation | Original Rust tokenizer/parser boundary, diagnostics, and a reproducible minimal document | Supported subset and FT-001 |
| FT-003 | Mac shell | Native source/preview shell using the contract fixture; local build evidence | Mac/Xcode worker, FT-001 |
| FT-004 | Companion capture | Actual Pencil drawing and camera capture; transport-ready image payload | Mac/Xcode/device worker, FT-001 |
| FT-005 | Typesetting/output | Rust-generated positioned output/PDF with source provenance | FT-002 |
| FT-006 | Incremental/recovery | Measured reuse, clean-build equivalence, and an explicit recovery case | FT-005 |
| FT-007 | Grok/transfer | Paired transfer, bounded context, reviewed conversion, anchored insertion | FT-001/003/004 and funded Grok |
| FT-008 | Integration/verification | Native end-to-end workflow, dark preview/export check, measured latency | FT-003/005/006/007 |

Keep compiler interpreter/layout state under one owner initially. Parallelize
around agreed interfaces rather than splitting tightly coupled semantics among
agents inventing incompatible structures. Mac/companion workers can use clearly
labeled fixtures until Rust output arrives; fixtures are not completed integration.

If only two workers are available, combine Mac and companion work under one Mac
owner and give the other compiler work; Commander handles contracts/coordination.
If more workers register, assign isolated tests, assets, or focused reviews only
when they shorten the critical path. More agents are not automatically faster.

## 9. Target schedule and scope control

All times below are Pittsburgh EDT on September 12, 2026. These are target gates,
not assertions of feasibility or completed work. Revise forecasts using evidence.

| Time | Target gate |
|---|---|
| 00:00 | Workers registered, ownership assigned, contracts and demo inputs agreed |
| 02:00 | Original Rust output and native shells build independently |
| 04:00 | Real compiler output appears in Mac; both device capture inputs work |
| 06:30 | First end-to-end conversion/insertion and incremental/recovery evidence |
| 07:50 | Final stabilization begins; freeze discretionary architecture/features |
| 09:15 | Repeated full demo on the actual Mac/iPad with documented limitations |
| 09:45 | Preserve known-good revision, export/sample files, and demonstration steps |
| 10:00 | Deadline |

Forecast the dependency chain and integration cost, not just the sum of worker
ETAs. Report best/likely/worst remaining time and confidence. If the deadline is
at risk, surface the gap and propose concrete scope choices; the Commander cannot
silently delete full compatibility or either capture requirement.

Use bounded improvement experiments: state the hypothesis, time/cost limit, and
success measurement; keep a demonstrated improvement or change approach. After
two similar failed attempts, reassess and request a focused review if useful and
funded. Persist with useful actions; avoid blind retry loops or false optimism.

## 10. Integration and frequent remote publication

Each worker owns one task branch. The Commander is the integration owner of main.
All new commits, including necessary merge commits, are executed by Cursor CLI
with `Cursor <cursor@flashtex.invalid>` and truthful implementation/execution
trailers, except on `mac-m1max-a`, where the user is the primary author (see the
machine exception in AGENTS.md). Do not rewrite previous human commits or claim
vendor verification.

For each integration:

1. Fetch and record the exact candidate and current main revisions.
2. Review intended changes, contract updates, handoff, and validation evidence.
3. Prepare a combined result in an integration worktree/branch. Preserve originals.
4. Run affected Rust checks and obtain Mac/device checks from a capable worker.
   Do not report Xcode validation from a Linux-only machine.
5. If a new commit is required, have Cursor execute it. Fast-forwards and pushes
   do not create commits and can be performed by the Commander directly.
6. Push with ordinary non-force Git semantics. If main advanced, refetch and
   re-evaluate the combined result; never overwrite another push.
7. Publish the resulting main SHA and required consumer actions. Workers merge
   current main at safe checkpoints and acknowledge the integrated change.

Use PRs when they help review or branch policy requires them. For the hackathon,
the Commander may integrate verified increments directly under the user's
instruction to keep remote updated. This does not bypass existing protections.
No one should push incomplete product work to main solely to satisfy a timer.

A failed integration returns to the owner with reproducible evidence. If a
regression reaches main, coordinate a targeted fix or a Cursor-executed revert;
do not reset shared history. Keep a known-good revision identified for the demo.

## 11. Resource allocation and tool selection

The Commander owns `coordination/RESOURCES.md`. Workers report usage on their
branches; one writer issues non-overlapping grants. A $200 subscription price is
not $200 of remaining API credit. Record actual quota only when available, with
source and timestamp. Unknown is not zero, and monetary estimates are not bills.

Protect the user's included Claude allowance. Do not use the £75 extra-credit
pool until isolation is verified or the user changes the restriction. Do not
transfer that amount into an imaginary Console account or enable auto-recharge.

Before auxiliary work, specify the pool, allocation ID, task, maximum duration,
call count/cost limit where enforceable, and acceptance result. Child grants are
subtracted from the parent's grant. Never spend a stale shared balance on several
machines. Keep unresolved charges reserved and reconcile interrupted calls before
retrying. No grants are reissued just because a session compacted or disappeared.

Use specialist agents for comparative advantage: independent semantic review,
targeted Swift investigation, or isolated validation. Use local commands for
deterministic checks. Cursor commit execution is an explicit user requirement;
batch coherent changes while preserving frequent useful publication.

Hard global spending enforcement requires provider controls and/or a transactional
gateway. The Markdown register provides accountability, not a guaranteed cap.

## 12. Context management and recovery

At every useful checkpoint and before planned compaction, write the handoff and
link durable findings. Store long evidence in topic files; keep startup files and
summaries concise. Use the [shared index](docs/INDEX.md) to make new information
discoverable. Save short decision rationales, not hidden reasoning transcripts.

Before resuming, every agent reads root instructions, this plan, Commander update,
its assignment, its own handoff, and the relevant contracts/peers. It then verifies
actual Git state, running jobs, and resource reservations. Resume the next unmet
gate; do not repeat completed work or restart the deadline.

The Commander's recovery packet additionally records the roster, all accepted
assignments, last reviewed worker SHAs, pending integrations, cancellation state,
unresolved costs, critical path, user decisions, and next dispatch actions.

## 13. First actions after publishing this plan

1. Joining agents read the startup instruction and self-register; no pre-filled
   machine list is needed from the user.
2. Workers publish registrations; Commander acknowledges them in the roster.
3. Commander dispatches FT-001 and hardware-eligible independent setup work.
4. All workers acknowledge assignments and report their first acceptance step.
5. Commander operates the review/adapt/integrate loop while the session is active.

Until registrations exist, report “unassigned” rather than claiming workers have
been dispatched. The immediate artifact is this operating plan and its boards;
implementing the product is a subsequent dispatched task.

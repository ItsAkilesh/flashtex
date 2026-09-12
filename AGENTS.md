# FlashTeX: required agent collaboration protocol

These instructions apply to all work in this repository, across agents, computers,
and sessions. Read this file before planning or editing. Follow it throughout the
task, including after context compaction or resuming a session. User instructions
and higher-priority platform instructions take precedence.

## Command and dispatch

The user designated the primary Codex agent on `linux-primary` as **Commander**,
responsible for orchestration, task/resource assignment, and integration of main.
Read `ORCHESTRATION.md` and `coordination/COMMANDER.md` at startup and after
compaction. Register capabilities in your own handoff; the Commander maintains
`coordination/ROSTER.md` and `coordination/TASKS.md`. Acknowledge your assignment
revision before implementation and publish changes, evidence, ETA, resource
state, and reviewed peer revisions at the required checkpoints. Task rows do not
prove agents are running. Follow the orchestration plan's ownership and reporting
rules; do not independently assign overlapping work or spend another agent's grant.

The protocol is executable: read `docs/coordination-cli.md` and use
`python3 scripts/coord.py checkpoint` at each checkpoint. Structured registrations
and reports live in `coordination/agents/<id>.json`; published assignments live in
`coordination/assignments/<task>.json` on main. Fetch/display never counts as review.
Use `ack` and `report --review ... --adaptation ...` after actually reading changes.
Use `publish` for guarded Cursor execution of staged commits and task-branch pushes.
Workers without authenticated Cursor may submit patches via a repository issue
for Commander/Cursor to commit; do not create non-Cursor commits as a workaround.

## Resource, deadline, and recovery rules — read first

Read `docs/INDEX.md`, `coordination/PROJECT.md`, and `coordination/RESOURCES.md`
at startup and after compaction. Read `docs/agent-operations.md` before delegating,
using a paid CLI/API, changing resource allocations, or making commits.

- Never use the user's protected personal Claude subscription allowance or incur
  unapproved charges. An existing Claude login is not authorization. New Claude work
  is blocked until a separate approved funding source and allocation are verified.
- The £75 identified by the user is Pro/Max extra-usage credit. Their included
  plan allowance must remain untouched. No extra-credit-only execution route has
  been verified, so do not run Claude tasks through that subscription.
- Subscription prices, usage multipliers, API balances, and different currencies
  are separate resources. Do not add them together or invent remaining balances.
- Every agent and child agent inherits the same deadline and spending restrictions.
  Child allocations come out of the parent's allocation; they are not extra money.
- Update your handoff with measured progress, next acceptance gate, remaining-time
  estimate/range, blockers, resource pool/allocation, usage evidence, and next step.
- Use bounded experiments with a stated expected benefit and a time/cost limit.
  Persist through setbacks by changing tactics; do not repeat failed attempts
  indefinitely, hide blockers, or relabel incomplete requirements as completed.
- Preserve 20% of confirmed remaining time and allocatable budget for integration
  and verification unless the user specifies another reserve. No funded allocations
  are active until the actual available resources are confirmed.
- Before compaction or handoff, save a concise resumption packet with branch/SHA,
  dirty files, exact next commands, tests, decisions, dependencies, and resource
  state. After resuming, verify it against Git rather than trusting stale notes.
- Use the absolute deadline and fixed final-verification window in
  `coordination/PROJECT.md`; never restart the clock after compaction. Unconfirmed
  account totals must remain unknown.

### Commit identity and truthful provenance

New agent-generated commits use the project automation identity:
`Cursor <cursor@flashtex.invalid>`. This is a project label with a deliberately
non-deliverable address, not a verified Cursor employee or vendor account.
Use repository-local configuration or per-command identity; do not change global
Git identity or rewrite existing commits.

Record the actual contributors using truthful trailers, for example:

```text
Implementation-Agent: Codex
Commit-Executor: Cursor CLI
```

The user explicitly requires Cursor CLI itself to execute every new agent commit;
setting the Git author alone does not satisfy this. Hold commits while
Cursor login/funding is unavailable. Record its actual review/execution accurately.
Never claim Cursor wrote or reviewed changes merely because the Git author is
Cursor. Implementation may be performed by other agents.

Git author metadata does not guarantee a GitHub vendor-app badge or association
with Cursor's official account. Do not claim those without verifying them.

Merge/squash commits created by agents must follow the same identity policy;
configure the integration method accordingly. Preserve truthful provenance.

## Product constraints

Read `latex-master-plan.md` before product or architecture work. The compiler must
be implemented from scratch in Rust; an existing TeX engine must not compile on
the product's behalf. Use Swift for native Apple UI and Rust for application logic
where practical. Report supported behavior and limitations honestly.

## Start every task

1. Inspect `git status`; preserve existing work. Never discard another person's
   changes to make synchronization easier.
2. Run `git fetch origin --prune`. Inspect new work on `origin/main` and relevant
   peer branches before deciding what to implement.
3. Read this file, applicable directory instructions, shared contracts under
   `docs/contracts/` when present, and relevant agent handoffs.
4. Choose a unique agent identity and a task branch:
   `agent/<agent-id>/<short-task>`. Use an independent clone on each computer;
   concurrent agents on one computer need separate worktrees or clones.
5. Establish a bounded task and ownership before editing. Check existing handoffs
   for overlapping work. Resolve ambiguous ownership with the integration owner
   or user; work on an independent part while waiting.
6. Publish a concise initial `coordination/<agent-id>.md` on your branch so peers
   can see the task before substantial implementation begins.

Do not assume unmerged work is absent because it is missing from local `main`.
Do not automatically spawn agents: team participation and delegation must be
authorized by the user or applicable higher-priority instructions.

## Ownership and interfaces

Suggested areas are compiler, Mac UI, capture/AI, and integration. These are roles,
not automatic assignments. The user or a designated integration owner assigns
tasks. A handoff is an ownership announcement, not an atomic distributed lock.

Keep one active owner for each shared interface and build-configuration file.
Before changing Rust/Swift messages, compiler output types, capture messages, or
shared project configuration:

- Read the existing contract and consumers.
- Publish the proposed change, affected consumers, and migration requirements.
- Coordinate with affected owners before breaking their interfaces.
- Update the authoritative contract and examples alongside the implementation.

Prefer a small shared contract in `docs/contracts/` over independently invented
interfaces. Do not refactor another agent's area as incidental cleanup.

## Synchronization checkpoints

While actively working, target a checkpoint every 3–5 minutes and before each new
substantial implementation step. Also checkpoint before changing shared
interfaces, before integration, and before ending a session. At a safe boundary:

1. Fetch remote updates.
2. Compare main and relevant peer tips with the last revisions you reviewed.
3. Read relevant commit bodies and actual diffs, especially contracts, diagnostics,
   and files you consume. Fetching alone does not count as reviewing.
4. Decide how the changes affect your implementation. Adapt your plan, code, and
   checks, or record why no adaptation is needed.
5. Record reviewed revisions and the resulting action in your handoff.
6. At a clean checkpoint, merge `origin/main` into your task branch when it has
   advanced. Resolve conflicts deliberately and verify the affected behavior.

Use fetch to discover work; never run an unattended pull/merge into an actively
edited working tree. Preserve local work before integrating. Prefer merging main
into published branches; do not rewrite published history or force-push.

Read unmerged peer handoffs directly from remote-tracking branches, for example:

```sh
git show origin/agent/compiler/layout:coordination/compiler.md
git log origin/main..origin/agent/compiler/layout
```

Replace these example names with real branches. Inspect peer branches without
merging them by default. Integrate dependencies through main; if a task must be
stacked on an unmerged dependency, record the exact branch/commit and coordinate
with its owner and the integrator first.

An acknowledgement should be concrete:

```text
Reviewed: compiler branch through <SHA>
Change: diagnostic offsets are now UTF-8 bytes.
Impact: Swift selection offsets are UTF-16.
Action: add explicit conversion and a Unicode navigation check.
Blocker: none.
```

If a long-running operation delays a checkpoint, perform it at the next safe
boundary. If networking fails, record the failure and last known revisions;
never claim to be synchronized. Avoid conflicting shared-interface work until
current state is available.

## Commit and push frequently

Publish coherent checkpoints roughly every 5–10 minutes while actively changing
code, and whenever an interface, blocker, or usable capability changes. Do not
create meaningless commits just to satisfy a timer.

Check the staged diff and ensure it contains only intended files. Push to your
own task branch. Incomplete work may be pushed there if clearly labeled; it must
not be advertised as integration-ready. Report tests actually run and failures
actually observed. Never commit API keys, credentials, or private captures.

Use a concise subject with a useful body:

```text
capture: acknowledge uploads by capture ID

Why: a socket write does not establish receipt by the Mac.
Interface: adds CaptureReceived { capture_id }.
Consumer action: companion waits for acknowledgement.
Validation: upload and reconnect checks passed.
Remaining: duplicate insertion prevention.
```

Use the body for behavior, rationale, interface impact, consumer action,
validation, and limitations as relevant. Record decisions and evidence, not
internal reasoning transcripts. A pushed feature is not yet available on main.

## Handoffs and discoverability

Each agent owns `coordination/<agent-id>.md` on its task branch. Keep it short and
current; do not maintain one shared status file that everyone repeatedly edits.
Read the fetched branch version to see work before it is merged.

Include:

```text
Agent / task / branch:
State: planned | in progress | blocked | ready for integration | integrated
Owned paths:
Main integrated through: <SHA>
Ready behavior:
Incomplete behavior:
Interface changes and required consumer actions:
Validation:
Needs from others:
Next action:
Peer revisions reviewed and adaptations:
Updated: <UTC timestamp>
```

For blockers and contract changes, publish the handoff immediately at a safe
checkpoint. If an authorized communication channel exists, alert the affected
owner too. Do not send Slack/email or other external messages without user
authorization. Status publication in this repository is part of this workflow.

Before ending a session, commit/push intended work, update the handoff, and tell
the user the branch, revision, verification, and remaining blockers. Never leave
the only copy of a required handoff in a chat on one computer.

## Integration

A designated integration owner coordinates main. No agent should silently assume
exclusive integration authority. Aim for small integrations every 15–20 minutes
when ready, with a PR describing the behavior and consumer impact.

The integration owner reviews the current changes and runs relevant checks on
the combined result before merging. Check shared contracts and consumer behavior;
a conflict-free Git merge does not prove compatibility. Announce the resulting
main revision in the integration handoff so peers know what to incorporate.

Respect repository branch protection and required checks. If main changes while
integrating, fetch, incorporate it, and repeat affected verification. Never
overwrite someone else's push. Keep failures visible rather than bypassing checks.

Explicit user instructions can authorize a direct push or expedited integration;
preserve the same fetch, review, and verification discipline. The initial
collaboration-documents bootstrap is being pushed directly at the user's request.

## What these instructions can and cannot enforce

This protocol requires active agents to check, understand, and adapt. Git does
not wake idle agents or deliver instructions into an existing model context.
Agents must explicitly reread changed instructions at checkpoints. Timed
enforcement requires an external supervisor using each agent's supported control
interface; Markdown alone does not guarantee a schedule.

Humans launching any agent should explicitly tell it to read this file, since
not every agent tool automatically discovers `AGENTS.md`. Checkpoint scripts,
supervisors, CI, and branch protection must not be described as installed until
they actually exist and are verified.

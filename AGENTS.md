> LATEST STAFFING OVERRIDE: this computer has FOUR active agents total: sole
> commander orchestrator-astra, root preview-controller, compiler_corpus fonts,
> supervisor_api_review rendering. Bridge-context, project-index and edit-ledger
> agents are paused; preserve all dirty/published work. Never revive or replace
> them automatically and do not compensate by increasing staffing elsewhere.
> This overrides older six-engineer staffing text below. Other authorization
> and explicit-user-stop-only project continuity remain unchanged.

# FlashTeX: required agent collaboration protocol

These instructions apply to all work in this repository, across agents, computers,
and sessions. Read this file before planning or editing. Follow it throughout the
task, including after context compaction or resuming a session. User instructions
and higher-priority platform instructions take precedence.

## Current user authorization — September 12

The user explicitly authorizes continuous autonomous project improvement until
they explicitly stop it. Verified completion starts another improvement cycle;
it is never an automatic stop condition. The
former 10am deadline and stabilization window are no longer stop conditions.
Do not stop solely because a task time estimate elapsed. Keep individual model
calls bounded, publish recovery evidence on failures, and continue other eligible
work. Do not wait indefinitely for a sleeping user's permission on actions already
authorized. Actual platform denials, missing authentication, and unapproved billing
remain blockers to report; never claim they have been bypassed or granted remotely.
Read `docs/autonomous-workers.md` for the executable startup and task loop.

## Command and dispatch

Current sole orchestrator: **orchestrator-astra**, hosted agent handle
`/root/runtime_validator`, on linux-primary, explicitly selected by the user.
All organizational work, resources, queues and global integration belong to this
role. The three active hosted product engineers perform product work only. The root agent
continues product engineering and does not concurrently write main/control files.
Read `coordination/authority.json` before every global mutation; obsolete role names
below are historical. Sol explicitly handed over after stopping publication jobs.


The user designated the primary Codex agent on `linux-primary` as **Commander**,
responsible for orchestration, task/resource assignment, and integration of main.
Read `ORCHESTRATION.md`, `coordination/COMMANDER.md`, and
`coordination/COMMANDER-RESUME.md` at startup and after
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
Use `publish` for guarded staged commits and task-branch pushes. After an observed
Cursor usage limit, the explicit user override below permits direct current-agent
Git commits with truthful provenance and the authenticated local user as coauthor.
Do not wait for exhausted Cursor quota. Other missing-authentication cases must
use an already-authorized execution route or publish a concrete recovery blocker.

There is exactly one active Commander. A successor may claim command only after
either (a) the current Commander publishes an explicit quiesced handoff naming that
successor and confirms its publication/integration jobs are stopped, or (b) the
successor independently verifies the exact Commander process/session terminated
and every Commander publication/dispatch/integration job is stopped. Silence, a
missed heartbeat, stale Git state, timeout, quota suspicion, or network failure is
never proof the Commander is offline. The successor fetches and pins current main,
selects one leader identity, publishes an atomic non-force authority claim, and
rereads that claim immediately before every main/control write. An old Commander
that resumes must reread authority and remain quiesced unless explicitly handed
command again. See the copyable revival prompt in `docs/autonomous-workers.md`.

Every blocked worker opens a GitHub recovery issue with task/revision, exact branch
and SHA, failing command, non-secret error, process state, resource state, and any
possibly in-flight call. The Commander triages each open recovery issue, assigns
one or more eligible non-overlapping resolvers after rereading every machine's
latest resource report, verifies the fix, and only then closes the issue. A comment
or task row is not proof that a local worker started; require an ACK plus actual
PID/session or equivalent live-process evidence.

## Current Claude Max authorization

The user explicitly requested more tasks and subagents on the 20x Claude Max plan
on mac-m1max-a. That machine's parent `mac-claude-a` may use its available plan
allowance for assigned project work and supervise `mac-pdf` and `mac-validation`
in separate worktrees. These share one account quota; they are not independent
balances. This scoped authorization supersedes older blanket Claude prohibitions
for that plan only. No overages/purchases or unrelated protected account use.

On `linux-primary`, Claude is API-only under the latest user instruction.
Never launch the local Max subscription, OAuth/keychain, or extra-usage route.
Opus work may start only after an existing funded API credential, actual available
credits, a provider-side cap, and a bounded project grant are verified. This grants
no purchase, new charge, auto-recharge, or overage. The Max 20x authorization above
remains confined to `mac-m1max-a`.

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

**Latest explicit user override (all computers):** when Cursor usage limits are
hit, the current implementing agent may execute Git commits directly. Do not wait
for Cursor quota, purchase more usage, or mislabel execution. Use the actual agent
identity and truthful `Implementation-Agent` / `Commit-Executor` trailers, plus
`Co-authored-by` for the GitHub user authenticated on that computer. Preserve the
mac-m1max-a primary-author exception. This supersedes older mandatory-Cursor wording
only for the observed Cursor-limit fallback; actual Cursor commits remain truthful.
The local Cursor limit was confirmed by terminal ActionRequiredError on the Astra
authority and bridge recovery publication attempts. Product work and publication
continue through direct-agent Git execution under this user authorization.


Commits actually executed by Cursor use the project automation identity
`Cursor <cursor@flashtex.invalid>` (except the Mac primary-author rule below).
Direct fallback commits use the actual implementing agent identity. This is a project label with a deliberately
non-deliverable address, not a verified Cursor employee or vendor account.
Use repository-local configuration or per-command identity; do not change global
Git identity or rewrite existing commits.


**Machine exception — `mac-m1max-a` (the user's own Mac):** the user directed
that everything committed and pushed from this computer carries the user as the
primary Git author, using the repository's configured `user.name`/`user.email`
(`jay3332`). Do not set the Cursor author on commits from this machine. The
The all-computer direct fallback remains available after an observed Cursor limit;
truthful trailers are still required here: name the implementing agent,
name the actual commit executor, and add `Co-authored-by: Cursor
<cursoragent@cursor.com>` only when Cursor CLI actually executed the commit.

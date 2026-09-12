# Resource authority and allocation register

Status: user confirmed £75 is Pro/Max extra usage and included allowance must not
be consumed. Balance/isolation remain UNVERIFIED. Claude task execution blocked.
Updated: 2026-09-12T03:22:36Z.
Resource owner: Commander, the primary Codex agent on linux-primary, appointed by user.

## Non-negotiable restriction

Do not use protected personal Claude subscription allowance or incur unapproved
charges. No automatic fallback to a logged-in Claude account, another provider,
or paid overage. Installing software or checking its version is not permission
to run inference. Existing interactive Codex usage is not measured by this file;
agents cannot control or infer the hosting application's billing from Git.

## User-reported resources awaiting reconciliation

Confirmed clarification: the £75 belongs to the user's Claude Code account, not
the combined multi-provider resource pool. Treat £75 as its stated project budget
ceiling pending confirmation of actual available funds. The user explicitly
confirmed the funds are Pro/Max extra usage. Do not treat them as API credits.
Included personal allowance remains protected, and the existing Max
login must not be used until an eligible billing route is verified.

Official documentation checked September 12 says extra credits apply after
included limits are reached. No credit-only launch mode preserving those limits
was verified. The help center also says the announced separate Agent SDK credit
was paused: `claude -p` is not a workaround for subscription usage. Keep this pool
blocked; separate approved API funding is the currently documented isolation route.

Sources: [usage credits](https://support.claude.com/en/articles/12429409-manage-usage-credits-for-paid-claude-plans),
[Agent SDK update](https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan).

The user mentioned $20 Claude and OpenAI plans, $100 credit amounts, a 20x Claude
plan, a 20x Codex plan, and a further $100. Their count, ownership, provider
assignment, expiry, remaining amount, and relation are not established. Do not
treat any mention as an independently additive pool.

Keep USD and GBP separate until an exchange-rate policy is explicitly approved.
Subscription prices are not spendable API balances. Usage multipliers are not
money, and subscription/API credits are not interchangeable by assumption.

The user identified a $200 OpenAI account on this Commander machine and authorized
it for orchestration. Record it as user-reported account/subscription access;
the exact plan, current quota, reset, and API balance have not been verified. Do
not subtract usage estimates from $200 as though it were an API-credit deposit.

## Pool register

| Pool alias | Funding type | Currency/unit | Confirmed remaining | Expiry/reset | Permission |
|---|---|---|---|---|---|
| claude-user-75gbp | Pro/Max extra usage, confirmed by user | GBP; stated budget £75 | Unknown, not assumed £75 | Unknown | BLOCKED: cannot verify included-allowance isolation |
| claude-project-api | Separate project API credits, proposed | Unknown | Unknown | Unknown | BLOCKED pending verification |
| claude-personal | Personal subscription/billing | Subscription usage | Not queried | Unknown | PROHIBITED |
| cursor-project | Authenticated Cursor account usage | Unknown | Unknown | Unknown | User explicitly authorized Cursor commit execution; no general development allocation |
| openai-commander | User-reported $200 OpenAI account on linux-primary | Subscription/account usage; exact plan unverified | Unknown | Unknown | Commander orchestration authorized; no new API charges inferred |
| openai-mac-plus | Worker-reported ChatGPT Plus on mac-m1max-a | Included subscription usage; no API credit reported | Unknown; prior quota snapshot stale | Reported weekly reset Sept 15, not independently verified | One bounded FT-003 task using user-provided OpenAI access; no API/overage purchases |
| openai-aarush-plus | Worker-reported ChatGPT Plus on aarush-macbook | Included subscription usage; no API credit reported | Unknown | Unknown | FT-004 conditional on available authenticated OpenAI tool; no Claude fallback |
| openai-project-api | Project API credits, proposed | Unknown | Unknown | Unknown | BLOCKED pending verification |
| grok-product | Product conversion API | Unknown | Unknown | Unknown | Credentials/budget not audited here |

`claude-user-75gbp` and the proposed `claude-project-api` may refer to the same
funds. Do not count them as two balances; reconcile aliases before allocation.

## Fixed allocations across computers

Use non-overlapping allocations issued by one resource owner. Before activating
an allocation, verify its funding pool in the provider's account/billing tools,
record evidence without secrets, and establish applicable provider-side limits.
Reserve 20% of confirmed allocatable funds for integration unless directed otherwise.

| Allocation ID | Pool | Agent/machine alias | Maximum | Spent | Reserved in-flight | State |
|---|---|---|---|---|---|---|
| cursor-docs-commit-001 | cursor-project | Cursor CLI / linux-primary | One bounded documentation commit session; monetary cost unknown | One completed session; monetary cost unknown | Charge unresolved; no inference in flight | Completed: b37237b committed by Cursor and pushed by Commander; zero Claude calls |
| cursor-orchestration-commit-002 | cursor-project | Cursor CLI / linux-primary | One bounded orchestration-docs commit session; monetary cost unknown | One completed session; monetary cost unknown | Charge unresolved; no inference in flight | Completed: 567d84b committed by Cursor and pushed to main; zero Claude calls |
| cursor-registration-commit-003 | cursor-project | Cursor CLI / linux-primary | One bounded self-registration clarification commit session | Completed 276bb1c; monetary cost unknown | No call in flight | Actual Cursor commit verified |
| commander-orch-001 | openai-commander | commander / linux-primary | Current orchestration task under user's account authorization | Usage unknown to repository | Unknown | Active; no delegated development agents started |
| commander-orch-002 | openai-commander | commander / linux-primary | Coordination CLI, shared contracts, and dispatch implementation | Usage unknown | Unknown | Active; no new API funding inferred |
| ORCH-002-R1 | openai-commander | coordination_review / linux-primary | One hosted read-only review, 5-minute timebox; no descendants | One completed review; quota/cost unknown | No model call in flight | Completed; findings incorporated; no external model/Claude/Cursor calls |
| cursor-protocol-commit-004 | cursor-project | Cursor CLI / linux-primary | One bounded tooling/dispatch commit session | Completed 6d096a3; monetary cost unknown | No call in flight | Actual Cursor commit verified and integrated |
| openai-mac-plus-ft003 | openai-mac-plus | mac-claude-a / mac-m1max-a | One 45-minute native-shell task; no descendants or paid API calls | Unknown | Awaiting acceptance | Use available user-provided OpenAI subscription access; do not consume protected Claude allowance |
| openai-aarush-plus-ft004 | openai-aarush-plus | aarush-macbook | One 45-minute capture task; no descendants or paid API calls | Unknown | Awaiting tool readiness/acceptance | No Claude fallback; report unavailable OpenAI tooling instead of purchasing |

The Cursor grant records the user's explicit tool-specific commit authorization,
not an invented dollar balance. Do not enable overages or change billing settings.
No Claude grant exists. Zero Claude calls in this Cursor commit session.

Agents report on their own branches. Only the designated resource owner changes
pool allocations. Git is an audit trail, not a transaction service: two machines
must never each spend the same apparent remaining global balance.

Each grant needs a unique ID, owner, pool, unit, maximum, and authorization record.
Descendant grants are deducted from the parent. Never reissue an allocation until
the old owner acknowledges cancellation and its in-flight calls are reconciled.
No silent top-ups, duplicated grants after restart, or resets after compaction.

Before each call, reserve its conservative maximum against the local grant. After
completion, reconcile provider-reported cost and leave unknown usage reserved.
Ledger availability = grant - confirmed spend - outstanding reservations. Do not
mix model-estimated dollar equivalents with actual subscription/API charges.

Local tracking and CLI budgets alone do not establish a strict cross-machine
billing cap. A shared gateway/transactional reservation service plus provider-side
controls is needed for stronger enforcement; neither is installed by these docs.

## Evidence and reporting

Each handoff reports allocation ID, actual/estimated/unknown usage, evidence source,
in-flight calls, remaining grant, quota reset if known, and next-call estimate.
Provider dashboards may lag. Unknown is not zero. Never commit keys or raw billing
credentials; use non-sensitive pool and machine aliases.

Do not purchase credits, enable auto-recharge, enable overages, or change account
plans without explicit user authorization.

## Kabir registration reconciliation — September 12, 03:48 UTC

Source: origin/agent/claude/machine-resource-inventory at ec0dac7c535d3de55dbc0b51694f30608717f5f0.
Worker reports ChatGPT **Plus**, not a 20x Codex plan, on mac-m5pro-kabir.
Pool `openai-kabir-plus` is subscription access, with no API balance. The worker
measured 4% weekly usage at 03:18Z; this is a historical snapshot, not live quota.
Three weekly resets are worker-reported user authorization, unverified; none
allocated or consumed by Commander. Do not assume resets are available or trigger
one automatically. Preserve the protected Claude restriction.

Grant `openai-kabir-plus-ft002`: one 45-minute original Rust compiler foundation
task for agent `claude` using **Codex**, no descendants, API purchases, or overages.
Acknowledgement pending; usage unknown. Authenticated Cursor may execute coherent
checkpoint commits under the user's existing commit authorization; report usage.

Grant `cursor-dispatch-commit-005`: one bounded Cursor session to commit this
registration/dispatch follow-up. Monetary cost unknown; no nested agents.

## Integration loop allocations — September 12, 03:56 UTC

`cursor-dispatch-commit-005` completed as b932149; monetary usage unknown, no call
in flight. `ORCH-003-R1`: one hosted Codex read-only integration-design review,
5-minute limit, no descendants or external calls; completed, findings incorporated.
`cursor-integration-tools-006`: one bounded Cursor commit session for integration
helper, tests, and instructions. `cursor-inventory-merge-007`: one bounded Cursor
merge session for reviewed Kabir capability inventory, after tooling validation.
Both Cursor sessions use the existing user-authorized commit route; no purchases,
overages, Claude calls, or invented monetary balances. Actual costs remain unknown.

## Continuing autonomous authorization — current user override

The user now explicitly authorizes ongoing work until they stop it or the entire
project including extras is implemented and fully tested. Former 45-minute task
limits are review/checkpoint intervals for the eligible OpenAI subscription routes,
not requirements to ask the sleeping user for permission again. Published queue
stage allocation IDs inherit their machine's existing authorized OpenAI pool.
No monetary balance, API access, reset availability, or protected Claude permission
is inferred. Provider quota remains authoritative; failures require recorded
recovery/reallocation, not unapproved fallback or an interactive permission wait.

Cursor commit and synchronization sessions remain authorized. All new commits
must additionally coauthor the authenticated GitHub user on the executing computer.
This Commander's identity resolved from `gh api user`: sixnat, GitHub user ID
266300832; use its GitHub noreply address, not a private email.

`cursor-integration-tools-006` completed 75648ea. `cursor-inventory-merge-007`
completed d432341 and passed 22 combined tests; both costs unknown, no calls in flight.
`worker_sync`, `worker_loop_tests`, and `dispatcher_loop` are bounded hosted Codex
implementation/test subtasks under current Commander authorization, no descendants
or external model calls. Their reports provide actual completion evidence.
`cursor-autonomous-loop-008`: one bounded Cursor commit session for coauthor policy,
continuous worker/dispatcher loop, and verification. Monetary cost unknown.

`openai-commander-bridge`: direct Commander Rust bridge implementation and validation
under the user's ongoing Commander authorization; no new API purchases. Grok calls
remain conditional on actual product credentials/funding. One local Codex
noninteractive preflight completed successfully in an isolated worktree, creating
and verifying one probe file without permission prompts; usage/cost unknown.

Machine-specific policy evidence: main d685879 and mac-claude-a's f13979c handoff
report direct local-user authorization for that Mac's Claude Max session and
jay3332 primary-author exception. Preserve that reported local instruction; it does
not authorize consuming this Commander's protected Claude allowance or generalize
to other accounts. No Claude inference was initiated by Commander.

`cursor-continuous-dispatch`: user-authorized Cursor publication of coherent queued
next-assignment batches, at most one call per prepared batch, no paid polling,
no automatic retry of an unresolved call. Uses the same existing Cursor account;
quota/cost unknown, no overages or purchases. Dispatcher advances explicit queues
only; Commander remains responsible for replenishment, review and integration.

## Explicit 20x Claude Max expansion — latest user instruction

The user requested more work and subagents on the 20x Claude Max plan, reporting
that it is barely using its allowance. This authorizes that plan's available
allowance on mac-m1max-a for ongoing project work. No new credit purchases, overages,
or use of other protected Claude accounts is authorized. Numeric quota is still
unknown until the Mac reports it. Parent and children share one pool.

Pool `claude-mac20x`: local Mac Max plan. Grants: parent mac-claude-a continues the
native editor and transport (`claude-mac20x-shell-stage2/3`); child mac-pdf owns
FT-009 (`claude-mac20x-pdf`); child mac-validation owns FT-010
(`claude-mac20x-validation`). Start parent plus these two children in independent
worktrees; each child has no descendants. Parent reports quota and child progress
at checkpoints and changes concurrency if actual resource evidence warrants it.

Hosted Commander grants `openai-commander-corpus` (FT-011) and
`openai-commander-protocol` (FT-012): two independent 30-minute product-validation
subtasks in isolated worktrees, no descendants or additional external model calls.
These share the already-authorized Commander account; usage/cost unknown.

## Current additional worker evidence — September 12, 04:42 UTC

`chatgpt-a` on `aarush-macbook-chatgpt` reports Codex CLI, Xcode 26.6, Swift,
Python, and Git push capability at registration `1d34323`; quota remains unknown.
Grant `openai-aarush-chatgpt-ft014` authorizes the independent FT-014 companion
validation harness under `tools/companion-validation`, using existing OpenAI
subscription access only. It does not authorize API purchases or overlapping
writes to FT-004's `apps/companion` ownership.

The Linux Claude credential metadata reports subscription type Max and tier
`default_claude_max_20x`, matching the user's authorization, but the CLI reports
logged out with no access or refresh token and no eligible API environment key.
Pool `claude-linux-max20x` is user-authorized for substantial project work after
login; usable quota is unknown. API-credit fallback is authorized only when an
actually funded existing API route is verified. New charges, purchases, and
overages remain prohibited. No inference has run and no grant is active while
authentication is absent.

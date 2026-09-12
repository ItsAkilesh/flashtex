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
| cursor-registration-commit-003 | cursor-project | Cursor CLI / linux-primary | One bounded self-registration clarification commit session | Pending; monetary cost unknown | One session | Authorized follow-up commit; no nested calls |
| commander-orch-001 | openai-commander | commander / linux-primary | Current orchestration task under user's account authorization | Usage unknown to repository | Unknown | Active; no delegated development agents started |

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

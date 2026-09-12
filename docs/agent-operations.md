# Agent operations: resources, persistence, and context

Owner: integration/resource coordinator when assigned.
Status: required operating procedures; enforcement limitations are explicit.

## Choose tools by task advantage

Use the current agent for ordinary work. An auxiliary agent should have a bounded
purpose that benefits from independent expertise or parallelism: a focused review,
a reproducible compiler bug investigation, or a self-contained Swift integration.
Do not spend a model call on work a deterministic command can perform unless the
user explicitly requires that tool, such as Cursor-mediated commits.

Before delegation, record objective, input revisions, owned paths, acceptance
criteria, deliverable location, timebox, funding allocation, and maximum cost.
Every child reads root instructions and the relevant context packet. The parent
tracks children and their resource reservations; nested layers cannot invent
budgets, extend deadlines, or delegate indefinitely. Unfunded auxiliary calls stop;
independent work through already-authorized means can continue.

## Claude: separate funding only

The installed CLI supports `--bare`, which excludes subscription OAuth and keychain
credentials. Use this mode for future project API tasks after funding is verified.
Do not use `claude`, `claude -p`, `claude setup-token`, or subscription login as an
unqualified project-task launcher. Version/help/auth-status checks do not invoke
inference and are allowed for setup.

Before running a funded call:

1. Verify a dedicated project API key belongs to the approved funding pool. Never
   paste it into chat, a commit, a prompt, or a command-line argument.
2. Supply it through a locally protected secret mechanism/environment. Isolate the
   launch from other auth tokens, cloud-provider selectors, endpoint overrides,
   user/project API key helpers, and unrelated settings.
3. Use `--bare`, disable inherited setting sources, and explicitly provide only the
   necessary project context and tools. Bare mode does not automatically read
   `CLAUDE.md`; the launcher must supply `AGENTS.md` and required task instructions.
4. Set the model, a per-call timebox, and `--max-budget-usd` for print-mode API work
   within the issued grant. Verify flags against the installed version's help.
5. Persist actual usage and unresolved reservations. Do not retry using a personal
   subscription if API authentication, quota, or funding fails.

This is a launch specification, not an installed billing firewall. Funding remains
blocked until verified. The user's existing subscription login must be preserved;
do not delete or repurpose their personal configuration.

The user confirmed £75 is Pro/Max extra usage, not Console API credit, and the
included subscription allowance must not be consumed. Official
help documents extra usage after included limits are reached. No reliable
extra-credit-only mode was established. Do not try to exhaust the included quota
to reach the credit balance. Also do not rely on outdated announcements about
separate Agent SDK credits: the help center's June 15 update says that change was
paused and subscription `claude -p` still draws from subscription limits.

Sources: [usage credits](https://support.claude.com/en/articles/12429409-manage-usage-credits-for-paid-claude-plans),
[Agent SDK billing update](https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan).

References: [Claude authentication](https://code.claude.com/docs/en/authentication),
[CLI reference](https://code.claude.com/docs/en/cli-reference).

## Cursor and commit provenance

Use Cursor CLI for a bounded commit review/execution task after login
and allocation are ready. Give it the exact repository, staged files, commit
message, expected author, allowed operations, and limits. Review its result.

All new agent commits use `Cursor <cursor@flashtex.invalid>` as the project bot
identity. Configure only this repository:

```sh
git config --local user.name Cursor
git config --local user.email cursor@flashtex.invalid
```

Explicit author/committer environment variables can override Git configuration;
verify the resulting identity after committing. Do not rewrite historical commits.
Add `Implementation-Agent` and `Commit-Executor` trailers reflecting what happened.
Bot identity is not evidence that Cursor authored the implementation. When Cursor
execution is unavailable, prepare changes and report the blocker; do not falsely
attribute execution. The user explicitly confirmed actual Cursor CLI execution
is required; the author label alone is insufficient. GitHub app badges and verified
account association are separate and are not promised by changing author metadata.

## Deadline-aware improvement loop

Work toward an observable acceptance gate. For each significant experiment:

1. State the hypothesis and the behavior/measurement it should improve.
2. Choose the cheapest informative test and bound time and spending.
3. Implement and measure using repeatable inputs.
4. Keep an improvement; diagnose or revert a regression deliberately, preserving
   unrelated work. Record the evidence and resulting decision.
5. Update remaining-work estimates and select the next highest-value step.

After two materially similar failed attempts, write down the blocker and change
approach, ask a focused question, or seek a bounded second opinion if funded.
Do not persist with blind retries. Do not abandon an authorized task simply
because it is difficult; identify a useful next action and make limitations clear.

Report optimistic/likely/pessimistic remaining-time estimates in minutes, including
integration and dependencies. Prefer completed acceptance gates over unsupported
percent-complete claims. A difficult forecast is a planning fact, not failure to
be positive. Honesty takes precedence over motivational language.

## Information volume and discoverability

Use three levels:

- Startup: `AGENTS.md`, `docs/INDEX.md`, project/resource control, own handoff.
- Working context: relevant contracts, peer handoffs, focused source and tests.
- Evidence archive: linked detailed findings, benchmark artifacts, and decisions.

Keep handoffs roughly one screen to a page when practical. Put longer findings in
topic documents with owner, date, status, evidence revisions, and links. Archive
superseded material with a pointer to its replacement. Do not repeatedly read all
logs, or substitute a verbose commit history for current state.

For every interface change, name affected consumers and required actions. Consumers
record the reviewed commit and adaptation. Crucial discoveries must be linked from
the index or a current handoff so another computer can locate them.

## Before compaction, context pressure, or stopping

Update your handoff proactively at normal checkpoints; do not rely on receiving a
compaction warning. Include:

- Objective, acceptance criteria, ownership, branch, and latest code revision.
- Dirty/untracked files, unpublished work, and running jobs.
- Completed gates and exact checks with results, including failures.
- Decisions and rejected approaches with short evidence summaries.
- Dependencies, reviewed peer revisions, and expected interface changes.
- Deadline, ETA range, budget/allocation, in-flight usage, and unknowns.
- Exact next commands or actions and relevant document paths.

Publish safe project artifacts and the handoff on your branch. Never include
credentials, private chain-of-thought, or entire unfiltered session transcripts.

## After compaction or restart

Read startup files and your handoff; inspect working-tree state and fetch peers.
Verify the referenced commits, actual files, and running jobs. Reconcile local
reservations before any new spending. Determine what changed since the handoff
and update the plan. Resume from the next unmet gate without repeating completed
work unless new evidence requires revalidation.

Claude discovers shared rules through the root `CLAUDE.md` import. Cursor receives
a small always-applied rule pointing to the same source. Other agents must be
explicitly instructed to read root `AGENTS.md`. None of these mechanisms alone
ensures that an idle process wakes or that every instruction is followed.

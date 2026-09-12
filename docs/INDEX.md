# Start here: shared project memory

Read this index after startup, resumption, or compaction. Follow links relevant to
your task; do not load the entire repository history into every prompt.

| Need | Authoritative location | Writer |
|---|---|---|
| Required collaboration behavior | [AGENTS.md](../AGENTS.md) | Integration owner with user direction |
| Product requirements | [Master plan](../latex-master-plan.md) | Product/integration owner |
| Deadline and acceptance gates | [PROJECT.md](../coordination/PROJECT.md) | Designated coordinator |
| Funding, permissions, allocations | [RESOURCES.md](../coordination/RESOURCES.md) | Designated resource owner |
| Delegation, progress, recovery procedures | [Agent operations](agent-operations.md) | Integration owner |
| Active work and blockers | `coordination/<agent-id>.md` on each task branch | That agent |
| Handoff format | [Agent template](../coordination/templates/agent.md) | Integration owner |
| Shared interfaces | `docs/contracts/<interface>.md` when created | Assigned interface owner |
| Durable decisions | `docs/decisions/<id>-<topic>.md` when created | Decision owner |
| Reproduction evidence / large outputs | Paths linked from the relevant handoff | Producing agent |

Some interface and decision directories will be created as implementation starts;
their listing here does not imply those designs already exist.

## Discover updates before they merge

Run `git fetch origin --prune`, list remote task branches, and inspect the relevant
branch's handoff with `git show <remote-branch>:coordination/<agent-id>.md`.
Record the peer commit you reviewed and the resulting adaptation. Main contains
integrated decisions; branch proposals remain proposals until accepted.

Every new lasting document must be linked from this index, a listed topical
index, or the producing agent's handoff. Include owner, status, date, relevant
revision, and what supersedes it. Do not create orphan knowledge files.

## Information hierarchy

Higher-priority instructions and explicit user decisions govern. Shared contracts
describe agreed interfaces; code and test results describe observed behavior.
Handoffs and plans can become stale. When evidence conflicts with a document,
report and resolve the discrepancy instead of silently choosing a convenient one.

Keep startup material short. Keep detailed findings in linked topic files, and
retain concise evidence summaries rather than huge terminal transcripts. Never
commit credentials, access tokens, login URLs, or private captures.

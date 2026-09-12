# Commander task board

Owner: Commander. Updated: 2026-09-12T03:43:54Z.
Full process: [orchestration master plan](../ORCHESTRATION.md).

| ID / revision | Task | Owner | State | Dependencies |
|---|---|---|---|---|
| ORCH-001 / 1 | Publish orchestration plan, roster, dispatch board, and discovery links | commander | integrated: 567d84b on main | Self-registration clarification follow-up |
| ORCH-002 / 1 | Executable coordination and discovery service | commander | ready_for_integration; 12 tests pass | Cursor publication |
| FT-001 / 1 | Shared compile/edit/capture contracts and fixtures | commander | ready_for_integration: runtime-v1 and fixtures | Cursor publication |
| FT-002 / 1 | Original Rust compiler foundation | Unassigned | unassigned | FT-001 |
| FT-003 / 1 | Native Mac shell | mac-claude-a | assigned; acknowledgement pending | runtime-v1; use available Codex instead of protected Claude |
| FT-004 / 1 | Pencil and camera capture | aarush-macbook | assigned; acknowledgement pending | runtime-v1; confirm OpenAI tool readiness; no protected Claude |
| FT-005 / 1 | Rust layout/output and source mapping | Unassigned | unassigned | FT-002 |
| FT-006 / 1 | Incremental reuse and recovery evidence | Unassigned | unassigned | FT-005 |
| FT-007 / 1 | Grok/transfer and reviewed insertion | Unassigned | unassigned | FT-001/003/004, Grok funding |
| FT-008 / 1 | Integrated demo verification | commander + future Mac worker | unassigned | FT-003/005/006/007 |

## Dispatch record required before changing a task to assigned

```text
Task ID / assignment revision / owner / branch:
Objective and acceptance criteria:
Owned paths / protected shared paths:
Dependencies and exact input revisions:
Contract and fixture links:
Timebox / next report time / hard deadline:
Resource allocation and descendant limits:
Validation / deliverable location:
Worker acknowledgement revision:
```

Assignment rows alone do not launch agents. Worker must acknowledge the dispatch
revision in its handoff before being counted as working. Commander owns this file.
Structured FT-003/FT-004 records under `coordination/assignments/` define the exact
ownership, branch, timebox, acceptance criteria, and funding references. Workers
should migrate their legacy Markdown registration with `scripts/coord.py register`
on the assigned branch, then acknowledge the published assignment. If local Cursor
is unavailable, submit a patch plus report via a repository issue for central commit.

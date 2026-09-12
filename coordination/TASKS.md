# Commander task board

Owner: Commander. Updated: 2026-09-12T03:22:36Z.
Full process: [orchestration master plan](../ORCHESTRATION.md).

| ID / revision | Task | Owner | State | Dependencies |
|---|---|---|---|---|
| ORCH-001 / 1 | Publish orchestration plan, roster, dispatch board, and discovery links | commander | ready_for_publication_after_commit | Cursor commit done locally; parent Commander push next |
| FT-001 / 1 | Shared compile/edit/capture contracts and fixtures | Unassigned | unassigned | Register eligible worker / define supported sample |
| FT-002 / 1 | Original Rust compiler foundation | Unassigned | unassigned | FT-001 |
| FT-003 / 1 | Native Mac shell | Unassigned | unassigned | Mac worker, FT-001 |
| FT-004 / 1 | Pencil and camera capture | Unassigned | unassigned | Mac/device worker, FT-001 |
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
Next action after ORCH-001 publication: worker registration and dispatch.

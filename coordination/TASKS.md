# Commander task board

Owner: Commander `orchestrator-jaysen-claude` (mac-m1max-a). Updated: 2026-09-13T03:30:00Z.
Full process: [orchestration master plan](../ORCHESTRATION.md).

> **Staffing rule (user, 2026-09-13):** allocate tasks ONLY to mac-m1max-a
> (parent `mac-claude-a` staffs lanes there) until another machine posts a fresh
> live report on GH issue #2. Nothing is dispatched to Daniel (mac-m5pro-dq222),
> Kabir (mac-m5pro-kabir), Aarush or linux-primary before that. Rows below for
> those machines are historical; their `assigned` states are not live evidence.

## Current queue (main ad71b648, all owned by mac-claude-a on mac-m1max-a)

| ID / revision | Task | Branch | State | Notes |
|---|---|---|---|---|
| FT-050 / 1 | Generic AI provider abstraction in crates/assistant-context + crates/bridge (GH2#5649521523 contract) | agent/mac-ai-generic/crates | assigned | Mac side is the parent's mac-ai-generic lane; no apps/mac edits from this lane |
| FT-051 / 1 | HW1.tex exact render: heading offset/skips, overfull lines, \setlist notice + three compiler asks | agent/mac-render-pipeline/hw1-math-10, agent/mac-compiler-hw1/compiler | assigned | compiler edits are a user-directed temporary allocation while FT-002 (Kabir) is not live |
| FT-052 / 1 | Corpus coverage gate from PR #42/#53 and first math/package coverage increment | agent/mac-coverage/corpus-gate | assigned | PR #42/#53 integration: only two add/add coordination conflicts |
| FT-053 / 1 | iPad companion finished: remaining durability cases, full simulator suite, device blockers listed | agent/mac-ios-app-2/finish | assigned | |
| FT-054 / 1 | Modern editor: highlighting, intellisense, navigation, scrollable problems panel | agent/mac-editor-intellisense/editor | assigned | extends mac-syntax-highlight / mac-completion-sync |
| FT-055 / 1 | CI/CD auto-deploy of compiler + GUI to the website; user docs | agent/mac-claude-a/cicd | assigned | parent already started; tracked to avoid duplication |

Integration facts at this update: `origin/agent/mac-pdf/searchable-text` b4b1513
is already on main (94a67130/9689384e/4893f3e7), nothing to merge. PR #42 and PR #53
are the only pending deliveries from this machine. Daniel's branches
`daniel-parent-b/lm-math-symbols` (11 ahead, touches apps/mac Fonts.swift) and
`daniel-math-accents/compiler` (6 ahead) are unintegrated and unowned until Daniel
reports live; do not merge without a fresh owner report and a Mac build/test.

| ID / revision | Task | Owner | State | Dependencies |
|---|---|---|---|---|
| ORCH-001 / 1 | Publish orchestration plan, roster, dispatch board, and discovery links | commander | integrated: 567d84b on main | Self-registration clarification follow-up |
| ORCH-002 / 1 | Executable coordination and discovery service | commander | integrated: 6d096a3; 12 tests pass | Discovery service active |
| FT-001 / 1 | Shared compile/edit/capture contracts and fixtures | commander | integrated: 6d096a3 runtime-v1 and fixtures | Native consumer verification pending |
| FT-002 / 1 | Original Rust compiler foundation | claude (Codex on Kabir Mac) | assigned; acknowledgement pending | FT-001 |
| FT-003 / 1 | Native Mac shell | mac-claude-a | assigned; acknowledgement pending | runtime-v1; use available Codex instead of protected Claude |
| FT-004 / 1 | Pencil and camera capture | aarush-macbook | assigned; acknowledgement pending | runtime-v1; confirm OpenAI tool readiness; no protected Claude |
| FT-005 / 1 | Rust layout/output and source mapping | Unassigned | unassigned | FT-002 |
| FT-006 / 1 | Incremental reuse and recovery evidence | Unassigned | unassigned | FT-005 |
| FT-007 / 1 | Rust bridge: Grok/transfer and reviewed insertion | commander | assigned; begins after loop publication | FT-001/003/004, Grok funding |
| FT-008 / 1 | Integrated demo verification | commander + future Mac worker | unassigned | FT-003/005/006/007 |
| FT-014 / 1 | Independent companion native validation and repair broker | chatgpt-a | assigned; ACK/PID pending | FT-004 branch, Xcode 26.6; no overlapping writes |
| FT-015 / 1 | Linux end-to-end demo, recovery, and regression harness | local-claude-opus | assigned; blocked on execution login | Compiler/bridge/PDF exact SHAs; offline fixtures required |
| FT-016 / 1 | Deterministic Claude worker supervisor and recovery safety | commander-supervisor | assigned; implementation via Commander OpenAI route | Local Claude FT-015 and auth recovery issue #6 |
| FT-017 / 1 | Reference TeX raster-diff visual oracle | mac-visual-oracle (Opus/Mac) | assigned; ACK/PID pending | Declared visual fixtures; test oracles only |
| FT-018 / 1 | TeX font shaping, metrics, and PDF embedding | mac-font-engine (Opus/Mac) | assigned; ACK/PID pending | Visual oracle; compiler adapter later |
| FT-019 / 1 | Paragraph line breaking, glue, kerning, and baselines | mac-paragraph-layout (Opus/Mac) | assigned; ACK/PID pending | FT-018 metric API |
| FT-020 / 1 | TeX-style math boxes, rules, delimiters, and spacing | mac-math-layout (Opus/Mac) | assigned; ACK/PID pending | FT-018 metric API |

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

## Commander implementation queue (current user instruction)

1. ORCH-003/004: worker execution, completion dispatch, resource/recovery reporting,
   coauthor enforcement, and real autonomous-startup verification.
2. FT-007: original Rust capture bridge and transfer-v1 consumer contract under
   crates/bridge; deduplication, revision-safe review and context/provider tests.
3. Integration: review and combine compiler/Mac/companion checkpoints, resolve
   conflicts and validate combined behavior; do not make workers wait on review
   when an independent next stage is already approved.
4. FT-008: end-to-end test scenarios, compatibility corpus and measured responsiveness;
   distinguish actual device/native evidence from Linux or protocol-only checks.

Compiler FT-002 acknowledgement is published at 25fe5c4. Mac FT-003 revision1 reports
ready at f13979c: six native tests/builds reported; Commander inspected package,
README and ShellModel and identified stale-source navigation as a follow-up check.
Mac next queued work now prioritizes real JSONLines subprocess transport, matching
its stated next experiment. Initial iPad acknowledgement remains pending.

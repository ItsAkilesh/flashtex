# Codex handoff

- Updated UTC: 2026-09-12T03:17:10Z.
- Agent / machine alias: Codex / linux-primary; no child tasks launched.
- Task: resource/deadline/context policy and Cursor/Claude CLI readiness.
- Branch: `agent/codex/resource-context-policy`; main reviewed through `92e9595`.
- Owned paths: instruction/discovery/control documents and handoff template;
  no product code.
- State: in progress; Cursor CLI reviewed the documentation and is committing it
  under `cursor-docs-commit-001`. Parent Codex will verify and push next.
- Ready: shared discovery rules, API-only Claude launch specification, personal
  usage prohibition, fixed budget allocation policy, deadline forecasts, bounded
  experiments, and compaction/restart recovery procedure.
- Tool evidence: Cursor CLI `2026.09.10-fd3934a` and Claude Code `2.1.229` already
  installed and executable. Cursor login verified and in use for this commit.
  Claude has a Max subscription login; zero Claude calls in this session; that
  login was left untouched.
- Validation: CLI version/help/auth-status and whitespace checks passed; Cursor
  reviewed the staged documentation set before commit.
- Deadline: `2026-09-12T14:00:00Z` (next 10 a.m. Pittsburgh); fixed final
  verification begins `2026-09-12T11:50:00Z`. At update, about 647 minutes remained.
- Incomplete: verified general billing source/limits,
  funded delegation, transactional resource enforcement, checkpoint daemon/CI.
  Push has not occurred yet.
- Resource use: zero Claude calls; authorized Cursor commit session
  `cursor-docs-commit-001` is being used now. Host-agent and Cursor monetary
  costs unknown; no invented balances.
- User confirmed £75 is Pro/Max extra usage and included allowance is protected.
  Official documentation does not establish a credit-only mode preserving included
  allowance; Claude calls remain blocked. Do not assume headless mode avoids
  subscription usage.
- User confirmed Cursor CLI must execute commits, not just supply the author
  name. Cursor is authenticated and executing this bounded commit.
- ETA: after this commit, parent Codex verifies and publishes; product ETA unknown.
- Peer review: no new main or other peer changes observed at latest fetch;
  adaptation is the user's provenance and personal-Claude restrictions.
- Dirty files: documents listed by `git status`; preserve rather than recreate.
  Git identity is repository-local `Cursor <cursor@flashtex.invalid>`; no
  historical authors were changed.
- Next: parent Codex verifies the Cursor commit, then pushes this branch.
- Resume reading: `AGENTS.md`, `docs/INDEX.md`, `coordination/RESOURCES.md`,
  `coordination/PROJECT.md`, `docs/agent-operations.md`, then actual Git state.
- Existing integration: earlier collaboration rules published at `fc7e8a2`
  with handoff follow-up at `92e9595`; this task extends those rules.

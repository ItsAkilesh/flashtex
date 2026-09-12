# Agent roster

Owner: Commander, designated by the user. Updated: 2026-09-12T03:23:15Z.

| Agent ID | Machine | Tool | Capabilities | Resource | State / assignment |
|---|---|---|---|---|---|
| commander | linux-primary | Codex | Git, coordination, local tooling; no native Xcode on Linux | User-reported $200 OpenAI account; quota unknown | Active: orchestration plan and integration |
| cursor-committer | linux-primary | Cursor CLI | Authenticated, verified Git commit execution | Explicitly authorized bounded commit sessions; cost unknown | On demand; no independent development assignment |

No other worker is registered yet. The user confirmed agents will populate the
system themselves. Discover `agent/*/register` branches at each checkpoint; do not
wait for a manual roster from the user or invent active workers.

Workers register via their own branch/handoff. Commander updates this roster;
personal account emails, credentials, and login links must not appear here.

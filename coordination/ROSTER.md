# Agent roster

Owner: Commander, designated by the user. Updated: 2026-09-12T03:43:54Z.

| Agent ID | Machine | Tool | Capabilities | Resource | State / assignment |
|---|---|---|---|---|---|
| commander | linux-primary | Codex | Git, coordination, Python integration tests; no native Xcode on Linux | User-reported $200 OpenAI account; quota unknown | Active: ORCH-002 tooling and FT-001 shared contract |
| cursor-committer | linux-primary | Cursor CLI | Authenticated, verified Git commit execution | Explicitly authorized bounded commit sessions; cost unknown | On demand; no independent development assignment |
| mac-claude-a | mac-m1max-a | Registration from Claude; Codex Plus also available | Reported M1 Max, Xcode 26.3, Swift, Rust, simulators; paired devices offline | openai-mac-plus-ft003; included Claude not allocated | Registered via 431889c; FT-003 assigned, acceptance pending |
| aarush-macbook | aarush-macbook | Registration from Cowork; ChatGPT Plus available | Reported M5, Xcode 26.6, Swift, Python; no simulators installed | openai-aarush-plus-ft004; tool readiness must be confirmed | Registered via 5db9d2c; FT-004 assigned, acceptance pending |
| chatgpt-a | aarush-macbook-chatgpt | Codex CLI | Xcode 26.6, Swift, Node, Python, Git push | openai-aarush-chatgpt-ft014; quota unknown | FT-014 assigned; active polling reported, ACK/PID pending |
| local-claude-opus | linux-primary | Claude Code Opus | Linux, substantial isolated implementation after verified API funding/auth | claude-linux-api; local subscription prohibited, API credits/auth unverified | FT-015 queued; blocked, no inference launched |

| claude | mac-m5pro-kabir | Codex Plus for implementation; authenticated Cursor for commits | Reported M5 Pro, Rust ready; no full Xcode | openai-kabir-plus-ft002; protected Claude excluded | Registered ec0dac7; FT-002 assigned, acceptance pending |

Two Mac registrations were read directly from their remote branches. These are
capability reports, not proof of active implementation. Their prior registration
commits were not executed by Cursor; do not merge/rewrite those commits to hide
that fact. The Commander records their inventory summaries here with source SHAs.
Use centralized patch submission if Cursor cannot run on the worker machine.
Discover further registration branches at each checkpoint; no manual user roster
is needed. Historical quota snapshots from workers are stale, not fresh balances.

Workers register via their own branch/handoff. Commander updates this roster;
personal account emails, credentials, and login links must not appear here.

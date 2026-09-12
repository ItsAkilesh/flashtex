# mac-claude-a handoff

Agent / task / branch: mac-claude-a (Claude Code on mac-m1max-a, user-directed) / FT-003 rev 1 native Mac shell / `agent/mac-claude-a/mac-shell`
State: ready for integration (first increment)
Owned paths: `apps/mac/`, `coordination/mac-claude-a.md`, `coordination/agents/mac-claude-a.json`
Main integrated through: d6858792737c0980cccaec5db835075e94a1ebc9
Ready behavior: Swift package builds (`swift build`, `xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build` → BUILD SUCCEEDED); editor + fixture preview with explicit FIXTURE status; clicking preview text selects the UTF-8 source range converted to UTF-16; diagnostics "Go to source"; dark preview toggle. 6/6 tests pass (`swift test`). See `apps/mac/README.md`.
Incomplete behavior: no Rust worker transport (fixture only); no PDF; no visual screenshot evidence (agent terminal lacks Screen Recording permission — user to eyeball the running app); no reverse sync.
Interface changes and required consumer actions: none. Consumes runtime-v1 as published; unknown item kinds are tolerated.
Validation: `cd apps/mac && swift build && swift test && xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build` on Xcode 26.3 / Swift 6.2.4, 2026-09-12T04:10Z.
Needs from others: Commander integration; FT-002/FT-005 to define worker launch (binary path + JSON Lines) so the shell can attach.
Next action: keep polling checkpoints; if no new dispatch, add JSON Lines transport stub behind the fixture (reads compile_result lines from a subprocess) as a bounded experiment.
Peer revisions reviewed and adaptations: origin/main d685879 (no change); origin/agent/claude/compiler-foundation 25fe5c4 (FT-002 ack only, no code) → no adaptation.
Resource: session runs on user's Claude Max at explicit user direction (override of Codex-only note on openai-mac-plus-ft003); no API spend; usage unknown.
Commit identity: jay3332 per AGENTS.md mac-m1max-a exception; Commit-Executor: git via Claude Code (Cursor CLI not logged in here).
Updated: 2026-09-12T04:10:00Z

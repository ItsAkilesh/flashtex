# mac-claude-a handoff

Agent / task / branch: mac-claude-a (Claude Code on mac-m1max-a, user-directed) / FT-003 rev 1 native Mac shell / `agent/mac-claude-a/mac-shell`
State: ready for integration (increments 1–2)
Owned paths: `apps/mac/`, `coordination/mac-claude-a.md`, `coordination/agents/mac-claude-a.json`
Main integrated through: d6858792737c0980cccaec5db835075e94a1ebc9
Ready behavior: increment 2 adds runtime-v1 JSON Lines worker transport (attach executable, ⌘B compile, WORKER/FIXTURE badge, stale-revision guard, error/violation/exit reporting) verified against a Python test double; 13/13 tests. Increment 1: Swift package builds (`swift build`, `xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build` → BUILD SUCCEEDED); editor + fixture preview with explicit FIXTURE status; clicking preview text selects the UTF-8 source range converted to UTF-16; diagnostics "Go to source"; dark preview toggle. 6/6 tests pass (`swift test`). See `apps/mac/README.md`.
Incomplete behavior: no real Rust worker to attach yet (needs FT-002 binary path); no PDF; no visual screenshot evidence (agent terminal lacks Screen Recording permission — user to eyeball the running app); no reverse sync.
Interface changes and required consumer actions: none. Consumes runtime-v1 as published; unknown item kinds are tolerated.
Validation: `cd apps/mac && swift build && swift test (13/13) && xcodebuild -scheme FlashTeXMac -destination 'platform=macOS' build → BUILD SUCCEEDED` on Xcode 26.3 / Swift 6.2.4, 2026-09-12T04:24Z.
Needs from others: Commander integration; FT-002 owner to publish the worker binary path/args (shell attaches any executable speaking runtime v1 on stdin/stdout). Offer: this Mac can run xcodebuild on origin/agent/aarush-macbook/companion-capture if the Commander wants build evidence for FT-004.
Next action: poll checkpoints for dispatch; next bounded experiment (≤20 min): source→preview sync (highlight preview items whose range contains the editor caret) and multi-page/diagnostic fixture for manual checks.
Peer revisions reviewed and adaptations: origin/main d685879 (no change); origin/agent/claude/compiler-foundation 25fe5c4 (FT-002 ack only) → none; origin/agent/aarush-macbook/companion-capture 5853ffc (apps/companion scaffold, no contract/fixture change) → none.
Resource: session runs on user's Claude Max at explicit user direction (override of Codex-only note on openai-mac-plus-ft003); no API spend; usage unknown.
Commit identity: jay3332 per AGENTS.md mac-m1max-a exception; Commit-Executor: git via Claude Code (Cursor CLI not logged in here).
Updated: 2026-09-12T04:24:00Z

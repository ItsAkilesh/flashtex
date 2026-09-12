# mac-claude-a registration and handoff

- Updated UTC: 2026-09-12T03:36:00Z
- Agent / parent / machine alias: mac-claude-a (Claude Code 2.1.269, Fable 5.1) / launched directly by the user / mac-m1max-a
- Task / acceptance gate / owned paths: self-registration and machine resource inventory so the Commander can allocate across computers. Owned: `coordination/mac-claude-a.md`, `coordination/machines/mac-m1max-a.md`.
- Branch / code revision / main integrated through: `agent/mac-claude-a/register` / see commit / main `276bb1c`
- State: ready for integration (registration only; no product code)
- Ready behavior and evidence: full inventory in [machines/mac-m1max-a.md](machines/mac-m1max-a.md). Summary: Apple M1 Max Mac with Xcode 26.3, Swift 6.2.4, Rust nightly, iPad and iPhone paired (offline at collection); Claude Max 20x login; Codex CLI on ChatGPT Plus (weekly 52% used at 2026-09-11T03:42Z, resets 2026-09-15T23:05Z); Cursor IDE free tier and Cursor CLI not logged in; no Anthropic, OpenAI, or xAI API keys found in the environment.
- Capabilities offered: Mac/Xcode builds, iOS/iPadOS simulator and physical-device checks (once a device is plugged in), Rust builds, Swift UI work. Eligible for FT-003, FT-004, FT-008 device checks, and Rust tasks.
- Incomplete behavior / blockers / needs from others:
  1. Live Claude session/weekly utilization not measured (keychain read denied). User can run `/usage` here or grant the read.
  2. Cursor CLI is not logged in on this machine, so the Cursor-executed-commit requirement cannot be met here. This registration commit was executed by plain git at the user's explicit request; trailers state that truthfully.
  3. OpenAI API key presence unknown (auth file not read). Grok/xAI key absent.
  4. Commander to reconcile: this Codex login is Plus, not the 20x plan named in RESOURCES.md; the Claude cached extra-usage cap read USD 20.00 on 2026-08-07, which does not match the £75 figure and may be stale.
- Interface changes / consumer actions: none.
- Reviewed peer revisions / resulting adaptations: origin/main `276bb1c`; branches `agent/codex/*` and `agent/commander/orchestration-plan` have no commits beyond main. Adaptation: followed self-registration procedure from ORCHESTRATION.md section 4.
- Validation commands / results / artifact paths: version, login-status, and log inspection commands listed in the inventory; no inference calls beyond this session.
- Exact deadline UTC / remaining time / integration reserve: 2026-09-12T14:00:00Z / about 624 minutes at update / final window from 11:50Z unchanged.
- ETA remaining: registration complete on push; product ETA not applicable.
- Resource pool / allocation ID / maximum: this session runs on `claude-personal` (user's Max 20x) at the user's direct instruction; no allocation ID issued. Codex on this machine: `openai-mac-plus` (proposed alias), Plus plan, no grant issued.
- Confirmed spend / estimated usage / in-flight reservation / remaining: unknown for this session; no API spend.
- Billing evidence / freshness / unknowns: see inventory table; Codex snapshot 2026-09-11T03:42Z, Claude snapshot 2026-08-07 (stale).
- Child tasks: none spawned.
- Dirty files / unpushed work / running jobs: untracked `Cargo.toml`, `Cargo.lock`, `src/main.rs` left uncommitted (not this agent's work). ChatGPT/Codex desktop and Claude desktop apps running.
- Decisions: committed with the project label identity and truthful `Commit-Executor: git via Claude Code` because Cursor CLI is unavailable here.
- Exact next action: Commander adds mac-m1max-a to ROSTER.md and reconciles pools; user decides on Cursor login and Claude usage permission for this machine.
- Resume reading list: AGENTS.md, ORCHESTRATION.md, docs/INDEX.md, coordination/COMMANDER.md, this file, the inventory.

# Resource registration: aarush-macbook

- Updated UTC: 2026-09-12T03:35:00Z
- Agent: Claude (Cowork) / Aarush's MacBook Pro
- Machine alias: aarush-macbook
- Operator: Aarush Raheja (AarushRaheja on GitHub)
- Task: resource inventory and registration

## Hardware

- **Chip:** Apple M5 (arm64)
- **Cores:** 10
- **RAM:** 32 GB
- **OS:** macOS 26.6.2 (Build 25G83)
- **Disk:** internal SSD (capacity not measured)

## Subscription plans and API resources

### Claude (Anthropic)
- **Plan:** Pro ($20/month)
- **Included usage:** standard Pro allowance (must be protected per project rules)
- **Extra usage credits:** £75 Pro extra-usage credit (previously identified)
- **API credits (Console):** none
- **Claude Desktop App:** installed, Cowork session active on this machine
- **Claude Code CLI:** not installed (needs Node.js first)
- **Allocation note:** Pro included allowance is protected. Extra-usage credit
  isolation has not been verified. Claude inference remains blocked per project
  resource policy until a credit-only execution route is confirmed or the user
  overrides.

### OpenAI / Codex
- **Plan:** ChatGPT Plus ($20/month)
- **API credits:** none reported; Plus subscription only
- **Codex CLI:** not installed (config dir ~/.codex exists, binary not on PATH;
  needs Node.js: `brew install node && npm i -g @openai/codex`)
- **Allocation note:** Plus plan provides web/app access. Codex CLI agent
  orchestration requires the CLI binary installed and may consume API credits
  or Plus usage depending on the mode. No separate API balance confirmed.

### Cursor
- **Plan:** not using Cursor (no subscription)
- **CLI:** not installed
- **Allocation note:** this machine cannot fulfill the project's Cursor-commit
  requirement. Commits from this machine need an alternative executor or the
  user must install/authenticate Cursor.

### Other APIs
- No Anthropic Console, xAI/Grok, Google/Gemini, or other API credits available.

## Development tools

| Tool | Version | Status |
|---|---|---|
| Xcode | 26.6 | installed |
| Swift | 6.3.3 (swiftlang-6.3.3.1.3) | installed |
| Git | 2.50.1 (Apple Git-155) | installed |
| Python | 3.14.6 | installed |
| GitHub CLI (gh) | 2.100.0 | installed, authenticated as AarushRaheja |
| Homebrew | installed | installed |
| Rust / Cargo | — | **not installed** |
| Node / npm | — | **not installed** |
| Docker | — | **not installed** |
| LaTeX (pdflatex etc.) | — | **not installed** |

## AI / agent tool configs

| Tool | Status |
|---|---|
| Claude Desktop App | running (Cowork session active) |
| Claude Code CLI | **not installed** |
| OpenAI Codex CLI | **not installed** (config at ~/.codex) |
| Cursor CLI | **not installed** |
| GitHub Copilot | config present (~/.copilot) |
| Kiro | config present (~/.kiro) |
| Conda / Anaconda | present (~/.anaconda, ~/.conda) |

## Capabilities summary

- **Native macOS/iOS builds:** Yes — Xcode 26.6, Swift 6.3.3.
  Best suited for Mac UI shell (FT-003) and companion capture (FT-004).
- **iOS simulators:** 0 currently downloaded (available via Xcode > Settings > Platforms).
- **Rust compiler work:** Not ready — Rust/Cargo must be installed first.
- **Git push:** Authenticated via `gh` (HTTPS). Can push to flash-tex/flashtex.
- **Agent orchestration:** Claude Desktop (Cowork) active. CLI agents (Claude Code,
  Codex) need Node.js and installation.
- **Cursor commits:** Cannot execute. No Cursor subscription or CLI.

## Install steps to unlock full participation

1. **Node.js:** `brew install node`
2. **Rust toolchain:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. **Claude Code CLI:** `npm install -g @anthropic-ai/claude-code` (after Node)
4. **Codex CLI:** `npm install -g @openai/codex` (after Node)
5. **iOS Simulators:** Xcode > Settings > Platforms > download iOS simulator

## Budget allocation constraints

- Claude Pro included allowance: **protected, do not consume**
- Claude £75 extra credit: **blocked until isolation verified or user overrides**
- OpenAI Plus: **web/app only, no confirmed API balance**
- Cursor: **none on this machine**
- Total confirmed spendable API budget on this machine: **$0** until user
  explicitly authorizes a funding source or installs/funds API access.

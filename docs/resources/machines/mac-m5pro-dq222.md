# Machine: mac-m5pro-dq222

Verified by: Claude Code agent `claude-dq222` on this machine, session of 2026-09-12.
Verified at: 2026-09-12T07:34Z. Quota readings are point-in-time and go stale.
Owner agent: `claude-dq222` (this machine).
Status: **capability evidence only.** Confers no spending permission; read
permissions from [RESOURCES.md](../../../coordination/RESOURCES.md).

## Registration intent and scope

The machine owner offers this computer's Claude and Cursor plan access to the
fleet so the Commander can allocate work to it. The owner set this scope:

- Work assigned to this machine is **executed on this machine**, under its own
  local login, by the local agent. **No token, API key, cookie, OAuth
  credential or session identifier is shared, published, or transferred to any
  other computer.** "Use this plan" means "route tasks here", not "authenticate
  as this user elsewhere".
- Access is **temporary** and revocable by the machine owner at any time.
- This file is capability evidence, not a grant. Allocation remains the
  Commander's to issue in `RESOURCES.md`.

## Hardware and OS

| Property | Value |
|---|---|
| Hostname | `Overpriced-Hardware.local` |
| Chip | Apple M5 Pro, 18 cores (6 Super + 12 Performance) |
| Memory | 48 GB |
| OS | macOS 26.6.2, arm64 |
| Free disk | 713 GiB of 926 GiB |

Distinct from `mac-m5pro-kabir`, which is a 15-core / 24 GB M5 Pro. This is the
larger configuration: 18 cores and 48 GB. Memory is not a binding constraint on
concurrent agents here.

## Agent CLIs installed

### Claude Code — the resource being offered

| Property | Value | How read |
|---|---|---|
| Version | 2.1.269, native install | `claude --version` |
| Binary | `~/.local/bin/claude` | `command -v claude` |
| Account | GitHub identity `d-q222`; account email deliberately withheld | `~/.claude.json` → `oauthAccount` |
| `organizationType` | `claude_max` | same |
| `organizationRateLimitTier` | `default_claude_max_20x` | same |
| `billingType` | `stripe_subscription` | same |
| Organization role | admin of a personal organization | same |
| Subscription created | 2026-07-26 | same |
| **`hasExtraUsageEnabled`** | **`false`** | same |
| API key on this machine | **none** | `ANTHROPIC_API_KEY` unset |

**This is a Claude Max 20x plan with extra usage switched off.** That second
fact is the one that matters for allocation risk. With `hasExtraUsageEnabled`
false, the account has no overage route: when plan quota is exhausted, requests
are refused rather than billed. There is no auto-recharge to trip and no credit
balance to drain.

That is a provider-side bound, enforced by Anthropic rather than by any file in
this repository — the kind of isolation `RESOURCES.md` repeatedly asks for
before unblocking Claude work. It bounds the downside to "the plan stops
working until reset". It is **not** permission to spend, and it must not be
read as authorization to enable extra usage, buy credits, or change the plan.

Live plan consumption **could not be read non-interactively.** `/usage` is an
interactive slash command inside the TUI and no CLI subcommand prints it. Any
percentage figure for this machine must come from a human running `/usage` and
pasting the result. Unknown is not zero.

### Cursor CLI — the resource being offered

| Property | Value | How read |
|---|---|---|
| Version | `2026.09.10-fd3934a` (up to date) | `cursor-agent --version` |
| Binaries | `cursor-agent`, `cursor` in `~/.local/bin` | `command -v` |
| Login state | logged in | `cursor-agent status` |
| Account | same owner as above; email deliberately withheld | `cursor-agent status` |
| Subscription tier | **Pro** | `cursor-agent about` |
| Default model | Cursor Grok 4.6 High Fast | `cursor-agent about` |

No quota, request count, or remaining balance is exposed by the Cursor CLI;
`about` and `status` report tier and identity only. Cursor consumption here is
**unmeasurable from the terminal** and must be read from the Cursor dashboard.

Cursor is logged in, so this machine can act as commit executor where AGENTS.md
asks for one. The "hold commits while Cursor login is unavailable" condition
does not apply here.

### Codex CLI — installed, NOT audited

`codex-cli 0.146.0` is present. Login state, plan, quota and funding were **not
audited** in this session, and the machine owner did not offer it. Treat as
unavailable until audited. Do not infer a third pool from its presence.

## Credentials NOT present

Checked and unset in this environment: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`,
`XAI_API_KEY`, `GROK_API_KEY`. Every model route on this computer is a
subscription login. There is no API-billed path here at all, so no Grok or xAI
handwriting-conversion work can run on this machine.

## Development toolchain

| Tool | Version | State |
|---|---|---|
| `swift` | 6.1.2 (swiftlang-6.1.2.1.2) | compiler only |
| `node` | 25.9.0 | ready |
| `python3` | 3.14.6 | ready |
| `uv` | 0.12.3 (Homebrew) | ready |
| `git` | 2.39.5 (Apple Git-154) | ready |
| `cargo` / `rustc` | **NOT INSTALLED** | blocker, below |
| `xcodebuild` | **unavailable** | blocker, below |

### Blocker 1: no Rust toolchain — this machine cannot build the project

`cargo` and `rustc` are not installed. This repository is a Rust workspace of 17
crates; **none of them can be built, tested, or benchmarked here as the machine
currently stands.** This is the single most important allocation fact about this
computer, and it outranks its hardware: 18 cores and 48 GB would make it the
strongest parallel `cargo` builder yet registered, and right now none of that is
reachable.

Resolution is one command, `rustup` install, which the machine owner has not yet
authorized. Until then, do not allocate FT-002 or any crate work here.

### Blocker 2: no full Xcode

`xcode-select -p` resolves to `/Library/Developer/CommandLineTools`:

```text
xcode-select: error: tool 'xcodebuild' requires Xcode, but active developer
directory '/Library/Developer/CommandLineTools' is a command line tools instance
```

`xcodebuild -showsdks` returns nothing and `simctl` lists zero available
simulators. The Swift *compiler* exists, but the Swift Mac app cannot be built,
run, or signed here, and neither can the iPad/iPhone capture companion. Do not
allocate FT-003, FT-004 or FT-008 native work to this machine.

Resolution is installing Xcode and running `sudo xcode-select -s`.

## What this machine is good for, today

Ranked by what the evidence supports, in its current unmodified state:

1. **Documentation, coordination and integration review** — no build required.
2. **Python coordination tooling** — `python3` 3.14.6 and `uv` both ready.
3. **Protocol, schema and contract review** — reading and reasoning work.
4. **Node/JavaScript work** — `node` 25.9.0 ready.
5. **Commit execution** via authenticated Cursor CLI.
6. **Not** Rust or crate work, until `rustup` is installed.
7. **Not** Swift/Apple UI or device capture, until Xcode is installed.
8. **Not** Grok/xAI capture work, until a key is supplied.

If the owner authorizes `rustup`, item 6 moves to the top of this list and this
becomes the fleet's largest Rust builder. That single install is the
highest-value change available to this machine.

## How to re-verify

```sh
claude --version                 # then /usage inside the TUI for live plan state
cursor-agent about               # tier and account
cursor-agent status              # login state
xcode-select -p                  # Xcode blocker check
command -v cargo rustc           # Rust blocker check
```

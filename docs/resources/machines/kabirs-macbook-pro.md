# Machine: mac-m5pro-kabir

Verified by: Claude agent, session of 2026-09-12.
Verified at: 2026-09-12T03:18Z; plan facts updated 2026-09-12T03:40Z (quota readings are point-in-time).
Owner agent: Claude on this machine.
Status: capability evidence. Confers no spending permission; see
[RESOURCES.md](../../../coordination/RESOURCES.md).

## Hardware and OS

| Property | Value |
|---|---|
| Computer name | Kabir's MacBook Pro |
| Hostname | `Kabirs-MacBook-Pro.local` |
| CPU | Apple M5 Pro, 15 cores |
| Memory | 24 GB |
| OS | macOS 26.6.2 (build 25G83), arm64 |
| Free disk | 742 GiB of 926 GiB |
| Terminal | Ghostty + tmux 3.7c |

Suitable for parallel Rust compilation and for running several agent CLIs
concurrently. Memory is the binding constraint on how many heavy agents run at once.

## Agent CLIs installed

### Claude Code

| Property | Value | How read |
|---|---|---|
| Version | 2.1.269, native install | `claude --version` |
| Binary | `~/.local/bin/claude` | symlink inspection |
| Account | `kabirgoyal@icloud.com` | `~/.claude.json` → `oauthAccount` |
| Organization role | admin of a personal organization | same |
| `organizationType` | `claude_pro` | same |
| `billingType` | `stripe_subscription` | same |
| `organizationRateLimitTier` | `default_claude_ai` | same |
| `hasExtraUsageEnabled` | `true` | same |
| Subscription created | 2025-12-07 | same |
| API key on this machine | **None** | no `ANTHROPIC_*` var in env or shell rc |

Live plan consumption could not be read non-interactively: `/usage` is an
interactive slash command, and there is no CLI subcommand that prints it. A human
must run `/usage` inside Claude Code and paste the result.

Local transcript totals on this machine only (not account-wide) — 4 sessions,
2026-09-04 to 2026-09-12, all `claude-opus-5`:

| Counter | Tokens |
|---|---|
| input | 470 |
| output | 156,289 |
| cache write | 776,383 |
| cache read | 12,686,614 |
| total | 13,619,756 |

The account is reachable through a subscription only. There is **no separate
API-key route configured on this machine**, which is the funding isolation
`RESOURCES.md` and `CLAUDE.md` require before Claude work is unblocked. Supplying
an `ANTHROPIC_API_KEY` funded by approved project credit is what would change that.

### Codex CLI

| Property | Value | How read |
|---|---|---|
| Version | `codex-cli 0.154.0`, standalone install | `codex --version`, `codex doctor` |
| Auth mode | `chatgpt` (subscription login, not an API key) | `codex doctor` → auth |
| Reported plan | `plus` | session rate-limit payload |
| Default model | `gpt-5.6-sol`, reasoning effort `high` | `~/.codex/config.toml` |
| Desktop app | 26.825.41651, installed, not running | `codex doctor` |
| Connectivity | websocket HTTP 101, endpoints reachable | `codex doctor` |
| Health | 22 ok · 1 idle · 0 warn · 0 fail | `codex doctor` |

Quota, read 2026-09-12T03:18:08Z from a deliberate 4,175-token probe:

| Window | Used | Resets |
|---|---|---|
| Primary, 300 min (5 h) | 0% | 2026-09-12T08:18:06Z |
| Secondary, 10080 min (7 d) | 4% | 2026-09-15T04:19:13Z |

Credits block reports `has_credits: false`, `unlimited: false`, `balance: "0"` —
this account has **no purchased credit balance**, only plan quota.

**Discrepancy resolved, 2026-09-12T03:40Z.** The user confirmed this is the $20
ChatGPT Plus plan, which matches the CLI's `plan_type: "plus"`. The "20x Codex
plan" mentioned in `RESOURCES.md` does **not** describe this account. The resource
owner should correct that entry; size Codex allocations off Plus.

User-reported availability, not readable from the CLI:

- Substantially all of the weekly and 5-hour session allowance is available for
  this project. The measured 0% / 4% readings above are consistent with that.
- **Three weekly resets** are available and may be used for this project.

Resets are a separate mechanism from purchased credits, so the `balance: "0"`
reading above does not contradict this; the CLI exposes no reset count at all, so
the figure of three is user-reported and cannot be verified from this machine.
Decrement it in `RESOURCES.md` as resets are consumed — Git is the only ledger
for it.

**This makes Codex the least-constrained measured resource on this machine.** It
is also the only route here that is simultaneously funded, authorized, and
unblocked: Claude work is policy-blocked pending funding isolation, and Cursor
exposes no quota to measure against. Allocate sustained autonomous work here to
Codex, on the Rust compiler, since the Xcode blocker below rules out Apple work.

Rate-limit percentages are only refreshed by making a request; between requests
the last snapshot goes stale. The snapshot preceding this one was 13 days old.

### Cursor CLI

| Property | Value | How read |
|---|---|---|
| Version | `2026.09.10-fd3934a` | `cursor-agent --version` |
| Installed | 2026-09-12, this session, at user request | install script |
| Binaries | `cursor-agent` and `agent` in `~/.local/bin` | symlink inspection |
| Account | `kabirgoyal@icloud.com` | `cursor-agent status` |
| Subscription tier | **Pro** | `cursor-agent about` |
| Default model | Auto | `cursor-agent about` |

No quota, request count, or remaining balance is exposed by the CLI; `about` and
`status` report tier and identity only. Cursor consumption on this machine is
therefore **unmeasurable from the terminal** and must be read from the Cursor
dashboard.

Cursor is the only CLI here that offers a choice across vendors — 40+ models
including `claude-opus-5-*`, `claude-sonnet-5-thinking-*`, `claude-fable-5-*`,
`gpt-5.6-sol-*`, `gpt-5.3-codex-*`, `cursor-grok-4.6-*`, `gemini-3.7-flash-high`,
and `composer-2.5`. Full list: `cursor-agent --list-models`.

Two capabilities matter for multi-machine orchestration:

- `cursor-agent worker` runs a self-hosted Cloud Agent worker that connects to
  Cursor and executes tool calls on this machine. Without `--pool` it is a personal
  "My Machines" worker and needs no Enterprise plan. This is a real route for
  driving this Mac from elsewhere; it is **not authorized or tested** here.
- `cursor-agent -w/--worktree` starts in an isolated git worktree under
  `~/.cursor/worktrees/`, which satisfies the AGENTS.md rule that concurrent
  agents on one computer use separate worktrees.

Cursor is also the commit executor required by AGENTS.md. It is now logged in, so
the "hold commits while Cursor login is unavailable" condition no longer applies
on this machine.

### opencode

Installed at `~/.opencode/bin/opencode`, version 1.18.27. Login state, provider
routing, and funding were **not audited**. Treat as unavailable until audited.

## No credentials present for

- **Grok / xAI.** The master plan names a Grok API key as the AI provider for
  handwriting conversion. No `XAI_*` or `GROK_*` variable exists in the
  environment or shell startup files on this machine. Capture/AI work cannot be
  executed here until a key is supplied.
- **Anthropic API**, **OpenAI API**, Gemini, or any other direct API key.

Every model route on this machine is a subscription login. There is currently no
API-billed path on this computer at all.

## Development toolchain

| Tool | Version | State |
|---|---|---|
| `cargo` / `rustc` | 1.98.0 (Homebrew) | ready |
| `swift` | 6.3.3 (swiftlang-6.3.3.1.3) | compiler only |
| `node` | 26.7.0 | ready |
| `python3` | 3.14.7 | ready |
| `git` | `/usr/bin/git`, repo detected | ready |
| `uv` | not installed | — |
| `xcodebuild` | **unavailable** | blocker, below |

### Blocker: no full Xcode

`xcode-select -p` resolves to `/Library/Developer/CommandLineTools`. `xcodebuild`
refuses to run and no SDK list is available:

```text
xcode-select: error: tool 'xcodebuild' requires Xcode, but active developer
directory '/Library/Developer/CommandLineTools' is a command line tools instance
```

Consequences for task allocation:

- The Rust compiler crate builds fine here. `cargo` is complete and independent
  of Xcode.
- The **Swift Mac UI cannot be built, run, or signed on this machine**, and
  neither can the iPad/iPhone capture companion. The master plan's demo items 1–8
  all terminate in a native app that this machine cannot currently produce.
- Device signing, mentioned in the master plan as available to the team, is not
  usable here in this state.

Resolution is installing Xcode and running `sudo xcode-select -s`. Until then,
allocate Swift/Apple UI and capture-companion tasks to a machine with full Xcode,
and allocate compiler work here. This is the single most important allocation
fact on this computer.

## Commit attribution on this machine

Every commit made from mac-m5pro-kabir must credit the user as co-author. The
user directed this explicitly on 2026-09-12.

```text
Co-authored-by: GoKubar <75052523+GoKubar@users.noreply.github.com>
```

The numeric id came from the public GitHub user API, and `GoKubar` is the identity
this machine's SSH key authenticates as (`ssh -T git@github.com`). A wrong id
silently attributes the commit to nobody, so do not guess one.

`scripts/coord.py publish` builds this trailer itself from `gh api user`, but it
treats `gh` as mandatory and fails outright when `gh` is unauthenticated, which it
is here. Until `gh auth login` is run on this machine, publish through Cursor CLI
directly with the trailer written into the message, and keep Cursor as the commit
executor as AGENTS.md requires. Commits made before 2026-09-12T05:30Z
(25fe5c4, 29221d8, e7127fb, 9f1033b) predate this rule and lack the trailer;
they are pushed, and AGENTS.md forbids rewriting published history, so they stay
as they are.

## Measured throughput capacity

Updated 2026-09-12T04:25Z with measurements, not estimates.

Implementing the whole FT-002 stage-1 compiler foundation on Codex — tokenizer,
parser, layout, JSON Lines transport and tests — consumed 958k tokens and moved
the meters from 0% to 8% of the 5-hour window, and from 4% to 5% of the weekly
window. One substantial engineering task costs roughly **1% of the weekly
allowance**.

| Window | Used at 04:20Z | Resets |
|---|---|---|
| Primary, 5 h | 8% | 2026-09-12T08:18Z |
| Weekly, 7 d | 5% | 2026-09-15T04:19Z |

Plus three weekly resets the user reports as available for this project. Quota is
not the binding constraint on this machine; Xcode is.

This machine runs **three Codex workers concurrently** in separate Git worktrees
(`/Users/kubar/code/ft-wt-*`), which is the isolation AGENTS.md requires for
concurrent agents on one computer. Parallel dispatch here is safe provided each
task names distinct owned paths. 15 cores and 24 GB support that comfortably;
memory is what would bind first if the count grew much beyond this.

**The Commander should route more Rust, protocol, coordination-tooling,
documentation and integration work to this machine.** Route Apple-platform work
(FT-003, FT-004, FT-008 native verification) elsewhere until Xcode is installed.

## What this machine is good for

Ranked by what the evidence supports:

1. **Rust compiler work** — full toolchain, 15 cores, no blockers.
2. **Documentation, coordination, and integration review** — cheap, no build needed.
3. **Commit execution** via Cursor CLI, as AGENTS.md requires.
4. **Not** Swift/Apple UI or device capture, until Xcode is installed.
5. **Not** Grok capture/AI integration, until a key is supplied.

## How to re-verify

```sh
claude --version                 # then /usage inside the TUI for live plan state
codex doctor                     # install, auth mode, connectivity
codex                            # then /status for live rate limits
cursor-agent about               # tier and account
cursor-agent status              # login state
xcode-select -p                  # Xcode blocker check
```

Codex rate limits also appear in the last `token_count` event of the newest
rollout under `~/.codex/sessions/`, but only refresh when a request is made.

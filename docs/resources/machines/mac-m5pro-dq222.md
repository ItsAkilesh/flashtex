# Machine: mac-m5pro-dq222

Verified by: Claude Code agent `claude-dq222` on this machine, session of 2026-09-12.
Verified at: 2026-09-12T08:09Z. Quota readings are point-in-time and go stale.
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
| `cargo` / `rustc` | 1.98.1 (rustup via Homebrew, keg-only) | ready |
| `rustfmt` / `clippy` | 1.9.0-stable / 0.1.98 | ready |
| `rust-analyzer` | bundled with toolchain | ready |
| `clang` / `cc` | Apple clang 21.0.0 | ready |
| `git` | 2.50.1 (Apple Git-155) | ready |
| `swift` | 6.3.3 (swiftlang-6.3.3.1.3) | compiles; see blocker |
| `python3` | 3.14.6 | ready |
| `uv` | 0.12.3 (Homebrew) | ready |
| `node` | 25.9.0 | ready |
| `gh` | 2.96.0, authenticated | ready |
| `pdflatex` | pdfTeX 3.141592653, TeX Live 2026 (BasicTeX) | ready, reference oracle |
| Command Line Tools | 26.6 | ready |
| Apple SDK licence | accepted, recorded 26.6 | ready |
| `xcodebuild` | Xcode 26.6 installed but **unusable** | blocker, below |

Three things were installed or repaired on 2026-09-12 to reach this state: the
Rust toolchain, the Command Line Tools (26.6, replacing a May 2025 build), and
BasicTeX. The Apple SDK licence was accepted, and `xcode-select` was pointed
back at the Command Line Tools.

## Verified build and test evidence

Measured 2026-09-12 after the repairs, not estimated.

| Suite | Result |
|---|---|
| Rust crates building | 17 of 17 |
| Rust tests | 772 passed, 0 failed |
| Python coordination tests | 191 passed, 0 failed, across 10 files |
| `swift build` (apps/mac) | succeeds |
| `swift test` (apps/mac) | blocked, see below |

The Python suite needs `jsonschema==4.23.0`, which the repository declares in
`protocol/rendering-v2.requirements.txt` and asks to be installed in an isolated
venv. That venv is at `.venv/` (gitignored); without it `tests/test_rendering_v2.py`
cannot import and its 23 tests do not run.

### Blocker: Xcode 26.6 is installed but incompatible with this macOS

An earlier revision of this file said Xcode was absent. **That was wrong**, and
the error is recorded here rather than quietly deleted. `Xcode.app` is present,
a full 3.5 GB install carrying every platform SDK: macOS, iPhoneOS and its
simulator, AppleTVOS, WatchOS, XROS and the rest.

It still cannot be used. Pointing the developer directory at it breaks the
entire toolchain:

```text
$ sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
$ cc hello.c -o hello
dlopen(@rpath/libxcodebuildLoader.dylib): Symbol not found: _XPCTypeBool
  Referenced from: /Library/Developer/PrivateFrameworks/CoreDevice.framework
  Expected in:     /Library/Apple/System/Library/PrivateFrameworks/Mercury.framework
```

With `xcode-select` on Xcode, `cc`, `clang`, `git` and `swift` all fail this way,
while `xcodebuild -version` alone still answers. Xcode's bundled CoreDevice
framework expects a symbol that this macOS 26.6.2 build does not export. The
machine was returned to the Command Line Tools, where everything works.

This was the machine's original fault. `xcode-select` was pointing at Xcode when
this session began, which is why `cc`, `git` and Homebrew were all refusing
before any change was made here.

Consequences for allocation:

- Rust, Python, Node, docs and coordination work are unaffected and verified above.
- `swift build` works, because the Swift compiler ships with the Command Line
  Tools. **`swift test` does not**: XCTest is not in the Command Line Tools SDK,
  so `apps/mac` tests fail with `no such module 'XCTest'`.
- No `xcodebuild`, no simulators, no app signing. Route FT-003, FT-004 and FT-008
  native work elsewhere until Xcode is repaired.

Repair is an Xcode update or reinstall, which needs an Apple ID and a
multi-gigabyte download. Do not simply re-point `xcode-select` at the current
Xcode; that reproduces the breakage above.

## What this machine is good for

Ranked by verified evidence:

1. **Rust crate work** — all 17 crates build, 772 tests green, 18 cores and
   48 GB. This is now the machine's strongest capability and the fleet's
   largest Rust builder.
2. **Python coordination tooling** — 191 tests green.
3. **Documentation, protocol and integration review** — no build needed.
4. **LaTeX reference-oracle validation** — `pdflatex` available for
   `tools/native-validation/oracle_compare`.
5. **Node/JavaScript work** — `node` 25.9.0.
6. **Commit execution** via authenticated Cursor CLI and `gh`.
7. **Swift compilation only** — `swift build` works, `swift test` does not.
8. **Not** `xcodebuild`, simulators or app signing, until Xcode is repaired.
9. **Not** Grok/xAI capture work, until a key is supplied.

## Commit attribution on this machine

Directed by the machine owner on 2026-09-12.

- **The owner is always the Git author** of everything committed from this
  computer, using the repository's configured identity `d-q222`
  (`279808976+d-q222@users.noreply.github.com>`). This mirrors the existing
  `mac-m1max-a` primary-author exception. Do **not** set the
  `Cursor <cursor@flashtex.invalid>` author on commits from this machine, even
  when Cursor executes them.
- **Cursor contribution attribution is enabled** at the owner's explicit request.
  `attributeCommitsToAgent` and `attributePRsToAgent` are both `true` in the
  local Cursor CLI configuration, so Cursor adds its own co-author trailer to
  work it performs. Per `AGENTS.md`, that trailer belongs on a commit only when
  Cursor CLI actually executed it.
- **Other AI attribution remains barred** by the owner's standing rule. A local
  `beforeShellExecution` hook denies `git commit`, `gh pr create` and
  `gh pr edit` carrying credit for Claude, Anthropic, Codex, ChatGPT, Copilot,
  Gemini or OpenAI. Cursor is exempt from that hook, and human `Co-authored-by`
  trailers were never affected.

## How to re-verify

```sh
claude --version                 # then /usage inside the TUI for live plan state
cursor-agent about               # tier and account
cursor-agent status              # login state
xcode-select -p                  # Xcode blocker check
cargo --version                  # Rust toolchain
cc /tmp/x.c -o /tmp/x            # must compile, not refuse
xcode-select -p                  # MUST be /Library/Developer/CommandLineTools
pdflatex --version               # reference oracle
```

# Machine: mac-m5pro-dq222

Verified by: Claude Code agent `claude-dq222` on this machine, session of 2026-09-12.
Verified at: 2026-09-12T08:20Z. Quota readings are point-in-time and go stale.
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
| `swift` | 6.3.3 (swiftlang-6.3.3.1.3) | ready, XCTest available |
| `python3` | 3.14.6 | ready |
| `uv` | 0.12.3 (Homebrew) | ready |
| `node` | 25.9.0 | ready |
| `gh` | 2.96.0, authenticated | ready |
| `pdflatex` | pdfTeX 3.141592653, TeX Live 2026 (BasicTeX) | ready, reference oracle |
| Command Line Tools | 26.6 | ready |
| Apple SDK licence | accepted, recorded 26.6 | ready |
| `xcodebuild` | Xcode 26.6 | ready |
| Xcode SDKs | macOS, iPhoneOS, iPhoneSimulator | ready |
| Simulator runtimes | **none installed** | see note |

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
| `swift test` (apps/mac) | 167 executed, 5 skipped, 1 failed |

The single Swift failure is a **pre-existing defect on `main`**, not an
environment problem. `CommandTableTests.testCommandTableMatchesREADMEShortcuts`
finds that `apps/mac/README.md` documents a shortcut, "Edit > Restore Discarded
Buffer", which is absent from the command table. It is reported here rather than
fixed: `apps/mac/` is owned by `mac-claude-a` and this agent does not edit
another agent's paths.

The Python suite needs `jsonschema==4.23.0`, which the repository declares in
`protocol/rendering-v2.requirements.txt` and asks to be installed in an isolated
venv. That venv is at `.venv/` (gitignored); without it `tests/test_rendering_v2.py`
cannot import and its 23 tests do not run.

### Xcode: was broken, now repaired

This section previously recorded Xcode as unusable. **It has been fixed**, and
the diagnosis is kept here because it explains why this machine appeared
completely broken at the start of the session.

Two independent stale components were at fault, not Xcode itself:

1. **`/Library/Developer/PrivateFrameworks/CoreDevice.framework` was version
   397.28, installed March 2025** — left behind by an older Xcode and more than
   a year out of date. It referenced `_XPCTypeBool`, a symbol this macOS 26.6.2
   build no longer exports, so every tool resolving through the developer
   directory died with `Symbol not found`. Xcode 26.6 ships a replacement in
   `Contents/Resources/Packages/` that had never been installed, because Xcode
   had never been first-launched. `xcodebuild -runFirstLaunch` installed it and
   moved CoreDevice to **518.33**.

2. **`MacOSX27.0.sdk` was shadowing the correct SDK.** `xcrun` selects the
   highest-numbered SDK it finds, and a macOS 27.0 SDK dated August 2026 was
   present alongside the correct 26.5 one. Its `libSystem.B.tbd` declares the
   `arm64e.x1` architecture, which this machine's linker (`ld-1267`) cannot
   parse, so every link failed with `tapi error: malformed file`. It has been
   moved to `/Library/Developer/CommandLineTools/SDKs-disabled/` rather than
   deleted, so the change is reversible. `xcrun` now correctly selects 26.5.

`xcode-select` now points at `/Applications/Xcode.app/Contents/Developer` and
the full Apple toolchain works: `cc`, `clang`, `git`, `swift`, `xcodebuild` and
XCTest. This machine can now build, test and sign native Mac work.

**Do not re-add the 27.0 SDK** unless the linker is also upgraded. Restoring it
reproduces the link failure across every crate and Swift target.

**One component is still missing: simulator runtimes.** `xcrun simctl list
runtimes` is empty, so there is no iOS or iPadOS simulator to run the capture
companion against. Native *Mac* work is unaffected. Installing a runtime is a
multi-gigabyte download and has not been done; request it before allocating
FT-004-style companion work here.

## What this machine is good for

Ranked by verified evidence:

1. **Rust crate work** — all 17 crates build, 772 tests green, 18 cores and
   48 GB. The fleet's largest Rust builder.
2. **Native Mac work** — `xcodebuild`, `swift build` and `swift test` all work
   since the repair above. `apps/mac` runs 167 tests here.
3. **Python coordination tooling** — 191 tests green.
4. **Documentation, protocol and integration review**.
5. **LaTeX reference-oracle validation** — `pdflatex` for
   `tools/native-validation/oracle_compare`.
6. **Node/JavaScript work**, and **commit execution** via Cursor CLI and `gh`.
7. **Not** iPad/iPhone companion work, until a simulator runtime is installed.
8. **Not** Grok/xAI capture work, until a key is supplied. See below.

## Supplying the Grok / xAI key

The key is read at `crates/bridge/src/main.rs:136` as the environment variable
**`XAI_API_KEY`**, and only inside the `capture_convert` handler. Two things
gate it, and both must be satisfied:

- The bridge binary must be started with **`--enable-grok`**. Without it the
  handler returns `provider_disabled` and never reads the key.
- `XAI_API_KEY` must be present in that process's environment. Without it the
  handler returns `provider_auth_missing`.

`FLASHTEX_GROK_MODEL` optionally overrides the default model.

Supply it per-process rather than writing it into a shell profile, so it is not
exported to every program on the machine:

```sh
XAI_API_KEY='...' flashtex-bridge --enable-grok
```

`.env` and `.env.*` are now in `.gitignore`, which they were not before. Nothing
in this repository reads a `.env` file, so the environment variable is the only
supported route. Never commit the key, and never record it in
`coordination/` or in this register.

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

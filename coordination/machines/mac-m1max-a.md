# Machine inventory: mac-m1max-a

Owner: mac-claude-a (Claude Code session launched directly by the user).
Status: measured inventory; items marked UNKNOWN were not verifiable from this
session. Collected: 2026-09-12T03:34:00Z. No credentials recorded here.

## Hardware and toolchain

| Item | Value |
|---|---|
| Machine alias | mac-m1max-a (Apple MacBook Pro, Apple M1 Max, 10 cores, 32 GiB RAM) |
| OS | macOS 26.3.1 (25D2128) |
| Xcode | 26.3 (17C529), Swift 6.2.4; simulators for iPad/iPhone 26.3.1 installed |
| Physical devices | iPhone (iOS 26.4) and iPad (iPadOS 26.4.1) are paired with Xcode but were offline at collection. Pencil/camera capture checks are possible once one is connected. |
| Rust | rustc 1.99.0-nightly (2026-07-29), cargo 1.99.0-nightly, `nightly-aarch64-apple-darwin` default |
| Disk | 220 GiB free of 926 GiB |
| GitHub | Push access to `flash-tex/flashtex` over HTTPS; `gh` CLI not installed |
| Local untracked work | `Cargo.toml`, `Cargo.lock`, `src/main.rs` (package `flashtex`, edition 2024). Not created by this agent; preserved and not committed. |

## AI tools and plans

| Tool | Version / state | Plan | Quota evidence | Permission status |
|---|---|---|---|---|
| Claude Code (homebrew) | 2.1.269, logged in via OAuth (Stripe subscription, org admin) | Claude Max, rate-limit tier `default_claude_max_20x` | Live session/weekly utilization NOT fetched: reading the OAuth token from the keychain was denied by the auto-mode classifier. Last cached snapshot in `~/.claude.json` is from 2026-08-07 and shows 0% session, 0% weekly, extra usage enabled with a USD 20.00 monthly cap and USD 0.00 used. That snapshot is stale and must not be used for allocation. | Per RESOURCES.md this is the protected personal pool. This registration session itself is running on it because the user launched Claude Code directly and requested this task. |
| Claude Code (~/.local/bin) | 2.1.47, second install, not on PATH first | same login | same | same |
| Claude desktop app | Installed and running | same account | n/a | n/a |
| Anthropic API key | None found in environment, fish config, or Claude settings (no `apiKeyHelper`) | none | none | No project API pool exists on this machine |
| Codex CLI | 0.154.0, "Logged in using ChatGPT"; default model `gpt-6-astra`, reasoning high | ChatGPT **Plus** (`plan_type: plus` in every rate-limit event since 2026-09-08) | Latest rate-limit event 2026-09-11T03:42:49Z: 5-hour window 16% used, reset 2026-09-11T05:17:07Z (already passed); 7-day window 52% used, reset **2026-09-15T23:05:08Z**; API credits: none (`has_credits: false`, balance 0). | Usable under user authorization; quota is a Plus plan, not the 20x plan mentioned elsewhere |
| ChatGPT desktop app (Codex) | 26.903.61454, running | same ChatGPT account | same limits as CLI | same |
| OpenAI API key | UNKNOWN: not present in environment; `~/.codex/auth.json` was not read (credential file) | unknown | unknown | unknown |
| Cursor IDE | 3.20.14 installed; signed up via GitHub | IDE state caches `stripeMembershipType = free` | n/a | No paid Cursor allocation on this machine unless the user upgrades |
| Cursor CLI (`cursor-agent`/`agent`) | 2026.09.10-fd3934a in `~/.local/bin` (not on PATH); **`cursor-agent status` reports "Not logged in"** | none | none | Cursor-executed commits are NOT possible on this machine until `cursor-agent login` is run |
| xAI / Grok key | None found in environment or shell config | none | none | Grok product conversion cannot run from this machine yet |

## Windows and resets relevant to the deadline (2026-09-12T14:00:00Z)

- Codex 7-day window resets 2026-09-15T23:05Z, after the deadline. Roughly 48% of
  the weekly Plus allowance remained as of 2026-09-11T03:42Z; usage since then is
  unknown until the next Codex turn records a new rate-limit event.
- Codex 5-hour windows reset every 5 hours from first use; the last known window
  ended 2026-09-11T05:17Z, so a fresh 5-hour window is available.
- Claude Max 20x session (5-hour) and weekly windows: unknown live state. The user
  can run `/usage` inside Claude Code on this machine and paste the result, or
  allow the OAuth usage fetch.

## How this was measured

- Versions: `claude --version`, `codex --version`, `~/.local/bin/cursor-agent --version`.
- Claude plan: non-secret `oauthAccount` and `cachedUsageUtilization` fields of `~/.claude.json`.
- Codex plan and limits: `rate_limits` events in `~/.codex/sessions/**/*.jsonl` (no tokens read).
- Cursor plan: `cursorAuth/stripeMembershipType` row in Cursor's `state.vscdb`; `cursor-agent status`.
- Devices: `xcrun xctrace list devices`.

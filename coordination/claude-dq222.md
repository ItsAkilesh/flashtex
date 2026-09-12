# claude-dq222 registration and handoff

- Updated UTC: 2026-09-12T08:10Z
- Agent / parent / machine alias: `claude-dq222` (Claude Code) / no parent, registers directly / `mac-m5pro-dq222`
- Task / acceptance gate / owned paths: **registration only.** Acceptance gate is the Commander recording this machine in `ROSTER.md` and deciding whether to open pools for it. Owned paths: `coordination/claude-dq222.md`, `docs/resources/machines/mac-m5pro-dq222.md`
- Branch / code revision / main integrated through: `agent/claude-dq222/register` / this commit / `2fd3026`
- State: **ready for integration** — registration published, toolchain installed and verified, awaiting Commander assignment
- Ready behavior and evidence: machine capability register published at [docs/resources/machines/mac-m5pro-dq222.md](../docs/resources/machines/mac-m5pro-dq222.md), every value read from this machine with the reading command recorded

## What is offered

The machine owner offers this computer's **Claude Max 20x** and **Cursor Pro**
access to the fleet, temporarily, so the Commander can allocate tasks to it.

The scope is routing, not credential sharing. Tasks assigned here execute **on
this machine** under its own local login. No token, API key, OAuth credential,
cookie or session identifier is shared, published, or transferred to another
computer, and none appears in this repository. A request to move credentials to
another machine is out of scope for this registration and would need the owner's
separate, explicit decision.

Access is temporary and the owner can withdraw it at any time.

## Funding isolation — read this before allocating

`hasExtraUsageEnabled` is **`false`** on this Claude account. There is no overage
route: when plan quota is exhausted, requests are refused rather than billed. No
auto-recharge, no credit balance. This is enforced provider-side.

That bounds the downside of allocating work here to "the plan stops until reset",
which is the isolation property `RESOURCES.md` asks for before Claude work is
unblocked. It is **not** permission to spend, and it is **not** authorization to
enable extra usage, purchase credits, or change the plan. Those remain prohibited.

Live quota is **unknown**. `/usage` is interactive-only, so no percentage can be
read non-interactively from this machine. Unknown is not zero. Do not size
allocations off an assumed remaining balance, and do not project task counts from
another machine's measured consumption.

- Resource pool / allocation ID / maximum: **none requested, none held.** Two candidate pools are offered for the Commander to register and bound: `claude-max20x-dq222` (Claude Max 20x, extra usage disabled, quota unread) and `cursor-pro-dq222` (Cursor Pro, quota not exposed by the CLI). Naming them here is a capability offer, not a grant. The Commander owns whether they become pools and what maximum each carries.
- Confirmed spend / estimated usage / in-flight reservation / remaining: zero project inference run on this machine to date / unknown / none / unknown
- Billing evidence / freshness / unknowns: plan tier fields read from `~/.claude.json` and `cursor-agent about` at 2026-09-12T07:35Z; live consumption for both plans unread and unreadable from the terminal

## Capability summary for allocation

**This machine is now a verified Rust builder.** Measured 2026-09-12 after
installing the toolchain and repairing the OS toolchain underneath it:

| Suite | Result |
|---|---|
| Rust crates building | 17 of 17 |
| Rust tests | 772 passed, 0 failed |
| Python coordination tests | 191 passed, 0 failed |
| `swift build` | succeeds |
| `swift test` | blocked, XCTest unavailable |

**Send here:** Rust crate work first — this is the fleet's largest Rust builder
at 18 cores and 48 GB, with all 17 crates green. Then Python coordination
tooling, documentation, protocol and integration review, LaTeX reference-oracle
validation via `pdflatex`, Node work, and commit execution.

**Installed or repaired on this machine, 2026-09-12:** Rust 1.98.1 with rustfmt,
clippy and rust-analyzer; Command Line Tools 26.6, replacing a May 2025 build;
BasicTeX for the `pdflatex` oracle; and an isolated `.venv` carrying the
`jsonschema==4.23.0` that `protocol/rendering-v2.requirements.txt` declares. The
Apple SDK licence was accepted and `xcode-select` was pointed back at the
Command Line Tools.

**Correction to the previous revision of this handoff.** It reported that Xcode
was absent here. That was wrong. Xcode 26.6 is installed, a full 3.5 GB with
every platform SDK. The error came from a shell glob that aborted on no-match
and a Spotlight query that returned nothing; neither was evidence of absence.

**Do not send Apple platform work here anyway — Xcode is installed but
unusable.** Pointing the developer directory at it breaks `cc`, `clang`, `git`
and `swift` with `Symbol not found: _XPCTypeBool`, because Xcode's bundled
CoreDevice framework expects a symbol this macOS 26.6.2 build does not export.
This was the machine's original fault: `xcode-select` was already pointing at
Xcode when this session began, which is why the whole toolchain was refusing
before anything was changed. The machine now runs on the Command Line Tools,
where everything above is green.

The practical limit is that `swift build` works but `swift test` does not, since
XCTest is not in the Command Line Tools SDK. There is no `xcodebuild`, no
simulator and no signing. Route FT-003, FT-004 and FT-008 elsewhere. Repair
needs an Apple ID and a multi-gigabyte download; do not simply re-point
`xcode-select`, which reproduces the breakage.

**Do not send here — no Grok/xAI credential.** No `XAI_*` or `GROK_*` key exists
on this machine.

**Not offered:** `codex-cli 0.146.0` is installed but its login, plan and funding
were not audited and the owner did not offer it. Do not infer a pool from it.

- Incomplete behavior / blockers / needs from others: needs a Commander assignment. The toolchain is installed and verified at 17/17 crates and 963 tests. The one remaining blocker is a broken Xcode, which needs an Apple ID and a large download; it does not affect Rust, Python or coordination work
- Interface changes / consumer actions: none
- Reviewed peer revisions / resulting adaptations: `main` at `2fd3026`; read `AGENTS.md`, `coordination/RESOURCES.md`, `coordination/ROSTER.md`, `docs/resources/machines/README.md` and `kabirs-macbook-pro.md` to match the register format and the commit-attribution rule
- Validation commands / results / artifact paths: `cargo build` and `cargo test` in each of `crates/*/` gives 17/17 building and 772 passing; each `tests/test_*.py` under `.venv/bin/python` gives 191 passing across 10 files; `swift build` in `apps/mac` succeeds and `swift test` fails on XCTest. Re-verification commands are listed at the end of the machine register
- Exact deadline UTC / remaining time / integration reserve: none set; registration is not time-boxed
- ETA remaining, optimistic / likely / pessimistic / confidence: n/a for registration
- Child tasks and their deducted allocations: none. This agent has no descendants and will not spawn any without a bounded grant naming them
## Commit attribution on this machine

Directed by the machine owner on 2026-09-12.

- **The owner is always the Git author** of everything committed here, using the
  configured identity `d-q222`. This mirrors the `mac-m1max-a` primary-author
  exception. Do not set the `Cursor <cursor@flashtex.invalid>` author on commits
  from this machine, even when Cursor executes them.
- **Cursor contribution attribution is enabled** at the owner's explicit request:
  `attributeCommitsToAgent` and `attributePRsToAgent` are both `true` locally, so
  Cursor adds its own co-author trailer to work it performs. Per `AGENTS.md` that
  trailer belongs on a commit only when Cursor CLI actually executed it, so it is
  absent from commits executed directly.
- **Other AI attribution remains barred** by the owner's standing rule: a local
  hook denies commits and PRs crediting Claude, Anthropic, Codex, ChatGPT,
  Copilot, Gemini or OpenAI. Cursor is exempt; human trailers are unaffected.

- Dirty files / unpushed work / running jobs: none
- Decisions / failed approaches / linked findings: account emails for both plans were deliberately withheld from this repository; the GitHub identity `d-q222` and its noreply address are used instead, per the register's rule against publishing account identifiers. `RESOURCES.md` and `ROSTER.md` were deliberately **not** edited — they are the resource owner's files and this agent only publishes capability evidence
- Exact next action or command: **wait for the Commander to assign a task.** This agent will not self-assign, will not start work from queue files on its own authority, and will surface any assignment to the machine owner before executing it
- Resume reading list: [docs/resources/machines/mac-m5pro-dq222.md](../docs/resources/machines/mac-m5pro-dq222.md), `coordination/RESOURCES.md`, `coordination/ROSTER.md`, `AGENTS.md`

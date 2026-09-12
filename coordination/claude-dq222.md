# claude-dq222 registration and handoff

- Updated UTC: 2026-09-12T07:35Z
- Agent / parent / machine alias: `claude-dq222` (Claude Code) / no parent, registers directly / `mac-m5pro-dq222`
- Task / acceptance gate / owned paths: **registration only.** Acceptance gate is the Commander recording this machine in `ROSTER.md` and deciding whether to open pools for it. Owned paths: `coordination/claude-dq222.md`, `docs/resources/machines/mac-m5pro-dq222.md`
- Branch / code revision / main integrated through: `agent/claude-dq222/register` / this commit / `2fd3026`
- State: **ready for integration** — registration published, awaiting Commander assignment
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

**Send here now:** documentation, coordination and integration review; Python
coordination tooling (`python3` 3.14.6, `uv` 0.12.3); protocol, schema and
contract review; Node work (`node` 25.9.0); commit execution via authenticated
Cursor CLI.

**Do not send here — no Rust toolchain.** `cargo` and `rustc` are not installed,
so none of the 17 crates can be built or tested here as the machine stands. This
outranks the hardware: 18 cores and 48 GB would make this the fleet's strongest
parallel `cargo` builder, and none of it is currently reachable. One `rustup`
install changes that; the owner has not yet authorized it.

**Do not send here — no full Xcode.** `xcode-select -p` is CommandLineTools only,
`xcodebuild` refuses to run, zero simulators available. No Swift app build, run
or signing, and no iPad/iPhone companion work. Route FT-003, FT-004 and FT-008
native work elsewhere.

**Do not send here — no Grok/xAI credential.** No `XAI_*` or `GROK_*` key exists
on this machine.

**Not offered:** `codex-cli 0.146.0` is installed but its login, plan and funding
were not audited and the owner did not offer it. Do not infer a pool from it.

- Incomplete behavior / blockers / needs from others: needs a Commander assignment. The two toolchain blockers above are owner decisions, not fleet blockers
- Interface changes / consumer actions: none
- Reviewed peer revisions / resulting adaptations: `main` at `2fd3026`; read `AGENTS.md`, `coordination/RESOURCES.md`, `coordination/ROSTER.md`, `docs/resources/machines/README.md` and `kabirs-macbook-pro.md` to match the register format and the commit-attribution rule
- Validation commands / results / artifact paths: every value in the machine register was read with the command recorded beside it; re-verification commands are listed at the end of that file
- Exact deadline UTC / remaining time / integration reserve: none set; registration is not time-boxed
- ETA remaining, optimistic / likely / pessimistic / confidence: n/a for registration
- Child tasks and their deducted allocations: none. This agent has no descendants and will not spawn any without a bounded grant naming them
- Dirty files / unpushed work / running jobs: none
- Decisions / failed approaches / linked findings: account emails for both plans were deliberately withheld from this repository; the GitHub identity `d-q222` and its noreply address are used instead, per the register's rule against publishing account identifiers. `RESOURCES.md` and `ROSTER.md` were deliberately **not** edited — they are the resource owner's files and this agent only publishes capability evidence
- Exact next action or command: **wait for the Commander to assign a task.** This agent will not self-assign, will not start work from queue files on its own authority, and will surface any assignment to the machine owner before executing it
- Resume reading list: [docs/resources/machines/mac-m5pro-dq222.md](../docs/resources/machines/mac-m5pro-dq222.md), `coordination/RESOURCES.md`, `coordination/ROSTER.md`, `AGENTS.md`

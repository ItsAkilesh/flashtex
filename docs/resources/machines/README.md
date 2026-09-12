# Machine capability register

One file per physical computer, named `<machine-alias>.md`. Each file records what
that machine can actually do: installed agent CLIs, which account each is logged
in as, the plan tier and quota state those tools report, local toolchains, and
build blockers. The producing agent owns its own machine file.

Scope boundary with [RESOURCES.md](../../../coordination/RESOURCES.md):

- `RESOURCES.md` is the authority on funding pools, permissions, and allocations.
  Only the designated resource owner changes it.
- These files are **capability evidence only**. A tool being installed, logged in,
  and quota-healthy is not permission to spend it. Read a permission from
  `RESOURCES.md`, never from this register.

Recording rules:

- Record only values actually read from the machine, with the command used.
- Timestamp every quota reading. Quota percentages are point-in-time and go stale.
- Unknown is not zero. If a figure could not be read, write what blocked reading it.
- Never commit API keys, tokens, account UUIDs, or session identifiers.
- Re-verify before relying on a file older than the current work session.

| Machine alias | File | Owner agent | Last verified |
|---|---|---|---|
| mac-m5pro-kabir | [kabirs-macbook-pro.md](kabirs-macbook-pro.md) | Claude (this machine) | 2026-09-12T03:40Z |
| mac-m5pro-dq222 | [mac-m5pro-dq222.md](mac-m5pro-dq222.md) | claude-dq222 (this machine) | 2026-09-12T08:20Z |

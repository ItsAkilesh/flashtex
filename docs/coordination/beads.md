# Beads coordination ledger (bd 1.2.2 / dolt 2.3.3)

This document covers how FlashTeX agents coordinate through Beads instead of GitHub issue comments.

- **What moves:** the ledger (tasks, claims, status, handoffs, heartbeats, authority) and agent-to-agent messages.
- **What stays on GitHub:** code integration (branches, PRs, review, merge) and product bugs.

Evidence comes from the 2026-09-13 trial on the private scratch repo `GoKubar/flashtex-beads-trial`:
- **Machine A:** mac-m5pro-kabir, macOS arm64.
- **Machine B:** Kabir's NixOS PC, x86_64.

**Labels:**
- **VERIFIED:** run and observed.
- **SOURCE:** read in the v1.2.2 source or docs.
- **BELIEVED:** not yet tested.

## 1. Pinned versions

| Tool | Version | Asset | sha256 |
|---|---|---|---|
| bd | 1.2.2 (`6c124203e`) | `beads_1.2.2_darwin_arm64.tar.gz` | `2aa1245c666419900d2d6993a05049e92c40e0e601d19579cca5b07a7bb8021d` |
| bd | 1.2.2 | `beads_1.2.2_linux_amd64.tar.gz` | `8140098a51d3b81d5548d1c5e6db1a2d9930e5d141efe2a4bff7d079c4d321e8` |
| bd | 1.2.2 | `beads_1.2.2_darwin_amd64.tar.gz` (untested) | `e192dcb60f0f48d9463cd3bcf425d41fc0a632080cf9a06153c97ce12368c4bb` |
| bd | 1.2.2 | `beads_1.2.2_linux_arm64.tar.gz` (untested) | `501f38a1070d4b9b3b6261a86a3c92c4a52366869021560430a4bb0036afd83a` |
| bd | release `checksums.txt` | | `25507c2d3ac43d17a1e7dc6ea25141b0354300cebf3f3f549b7ba3d266ee4f69` |
| dolt | 2.3.3 | `dolt-darwin-arm64.tar.gz` | `55c11d34df78d7583f1130a1adef7763340f2aade33e68ccb0046854134ad08b` |
| dolt | 2.3.3 | `dolt-linux-amd64.tar.gz` | `4acd730a4c53991996854a72fbb1add102b0a583bd07411320efb65037a43d9d` |
| dolt | 2.3.3 | `dolt-darwin-amd64.tar.gz` (untested) | `e33f4fa00032054e38da78b31314f8e93fa9eb950c587ac5f29ba5c6402b3f2a` |
| dolt | 2.3.3 | `dolt-linux-arm64.tar.gz` (untested) | `850a880aece6587cb9251ea0f07eb51fcc0a37450471fd89e03ac2fba1fdaed3` |

Dolt publishes no checksums file, so its hashes are the GitHub release asset digests.

**Rules:**
- Never install bd 1.2.0 or 1.2.1. Running 1.2.1 once migrates the DB to schema v65, which 1.2.2 refuses to open.
- No `1.3.0-rc`. Stay on 1.2.2 until 1.3.0 is stable **and** the owner approves.
- No Homebrew or npm, because they cannot hold a version.

**Install:** `scripts/beads/install-pinned.sh`
- It downloads, verifies the sha256, and installs into `~/.local/share/flashtex-beads/`. It is idempotent.
- VERIFIED on both trial machines; the re-run skips the download.
- **NixOS:** bd is a dynamically linked glibc binary, so it fails with `Could not start dynamically linked executable` (VERIFIED). dolt is static and runs as-is.
  - Without nix-ld, the installer writes a shim that runs the unmodified verified binary through nixpkgs' glibc loader, with a GC root (VERIFIED on NixOS 26.11).
  - The cleaner system fix is `programs.nix-ld.enable = true;` in the NixOS configuration (owner action).
  - nixpkgs `beads` is 1.0.3, so it is not usable for the pin (VERIFIED `nix eval nixpkgs#beads.version`).

**Always call `scripts/beads/bd`, never bd directly.** VERIFIED behaviours:

| Guard | Observed |
|---|---|
| Version mismatch | `flashtex-bd: bd version mismatch: 'bd version 1.2.1 (x)' (pinned 1.2.2). Refusing.` (exit 3) |
| `BD_SMART_GATE=0`, `BD_DISABLE_METRICS=1` | exported on every call; pinned `dolt` first on PATH (bd shells out to `dolt` for git remotes, SOURCE) |
| `bd dolt push --force` | always refused |
| `bd migrate` (not `--inspect`/`--dry-run`), `bd upgrade ack`, `bd doctor --fix`, `flatten`, `rename-prefix`, `admin`, `bd init` in flash-tex/flashtex, or `BD_ALLOW_REMOTE_MIGRATE` set | refused unless the machine (`FLASHTEX_MACHINE` or `~/.config/flashtex/machine`) equals the authority record's `machine` **and** `authority_state=active`. Refusals seen: `this machine is 'mac-m5pro-kabir', Commander machine is 'mac-m1max-a' (origin/main:coordination/authority.json)`; `authority_state='vacant' in beads:trial-authority (upgrades frozen while no Commander is active)` |

- **Authority source for the wrapper:** the Beads bead `ft-authority` (metadata `machine`, `authority_state`), falling back to `coordination/authority.json` on origin/main.

## 2. Metrics off (owner decision 2)

- bd 1.2.2 enables anonymous metrics by default. A fresh HOME prints `Anonymous usage metrics: ON` and writes `metrics: {disabled: false}` (VERIFIED).
- **Two mechanisms, both used:**
  1. `bd metrics off` writes `metrics.disabled: true` to the user-global `~/.config/bd/config.yaml` (VERIFIED on macOS and NixOS; SOURCE `cmd/bd/metrics.go`). `install-pinned.sh` runs it.
  2. The env var `BD_DISABLE_METRICS=1` overrides config. `bd metrics` then reports `OFF` with `Note: BD_DISABLE_METRICS=1 is overriding your saved config (which is "on")` (VERIFIED). The wrapper and NixOS shim always export it.
- The shim also needs metrics off for correctness: the metrics flusher re-executes `os.Executable()` (SOURCE `internal/metrics/spawn.go`).

## 3. Schema migrations and upgrades (owner decision 3)

**The gate:**
- Every machine runs `BD_SMART_GATE=0`. bd's default "smart" gate may auto-migrate as a "safe first-mover" when the remote is at the same schema (SOURCE `internal/storage/schema/smart_remote_migrate_gate.go`, convergence floor v43).
- **VERIFIED:** a remote-backed DB created by bd 1.0.4 (schema v32), opened with 1.2.2 and `BD_SMART_GATE=0`:
  - Writes and `bd dolt pull` exit 1 with `refusing to auto-apply 21 pending schema migrations to a remote-backed database (v32 -> v53): migrating clones independently forks the schema (#4259)`.
  - Reads continue with `Read-only command: continuing on schema v32 without migrating. Writes are blocked`.
- With the gate unset, the same v32 DB also blocked, because it is below the v43 floor. The smart-gate auto-migrate at ≥v43 is SOURCE/BELIEVED only.
- **Only the Commander machine migrates.** Upgrades are frozen while `authority_state` is not `active`. Migration rights move with a completed Commander claim (§8).

**Upgrade procedure** (after owner approval of a new pin; SOURCE `getting-started/upgrading.md` v1.2.2):
1. Commander records `upgrade_state=planned` on `ft-authority` and announces a freeze bead.
2. Every machine, **with the old binary**: `scripts/beads/bd dolt pull && scripts/beads/bd dolt push`, then stops writing. Pushes and pulls are refused once a newer binary with pending migrations is installed.
3. Commander machine only:
   - `bd export --all -o <backup>.jsonl` and `bd backup`
   - install the new pin (update `install-pinned.sh` hashes in a PR first)
   - `BD_ALLOW_REMOTE_MIGRATE=1 scripts/beads/bd migrate`
   - `scripts/beads/bd dolt push`
4. Every other machine: run the updated `install-pinned.sh`, move `.beads/embeddeddolt` aside, then `scripts/beads/bd bootstrap --yes`. Never `bd migrate` locally: two independent migrations fork the schema irrecoverably (#4259).
5. Commander sets `upgrade_state=frozen` and updates the pins in this file.

## 4. Storage mode: embedded, one DB per machine, one sync loop per machine

Measured on the Mac with 8 and 16 concurrent `bd` processes (create / list / `ready --claim`):

| Mode | Writers × ops | Wall | Succeeded | Failures |
|---|---|---|---|---|
| embedded | 8 × 20 | 53 s | 160/160 | none; 40 claims, no duplicate IDs |
| embedded | 16 × 10 | 52 s | 160/160 | none |
| server (`bd init --server`, auto-started per-project `dolt sql-server`) | 8 × 20 | 11 s | creates all OK | 9/40 claims returned `Error 1213 (40001): serialization failure` |
| server | 16 × 10 | 10 s | 134/160 | 26/53 claims exit 1 with `serialization failure` |

**Decision: embedded mode.**
- It serialises through a file lock, was about 5× slower, and produced zero failures.
- Server mode drops concurrent claims with exit 1, which would need a retry wrapper around every write, plus a server lifecycle on every machine.
- Revisit only if lock latency becomes the bottleneck.

**Consequences:**
- All git worktrees of a clone share the main clone's `.beads/embeddeddolt`: `bd where` from a worktree prints the main clone's DB, and a bead created in a worktree is visible in main (VERIFIED). So a 15–20-agent machine has **one** ledger DB.
- The embedded lock is held for the whole network sync. A local `bd create` issued during a 7.6 s `bd dolt push` took 9.18 s (VERIFIED).
  - Therefore agents do **not** each run pull/push loops. One machine-level sync loop (supervisor script, no model call) runs `bd dolt pull` every 60 s and pushes after local writes.
  - Agents push immediately only after claim/close (§6).
- Expect about 3 local ops/s of throughput with contention. That is ample for task-boundary use, not for per-token logging.

## 5. Sync convention (staleness is explicit)

- **Transport:** the ledger lives on the code repo's remote under `refs/dolt/data` (VERIFIED via `git ls-remote`).
  - The first `bd dolt push` also creates a visible **branch** `refs/heads/__dolt_remote_info__` containing `DOLT_REMOTE.md` (VERIFIED). Do not delete it; exclude it from branch-cleanup scripts.
  - Plain PR flow is unaffected: branch, push, `gh pr create`, `gh pr merge --squash --delete-branch` all worked with both refs present, and the refs survived (VERIFIED, trial PR #1).
- **Fresh clone:** `scripts/beads/bd bootstrap --yes`, which clones `refs/dolt/data` (VERIFIED). Then `git config beads.role maintainer`, because bd warns `beads.role not configured (GH#2950)`, and `chmod 700 .beads`, because bd warns about 0755.
- **Read at task boundaries:** before choosing work, before claiming, before closing, and at each checkpoint. Run `bd dolt pull` first, or rely on the machine loop's last pull. Any decision must state the pull time it relied on: "ledger as of <UTC of last successful pull>".
- **Push:** immediately after claim, close, handoff or message; otherwise the machine loop pushes. Never `--force`.
- **Rejected push** (`! [rejected] main -> main (non-fast-forward)`, VERIFIED): pull, then push again. Non-conflicting concurrent work merges cleanly: two machines each created beads offline, the loser pulled, and both beads were present everywhere with distinct hash IDs (VERIFIED).

## 6. Claim rule

**A claim counts only after `bd update <id> --claim` has been followed by a successful `bd dolt push` and a re-read shows you as assignee.**

**What actually happens on a collision** (VERIFIED, trial bead `trial-q7g`):
1. A and B both run `bd update trial-q7g --claim` offline. Both print `✓ Updated issue`.
2. A pushes first: `Push complete.`
3. B pushes: `! [rejected] main -> main (non-fast-forward)` (exit 1).
4. B pulls: `Error: merge origin/main: merge conflicts in issues require operator resolution; merge aborted and working set restored` (exit 1).
   - B's local copy still says `assignee: agent-B`.
   - Every further push from B is rejected, so **B's ledger is wedged**.
5. A and the remote say `assignee: agent-A`. First successful pusher wins.
6. `bd doctor` (the documented conflict fix) prints `'bd doctor' is not yet supported in embedded mode`.
7. **Recovery that worked:** move `.beads/embeddeddolt` aside, run `bd bootstrap --yes`. B then showed `assignee: agent-A` and pushed a new bead successfully.
   - This discards B's unpushed ledger writes, so keep them minimal.

**Protocol derived from this:**
- `pull` → `show` (still open/unassigned?) → `--claim` → `push`, with **no other unpushed ledger writes** in between.
- If the push is rejected: pull. If the pull succeeds, re-read. If you are not the assignee, yield.
- If the pull reports `merge conflicts … require operator resolution`, your claim lost:
  1. Keep code on your git branch.
  2. Copy any unpushed notes into the branch handoff file.
  3. Move `.beads/embeddeddolt` aside, run `bd bootstrap --yes`, and re-read.
  4. Notify the machine's other agents: a re-bootstrap affects the whole machine's shared DB, so the machine sync loop should do it.
- Never resolve by `--force` push. `bd vc merge --strategy theirs` exists for branch merges but was not tested for the remote case.

## 7. Messaging convention

- **`-t message` does not work cross-machine.** In 1.2.2 `message` is a built-in infrastructure type that bd routes to the local `wisps` table, which is dolt-ignored.
  - `bd create -t message …` returned `trial-wisp-bde`, and B then saw an empty inbox and `no issue found` (VERIFIED; SOURCE `types.infra`).
  - `bd promote <wisp>` makes it permanent: B could then `bd show` it, but `bd list -t message` on B stayed empty. Do not rely on this path.
  - `bd mail` only delegates to an external provider (`mail.delegate`), and none is configured.
- **Convention (VERIFIED end-to-end A→B):**
  - **Send:** `bd create "<subject>" -t task --assignee <recipient> -l msg,to:<recipient>,from:<sender> --description "<body>"`, then `bd dolt push`.
  - **Receive:** `bd dolt pull`, then `bd list -l to:<me> --status open --json` (493 bytes for one message).
  - **Read:** `bd show <id>` (293–360 bytes).
  - **Acknowledge:** `bd close <id> -r read`, then push.
  - **Reply:** create a message and `bd dep add <reply> <original> --type replies-to`. The dep type was accepted locally (VERIFIED); a cross-machine thread was not tested.
  - **Broadcast:** `-l msg,to:all`. Machine-wide: `-l msg,to:machine:<alias>`.
  - Messages are pull-based, so latency is the sync-loop interval (≤60 s) plus push/pull (about 6–8 s each, measured).
- **Other sync facts (VERIFIED):** `bd kv set/get` values, `--set-metadata` fields and `bd config set types.custom` all reach the other machine after push/pull.
- **Output sizes** (trial, 9–11 beads):
  - `bd ready` 443 B; `bd ready --json` 2273–2763 B
  - `bd show <id>` 360 B; `--json` 543 B
  - `bd list --json` 3135 B; `bd prime` 4694 B

  Compare about 437 KB for one read of GitHub issue #2.

## 8. Low-usage signalling and reallocation

**Agent, the moment its tool reports a usage warning, 429/limit error, or low reading:**
1. Push code to the task branch.
2. `bd update <task> --append-notes "handoff: branch=<b> sha=<sha> tests=<…> next=<…>"`
3. `bd update ft-agent-<id> --set-metadata capacity=low --set-metadata usage_verbatim="<exact tool text>" --set-metadata observed_utc=<utc>`. Use `capacity=exhausted` if no more calls are possible.
4. If it cannot finish: `bd assign <task> ""` and `bd update <task> --status open --add-label handoff-ready`.
5. `bd dolt push`.

Report the exact tool text only. Never estimate or sum unlike resources.

**Commander (scripted poll, no model call):** `bd list -l handoff-ready --status open --json` plus agent beads whose metadata shows `capacity=low`. Grants still come from `coordination/RESOURCES.md`; Beads confers no spend permission. Two ways to reassign:
- `bd assign <task> <eligible-agent>`, then a message (§7), then push.
- Leave it unassigned with capability labels for `bd ready --claim --label cap:<x> --unassigned`.

## 9. Commander authority and failover

**Decision 5:** after cutover the authority record is the bead `ft-authority` (type `decision`). `coordination/authority.json` is a read-only mirror the Commander regenerates until retired.

**Metadata:**
- `commander_id`, `machine`, `authority_state` (active | handoff | vacant)
- `epoch`, `claim_mode`, `designated_successor`, `updated_utc`, `heartbeat_utc`
- `bd_pin`, `dolt_pin`, `upgrade_state` (frozen | planned | in_progress)

**Heartbeat:** the Commander sets `heartbeat_utc` every ≤5 min, then pushes. A stale heartbeat is **only a prompt to investigate**, never grounds to claim (AGENTS.md rule, unchanged).

**Claim protocol:**
1. Run `bd dolt pull` and `git fetch`, then read `ft-authority`.
2. Eligibility is unchanged. One of:
   - an explicit quiesced handoff naming you;
   - independently verified termination of the exact session plus stopped publishers;
   - a direct user instruction naming you.
3. `bd update ft-authority --set-metadata epoch=<n+1> --set-metadata commander_id=<me> --set-metadata machine=<alias> --set-metadata authority_state=active`, then `bd dolt push` (never `--force`).
4. If the push is rejected, or the pull conflicts (§6), another claimant won: re-bootstrap, re-read, stand down.
5. The claim is complete only when the push succeeded **and** a fresh pull shows your epoch. Only then does the wrapper grant migration rights on your machine: it reads `machine` and `authority_state`.
6. A resumed old Commander re-reads, sees the higher epoch, and stays quiesced.

## 10. Verified vs believed

| Claim | Status |
|---|---|
| Identical `bd version 1.2.2` / `dolt version 2.3.3` on macOS arm64 and NixOS x86_64, sha256-verified | VERIFIED |
| Create on A → push → B `bootstrap` + `show` | VERIFIED |
| Claim on B → push → A pull shows `in_progress`/`agent-B`; `bd ready` on A no longer offers it | VERIFIED |
| Blocked bead absent from `bd ready`, appears after blocker closed | VERIFIED |
| Offline creates on both machines merge with no ID collision or loss | VERIFIED |
| Claim collision: first pusher wins; loser's pull aborts (exit 1) and ledger is wedged until re-bootstrap | VERIFIED |
| Embedded mode: 16 concurrent writers, 0 failures; server mode drops concurrent claims (40001) | VERIFIED (Mac) |
| Worktrees share one embedded DB | VERIFIED |
| `-t message` is local-only (wisp); task+labels messages sync | VERIFIED |
| kv, metadata, `types.custom` config sync | VERIFIED |
| `BD_SMART_GATE=0` blocks writes/pull on pending remote-backed migration | VERIFIED (v32→v53) |
| Smart gate auto-migrates at ≥v43 when unset | SOURCE / BELIEVED |
| Metrics off via config + env | VERIFIED |
| PR flow unaffected by `refs/dolt/data` and `__dolt_remote_info__` | VERIFIED |
| Wrapper refusals (version, force, migrate, vacant authority) | VERIFIED |
| Sync on Daniel's / Jaysen's machines | BELIEVED (runbook) |
| Behaviour with 15–20 real agents over hours, and dolt storage growth in the repo | BELIEVED |
| `bd init` git hooks (`core.hooksPath=.beads/hooks`) are harmless for FlashTeX | NOT VERIFIED → use `--skip-hooks` |

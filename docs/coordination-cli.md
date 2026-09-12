# Executable coordination protocol v1

Implementation: `scripts/coord.py`, Python 3.9+ standard library on macOS/Linux.
No API keys or model calls are needed except the explicit `publish` command.

## Join

Choose a unique ID, fetch main, and create `agent/<id>/register` from current main
in a clean clone/worktree. Then:

```sh
python3 scripts/coord.py register --id worker-a --machine mac-a --tool Codex --capability swift --capability xcode
git add coordination/agents/worker-a.json
python3 scripts/coord.py publish --allocation YOUR_AUTHORIZED_CURSOR_GRANT --implementation Codex -m 'coordination: register worker-a'
```

Use real IDs/grants. `register` writes only your JSON report and does not switch
branches, stage files, spend money, or publish. Only your own `agent/<id>/...`
branch may carry updates. Markdown registrations are discovered as legacy reports
for manual review; they are not silently converted to active structured workers.

## Check, read, adapt, acknowledge

```sh
python3 scripts/coord.py checkpoint
```

This fetches and outputs JSON containing new branch tips, changed file paths,
structured/legacy reports, stale/duplicate warnings, and authoritative assignments
from `origin/main`. Local state lives under `git rev-parse --git-path flashtex`;
it works in linked worktrees. The command never merges or modifies tracked files.
Observed commits are NOT marked reviewed. A failed fetch preserves the prior
snapshot with `fetch_ok: false`; do not rely on that snapshot as current.

Read the actual task and relevant diffs. On the branch named in the assignment,
carry your registration forward and acknowledge the exact published revision:

```sh
python3 scripts/coord.py ack --id worker-a --task FT-003 --revision 1 --adaptation 'Accepted the Swift shell task; shared messages come from protocol v1.'
python3 scripts/coord.py report --id worker-a --state in_progress --summary 'Source editor builds; preview pending' --next 'Connect the compile-result fixture' --eta 10 20 35 --usage unknown --evidence 'Subscription dashboard not available'
```

Use `report --review origin/main=FULL_SHA --adaptation '...'` to record a real
review and response. Publish the updated report with a coherent work checkpoint.
An assignment acknowledgement is tied to a revision and the source main commit;
new dispatch revisions require new acknowledgement. No automatic ACKs exist.

## Commander dispatch

On a Commander branch containing current main:

```sh
python3 scripts/coord.py dispatch --task FT-003 --agent worker-a --branch agent/worker-a/mac-shell --objective 'Build a native source/preview shell' --path apps/mac --acceptance 'Build with Xcode and render the versioned fixture' --minutes 45 --allocation APPROVED_GRANT
```

Dispatch writes `coordination/assignments/<task>.json`; it is provisional until
Cursor commits it and Commander publishes it to main. Active path overlaps and
silent transfer of active ownership are rejected. Commander verifies registrations,
funding, and capability before issuing the command. A grant string is an audit
reference, not a billing enforcement mechanism.

## Publication

Stage ONLY intended changes, resolve/preserve unstaged files, and call `publish`.
It actually invokes Cursor CLI, then requires exactly one new commit on the same
branch with the original staged tree, Cursor author/committer, truthful trailers,
and a clean working tree. Only then does it non-force push that task branch.
It never pushes main. Commander reviews/integrates the candidate separately.

On a timeout, rejected push, or validation failure, inspect actual Git state and
the remote before retrying. Cursor may have committed even when its process failed.
Do not rerun a paid session or overwrite history blindly.

### If a worker cannot use Cursor

Do not make another non-Cursor commit. Prepare a `git diff --binary` patch including
intended new files (use the index to include them), validation results, and a
handoff. Submit it as a GitHub issue under this repository so Commander can apply
it in an isolated worktree, review it, and have Cursor commit it on this machine.
Use a structured issue body / `gh issue create --body-file` to preserve literals.
Include task ID, base SHA, exact paths, and `Implementation-Agent`. Do not include
secrets, private captures, binaries exceeding issue limits, or unrelated changes.
If the patch exceeds issue limits, ask for an approved artifact-transfer route.
This is a manual commit-broker route, not an automatic patch execution service.
Commander treats submitted patches as untrusted changes and inspects them first.

## Background discovery

```sh
python3 scripts/coord.py watch --interval 60
```

The watcher fetches once per minute, updates local snapshots, and exits at the
fixed deadline. It never calls models, creates commits, pushes, applies changes,
or marks anything reviewed. Run under a session/service manager for persistence.
It discovers work; an active agent or human still performs review/dispatch.

## Validation

```sh
python3 -m unittest discover -s tests -v
```

Tests use temporary bare remotes/clones/worktrees and synthetic commits. Cursor is
replaced by a test double ONLY inside those isolated fixtures; no model calls or
real repository commits occur during testing. Production publication invokes the
real Cursor executable and verifies its output.

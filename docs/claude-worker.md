# Local Opus worker

Owner: Commander, FT-016. September 12, 2026. Implemented deterministic supervisor;
unit tests use synthetic tools only. Actual local Claude execution is blocked until an authorized API key, successful
API authentication and available credits with a bounded grant are verified.
The user's latest instruction prohibits subscription and extra-usage billing on
this Linux computer. This does not change Jaysen's M1 Max subscription grant.
See recovery issue #6 and the authoritative per-worker assignments.

Each local Opus worker has a separate worktree, assignment, queue, report and
non-overlapping API allocation. Sol reads all reports before dispatch. Two workers
are the initial target; neither is counted active before exact ACK/session evidence.

## Startup and continuation

After fetching integrated main, create a dedicated clean checkout on the assigned
`agent/local-claude-opus/e2e-harness` branch. After the resource owner verifies a private API grant and supplies its key:

```sh
python3 scripts/claude_worker.py --id local-claude-opus --machine linux-primary --api-grant /private/verified-grant.json
```

On Linux a transient user service can keep the process alive across terminal closure
(run from that checkout; it does not log in or create credentials):

```sh
systemd-run --user --unit=flashtex-claude-opus --collect \
  --working-directory="$PWD" \
  --setenv=PATH="$PATH" \
  /usr/bin/python3 scripts/claude_worker.py --id local-claude-opus --machine linux-primary --api-grant /private/verified-grant.json
journalctl --user -u flashtex-claude-opus -f
systemctl --user stop flashtex-claude-opus
```

Pass explicit product verification commands through repeated `--check '["argv",...]'`
options where appropriate. The default Python infrastructure checks do not establish
product correctness. Every integration must additionally satisfy its assigned gates.

The supervisor polls Git every 30 seconds without model inference. It checks shared
stop control, authoritative assignment and next-pointer agreement; synchronizes clean
branches through the guarded Cursor helper; acknowledges exact revisions; launches
`claude -p --model opus`; publishes reports and code through actual Cursor. Completed
assignments wait for the next revision; an individual completion never stops the
project. One lock prevents duplicate supervisors in the same checkout.

Calls default to five-minute checkpoints, not a project stop time. The worker reads
shared instructions and relevant peer changes each cycle and reports exact reviewed
SHAs. No paid polling or repeated empty task prompts. Missing API credentials or grants produce recovery issues without inference.
After credentials and the allocation are verified, restart the supervisor. No launch success is claimed without actual PID,
assignment ACK and successful CLI evidence.

The installed CLI's `dontAsk` and `--permission-prompts none` prevent unattended
permission waits. Explicit file/command tools support authorized development;
safe mode disables custom hooks/plugins/settings. This is tool configuration, not
an OS security sandbox. Platform refusals remain refusals and become recovery work.
The agent does not commit, manipulate billing, invoke another paid tool, or start
detached work; the supervisor owns publication. Nested paid agents require separate
resource allocation rather than inventing extra quota.

## Usage and credits

Published `coordination/agents/local-claude-opus.json` includes CLI token counts,
client-estimated equivalent USD when supplied, PID/cycle/assignment, tests, ETA and
last completed timestamp. Equivalent USD is not an actual subscription charge or
remaining credit balance. Remaining plan quota stays unknown unless separately
verified and reported. Raw logs stay in Git-local coordination state, never commits.

All launches use `--bare` to exclude subscription OAuth. Provider and OAuth
overrides are rejected. No subscription login can unblock this worker. No API key
or credit balance has been verified here. Do not purchase credits or enable overages.

An API grant contains no secret and must name the exact worker, assignment allocation
and absolute worktree, plus the verified key’s `credential_sha256`; `verified_api_auth:true`, `verified_prepaid_credits:true`,
`provider_spend_cap_verified:true`, a future `expires_utc`, and positive
`max_total_usd`/`max_call_usd`. These flags record actual checks by the resource owner;
they are not proof created by this script. The approved key is supplied privately as
`ANTHROPIC_API_KEY`. Other provider/OAuth overrides are rejected. The API command
uses `--bare` to exclude subscription OAuth and a per-call `--max-budget-usd`.

The full call cap is reserved durably before inference and remains reserved even
when a client estimate is lower or the call is interrupted. Only audited provider
reconciliation can release it. Do not reset state or reuse the grant on another
checkout. Local bookkeeping is not a cross-machine transactional billing service;
verified provider controls are required for the stated credit cap.

## Recovery

Inspect Git-local `claude-worker.json`, `claude-cycle-N.json`, running process tree,
and task branch before retry. A `model_running`, `model_failed` or
`publication_started` phase blocks another inference call. It may represent paid
work already completed or committed. Recover that work/publication first; never
clear the phase just to make the error disappear. Report an unresolved blocker as
a GitHub issue and assign a resolver. Distinct failures get fresh notifications;
repeated identical failures are deduplicated. Cursor publication failures do not
trigger a new Claude call.

Stop the user service before manual reconciliation. Preserve dirty files and logs;
verify no model process remains. If the model completed, validate its result and
publish it through Cursor rather than rerunning it. After reconciling calls,
reservations and publication, the orchestrator may restore an idle phase and restart
the service. Never infer process death solely from a stale Git heartbeat.

References: [Claude headless CLI](https://code.claude.com/docs/en/headless),
[CLI reference](https://code.claude.com/docs/en/cli-reference). Flags were also checked
against the installed CLI's `--help`; tests do not replace a live authenticated
preflight.

Blocked-result issue delivery is journaled before completion. Restart retries pending
issue delivery without repeating inference. API reservations are fsynced before launch.

# Autonomous worker operation

The user authorized autonomous work until an explicit stop or verified whole-project
completion, including extra features. The old 10am stop is superseded. Each machine
must launch its worker once; Git cannot start a process on a machine with no running
supervisor. Registration and a published assignment do not prove a worker is running.

## Initial machine assignments

| Agent ID | Machine | Task / branch | Authorized implementation route |
|---|---|---|---|
| claude | mac-m5pro-kabir | FT-002 / agent/claude/compiler-foundation | Existing Codex ChatGPT Plus; original Rust compiler |
| mac-claude-a | mac-m1max-a | FT-003 / agent/mac-claude-a/mac-shell | Existing Codex ChatGPT Plus; native Mac app |
| aarush-macbook | aarush-macbook | FT-004 / agent/aarush-macbook/companion-capture | User-provided OpenAI access after local login; iPad capture |

The `claude` ID is retained for continuity; it does not authorize protected Claude
inference. All new commits are executed by actual Cursor CLI and coauthor the
GitHub user authenticated on the executing machine.

## One-time local startup

An agent on each machine performs these checks under the user's existing authorization:

1. Fetch current main. Read AGENTS.md, this file, and its exact assignment.
2. Preserve existing work; use a dedicated checkout/worktree on the assigned branch
   based on current main. Never reset or discard an existing branch to get clean.
3. Check `codex login status`, `cursor-agent status`, and `gh auth status`.
   Confirm the eligible subscription account. No secrets or auth files go into Git.
4. Verify Xcode/build tools and any already-authorized macOS app permissions needed
   for that task. Authentication, camera/local-network OS prompts, and signing
   prerequisites must be reported while a person is available. Their presence
   cannot be inferred or granted by the Commander from another computer.
5. Start the worker in that dedicated task checkout, substituting its real ID:

```sh
python3 scripts/worker.py --id claude --machine mac-m5pro-kabir --capability rust
```

Keep this process under a local session manager such as an existing tmux session
or a per-user service. Do not create a global always-awake setting. Publish actual
PID/session, startup result, tool/auth capability (no secrets), and blockers in the
assignment issue. The first successful worker checkpoint publishes a structured ACK.

The supervisor uses supported `codex exec` noninteractive mode with workspace-write
and approval policy `never`; denied operations fail rather than waiting forever.
It does not bypass platform security. Saved CLI authentication is reused; this
runner requires the authorized ChatGPT route and rejects environment API overrides.
See [official noninteractive documentation](https://learn.chatgpt.com/docs/non-interactive-mode).

## Development loop

- Poll Git every 30 seconds while awaiting work. Fetching consumes no model call.
- Read the active assignment and `coordination/next/<id>.json`. Commander maintains
  the next-task pointer and `coordination/queues/<id>.json`; workers own their report.
- At clean checkpoints, synchronize main through `integrate.py sync`. Fast-forwards
  need no new commit; divergent merges use actual Cursor and validation.
- Run a bounded Codex implementation cycle. Each fresh cycle reads durable handoffs,
  current instructions/contracts, and relevant peer revisions instead of depending
  on a large chat history. Return structured progress, tests, ETA and next action.
- Verify branch/HEAD and changed-file scope. Have Cursor commit and push the checkpoint,
  including authenticated GitHub coauthor and truthful implementation provenance.
- Continue immediately while in progress. When ready, publish `ready_for_integration`
  and wait for a changed assignment revision. Worker completion is not global success.
- Commander archives the completion SHA/evidence and advances its pre-reviewed queue
  by writing the next assignment and next-task file. Queue advancement does not claim
  the previous change has passed integration; Commander still reviews and integrates.
- Add appropriate product build checks with repeated `--check` JSON argv arguments.
  The default coordination tests cover infrastructure only, not native app correctness.

## Commander continuity and recovery issues

Only one Commander may write global control state or promote main. Planned transfer
uses an explicit handoff that names one successor, records the pinned main SHA and
all running publication jobs, and stops those jobs before the successor claims
authority. Unplanned revival requires positive local evidence that the exact former
Commander process/session has terminated and that dispatch, Cursor publication,
integration, and promotion jobs are stopped. Missed heartbeats, silence, Git fetch
failure, network partitions, stale reports, and model quota errors do not satisfy
this test. If termination cannot be verified, workers continue their existing
non-overlapping assignments and open a recovery issue; they do not elect another
Commander.

The one chosen successor fetches current main, creates its authority claim from that
exact SHA, publishes through actual Cursor, and pushes main non-force. If main moved,
the claim is abandoned and rebuilt from the new tip. Before each later global write,
the leader fetches and rereads the current authority record. A resumed former leader
must do the same and must not write until a new explicit handoff names it.

For any implementation, tool, permission, quota, build, or publication blocker,
open a repository issue titled `[recovery] AGENT TASK-rN: short cause`. Include:

- exact task revision, branch, SHA, owned paths, and last integrated main;
- exact failing command and concise non-secret output;
- process/PID/session state and whether a model or Cursor call may still be in flight;
- latest provider/quota evidence, allocation ID, and prohibited fallbacks;
- attempted bounded remedies and the smallest reproducible acceptance check.

At every Commander checkpoint and before dispatch, fetch all reports and reread the
latest resource state for every registered computer. List open recovery issues,
classify ownership and dependency impact, then assign one or more eligible resolvers
on isolated branches without duplicating active path ownership or uncertain spend.
The resolver publishes exact evidence; the Commander reproduces the fix or obtains
required native-machine verification before closing the issue. Keep the issue open
when only a comment, proposed patch, stale report, or remote task row exists.

## Quota, failures, and permissions

Normal startup does not ask for approval again for authorized project work. Each
model call and Cursor commit has a timeout. Unexpected permission/auth failures
produce a recovery issue using the required template above, not an indefinitely
pending interactive question.

The worker persists phase/cycle state before inference and publication. A timeout
or failure may leave useful work or a completed commit. Inspect its Git state and
local `.git/.../flashtex/worker.json` before repair/resumption; never automatically
repeat an unresolved paid call. The worker reports a concise recovery issue without
uploading raw model logs. Commander assigns recovery or uses another eligible worker.

Ongoing OpenAI subscription work is authorized. Task timeboxes are reporting/review
intervals, not project stop conditions. `--max-cycles N` can impose an explicit local
per-assignment cap; `--bounded` retains a limited-run mode for controlled checks.

Do not automatically purchase credits, enable overages, consume protected Claude
included allowance, or activate a reset based only on an old Markdown claim.
Currently no separately verified API fallback is configured; Kabir's three resets
are worker-reported but have no verified callable reset mechanism. Eligible routes
must be confirmed and recorded before automation uses them. An exhausted account
must report its quota/reset evidence so Commander can reallocate work; other eligible
computers continue. The runner does not promise a provider-independent billing cap.

## Stop and completion

`coordination/control.json` is authoritative. `running` continues; `user_stopped`
and `verified_complete` stop workers at their next boundary. Only Commander changes
this, using the user's instruction or whole-project acceptance evidence. A finished
queue is a request for Commander planning, not completion of the product.

Local Ctrl-C/service stop remains available. Sleeps and waiting polls do not mean
an implementation task is active; check the actual process and published reports.

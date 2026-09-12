# Background conversion jobs

Owner: bridge_context, delegated by orchestrator-astra. Independent Rust product
scheduler; no provider request, bridge process adapter or native UI integration is
claimed. Generic `Send` closures perform conversion on a fixed-size thread pool.

`Scheduler<T>::new(workers, max_queued, max_retained)` bounds workers (1–64),
pending tasks and all retained job identities/results. `submit(id, fingerprint,
closure)` accepts a unique capture-attempt identity and queues it. The closure
receives a cooperative cancellation token and returns `Result<T, Failure>`.
`state` reports queued, running, completed, failed or cancelled. Same-ID submissions
are rejected while retained, including terminal failures: there is no automatic
retry, particularly after a provider timeout with ambiguous completion/billing.

`ContextFingerprint` carries a revision and lowercase SHA-256. The integrating
adapter must hash its exact source/dependency snapshot set; this crate validates
the digest format, not its correspondence to source. `update_context` rejects a
stale queued call before provider execution, rejects stale running results when
they finish, and revokes a previously completed result after context changes.
The adapter must serialize document updates, fingerprint updates and consuming a
result in the same document transaction; do not insert an earlier result clone
after its context changes. Continue to use bridge proposal-review/prepare checks.

`cancel` immediately marks queued/running jobs cancelled and signals the token.
Queued calls never execute. Running jobs keep their physical worker slot and
identity until the closure actually returns; late results are discarded. The
closure must enforce provider/network deadlines: cancellation cannot kill a Rust
thread or guarantee the provider did not charge. Dropping the scheduler requests
cancellation and stops new dispatch without blocking editor shutdown; a running
worker exits when its closure returns. Provider panics become a failed state with
no retry; Rust's process panic hook may separately report the panic on stderr.

`forget` explicitly retires a terminal record; it refuses still-executing cancelled
calls. Retirement permits identity reuse, so the bridge's durable capture journal
must retain cross-session/idempotency history. Prefer a new explicit attempt ID
when the user chooses to retry. The scheduler itself is memory-only; process
recovery must classify interrupted running calls as ambiguous and reconcile the
journal before issuing another paid request. Result/image byte limits belong to
the adapter; count limits here cannot bound arbitrary generic payload sizes.

Verification: `cargo test --manifest-path crates/conversion-jobs/Cargo.toml` and
`cargo clippy --manifest-path crates/conversion-jobs/Cargo.toml --all-targets -- -D warnings`.
Eight deterministic channel-controlled tests exercise duplicate rejection, bounded
queue/retention, cancellation, stale revisions/hashes, concurrent workers, panic
recovery and nonblocking shutdown. No Grok or other provider calls occur.

## Durable recovery

`save_snapshot(path, Limits, encode)` captures coherent metadata, then invokes the
caller codec outside the scheduler lock. It atomically replaces and fsyncs a file
in an existing caller-owned private directory. Individual result and aggregate
JSON byte limits are checked before publication. Existing checkpoints survive a
codec/size failure. The caller serializes writers and validates allocation limits
inside its codec; snapshot generation cannot bound arbitrary codec allocations.

`restore_snapshot(..., decode)` checks version, identities, digests and limits.
Restoration never schedules closures. Former queued jobs become
`RecoveryRequired(QueuedAwaitingAuthorization)`; running, cancelled-but-executing,
ambiguous-failure and panicked calls become `ProviderMayHaveCompleted`.
`authorize_resume` requires an explicit caller decision and new closure; queued-only
authorization cannot restart a possibly completed call. Confirm the previous
process is terminal and reconcile provider/journal evidence first. A snapshot is
not a distributed lease or exactly-once billing guarantee. This metadata checkpoint
must be coordinated with the capture journal by the eventual bridge adapter.

Twelve tests now pass, including atomic old-file preservation, bounded codecs,
malformed snapshots, duplicate identities and explicit recovery authorization.

## Progress and resource evidence

`subscribe(capacity)` returns a bounded event receiver. Enqueue, start, completion,
failure, cancellation and `CancellationToken::progress(percent,message)` emit
sequenced metadata events using nonblocking sends. Full consumers lose events;
`usage().events_dropped` records missed deliveries and clients resynchronize from
`state`. No consumer callback runs inside the scheduler lock. Progress messages
are limited to 1024 bytes and percentages to 0–100; do not include credentials.
Disconnected receivers are pruned on the next event. Up to 64 subscriptions with
capacity 1–4096 are permitted; callers should use small UI buffers.

`usage` exposes queued, physically executing, retained and converter-call counts.
These are process-local measurements, not cumulative provider billing or quota;
`provider_billing_known` is always false. Cancellation does not free an executing
slot early. Fifteen tests cover the scheduler, recovery and event/backpressure paths.

## Actual bridge adapter example (offline)

`cargo run --manifest-path crates/conversion-jobs/Cargo.toml --example capture_pipeline`
uses the real bridge document/context/journal APIs with an explicitly offline
converter. It persists a conservative attempt-intent record before starting a job,
so a crash before the first job checkpoint cannot silently retry a possibly charged
call. A durable existing proposal wins over stale scheduler metadata. A pending
intent without a proposal requires explicit reconciliation; it never retries itself.

The example keeps document ownership separate from the worker closure, fingerprints
complete supplied source snapshots, checks current context before promoting a result,
and persists the bridge proposal before retiring the job/intent. It does not edit
source. Two example tests exercise persisted-result deduplication across owner reopen
and the unresolved-intent retry gate. The example uses a single capture and an
exclusive bridge store; production needs per-capture intent names and a native
document transaction, not one shared static intent filename.

Compiler validation is a subsequent independent step. Root's separately published
`Bridge::validate_capture`/`capture_validate` adapter builds a hypothetical source
snapshot and runs the explicitly configured original Rust compiler. Integrate that
API through the sole integration owner, display its actual diagnostics, and require
review before preparing an edit. This example does not invoke compilation and makes
no full-compatibility or live-Grok claim.

Final adapter checkpoint: `cargo test --all-targets` passes 17 tests; strict Clippy
and formatting pass; actual example run prints `Offline fixture journaled for
review: $x^2$`. No Grok, Claude or other provider request is made by these checks.

## Optional reusable asynchronous bridge adapter

Enable `bridge-integration` for `bridge_adapter::BridgeAdapter`. Its injected
`Arc<Provider>` accepts immutable `CaptureSubmit`, `Context` and cancellation token;
the real bridge stays exclusively on the background document/journal actor.
`start` first checks the durable capture and exclusively fsyncs a per-capture intent,
then dispatches at most one provider attempt. An existing intent with no journaled
proposal exposes `RecoveryRequired`; adapter restart, terminal failure, retirement
and repeated start do not silently retry that capture identity. A new attempt must
be an explicit application decision with new capture identity after reconciliation.

`status_handle()` gives a cloneable handle that performs no filesystem/provider IO.
Native UI can poll queued/running/awaiting-journal/recovery/terminal states without
waiting for conversion. `reconcile(bridge,id)` belongs on the background document
actor: compare the exact current Context (including dependency revision/hash set),
then fsync the proposal before exposing `Proposal`. Source changes, cancellation
before journal promotion, prepared/applied/rejected captures and conflicting results
cannot be silently overwritten. Call reconciliation as part of every document
update/result-consumption transaction; a cached status alone does not authorize
insertion. Actual compiler validation and user review remain the next separate gate.

`retire` releases bounded in-memory state only after the physical call finishes;
durable intents/capture journal remain for deduplication. Queue rejection removes
only the newly created intent because submission proves no provider started, so
explicit backpressure retry remains possible. `usage` reports actual process-local
calls and physical execution, not billing. Provider closures still need timeouts.
The adapter does not implement the bridge CLI, networking or a native screen.

## Typed background command/event boundary

`bridge_adapter::native` provides bounded typed requests (`protocol_version`,
correlation `id`, `command`) for Admit, Start, Status, Reconcile and Cancel. This
is an internal adapter API, not an added bridge CLI message. Each command includes
an exact capture ID and ContextIdentity (project, path, revision, context SHA-256).
Admission confirms an existing durable capture without starting a provider. Start
requires the unchanged admitted context; reconciliation takes the exact current
context and preserves the original conversion identity in returned snapshots.

Snapshots distinguish durable receipt, absent/durable attempt intent, queued/running,
awaiting journal promotion, proposal-ready and recovery-required states. Restored
intents are visible at admission before any dispatch. `NativeStatusHandle` reads
only memory; typed events carry capture and admitted context identity. Full event
channels remain lossy and clients resynchronize from status. Old buffered events
are excluded when an identity is retired and newly admitted.

Command bytes are bounded to 16 KiB before parsing. Snapshot serialization uses a
writer that stops before exceeding its output budget (maximum 512 KiB); it never
returns partial JSON. Unknown fields/types and mismatched identities fail closed.
Provider failure messages are bounded to 2048 UTF-8 bytes. Twenty-five tests,
strict Clippy and formatting pass for this checkpoint. No new CLI/provider call.

## Ledger handoff and concurrent projects

The optional-feature suite now contains 28 passing tests. The actual edit-ledger
handoff test verifies its durable document bytes remain unchanged throughout
conversion and proposal promotion. Only the explicit caller then prepares/reviews
an edit, applies it through edit-ledger, acknowledges the bridge receipt, and
confirms the ledger transaction. Reopening both stores and replaying the same
receipt does not insert twice. This verifies storage handoff, not native review UI
or compiler acceptance. Conversion jobs do not write document source.

A two-project test holds one converter pending while another project completes;
cancelling the first does not cancel or overwrite the second. Admission is bounded
FIFO with fixed worker count, not weighted per-project quota scheduling. Native
reconciliation also revokes exposed status when a full snapshot replacement
invalidates the original anchor and no current context can be assembled.

## Durable multi-capture review inbox

`bridge_adapter::review::ReviewInbox` persists bounded proposal cards, explicit
current selection and decision history under an exclusive journal lock. The caller
admits only journaled proposals with their actual ContextIdentity. No card is
selected or accepted automatically. `decide(DecisionRequest)` requires the selected
capture, exact proposal hash and explicit `AcceptForPreparation` or `RejectCapture`
intent. Identical decision retries return the same context-bound handoff; reuse of
an ID with altered data or a contradictory second decision fails closed.

A `ReviewIntentHandoff` is a durable intention, not proof of human presence or final
approval of a PreparedEdit. As coordinated with the controller owner, the caller
must validate it against actual current context, request preparation, display the
exact resulting edit and obtain explicit approval before constructing ApprovedEdit
or invoking the controller's apply action. The inbox never calls bridge prepare,
reject, apply, providers or source-writing APIs itself.

Context changes and cancellation revoke handoffs. Explicit rejection of a stale
proposal is possible using its current context. Terminal cards can be retired while
bounded decision history and capture tombstones preserve deduplication; full limits
return capacity errors rather than silently evicting history. A cloneable InboxView
returns immutable snapshots without disk IO. Any persistence uncertainty poisons
this handle and its view until reopen; bounded-serialization rejection leaves the
previous checkpoint intact. Reopen validates record and decision identity invariants.

The inbox adds eight tests; 36 all-feature/all-target tests, strict Clippy and
formatting pass. Cases cover reopen, selection, duplicate/tampered decisions,
context changes, cancellation, retirement, bounded serialization, corrupt state,
exclusive ownership and recovery after real storage failure.

Review follow-up: `InboxView::visible_ids(InboxFilter)` returns arrival-ordered
IDs filtered by project and optional undecided status, without copying proposal
bodies or reading disk. `navigate` changes selection only; it never accepts a card.
Selection persists across reopen; reaching an end keeps it unchanged, and navigating
an empty filtered view clears it. `admit_ready` consumes only the native adapter's
fresh, durably journaled proposal state and leaves selection/decision empty.

Inbox event subscribers use bounded nonblocking channels. Notifications carry
capture/project IDs and durable generation, not proposal bodies or approval tokens.
Full consumers lose notifications; dropped-delivery counts and immutable view
snapshots support resynchronization. Persistence uncertainty emits a distinct event
and makes view reads fail until reopen. Thirty-nine tests, strict Clippy and
formatting pass, including actual native-ready snapshot admission into this inbox.

### Review expiry and recovery

Review cards optionally receive a caller-clock deadline with `set_expiry` before
any decision; a deadline can only be shortened. Use `open_at`, `decide_at`, and
`validate_handoff_at` with one consistent clock domain (for example persisted Unix
seconds). These entry points durably run bounded `expire_due` before recovery or
acceptance. The clock-free methods remain for controllers that explicitly run
expiry themselves. No background wall-clock timer is implied. Expired cards clear
current selection, revoke existing handoffs and remain expired across restart or
clock rollback; explicit retirement retains the existing replay tombstone.

Context revocation is sticky, including changes to included-document hashes with
an unchanged destination revision. Returning to an earlier hash cannot revive a
previous acceptance. After restoring native document snapshots, call
`refresh_from_bridge` before using a recovered card. Missing context also durably
revokes it; a new capture/conversion is required. This does not mutate the capture
journal, prepared edits, receipts or source. `AcceptForPreparation` continues to
require a separate exact-`PreparedEdit` review/approval in the controller.

### Native review JSONL helper

Build `cargo build --features bridge-integration --bin flashtex-review-inbox`.
Run `flashtex-review-inbox PRIVATE_INBOX_DIRECTORY`. This exclusive-lock process
only stores review metadata. It never calls a provider, prepares an edit, writes
source, or treats a review intention as approval of an exact PreparedEdit.

Every UTF-8 JSON request must end with a newline:

```json
{"protocol_version":1,"id":"request-1","now":1730000000,"type":"snapshot","payload":null}
```

`now` is mandatory in the caller's consistent persisted-clock domain. All commands
run durable expiry first, including the first request after restart. Response
`type` is `review_result` or `review_error`, echoing the ID for valid requests;
errors carry `payload.code`. Malformed envelopes have null ID. Additive command
names apply to this review helper, not the existing capture transfer endpoint.

Commands and payloads:

- `snapshot`: null; returns bounded persisted cards, generation and selection.
- `select`: `{capture_id: string|null}`.
- `admit_ready`: `{status: StatusSnapshot}` from the trusted local native adapter.
- `update_context`: `{capture_id, context: ContextIdentity}`; controller must
  provide actual current context before any recovered decision/handoff.
- `decide`: `{decision: DecisionRequest}`; returns preparation/rejection intent.
- `validate_handoff`: `{handoff: ReviewIntentHandoff, context: ContextIdentity}`;
  success means `valid_for_preparation_only`, never exact-edit approval.
- `set_expiry`: `{capture_id, expires_at}`; deadlines already due expire immediately.
- `expire`: null; returns current durable generation.
- `cancel` / `retire`: `{capture_id}`.

This is a trusted local IPC interface, not authenticated network admission. A
client-supplied context or proposal-ready status is not independently attested.
The native controller must derive these from the actual bridge. Startup invalid
journal/lock errors exit nonzero with stderr; request persistence uncertainty
returns `recovery_required`. Reopen and reconcile before further decisions.

#### Background I/O and deadline supervision

Host the process on a background I/O actor, never the UI thread. Use one in-flight
command and a bounded queue (suggested 32), capped at 512 KiB per newline-delimited
request and 1 MiB per response. The helper retains at most 64 cards and 512 KiB of
journal JSON; oversize input is drained to the next newline without retaining it.
Invalid/partial EOF frames cannot execute; clean EOF exits and releases the lock.
Responses flush individually; the host must continuously drain stdout and stderr.

The helper deliberately serializes filesystem transactions. Its OS reads, writes
and fsync may block; no hard filesystem deadline is claimed. The host should use
an explicit monotonic request deadline (suggested five seconds), stop admission
on timeout, terminate and reap the exact child before reopening its directory.
A timeout is an ambiguous durable outcome, not evidence of rollback. Recover via
snapshot and the original decision ID; never mint a new decision or apply source
automatically to compensate for a missing response. Request IDs are correlation
IDs, not a durable deduplication ledger. No unbounded retries or background
provider work are started by this helper.

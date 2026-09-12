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

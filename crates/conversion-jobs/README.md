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

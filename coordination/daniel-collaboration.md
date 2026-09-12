# daniel-collaboration handoff (FT-044)

Agent / task / branch: `daniel-collaboration` / FT-044 "Offline bounded
collaborative text operation core" / `agent/daniel-collaboration/collaboration-core`

State: ready for integration (revision 2)

Owned paths: `crates/collaboration-core/**`, `coordination/daniel-collaboration.md`.
No other crate is touched, in particular `crates/edit-ledger` is untouched.

Tested commit (this branch, HEAD at the time `cargo build`, `cargo test`, and
`cargo clippy --all-targets -- -D warnings` were all run clean): see
`coordination/agents/daniel-collaboration.json`'s `code_revision` (updated at
every revision; do not hand-copy a stale SHA into this file).

## Revision 2: checkpoints and interrupted delivery recovery

Adds two additive capabilities on top of revision 1's unchanged core
(`Document::apply`'s behavior, including `MissingDependency` rejection, is
untouched):

- **`Document::checkpoint(max_entries) -> Result<Checkpoint, CheckpointError>`**
  (`src/checkpoint.rs`) takes a bounded, serializable snapshot of a
  document's full structure (elements in order, including tombstones) plus
  its causal frontier (every applied operation id, insert and delete
  alike). `Document::from(checkpoint)` resumes a fully functional document
  from it - not a read-only view. Exceeding `max_entries` returns
  `CheckpointError::TooLarge` instead of an unboundedly large snapshot.
  `Checkpoint::to_bytes`/`from_bytes` give one concrete, dependency-free
  byte encoding with bounded, non-panicking decoding
  (`CheckpointDecodeError::Truncated`/`InvalidChar`).
- **Checkpoint equivalence** (the load-bearing property for this revision):
  resuming from a checkpoint and applying the remaining operations produces
  a document structurally identical - not just text-identical - to
  replaying the whole log from scratch. Proven in
  `tests/checkpoint_equivalence.rs` over three independent scenarios: a
  sequential single-replica build with deletes, a Unicode/multibyte
  sequence, and (most thoroughly) every causally valid ordering of a
  5-operation concurrent insert/insert/delete scenario, each checked at
  every possible cut point (60+ (ordering, cut-point) combinations). Every
  check compares the two documents' own checkpoints (structural: elements,
  tombstones, applied-id set) in addition to visible text.
- **Interrupted delivery recovery: buffering, via the additive
  `PendingOps` type** (`src/recovery.rs`). `Document::apply` itself still
  rejects a missing dependency with `CrdtError::MissingDependency` exactly
  as in revision 1 - `PendingOps::receive(&mut doc, op)` wraps it: on a
  missing dependency it buffers `op` (bounded by a configured
  `max_pending`, `CrdtError::PendingBufferFull` past that) and returns
  `ApplyOutcome::Buffered` instead of erroring; whenever anything is newly
  applied, the whole buffer is retried, so multi-level dependency chains
  cascade in once their root arrives, in any delivery order.
  `PendingOps::missing_dependencies(&doc)` additionally exposes exactly
  which ids the buffer is still blocked on, for a caller that wants to
  proactively request redelivery rather than only wait. Tested in
  `src/recovery.rs` (single-gap buffering, multi-level chains, the buffer
  bound, idempotent redelivery, `IdConflict` still surfacing through
  `receive`) and end to end in `tests/interrupted_delivery.rs` (a scrambled
  reconnect backlog recovering to the same state as in-order delivery; the
  missing-dependency report across two rounds of a chain).

## What this is, and what it explicitly is not

`flashtex-collaboration-core` (crate `crates/collaboration-core`) is an
offline, bounded core for merging **concurrent** edits to a small text (a
note, a card field) made by two or more replicas that were never serialized
against a shared revision counter. It has zero dependencies — no network
I/O, no networking crate, nothing beyond `std`.

It is **not** a replacement for `flashtex-edit-ledger`, and does not
duplicate it:

- `edit-ledger` owns **one authoritative document** with a single serial
  writer: durable fsynced storage, `expected_revision`-guarded transactions,
  receipts, crash recovery, undo/redo history. It has no concept of two
  replicas editing independently and later reconciling — every writer must
  already agree on the current revision before it acts.
- `collaboration-core` owns the piece `edit-ledger` explicitly does not
  attempt: what happens when two replicas *did* edit independently (e.g.
  offline, or before either one talked to the ledger) and their operations
  now need to be merged into one consistent text, in whatever order they
  happen to arrive.

The intended composition is sequential, not overlapping: replicas
reconcile concurrent edits through a `collaboration-core::Document` first;
once reconciled, a single owner replays the converged text into
`edit-ledger` as one ordinary `replace_document`/`apply` transaction against
its current revision. `collaboration-core` has no dependency on, and makes
no calls into, `edit-ledger`.

## Typed contract

```rust
pub struct ReplicaId(pub u64);

pub struct OpId {
    pub counter: u64,      // per-replica Lamport-style counter, compared first
    pub replica: ReplicaId, // deterministic tie-break when counters are equal
}

pub enum OpPayload {
    Insert { left: Option<OpId>, right: Option<OpId>, value: char },
    Delete { target: OpId },
}

pub struct Op { pub id: OpId, pub payload: OpPayload }

pub enum ApplyOutcome { Applied, Duplicate }

pub enum CrdtError {
    MissingDependency(OpId), // left/right/target not applied yet
    IdConflict(OpId),        // id reused with a different insert value
    DocumentFull,            // at the configured element bound
}

pub struct Document { /* opaque */ }
impl Document {
    pub fn new() -> Self;                                  // DEFAULT_MAX_ELEMENTS = 200_000
    pub fn with_max_elements(max_elements: usize) -> Self;
    pub fn apply(&mut self, op: Op) -> Result<ApplyOutcome, CrdtError>;
    pub fn text(&self) -> String;
    pub fn len_chars(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn is_applied(&self, id: OpId) -> bool;
    pub fn char_id_at(&self, index: usize) -> Option<OpId>; // by char, never by byte
}

pub struct OpBuilder { /* opaque */ }
impl OpBuilder {
    pub fn new(replica: ReplicaId) -> Self;
    pub fn insert_at(&mut self, doc: &Document, index: usize, value: char) -> Option<Op>;
    pub fn delete_at(&mut self, doc: &Document, index: usize) -> Option<Op>;
}

// --- Revision 2 additions -------------------------------------------------

pub struct Checkpoint { /* opaque, plain data */ }
impl Checkpoint {
    pub fn entry_count(&self) -> usize;   // elements + delete-op ids; what max_entries bounds
    pub fn visible_len(&self) -> usize;
    pub fn to_bytes(&self) -> Vec<u8>;
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CheckpointDecodeError>;
}
pub enum CheckpointError { TooLarge { entries: usize, max_entries: usize } }
pub enum CheckpointDecodeError { Truncated, InvalidChar(u32) }

impl Document {
    pub fn checkpoint(&self, max_entries: usize) -> Result<Checkpoint, CheckpointError>;
}
impl From<Checkpoint> for Document { /* resumes a fully functional Document */ }

pub struct PendingOps { /* opaque */ }
impl PendingOps {
    pub fn new(max_pending: usize) -> Self;
    pub fn receive(&mut self, doc: &mut Document, op: Op) -> Result<ApplyOutcome, CrdtError>;
    pub fn missing_dependencies(&self, doc: &Document) -> Vec<OpId>;
    pub fn pending_len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
// ApplyOutcome gained a third variant, `Buffered` — only ever returned by
// PendingOps::receive, never by Document::apply itself.
// CrdtError gained a fourth variant, `PendingBufferFull { max_pending: usize }`.
```

Positions are only ever expressed as `OpId`s or visible character indices —
never byte offsets — so no operation can split a multi-byte UTF-8 character;
`Document::text()` always returns a `String` built by collecting whole
`char`s. `OpBuilder` indices are bounds-checked (`Option`, no panic).

## Ordering and tie-break rule

Text is a sequence of Unicode scalar values, each with a stable `OpId` that
is never reassigned or moved. Deletion never removes an element — it only
sets a tombstone flag — so `Document::text()` is a pure filter over that
structure and can never desynchronize with the underlying order.

Insert operations carry the `left`/`right` neighbor ids the inserting
replica observed at the time of insertion (a YATA-style anchored sequence
CRDT, the same family of algorithm used by Yjs). Integration
(`Document::integrate`, private) places a new element within the exact gap
`(left, right)` it was anchored to:

1. Scan forward from just after `left`, stopping before `right`.
2. An existing element anchored to a strictly different, non-overlapping gap
   is skipped over as part of its own subtree.
3. An existing element anchored to the **same** `(left, right)` gap is a true
   concurrent sibling: ties are broken by comparing `OpId` as the tuple
   `(counter, replica)` — the sibling with the *larger* id is placed closer
   to `left`.

Because `(counter, replica)` pairs are globally unique and this comparison
is a pure function of already-applied state, any two replicas that have
applied the same set of operations compute the same order, regardless of
delivery order — this is exactly what `tests/convergence.rs` demonstrates
rather than asserts.

Delivery contract: operations must be delivered **causally** per replica — a
Insert/Delete's referenced `left`/`right`/`target` id must already be
applied locally before the operation itself. `apply()` rejects an operation
whose dependency is missing with `CrdtError::MissingDependency` rather than
buffering or guessing — this is unchanged from revision 1. Transport and
causal delivery (e.g. via a vector clock, or "replicas only exchange after
syncing history") remain the caller's responsibility for direct `apply()`
use. As of revision 2, a caller that instead wants to tolerate a partial or
out-of-order backlog (e.g. after a reconnect) can route delivery through
`PendingOps::receive` — see "Interrupted delivery recovery" below — rather
than handling `MissingDependency` itself.

## Convergence proof (fixtures, not assertions)

`crates/collaboration-core/tests/convergence.rs`:

- **Concurrent insert at the same position.** Base text `"ac"`. Replicas A
  and B each independently insert between `a` and `c` (`'b'` from A,
  `'z'` from B), both anchored to the identical gap `(a, c)`. Hand-derived:
  since counters are equal, replica id breaks the tie and the larger
  `OpId` (B's) sorts closer to `a`, giving `"azbc"` — verified identical
  whether A applies its own op first then B's, or vice versa.
- **Concurrent insert versus delete, overlapping.** Base text `"abc"`.
  Replica A deletes `b`; replica B concurrently inserts `'X'` anchored
  `(left=b, right=c)`. Hand-derived: delete only flips a tombstone flag and
  never changes any id's structural position, so `'X'` lands between `b`'s
  (tombstoned) slot and `c` regardless of order, giving visible text
  `"aXc"` — verified both ways (delete-then-insert and insert-then-delete).
- **Replay of a concurrent op.** Confirms redelivering an already-applied
  operation inside a convergence scenario (not just the isolated case in
  `src/lib.rs`) stays a no-op.
- **Exhaustive sweep, bonus.** A 5-operation scenario (two concurrent
  absolute-start inserts, two ops depending on one of them, one depending
  on the other) is applied in every one of its 120 permutations; orderings
  that violate causal delivery are skipped, and the remaining ≥10 valid
  orderings are cross-checked to produce byte-for-byte identical text.

This proves convergence for the crate's supported operation set — single
Unicode-scalar `Insert` anchored to observed neighbors, and `Delete` by
target id, under causally-ordered-per-replica delivery. It is not a claim
about arbitrary out-of-causal-order delivery (rejected explicitly, see
above) or about multi-character batch/string operations (not part of this
API; callers apply one `Op` per character).

## Replay deduplication

`Document::apply` keeps a `HashSet<OpId>` of already-applied operation ids.
Reapplying an id already in that set returns `ApplyOutcome::Duplicate`
without any structural change (`src/lib.rs` tests
`replaying_the_same_insert_is_a_no_op`,
`replaying_the_same_delete_is_a_no_op`; `tests/convergence.rs`
`replay_of_a_concurrent_op_does_not_double_apply`). Reusing an id with a
*different* insert value — a malformed/misbehaving sender — is rejected as
`CrdtError::IdConflict` instead of silently accepted (`reusing_an_id_with_different_value_is_rejected`).

## UTF-8 safety

All positions are `OpId`s or char (not byte) indices; the CRDT structure is
built and iterated per `char`, so `Document::text()` cannot construct
invalid UTF-8 and no operation can split a multi-byte scalar. Tested with a
mixed 1/2/3/4-byte-UTF-8 string plus a zero-width joiner
(`multibyte_characters_round_trip_exactly`), deletion immediately adjacent
to CJK/emoji characters (`deleting_around_multibyte_characters_never_splits_one`),
and insertion exactly between two 4-byte emoji
(`insert_at_index_between_multibyte_neighbors`).

## Malformed and bounded input

- Insert/Delete referencing an unknown id: `CrdtError::MissingDependency`,
  no panic (`insert_with_unknown_left_is_rejected_not_panicking`,
  `delete_of_unknown_target_is_rejected_not_panicking`).
- `OpBuilder` index past the end of the document: returns `None`, no panic
  or clamping (`out_of_range_builder_indices_return_none_not_panic`).
- Structurally nonsensical `(left, right)` (right resolves to a position
  before left): accepted without a dedicated validation pass beyond "both
  referenced ids exist" — placed at a definite, bounded position rather
  than panicking (`malformed_left_right_ordering_does_not_panic`). This is
  a known, documented limitation: the crate does not attempt to detect or
  reject an inconsistent gap, only a missing one.
- `Document::with_max_elements` bounds total structural elements (visible +
  tombstoned); exceeding it returns `CrdtError::DocumentFull` instead of
  growing without limit (`document_full_is_rejected_not_grown_unbounded`).
  Default bound is 200,000 elements — sized for a note/card field, not a
  manuscript; `edit-ledger` is the durable home for the latter once
  reconciled.

## Build and check

```sh
cargo build --manifest-path crates/collaboration-core/Cargo.toml
cargo test --manifest-path crates/collaboration-core/Cargo.toml
cargo clippy --manifest-path crates/collaboration-core/Cargo.toml --all-targets -- -D warnings
```

33 tests pass: 24 unit tests in `src/lib.rs` (13 core + 5 in
`checkpoint::tests` + 6 in `recovery::tests`), plus integration tests in
`tests/convergence.rs` (4, revision 1, unchanged), `tests/checkpoint_equivalence.rs`
(3), and `tests/interrupted_delivery.rs` (2). Clippy passes with warnings
denied (`cargo clippy --all-targets -- -D warnings`). Zero dependencies
(`Cargo.lock` lists only this crate itself), so no network access is
possible at build or run time — revision 2 added no dependency either.

## Known limitations / not done

- Convergence is proven for single-character `Insert`/`Delete` under
  causally-ordered-per-replica delivery only (see "Convergence proof"
  above) — not for a multi-character batch op (there isn't one; a string
  edit is one `Op` per `char`). `integrate`'s neighbor lookup is a linear
  scan (`O(n)` per operation), an intentional simplicity/boundedness
  tradeoff for small, bounded text — not tuned for large documents.
  Out-of-causal-order delivery is no longer only rejected: `PendingOps` (see
  above) buffers and recovers it, but `Document::apply` itself is still
  strict, by design, so the convergence proof's own delivery contract is
  unchanged from revision 1.
- `Checkpoint::to_bytes`/`from_bytes` is one concrete, hand-rolled
  dependency-free encoding, not a stability promise across crate versions;
  a consumer that needs a versioned/cross-version wire format should treat
  this as a starting point, not a guarantee. `Op`/`OpId` themselves still
  have no crate-defined wire format beyond being plain `Copy` primitives.
- `PendingOps`'s buffer bound is a flat operation count (`max_pending`),
  not a byte-size bound; a caller with adversarial-sized payloads should
  size `max_pending` accordingly (this crate's `char`-per-op model already
  caps any single buffered operation's size).
- No integration with `edit-ledger` or any native/bridge caller is
  implemented or claimed; this is a standalone additive crate per FT-044's
  scope, awaiting a consumer contract before integration.

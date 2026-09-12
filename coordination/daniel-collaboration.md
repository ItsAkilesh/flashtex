# daniel-collaboration handoff (FT-044)

Agent / task / branch: `daniel-collaboration` / FT-044 "Offline bounded
collaborative text operation core" / `agent/daniel-collaboration/collaboration-core`

State: ready for integration

Owned paths: `crates/collaboration-core/**`, `coordination/daniel-collaboration.md`.
No other crate is touched, in particular `crates/edit-ledger` is untouched.

Tested commit (this branch, HEAD at the time `cargo build`, `cargo test`, and
`cargo clippy --all-targets -- -D warnings` were all run clean):
`1989b17dc37348077e88e4a0fcd6b3d01f0823f2`

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
buffering or guessing; there is no out-of-order reordering buffer in this
crate. Transport and causal delivery (e.g. via a vector clock, or "replicas
only exchange after syncing history") are the caller's responsibility.

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

17 tests pass (13 unit tests in `src/lib.rs`, 4 integration tests in
`tests/convergence.rs`); clippy passes with warnings denied. Zero
dependencies (`Cargo.lock` lists only this crate itself), so no network
access is possible at build or run time.

## Known limitations / not done

- Convergence is proven for single-character `Insert`/`Delete` under
  causally-ordered-per-replica delivery only (see "Convergence proof"
  above) — not for out-of-causal-order delivery, which this crate rejects
  rather than buffers or reorders, and not for a multi-character batch op
  (there isn't one; a string edit is one `Op` per `char`).
  `integrate`'s neighbor lookup is a linear scan (`O(n)` per operation),
  an intentional simplicity/boundedness tradeoff for small, bounded text —
  not tuned for large documents.
- No serialization/wire format is defined or implied by this crate; a
  consumer wiring transport between replicas defines its own encoding of
  `Op`/`OpId` (all fields are plain, `Copy` primitives).
- No integration with `edit-ledger` or any native/bridge caller is
  implemented or claimed; this is a standalone additive crate per FT-044's
  scope, awaiting a consumer contract before integration.

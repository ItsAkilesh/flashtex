# daniel-collaboration handoff (FT-044)

Agent / task / branch: `daniel-collaboration` / FT-044 "Offline bounded
collaborative text operation core" / `agent/daniel-collaboration/collaboration-core`

State: ready for integration (revision 4)

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

57 tests pass: 26 unit tests in `src/lib.rs` (15 core, incl. the two
revision-3 counter-exhaustion cases, + 5 in `checkpoint::tests` + 6 in
`recovery::tests`; unchanged since revision 3), plus integration tests in
`tests/convergence.rs` (4, revision 1, unchanged),
`tests/checkpoint_equivalence.rs` (3, revision 2, unchanged),
`tests/interrupted_delivery.rs` (2, revision 2, unchanged),
`tests/adversarial.rs` (17: the 9 from revision 3 plus 8 new in revision 4),
`tests/measured_fixtures.rs` (3, new in revision 4), and
`tests/compatibility_evidence.rs` (2, new in revision 4). Clippy passes with
warnings denied (`cargo clippy --all-targets -- -D warnings`). Zero
dependencies (`Cargo.lock` lists only this crate itself), so no network
access is possible at build or run time — revision 4 added no dependency
either, including in test code (the byte-hash helper in
`tests/compatibility_evidence.rs` is a locally written, dependency-free
FNV-1a implementation, not an external crate).

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

## Revision 3: adversarial hardening and the published consumer contract

Two additions, both additive; revision 1's convergence/dedup and revision 2's
checkpoint equivalence are unchanged and still pass (see "Build and check"
below for the updated count).

### Bug fixed while adding adversarial coverage: `OpBuilder` counter exhaustion

Writing the "counter at integer maximum" adversarial case surfaced a real,
if extremely unlikely, defect: `OpBuilder::next_id` incremented its
per-replica counter with plain `+= 1`, which panics on overflow in a debug
build and silently wraps to a reused counter (a latent `IdConflict`) in a
release build. Fixed with `checked_add`: `OpBuilder::insert_at`/`delete_at`
now return `None` — the same bounded, already-documented signal they use for
an out-of-range index — if this replica's counter is already at `u64::MAX`.
`OpId.counter` itself is an unconstrained `u64`; `u64::MAX` is a perfectly
valid id and applies normally (`op_with_max_counter_value_applies_normally`
in `src/lib.rs`). This does not change any public type signature.

### New adversarial tests (`crates/collaboration-core/tests/adversarial.rs`
unless noted), each asserting a typed `Err`/`None` result or a well-defined
success — never a panic:

- **Operation referencing an unknown replica**:
  `insert_referencing_an_id_from_a_replica_that_never_produced_any_operation_is_rejected`
  and the `Delete` counterpart — an `OpId` whose replica has never applied
  anything to the document is rejected with `CrdtError::MissingDependency`,
  identically to any other unknown id.
- **Counter at integer maximum**: `op_builder_returns_none_when_counter_is_exhausted_not_panicking`
  and `op_with_max_counter_value_applies_normally` (`src/lib.rs`, private-field
  access needed to construct a pre-exhausted builder) — see the bugfix above.
- **Pending buffer at and past capacity**:
  `pending_buffer_exactly_at_capacity_succeeds_next_one_is_rejected` — fills
  `PendingOps` to exactly `max_pending`, confirms all of them are accepted as
  `Buffered`, then confirms the very next one is rejected with
  `CrdtError::PendingBufferFull` and the buffer length does not grow past the
  bound.
- **Checkpoint at and past its entry bound**:
  `checkpoint_exactly_at_bound_succeeds_one_less_bound_fails` — a document
  with exactly 5 entries checkpoints successfully at `max_entries: 5` and
  fails with `CheckpointError::TooLarge` at `max_entries: 4`.
- **Truncated and corrupted checkpoint bytes**: beyond revision 2's
  `Truncated`/`InvalidChar` cases, adds a header that declares
  `u64::MAX` elements backed by no data
  (`checkpoint_bytes_with_declared_counts_far_exceeding_actual_data_is_rejected` —
  confirms the bounded `Reader` fails on the first missing field rather than
  attempting to allocate or iterate anything unbounded), a sweep of
  arbitrary byte patterns of varying length wrapped in
  `std::panic::catch_unwind` (`checkpoint_bytes_corrupted_with_arbitrary_patterns_never_panics`),
  and a single flipped byte inside an otherwise-valid encoding
  (`checkpoint_bytes_with_a_flipped_byte_in_the_middle_is_rejected_or_harmlessly_decoded` —
  asserts only "does not panic", since a flipped byte may or may not
  produce a structurally-valid-looking decode).
- **Insert whose position references a deleted element**:
  `insert_anchored_to_a_tombstoned_element_still_integrates_deterministically` —
  a tombstone is a structurally *present* element (only its `deleted` flag is
  set), so `left`/`right` anchoring to one is not a missing dependency; the
  insert integrates at a definite position. Also adds
  `delete_of_an_already_deleted_element_is_idempotent_not_double_counted` for
  the adjacent case of two independent delete operations targeting the same
  (already-deleted) element.

## Published consumer contract

**No consumer exists today.** Confirmed by grepping the whole repository
(not just this crate) for every plausible reference to this crate's package
and crate names:

```sh
grep -rn "collaboration.core\|collaboration_core\|flashtex-collaboration-core\|flashtex_collaboration_core" \
  --include="*.rs" --include="*.toml" --include="*.md" --include="*.json" . \
  | grep -v "^./crates/collaboration-core/" | grep -v "/target/"
```

Every hit outside `crates/collaboration-core/` itself is coordination
bookkeeping (this file, the assignment/agent-registry JSON) — no `.rs` or
`Cargo.toml` anywhere else in the repository names this crate. There is also
no workspace-root `Cargo.toml` (none exists in this repository at all), so
nothing pulls this crate in implicitly either. Per FT-044's ownership
boundary this agent does not edit `crates/edit-ledger` or any other crate to
manufacture a consumer; what follows is the interface a future consumer
would use, published so an integrator can wire it up without needing to read
`src/`.

### The contract, by consumer-facing operation

```rust
use flashtex_collaboration_core::{
    ApplyOutcome, Checkpoint, CheckpointDecodeError, CheckpointError,
    CrdtError, Document, Op, OpBuilder, OpId, PendingOps, ReplicaId,
};

// 1. Construct a replica's document (and its op-authoring helper).
let mut doc: Document = Document::new(); // DEFAULT_MAX_ELEMENTS = 200_000
// or: Document::with_max_elements(max_elements: usize) -> Document
let mut builder: OpBuilder = OpBuilder::new(ReplicaId(my_replica_id));

// 2. Submit a local edit: build an `Op` anchored to the document's current
//    visible state, then apply it to this replica's own document.
let op: Option<Op> = builder.insert_at(&doc, index /* char index */, value /* char */);
// or: builder.delete_at(&doc, index) -> Option<Op>
// `None` means `index` was out of range, or (see revision 3) this builder's
// counter is exhausted at `u64::MAX` — never a panic.
let outcome: Result<ApplyOutcome, CrdtError> = doc.apply(op.unwrap());
// Send `op` (it is `Copy`) to every other replica over whatever transport
// the caller owns; this crate does no I/O itself.

// 3. Receive a remote operation.
//    Direct (revision 1): requires causal delivery (op's left/right/target
//    already applied locally); a gap is a typed error, not a panic:
let outcome: Result<ApplyOutcome, CrdtError> = doc.apply(remote_op);
// `Ok(Applied)` | `Ok(Duplicate)` (already-seen id, safe to redeliver) |
// `Err(MissingDependency(id))` | `Err(IdConflict(id))` | `Err(DocumentFull)`.

// 4. Recover from a gap (out-of-order/partial backlog, e.g. after a
//    reconnect): route delivery through `PendingOps` instead of `apply`
//    directly. It buffers on a missing dependency instead of erroring, and
//    auto-drains (including multi-level chains) as dependencies arrive.
let mut pending: PendingOps = PendingOps::new(max_pending: usize);
let outcome: Result<ApplyOutcome, CrdtError> = pending.receive(&mut doc, remote_op);
// `Ok(Buffered)` on a gap; `Err(PendingBufferFull { max_pending })` past the
// configured bound. To proactively re-request instead of only waiting:
let still_missing: Vec<OpId> = pending.missing_dependencies(&doc);

// 5. Take a checkpoint (bounded snapshot: structure + causal frontier).
let cp: Result<Checkpoint, CheckpointError> = doc.checkpoint(max_entries: usize);
// Err(TooLarge { entries, max_entries }) instead of an unbounded snapshot.
let bytes: Vec<u8> = cp.unwrap().to_bytes();

// ...and restore one (a fully functional Document, not a read-only view):
let restored_cp: Result<Checkpoint, CheckpointDecodeError> = Checkpoint::from_bytes(&bytes);
// Err(Truncated) | Err(InvalidChar(code)) on malformed/corrupted bytes.
let mut restored_doc: Document = Document::from(restored_cp.unwrap());
// Apply whatever operations postdate the checkpoint exactly as in step 3/4.

// 6. Read the resulting text and its causal frontier.
let text: String = doc.text();                    // current visible text
let n: usize = doc.len_chars();                    // visible char count
let seen: bool = doc.is_applied(some_op_id);        // frontier membership
let id_at_i: Option<OpId> = doc.char_id_at(index);  // stable per-char id
```

### Worked example: two replicas, offline, then reconciled

```rust
let mut a = Document::new();
let mut ba = OpBuilder::new(ReplicaId(1));
a.apply(ba.insert_at(&a, 0, 'a').unwrap()).unwrap();
a.apply(ba.insert_at(&a, 1, 'c').unwrap()).unwrap(); // replica A: "ac"

let mut b = a.clone();                     // replica B starts from the same state
let mut bb = OpBuilder::new(ReplicaId(2));
let op_b = bb.insert_at(&b, 1, 'z').unwrap();
b.apply(op_b).unwrap();                    // replica B, offline: "azc"

let op_a = ba.insert_at(&a, 1, 'b').unwrap();
a.apply(op_a).unwrap();                    // replica A, offline: "abc"

// Reconcile: exchange operations in either order.
a.apply(op_b).unwrap();
b.apply(op_a).unwrap();
assert_eq!(a.text(), b.text());            // both converge to "azbc"

// A single owner now replays the converged text into edit-ledger as one
// ordinary transaction (see "What this is, and what it explicitly is
// not" above) — collaboration-core makes no call into edit-ledger itself.
```

### Relationship to `edit-ledger`, restated for revision 3

FT-044's objective is explicit that this is "no ... ledger replacement",
and that remains true after the consumer contract above: nothing in this
contract talks to `edit-ledger`, imports it, or duplicates its
responsibilities (`crates/edit-ledger`'s own public surface —
`Store::open`/`apply`/`replace_document`/`confirm`/`recovery`,
`Document::new(project_id, path, revision, text)` — is single-writer,
revision-guarded, durable, fsynced, with receipts and crash recovery; none
of that is reimplemented here, and `flashtex-collaboration-core` has zero
dependencies and no I/O of any kind). This crate only prepares converged
text that a consumer would *then* hand to `edit-ledger` as one ordinary
edit, per step 5 of the worked example above; whether and how a caller does
that wiring is exactly the "no consumer exists today" gap this section
reports rather than fills.

## Revision 4: expanded measured fixtures, adversarial bounds, and reproducible compatibility evidence

Revision 4's objective adds one clause on top of revision 3's scope
(unchanged): "Expand measured fixtures and adversarial bounds with
reproducible compatibility evidence." No public type or signature changed;
this revision is tests and documentation only. Test count: 44 -> 57 (13
new), all listed exactly in "Build and check" above.

### Expanded measured fixtures (`tests/measured_fixtures.rs`, new file, 3 tests)

Each fixture states its replica count and operation count as an assertion
computed from the fixture's own generated operations (not hand-typed), and
its converged result as a pinned `const` literal, checked across 2-3
independent, causally valid delivery orders per fixture:

- **Fixture 1** (`fixture_1_three_replica_word_merge_pinned_at_eight_ops`):
  3 replicas, 8 operations, converged result `"[dogcat]"`.
- **Fixture 2**
  (`fixture_2_five_replica_single_anchor_insert_with_delete_pinned_at_seven_ops`):
  5 replicas, 7 operations, converged result `"[srq]"`.
- **Fixture 3** (`fixture_3_five_replica_four_word_merge_pinned_at_ten_ops`):
  5 replicas, 10 operations, converged result `"[dwczbyax]"`.

These are additional to, not replacements for, `tests/convergence.rs`'s
revision-1 hand-derived cases and 120-permutation exhaustive sweep.

### Expanded adversarial bounds (`tests/adversarial.rs`, 9 -> 17 tests)

8 new cases, each still asserting a specific typed `Err`/`None` (or a
well-defined `Ok`) — never a panic — covering every category revision 4
called out beyond revision 3's 9:

- **Concurrent inserts at the identical anchor, from N replicas**
  (`concurrent_inserts_at_identical_anchor_from_five_replicas_produce_one_deterministic_total_order`):
  5 replicas insert into the same `(left, right)` gap; checked across 4
  distinct delivery orders that the result is the *specific* deterministic
  descending-`OpId` order (`"a65432c"`), not merely "some" consistent order.
- **Interleaved delete/insert on the same element**
  (`interleaved_delete_then_insert_on_the_same_element_converges_across_delivery_orders`):
  a delete of `'a'`, an insert anchored to `'a'`, and a delete of that new
  insert, across all 3 causally valid orderings of the 3 ops.
- **Replay of an entire op log, reversed and shuffled**
  (`replaying_the_full_op_log_in_reverse_order_is_idempotent`,
  `replaying_the_full_op_log_in_shuffled_order_is_idempotent`): an 8-op log
  (5 inserts, 1 concurrent insert, 2 deletes) replayed in full reverse
  order and in a fixed shuffle; every replayed op reports
  `ApplyOutcome::Duplicate` and the document is byte-unchanged, because
  dedup checks `applied.contains(&op.id)` before any dependency lookup.
- **Pending buffer at a zero bound**
  (`pending_buffer_zero_capacity_rejects_the_very_first_blocked_operation`):
  `PendingOps::new(0)` rejects the very first blocked operation with
  `PendingBufferFull { max_pending: 0 }` rather than buffering it
  momentarily — complementing revision 3's "fill to N, then one more"
  case with the degenerate zero-capacity bound.
- **Checkpoint bound crossed by document growth**
  (`checkpoint_bound_fixed_document_grows_from_within_bound_to_one_past_it`):
  a fixed `max_entries` bound; the document grows from exactly at the bound
  (succeeds) to exactly one past it (fails `TooLarge`) — complementing
  revision 3's "fixed document, shrinking bound" framing of the same edge.
- **Truncated bytes inside the delete-op-ids section specifically**
  (`checkpoint_bytes_truncated_inside_the_delete_op_ids_section_is_rejected`):
  revision 3's truncation cases hit the header or the element list; this
  one truncates 4 bytes out of a delete-op-id's own 16-byte encoding.
- **Counter at `u64::MAX` across two different replicas**
  (`two_replicas_both_at_u64_max_counter_coexist_without_id_conflict`):
  confirms `OpId` uniqueness is the pair `(counter, replica)`, not
  `counter` alone — two replicas both issuing `u64::MAX` coexist without
  `IdConflict`.

### Reproducible compatibility evidence (`tests/compatibility_evidence.rs`, new file, 2 tests)

- **A checkpoint written under this contract is readable byte-for-byte**
  (`checkpoint_bytes_for_a_fixed_fixture_are_pinned_byte_for_byte`): a
  fixed 2-insert-plus-1-delete fixture's `Checkpoint::to_bytes()` output is
  compared against a 102-byte literal pinned inline (with a field-by-field
  layout comment), and `Checkpoint::from_bytes` of that exact pinned
  literal is asserted to decode to a checkpoint equal to the one written.
  A future change to the wire encoding breaks this test with a literal
  byte mismatch, not just a passing round trip that could mask a
  compatible-looking change.
- **A converged document is byte-identical across every replica ordering,
  pinned, not just mutually equal**
  (`converged_document_is_byte_identical_across_replica_orderings_pinned`):
  3 replicas concurrently insert into an identical gap; all 6 permutations
  of delivery order are checked against a pinned literal (`"arqpc"`) *and*
  a pinned `u64` FNV-1a hash of its bytes (`7_532_153_511_834_115_664`) —
  two independent, checkable invariants rather than one. The hash function
  is a small, locally written, dependency-free implementation (no crate
  added), consistent with this crate's zero-dependency property.

No public API changed in revision 4; the "Typed contract" section above
and the worked example are unchanged and still accurate.

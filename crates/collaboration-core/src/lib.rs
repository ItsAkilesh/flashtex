//! `flashtex-collaboration-core`: an offline, bounded collaborative text
//! operation core.
//!
//! This crate gives two or more replicas of a small text (e.g. a note or
//! flashcard field) a way to edit **independently while offline** and later
//! reconcile their operations in any order, converging on an identical
//! result. It has no network I/O and no networking dependency: transport of
//! operations between replicas is entirely the caller's responsibility.
//!
//! It is explicitly **not** a replacement for `flashtex-edit-ledger`, which
//! owns durable, single-writer document storage, revision-guarded transaction
//! receipts, and crash recovery for one authoritative document. This crate
//! owns the opposite half of the problem: merging *concurrent* edits from
//! multiple replicas that were never serialized against a single revision
//! counter. A typical composition is: each replica keeps its own
//! [`Document`] while offline, and once reconciled a single owner replays the
//! converged text through the edit ledger as one ordinary edit.
//!
//! # Model
//!
//! Text is a sequence of Unicode scalar values (`char`). Every inserted
//! character gets a globally unique, causally-anchored [`OpId`] and is never
//! renumbered or moved by later operations. Deletion never removes a
//! character from the underlying structure; it only sets a tombstone flag.
//! Visible text is a pure filter over that structure, so insert/delete
//! ordering can never desynchronize replicas (see [`Document::apply`]).
//!
//! Insert operations carry the *left* and *right* neighbor IDs the inserting
//! replica observed at the time of insertion (a YATA-style anchored
//! sequence CRDT). Concurrent inserts anchored to the same
//! `(left, right)` gap are ordered deterministically by comparing their
//! [`OpId`]s, so any two replicas that have applied the same set of
//! operations compute the same final order regardless of arrival order.
//! See [`Document::apply`] for the precise integration algorithm and the
//! module-level tests / `tests/convergence.rs` for hand-worked fixtures.
//!
//! Because positions are expressed only as operation IDs or visible
//! character indices (never byte offsets), no operation can ever split a
//! multi-byte UTF-8 character: [`Document::text`] always returns a valid
//! `String` built by collecting whole `char`s.

use std::collections::HashSet;
use std::fmt;

/// Identifies one replica (site) participating in a collaboration session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReplicaId(pub u64);

/// A stable, globally unique identifier for one operation.
///
/// Field order matters for the derived [`Ord`]: `counter` is compared first
/// (each replica's own per-replica Lamport-style counter, incremented once
/// per operation it originates), and `replica` is the deterministic
/// tie-break when two different replicas produced operations with the same
/// counter value. This total order has no causal meaning by itself; it only
/// needs to be unique and consistent across replicas, which it is because
/// `(counter, replica)` pairs are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpId {
    pub counter: u64,
    pub replica: ReplicaId,
}

/// The payload of one operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpPayload {
    /// Insert `value` anchored between `left` and `right`, the neighbor IDs
    /// the originating replica observed at the time of insertion.
    /// `None` for `left` means "at the very start"; `None` for `right` means
    /// "at the very end" (as the originating replica saw the document).
    Insert {
        left: Option<OpId>,
        right: Option<OpId>,
        value: char,
    },
    /// Tombstone the character created by the insert operation `target`.
    Delete { target: OpId },
}

/// One collaboration operation: a stable ID plus its payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Op {
    pub id: OpId,
    pub payload: OpPayload,
}

/// Result of a successful [`Document::apply`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// The operation was new and has been integrated.
    Applied,
    /// This exact operation ID was already applied; nothing changed.
    Duplicate,
}

/// Failure modes of [`Document::apply`]. All are bounded, non-panicking
/// rejections of malformed or out-of-bounds input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrdtError {
    /// The operation references an `OpId` (as `left`, `right`, or `target`)
    /// that has not been applied to this document yet. Callers must deliver
    /// operations in causal order: an operation's dependencies must be
    /// applied before the operation itself.
    MissingDependency(OpId),
    /// This `OpId` was already used by a *different* insert (different
    /// `value`). IDs must be unique per originating replica/counter pair;
    /// reusing one with different content is rejected rather than silently
    /// accepted or overwritten.
    IdConflict(OpId),
    /// The document is already at its configured element bound
    /// ([`Document::with_max_elements`]); the insert was rejected instead of
    /// growing without limit.
    DocumentFull,
}

impl fmt::Display for CrdtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrdtError::MissingDependency(id) => {
                write!(f, "missing dependency: operation {id:?} not yet applied")
            }
            CrdtError::IdConflict(id) => {
                write!(f, "operation id {id:?} reused with different content")
            }
            CrdtError::DocumentFull => write!(f, "document is at its bounded element limit"),
        }
    }
}

impl std::error::Error for CrdtError {}

/// Default cap on total structural elements (including tombstones) a
/// [`Document`] will hold. This crate is for bounded, offline, small-scale
/// text (a note, a card field) reconciled between a handful of replicas, not
/// for large manuscripts; `flashtex-edit-ledger` owns durable storage of the
/// authoritative large document once reconciled.
pub const DEFAULT_MAX_ELEMENTS: usize = 200_000;

#[derive(Debug, Clone)]
struct Element {
    id: OpId,
    left: Option<OpId>,
    right: Option<OpId>,
    value: char,
    deleted: bool,
}

/// One replica's view of a collaboratively edited text.
///
/// Two `Document`s that started from the same state and have applied the
/// same set of operations (in any order, as long as each operation's
/// dependencies were applied first) always have identical [`text`](Document::text).
#[derive(Debug, Clone)]
pub struct Document {
    elements: Vec<Element>,
    applied: HashSet<OpId>,
    max_elements: usize,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    /// Create an empty document with the default bound
    /// ([`DEFAULT_MAX_ELEMENTS`]).
    pub fn new() -> Self {
        Self::with_max_elements(DEFAULT_MAX_ELEMENTS)
    }

    /// Create an empty document bounded to at most `max_elements`
    /// structural elements (visible characters plus tombstones).
    pub fn with_max_elements(max_elements: usize) -> Self {
        Self {
            elements: Vec::new(),
            applied: HashSet::new(),
            max_elements,
        }
    }

    /// The current visible text, in document order.
    pub fn text(&self) -> String {
        self.elements
            .iter()
            .filter(|e| !e.deleted)
            .map(|e| e.value)
            .collect()
    }

    /// Number of currently visible characters.
    pub fn len_chars(&self) -> usize {
        self.elements.iter().filter(|e| !e.deleted).count()
    }

    /// Whether there are no visible characters left.
    pub fn is_empty(&self) -> bool {
        self.len_chars() == 0
    }

    /// Whether `id` has already been applied to this document.
    pub fn is_applied(&self, id: OpId) -> bool {
        self.applied.contains(&id)
    }

    /// The stable ID of the visible character currently at `index`
    /// (0-based, counting only non-deleted characters), or `None` if
    /// `index >= len_chars()`. Because this indexes by character, not byte,
    /// it can never land inside a multi-byte UTF-8 sequence.
    pub fn char_id_at(&self, index: usize) -> Option<OpId> {
        self.elements
            .iter()
            .filter(|e| !e.deleted)
            .nth(index)
            .map(|e| e.id)
    }

    /// Structural position (including tombstones) of `id`, if applied.
    fn position_of(&self, id: OpId) -> Option<usize> {
        self.elements.iter().position(|e| e.id == id)
    }

    /// `-1` for `None`, else the structural index of the given id.
    /// Panics only if `id` is `Some` and not present, which callers must
    /// have already ruled out (see [`Document::apply`]'s dependency checks).
    fn signed_index(&self, id: Option<OpId>) -> isize {
        match id {
            None => -1,
            Some(id) => self
                .position_of(id)
                .expect("caller must validate dependencies before integrating") as isize,
        }
    }

    /// `elements.len()` for `None` (meaning "open end"), else the structural
    /// index of the given id.
    fn right_bound(&self, id: Option<OpId>) -> isize {
        match id {
            None => self.elements.len() as isize,
            Some(id) => self
                .position_of(id)
                .expect("caller must validate dependencies before integrating") as isize,
        }
    }

    /// YATA-style anchored integration: place `elem` in the unique
    /// structural position that every replica applying the same operation
    /// set will independently compute, regardless of application order.
    ///
    /// `elem` is scoped to the gap `(left, right)` it was created in. Among
    /// elements concurrently inserted into exactly the same gap, ties are
    /// broken by comparing `OpId` (larger id sorts closer to `left`); an
    /// element anchored to a strictly narrower/later sub-gap is skipped over
    /// as part of that gap's own contents rather than compared directly.
    fn integrate(&mut self, elem: Element) {
        let left_idx = self.signed_index(elem.left);
        let right_idx = self.right_bound(elem.right);
        let mut i = left_idx + 1;
        while i < right_idx {
            let existing = &self.elements[i as usize];
            let existing_left_idx = self.signed_index(existing.left);
            if existing_left_idx < left_idx {
                break;
            } else if existing_left_idx == left_idx {
                let existing_right_idx = self.right_bound(existing.right);
                match existing_right_idx.cmp(&right_idx) {
                    std::cmp::Ordering::Less => break,
                    std::cmp::Ordering::Equal => {
                        if existing.id > elem.id {
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    std::cmp::Ordering::Greater => i += 1,
                }
            } else {
                i += 1;
            }
        }
        self.elements.insert(i as usize, elem);
    }

    /// Apply one operation to this document.
    ///
    /// Returns [`ApplyOutcome::Duplicate`] without changing anything if
    /// `op.id` has already been applied (replay deduplication), so
    /// delivering the same operation twice — over an unreliable transport,
    /// say — is always safe. Returns [`CrdtError::MissingDependency`] if a
    /// referenced ID has not been applied yet: callers must deliver
    /// operations in causal order (an operation's `left`/`right`/`target`
    /// must already be applied).
    pub fn apply(&mut self, op: Op) -> Result<ApplyOutcome, CrdtError> {
        if self.applied.contains(&op.id) {
            if let OpPayload::Insert { value, .. } = &op.payload {
                if let Some(existing) = self.elements.iter().find(|e| e.id == op.id) {
                    if existing.value != *value {
                        return Err(CrdtError::IdConflict(op.id));
                    }
                }
            }
            return Ok(ApplyOutcome::Duplicate);
        }

        match op.payload {
            OpPayload::Insert { left, right, value } => {
                if let Some(l) = left {
                    if self.position_of(l).is_none() {
                        return Err(CrdtError::MissingDependency(l));
                    }
                }
                if let Some(r) = right {
                    if self.position_of(r).is_none() {
                        return Err(CrdtError::MissingDependency(r));
                    }
                }
                if self.elements.len() >= self.max_elements {
                    return Err(CrdtError::DocumentFull);
                }
                self.integrate(Element {
                    id: op.id,
                    left,
                    right,
                    value,
                    deleted: false,
                });
            }
            OpPayload::Delete { target } => {
                let pos = self
                    .position_of(target)
                    .ok_or(CrdtError::MissingDependency(target))?;
                self.elements[pos].deleted = true;
            }
        }

        self.applied.insert(op.id);
        Ok(ApplyOutcome::Applied)
    }
}

/// Convenience generator of well-formed operations anchored to visible
/// character indices, for one replica. Using this (rather than hand-built
/// `Op`s) guarantees `left`/`right` are always real neighbor IDs or `None`,
/// and that positions are expressed in characters, never bytes.
#[derive(Debug, Clone)]
pub struct OpBuilder {
    replica: ReplicaId,
    counter: u64,
}

impl OpBuilder {
    pub fn new(replica: ReplicaId) -> Self {
        Self { replica, counter: 0 }
    }

    fn next_id(&mut self) -> OpId {
        self.counter += 1;
        OpId {
            counter: self.counter,
            replica: self.replica,
        }
    }

    /// Build an insert of `value` so that, from `doc`'s current point of
    /// view, it becomes the character at visible index `index` (0-based;
    /// `index == doc.len_chars()` appends at the end). Returns `None` if
    /// `index > doc.len_chars()` rather than panicking or clamping.
    pub fn insert_at(&mut self, doc: &Document, index: usize, value: char) -> Option<Op> {
        if index > doc.len_chars() {
            return None;
        }
        let left = if index == 0 {
            None
        } else {
            doc.char_id_at(index - 1)
        };
        let right = doc.char_id_at(index);
        Some(Op {
            id: self.next_id(),
            payload: OpPayload::Insert { left, right, value },
        })
    }

    /// Build a delete of the visible character currently at `index`.
    /// Returns `None` if `index >= doc.len_chars()`.
    pub fn delete_at(&mut self, doc: &Document, index: usize) -> Option<Op> {
        let target = doc.char_id_at(index)?;
        Some(Op {
            id: self.next_id(),
            payload: OpPayload::Delete { target },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(n: u64) -> ReplicaId {
        ReplicaId(n)
    }

    #[test]
    fn single_replica_sequential_build() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        for (i, ch) in "hello".chars().enumerate() {
            let op = b.insert_at(&doc, i, ch).unwrap();
            assert_eq!(doc.apply(op).unwrap(), ApplyOutcome::Applied);
        }
        assert_eq!(doc.text(), "hello");
        assert_eq!(doc.len_chars(), 5);
    }

    #[test]
    fn delete_is_tombstoned_not_removed() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        for (i, ch) in "cat".chars().enumerate() {
            doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
        }
        let del = b.delete_at(&doc, 1).unwrap(); // delete 'a'
        doc.apply(del).unwrap();
        assert_eq!(doc.text(), "ct");
        assert_eq!(doc.len_chars(), 2);
    }

    // --- Replay deduplication -------------------------------------------

    #[test]
    fn replaying_the_same_insert_is_a_no_op() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        let op = b.insert_at(&doc, 0, 'x').unwrap();
        assert_eq!(doc.apply(op).unwrap(), ApplyOutcome::Applied);
        assert_eq!(doc.text(), "x");
        // Redeliver the identical operation (e.g. an at-least-once transport).
        assert_eq!(doc.apply(op).unwrap(), ApplyOutcome::Duplicate);
        assert_eq!(doc.text(), "x", "duplicate delivery must not double-insert");
        assert_eq!(doc.len_chars(), 1);
    }

    #[test]
    fn replaying_the_same_delete_is_a_no_op() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        doc.apply(b.insert_at(&doc, 0, 'x').unwrap()).unwrap();
        let del = b.delete_at(&doc, 0).unwrap();
        assert_eq!(doc.apply(del).unwrap(), ApplyOutcome::Applied);
        assert_eq!(doc.text(), "");
        assert_eq!(doc.apply(del).unwrap(), ApplyOutcome::Duplicate);
        assert_eq!(doc.text(), "");
    }

    #[test]
    fn reusing_an_id_with_different_value_is_rejected() {
        let mut doc = Document::new();
        let id = OpId {
            counter: 1,
            replica: r(1),
        };
        doc.apply(Op {
            id,
            payload: OpPayload::Insert {
                left: None,
                right: None,
                value: 'a',
            },
        })
        .unwrap();
        let conflicting = Op {
            id,
            payload: OpPayload::Insert {
                left: None,
                right: None,
                value: 'b',
            },
        };
        assert_eq!(doc.apply(conflicting), Err(CrdtError::IdConflict(id)));
        assert_eq!(doc.text(), "a", "the original content must be untouched");
    }

    // --- Malformed / bounded input ---------------------------------------

    #[test]
    fn insert_with_unknown_left_is_rejected_not_panicking() {
        let mut doc = Document::new();
        let ghost = OpId {
            counter: 99,
            replica: r(7),
        };
        let op = Op {
            id: OpId {
                counter: 1,
                replica: r(1),
            },
            payload: OpPayload::Insert {
                left: Some(ghost),
                right: None,
                value: 'z',
            },
        };
        assert_eq!(doc.apply(op), Err(CrdtError::MissingDependency(ghost)));
        assert_eq!(doc.text(), "");
    }

    #[test]
    fn delete_of_unknown_target_is_rejected_not_panicking() {
        let mut doc = Document::new();
        let ghost = OpId {
            counter: 5,
            replica: r(2),
        };
        let op = Op {
            id: OpId {
                counter: 1,
                replica: r(1),
            },
            payload: OpPayload::Delete { target: ghost },
        };
        assert_eq!(doc.apply(op), Err(CrdtError::MissingDependency(ghost)));
    }

    #[test]
    fn out_of_range_builder_indices_return_none_not_panic() {
        let doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        assert!(b.insert_at(&doc, 5, 'x').is_none());
        assert!(b.delete_at(&doc, 0).is_none());
    }

    #[test]
    fn malformed_left_right_ordering_does_not_panic() {
        // A deliberately nonsensical operation where `right` structurally
        // precedes `left`. This crate does not validate gap consistency
        // beyond "both referenced ids exist"; it must still never panic and
        // must place the element at a definite, bounded position.
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        for (i, ch) in "ab".chars().enumerate() {
            doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
        }
        let a_id = doc.char_id_at(0).unwrap();
        let b_id = doc.char_id_at(1).unwrap();
        let op = Op {
            id: OpId {
                counter: 100,
                replica: r(9),
            },
            payload: OpPayload::Insert {
                left: Some(b_id),
                right: Some(a_id),
                value: 'z',
            },
        };
        // Must not panic; must still be a well-formed, boundedly-placed document.
        assert_eq!(doc.apply(op), Ok(ApplyOutcome::Applied));
        assert_eq!(doc.len_chars(), 3);
    }

    #[test]
    fn document_full_is_rejected_not_grown_unbounded() {
        let mut doc = Document::with_max_elements(2);
        let mut b = OpBuilder::new(r(1));
        doc.apply(b.insert_at(&doc, 0, 'a').unwrap()).unwrap();
        doc.apply(b.insert_at(&doc, 1, 'b').unwrap()).unwrap();
        let op = b.insert_at(&doc, 2, 'c').unwrap();
        assert_eq!(doc.apply(op), Err(CrdtError::DocumentFull));
        assert_eq!(doc.text(), "ab");
    }

    // --- UTF-8 / Unicode safety -------------------------------------------

    #[test]
    fn multibyte_characters_round_trip_exactly() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        // Mix of 1, 2, 3 and 4-byte UTF-8 scalars, plus a zero-width joiner.
        let source = "a\u{00e9}\u{4e2d}\u{1f389}\u{200d}b";
        for (i, ch) in source.chars().enumerate() {
            doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
        }
        assert_eq!(doc.text(), source);
        assert_eq!(doc.len_chars(), source.chars().count());
        // text() must always be valid UTF-8 by construction (String::from(char)).
        assert!(std::str::from_utf8(doc.text().as_bytes()).is_ok());
    }

    #[test]
    fn deleting_around_multibyte_characters_never_splits_one() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        let source = "x\u{4e2d}\u{1f389}y"; // x, CJK char, emoji, y
        for (i, ch) in source.chars().enumerate() {
            doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
        }
        // Delete the CJK character (index 1) and the emoji (now index 1).
        doc.apply(b.delete_at(&doc, 1).unwrap()).unwrap();
        doc.apply(b.delete_at(&doc, 1).unwrap()).unwrap();
        assert_eq!(doc.text(), "xy");
        for ch in doc.text().chars() {
            assert!(ch.len_utf8() >= 1); // every remaining scalar is intact
        }
    }

    #[test]
    fn insert_at_index_between_multibyte_neighbors() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        for (i, ch) in "\u{1f600}\u{1f601}".chars().enumerate() {
            doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
        }
        // Insert between two 4-byte emoji; must land exactly between them,
        // never inside either one's byte sequence.
        let op = b.insert_at(&doc, 1, 'X').unwrap();
        doc.apply(op).unwrap();
        assert_eq!(doc.text(), "\u{1f600}X\u{1f601}");
    }
}

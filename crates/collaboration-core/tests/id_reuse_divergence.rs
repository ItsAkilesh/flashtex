//! Does reusing one `OpId` across two *different* operations diverge replicas?
//!
//! `Document::apply` is public and `Op` is fully public, so operations arriving
//! from a peer are attacker- or bug-controlled. `CrdtError::IdConflict` exists
//! precisely to catch an ID reused with different content. The question these
//! tests pin down is whether that guard covers every combination, because a
//! combination it misses produces *silent divergence*: two replicas that saw
//! the same operation set in different orders end up with different text and
//! neither reports an error. For a CRDT whose entire contract is convergence,
//! that is the worst possible failure mode.

use flashtex_collaboration_core::{
    ApplyOutcome, CrdtError, Document, Op, OpId, OpPayload, ReplicaId,
};

fn r(n: u8) -> ReplicaId {
    ReplicaId(n as u64)
}

fn ins(id: OpId, left: Option<OpId>, right: Option<OpId>, value: char) -> Op {
    Op {
        id,
        payload: OpPayload::Insert { left, right, value },
    }
}

fn del(id: OpId, target: OpId) -> Op {
    Op {
        id,
        payload: OpPayload::Delete { target },
    }
}

fn oid(counter: u64, replica: u8) -> OpId {
    OpId {
        counter,
        replica: r(replica),
    }
}

/// The decisive experiment. One ID, two different payloads, two delivery
/// orders. If the two documents disagree and neither errored, convergence is
/// broken silently.
#[test]
fn same_id_two_payloads_two_orders_must_not_diverge_silently() {
    let seed = oid(1, 1);
    let reused = oid(2, 1);

    // Replica A: the Delete lands first, then the Insert reusing its ID.
    let mut a = Document::new();
    a.apply(ins(seed, None, None, 'a')).unwrap();
    let a_first = a.apply(del(reused, seed));
    let a_second = a.apply(ins(reused, None, None, 'z'));

    // Replica B: the same two operations, opposite order.
    let mut b = Document::new();
    b.apply(ins(seed, None, None, 'a')).unwrap();
    let b_first = b.apply(ins(reused, None, None, 'z'));
    let b_second = b.apply(del(reused, seed));

    let diverged = a.text() != b.text();
    let errored = a_first.is_err() || a_second.is_err() || b_first.is_err() || b_second.is_err();

    println!("A: {a_first:?} then {a_second:?} -> text {:?}", a.text());
    println!("B: {b_first:?} then {b_second:?} -> text {:?}", b.text());
    println!("diverged={diverged} errored={errored}");

    assert!(
        !(diverged && !errored),
        "SILENT DIVERGENCE: A={:?} B={:?}, no error from any apply",
        a.text(),
        b.text()
    );
}

/// Delete-then-Insert on one ID: the reused Insert must not be swallowed.
#[test]
fn delete_then_insert_reusing_the_id_is_a_conflict_not_a_duplicate() {
    let seed = oid(1, 1);
    let reused = oid(2, 1);
    let mut doc = Document::new();
    doc.apply(ins(seed, None, None, 'a')).unwrap();
    doc.apply(del(reused, seed)).unwrap();
    assert_eq!(
        doc.apply(ins(reused, None, None, 'z')),
        Err(CrdtError::IdConflict(reused))
    );
}

/// Insert-then-Delete on one ID: likewise.
#[test]
fn insert_then_delete_reusing_the_id_is_a_conflict_not_a_duplicate() {
    let seed = oid(1, 1);
    let reused = oid(2, 1);
    let mut doc = Document::new();
    doc.apply(ins(seed, None, None, 'a')).unwrap();
    doc.apply(ins(reused, None, None, 'b')).unwrap();
    assert_eq!(
        doc.apply(del(reused, seed)),
        Err(CrdtError::IdConflict(reused))
    );
}

/// Two Deletes sharing an ID but naming different targets.
///
/// KNOWN GAP, deliberately not fixed here. Detecting this requires storing
/// each applied delete's target, and the checkpoint wire format currently
/// serializes `delete_op_ids` as bare ids with no target. Adding targets is a
/// serialization-compatibility change: it would alter the pinned byte layout
/// that `compatibility_evidence.rs` asserts, and any checkpoint written by an
/// older build would no longer round-trip. That decision belongs to the
/// Commander, not to this lane, so the gap is pinned as an ignored test rather
/// than silently omitted - the test documents the exact behaviour and will
/// start failing the moment someone closes it.
#[test]
#[ignore = "needs delete targets in the checkpoint wire format; see coordination/daniel-collaboration.md"]
fn delete_then_different_delete_reusing_the_id_is_a_conflict() {
    let first = oid(1, 1);
    let second = oid(2, 1);
    let reused = oid(3, 1);
    let mut doc = Document::new();
    doc.apply(ins(first, None, None, 'a')).unwrap();
    doc.apply(ins(second, Some(first), None, 'b')).unwrap();
    doc.apply(del(reused, first)).unwrap();
    assert_eq!(
        doc.apply(del(reused, second)),
        Err(CrdtError::IdConflict(reused))
    );
}

/// The other direction must keep working: a genuinely identical replayed
/// operation stays idempotent. Causal-delivery recovery replays whole op logs,
/// so breaking this would be worse than the bug.
#[test]
fn identical_replay_remains_idempotent_for_both_payload_kinds() {
    let seed = oid(1, 1);
    let d = oid(2, 1);
    let mut doc = Document::new();
    doc.apply(ins(seed, None, None, 'a')).unwrap();
    assert_eq!(
        doc.apply(ins(seed, None, None, 'a')).unwrap(),
        ApplyOutcome::Duplicate
    );
    doc.apply(del(d, seed)).unwrap();
    assert_eq!(doc.apply(del(d, seed)).unwrap(), ApplyOutcome::Duplicate);
    assert_eq!(doc.text(), "");
}

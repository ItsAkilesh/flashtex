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
    ApplyOutcome, Checkpoint, CrdtError, Document, Op, OpId, OpPayload, ReplicaId,
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
/// Previously a KNOWN GAP, pinned as an ignored test rather than silently
/// omitted: closing it needed each applied delete's target stored somewhere,
/// and the checkpoint wire format used to serialize deletes as bare ids with
/// no target. GH#23 approved a v2 wire-format bump (see `src/checkpoint.rs`)
/// that pairs each delete-operation id with its target, and `Document` now
/// tracks that same target in memory independent of any checkpoint, so this
/// is closed for good: two deletes sharing an id with different targets is
/// now a conflict, not a silent duplicate.
#[test]
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

/// The same conflict must still be detected after a round trip through the
/// v2 checkpoint wire format: the known target has to survive
/// serialization, not just live in the original `Document`'s in-memory map.
#[test]
fn delete_target_mismatch_is_still_a_conflict_after_a_checkpoint_round_trip() {
    let first = oid(1, 1);
    let second = oid(2, 1);
    let reused = oid(3, 1);
    let mut doc = Document::new();
    doc.apply(ins(first, None, None, 'a')).unwrap();
    doc.apply(ins(second, Some(first), None, 'b')).unwrap();
    doc.apply(del(reused, first)).unwrap();

    let bytes = doc.checkpoint(10).unwrap().to_bytes();
    let mut restored = Document::from(Checkpoint::from_bytes(&bytes).unwrap());

    assert_eq!(
        restored.apply(del(reused, second)),
        Err(CrdtError::IdConflict(reused)),
        "the v2 checkpoint must carry the known target through the round trip"
    );
}

/// A delete restored from a *legacy* v1 checkpoint has an unknown target -
/// v1 never carried one - so it cannot be proven to conflict with a
/// same-id delete naming a different target. That must still behave as an
/// idempotent replay rather than erroring, exactly as it did before v2
/// existed.
#[test]
fn delete_with_unknown_target_from_a_legacy_checkpoint_stays_idempotent() {
    // Hand-assembled v1 bytes (no v2 magic prefix, bare delete ids): one
    // element 'x' (already tombstoned, standing in for a document that
    // applied and then serialized a delete before v2 existed), plus that
    // delete's bare id in the frontier.
    let el_id = oid(1, 1);
    let del_id = oid(2, 1);
    let mut v1_bytes = Vec::new();
    v1_bytes.extend_from_slice(&100u64.to_le_bytes()); // max_elements
    v1_bytes.extend_from_slice(&1u64.to_le_bytes()); // element_count
    v1_bytes.extend_from_slice(&el_id.counter.to_le_bytes());
    v1_bytes.extend_from_slice(&el_id.replica.0.to_le_bytes());
    v1_bytes.push(0); // left = None
    v1_bytes.push(0); // right = None
    v1_bytes.extend_from_slice(&('x' as u32).to_le_bytes());
    v1_bytes.push(1); // deleted = true
    v1_bytes.extend_from_slice(&1u64.to_le_bytes()); // delete_count
    v1_bytes.extend_from_slice(&del_id.counter.to_le_bytes());
    v1_bytes.extend_from_slice(&del_id.replica.0.to_le_bytes());

    let cp = Checkpoint::from_bytes(&v1_bytes).expect("legacy v1 bytes must still decode");
    let mut restored = Document::from(cp);
    assert_eq!(restored.text(), "");

    // Re-applying `del_id` with a target that does not match what actually
    // deleted 'x' must NOT error: the restored document never learned the
    // original target, so it cannot prove a conflict.
    let different_target = oid(99, 5);
    assert_eq!(
        restored.apply(del(del_id, different_target)),
        Ok(ApplyOutcome::Duplicate),
        "an unknown (legacy) delete target must be treated as a wildcard, not a conflict"
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

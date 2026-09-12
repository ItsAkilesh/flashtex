//! Integration-level test for FT-044 revision 2's chosen interrupted-delivery
//! recovery strategy: `PendingOps` buffers operations that arrive before
//! their causal dependency instead of the caller having to discard them and
//! start over. `Document::apply` itself is unchanged from revision 1 (still
//! `MissingDependency`, still no buffering) - `PendingOps` is a purely
//! additive layer on top of it. See `src/recovery.rs` for mechanism-level
//! unit tests (single-gap buffering, multi-level chains, the buffer bound,
//! idempotent redelivery, `IdConflict` still surfacing through `receive`).
//! This file exercises it end to end in a disconnect-and-catch-up scenario.

use flashtex_collaboration_core::{ApplyOutcome, Document, OpBuilder, PendingOps, ReplicaId};

#[test]
fn interrupted_backlog_recovers_to_the_same_state_as_in_order_delivery() {
    // Replica R1 builds "hello" sequentially and deletes one character;
    // this is the backlog a peer receives after reconnecting.
    let mut source = Document::new();
    let mut b = OpBuilder::new(ReplicaId(1));
    let mut backlog = Vec::new();
    for (i, ch) in "hello".chars().enumerate() {
        let op = b.insert_at(&source, i, ch).unwrap();
        source.apply(op).unwrap();
        backlog.push(op);
    }
    let del = b.delete_at(&source, 1).unwrap(); // delete 'e'
    source.apply(del).unwrap();
    backlog.push(del);
    assert_eq!(source.text(), "hllo");

    // The transport delivers the backlog scrambled (reversed), not as a
    // clean in-order prefix.
    let mut scrambled = backlog.clone();
    scrambled.reverse();

    let mut peer = Document::new();
    let mut pending = PendingOps::new(backlog.len());
    let mut buffered_at_least_once = false;
    for op in scrambled {
        match pending.receive(&mut peer, op).unwrap() {
            ApplyOutcome::Buffered => buffered_at_least_once = true,
            ApplyOutcome::Applied | ApplyOutcome::Duplicate => {}
        }
    }

    assert!(
        buffered_at_least_once,
        "reversed delivery must actually exercise buffering, not accidentally arrive in order"
    );
    assert!(pending.is_empty(), "the full backlog must eventually drain");
    assert_eq!(peer.text(), source.text());
    assert_eq!(peer.text(), "hllo");
}

#[test]
fn missing_dependencies_reports_exactly_the_blocking_ids_for_redelivery() {
    let mut probe = Document::new();
    let mut b = OpBuilder::new(ReplicaId(1));
    let op_a = b.insert_at(&probe, 0, 'a').unwrap();
    probe.apply(op_a).unwrap();
    let op_b = b.insert_at(&probe, 1, 'b').unwrap();
    probe.apply(op_b).unwrap();
    let op_c = b.insert_at(&probe, 2, 'c').unwrap();
    probe.apply(op_c).unwrap();

    let mut peer = Document::new();
    let mut pending = PendingOps::new(10);

    // Peer receives only the tail of the chain first.
    pending.receive(&mut peer, op_c).unwrap();
    assert_eq!(pending.missing_dependencies(&peer), vec![op_b.id]);

    // The caller requests exactly that id and it arrives - but it too
    // depends on something not yet delivered, which is now also surfaced.
    pending.receive(&mut peer, op_b).unwrap();
    let missing: std::collections::HashSet<_> =
        pending.missing_dependencies(&peer).into_iter().collect();
    assert_eq!(missing, [op_a.id, op_b.id].into_iter().collect());

    // Finally the root arrives and the whole chain drains.
    pending.receive(&mut peer, op_a).unwrap();
    assert!(pending.missing_dependencies(&peer).is_empty());
    assert!(pending.is_empty());
    assert_eq!(peer.text(), "abc");
}

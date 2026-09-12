//! Revision 3: bounded adversarial edge cases beyond revision 1's convergence
//! fixtures and revision 2's checkpoint/recovery equivalence proofs.
//!
//! Every case here is a deliberately hostile or boundary input the crate
//! must handle with a typed `Err`/`None` result (or, where the input is
//! merely unusual rather than invalid, a well-defined success) — never a
//! panic. `counter`-exhaustion is covered separately in `src/lib.rs`'s unit
//! tests because it requires constructing an `OpBuilder` at a specific
//! private counter value.

use flashtex_collaboration_core::{
    ApplyOutcome, Checkpoint, CheckpointDecodeError, CheckpointError, CrdtError, Document, Op,
    OpBuilder, OpId, OpPayload, PendingOps, ReplicaId,
};

fn r(n: u64) -> ReplicaId {
    ReplicaId(n)
}

// --- An operation referencing an unknown replica -------------------------

#[test]
fn insert_referencing_an_id_from_a_replica_that_never_produced_any_operation_is_rejected() {
    let mut doc = Document::new();
    // Replica 999 has never applied anything to `doc`; its claimed id is
    // referenced as a neighbor by an insert from a different, real replica.
    let unknown = OpId {
        counter: 1,
        replica: r(999),
    };
    let op = Op {
        id: OpId {
            counter: 1,
            replica: r(1),
        },
        payload: OpPayload::Insert {
            left: Some(unknown),
            right: None,
            value: 'z',
        },
    };
    assert_eq!(doc.apply(op), Err(CrdtError::MissingDependency(unknown)));
    assert_eq!(doc.text(), "", "rejected op must not be partially applied");
}

#[test]
fn delete_referencing_a_target_from_an_unknown_replica_is_rejected() {
    let mut doc = Document::new();
    let unknown = OpId {
        counter: 42,
        replica: r(777),
    };
    let op = Op {
        id: OpId {
            counter: 1,
            replica: r(1),
        },
        payload: OpPayload::Delete { target: unknown },
    };
    assert_eq!(doc.apply(op), Err(CrdtError::MissingDependency(unknown)));
}

// --- Pending buffer at and past capacity ----------------------------------

#[test]
fn pending_buffer_exactly_at_capacity_succeeds_next_one_is_rejected() {
    let mut doc = Document::new();
    let max_pending = 3;
    let mut pending = PendingOps::new(max_pending);

    // Three operations that each reference a distinct id nothing will ever
    // supply, so all three stay buffered at once.
    for i in 0..max_pending as u64 {
        let ghost = OpId {
            counter: i,
            replica: r(500),
        };
        let op = Op {
            id: OpId {
                counter: i,
                replica: r(1),
            },
            payload: OpPayload::Insert {
                left: Some(ghost),
                right: None,
                value: 'x',
            },
        };
        assert_eq!(
            pending.receive(&mut doc, op),
            Ok(ApplyOutcome::Buffered),
            "buffering op {i} should still be within capacity"
        );
    }
    assert_eq!(pending.pending_len(), max_pending);

    // The bound-th-plus-one operation must be rejected with a typed error,
    // not silently dropped or grown past the configured bound.
    let overflow = Op {
        id: OpId {
            counter: 100,
            replica: r(1),
        },
        payload: OpPayload::Insert {
            left: Some(OpId {
                counter: 999,
                replica: r(500),
            }),
            right: None,
            value: 'y',
        },
    };
    assert_eq!(
        pending.receive(&mut doc, overflow),
        Err(CrdtError::PendingBufferFull { max_pending })
    );
    assert_eq!(pending.pending_len(), max_pending, "buffer must not grow past its bound");
}

// --- Checkpoint at and past its entry bound -------------------------------

#[test]
fn checkpoint_exactly_at_bound_succeeds_one_less_bound_fails() {
    let mut doc = Document::new();
    let mut b = OpBuilder::new(r(1));
    for (i, ch) in "hello".chars().enumerate() {
        doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
    }
    // 5 elements, 0 delete-op ids -> entry_count() == 5.
    let at_bound = doc.checkpoint(5).expect("exactly at the bound must succeed");
    assert_eq!(at_bound.entry_count(), 5);

    assert_eq!(
        doc.checkpoint(4),
        Err(CheckpointError::TooLarge {
            entries: 5,
            max_entries: 4
        }),
        "one below the bound must be a typed error, not a truncated snapshot"
    );
}

// --- Truncated and corrupted checkpoint bytes -----------------------------

#[test]
fn checkpoint_bytes_with_declared_counts_far_exceeding_actual_data_is_rejected() {
    // A header claiming millions of elements, backed by no element data at
    // all. Must fail with `Truncated` on the first missing field rather than
    // attempting to allocate or read anything unbounded.
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0u64.to_le_bytes()); // max_elements
    bytes.extend_from_slice(&u64::MAX.to_le_bytes()); // "element_count": huge lie
    assert_eq!(
        Checkpoint::from_bytes(&bytes),
        Err(CheckpointDecodeError::Truncated)
    );
}

#[test]
fn checkpoint_bytes_corrupted_with_arbitrary_patterns_never_panics() {
    let patterns: &[&[u8]] = &[
        &[],
        &[0x00],
        &[0xFF],
        &[0xFF; 7],
        &[0xFF; 40],
        &[0x00; 40],
        &[0xAB, 0xCD, 0xEF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
    ];
    for pattern in patterns {
        // The only contract under test: decoding never panics. Whatever
        // `Result` comes back (most of these are not valid checkpoints) is
        // acceptable as long as it is a `Result`, not a crash.
        let result = std::panic::catch_unwind(|| Checkpoint::from_bytes(pattern));
        assert!(result.is_ok(), "from_bytes panicked on pattern {pattern:?}");
    }
}

#[test]
fn checkpoint_bytes_with_a_flipped_byte_in_the_middle_is_rejected_or_harmlessly_decoded() {
    let mut doc = Document::new();
    let mut b = OpBuilder::new(r(1));
    for (i, ch) in "hello world".chars().enumerate() {
        doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
    }
    let cp = doc.checkpoint(100).unwrap();
    let mut bytes = cp.to_bytes();
    // Flip a byte roughly in the middle of the encoded element data. This
    // may still decode (some fields tolerate arbitrary values) or may not;
    // either way it must never panic.
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    let result = std::panic::catch_unwind(|| Checkpoint::from_bytes(&bytes));
    assert!(result.is_ok(), "from_bytes must never panic on corrupted bytes");
}

// --- Insert whose position references a deleted element ------------------

#[test]
fn insert_anchored_to_a_tombstoned_element_still_integrates_deterministically() {
    let mut doc = Document::new();
    let mut b = OpBuilder::new(r(1));
    for (i, ch) in "ac".chars().enumerate() {
        doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
    }
    let a_id = doc.char_id_at(0).unwrap();
    let c_id = doc.char_id_at(1).unwrap();

    // Delete 'a'. It stays structurally present as a tombstone.
    doc.apply(b.delete_at(&doc, 0).unwrap()).unwrap();
    assert_eq!(doc.text(), "c");

    // A concurrent insert anchored to the now-deleted 'a' as `left` (e.g. a
    // replica that observed 'a' before the delete arrived) must still
    // integrate at a definite position, not be rejected as "missing" and not
    // panic — a tombstone is a structurally present element, not an absent
    // one.
    let op = Op {
        id: OpId {
            counter: 100,
            replica: r(2),
        },
        payload: OpPayload::Insert {
            left: Some(a_id),
            right: Some(c_id),
            value: 'X',
        },
    };
    assert_eq!(doc.apply(op), Ok(ApplyOutcome::Applied));
    assert_eq!(doc.text(), "Xc", "X lands in the gap after the tombstoned 'a', before 'c'");
    assert_eq!(doc.len_chars(), 2, "the tombstoned 'a' is not counted as visible");
}

#[test]
fn delete_of_an_already_deleted_element_is_idempotent_not_double_counted() {
    // Deleting the same target twice, via two distinct delete operations
    // (not a replayed duplicate id), must not panic or under/over-count.
    let mut doc = Document::new();
    let mut b = OpBuilder::new(r(1));
    doc.apply(b.insert_at(&doc, 0, 'a').unwrap()).unwrap();
    let a_id = doc.char_id_at(0).unwrap();

    let del1 = Op {
        id: OpId {
            counter: 10,
            replica: r(2),
        },
        payload: OpPayload::Delete { target: a_id },
    };
    let del2 = Op {
        id: OpId {
            counter: 11,
            replica: r(3),
        },
        payload: OpPayload::Delete { target: a_id },
    };
    assert_eq!(doc.apply(del1), Ok(ApplyOutcome::Applied));
    assert_eq!(doc.apply(del2), Ok(ApplyOutcome::Applied));
    assert_eq!(doc.text(), "");
    assert_eq!(doc.len_chars(), 0);
}

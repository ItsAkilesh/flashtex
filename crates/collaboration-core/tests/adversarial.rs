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
    assert_eq!(
        pending.pending_len(),
        max_pending,
        "buffer must not grow past its bound"
    );
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
    let at_bound = doc
        .checkpoint(5)
        .expect("exactly at the bound must succeed");
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
    assert!(
        result.is_ok(),
        "from_bytes must never panic on corrupted bytes"
    );
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
    assert_eq!(
        doc.text(),
        "Xc",
        "X lands in the gap after the tombstoned 'a', before 'c'"
    );
    assert_eq!(
        doc.len_chars(),
        2,
        "the tombstoned 'a' is not counted as visible"
    );
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

// ==========================================================================
// Revision 4: expanded adversarial bounds (beyond revision 3's 9 cases).
// Every case below still asserts a specific typed `Err`/`None` result (or,
// where noted, a well-defined `Ok`) — never a panic.
// ==========================================================================

// --- Concurrent inserts at the identical anchor from N replicas -----------

#[test]
fn concurrent_inserts_at_identical_anchor_from_five_replicas_produce_one_deterministic_total_order()
{
    // Base "ac"; replicas 2..=6 (five replicas) each concurrently insert a
    // digit character equal to their own replica id, anchored to the
    // identical gap (left = 'a', right = 'c'). Every op shares counter 1,
    // so the total order is decided purely by comparing `OpId.replica`:
    // per the documented tie-break, the *largest* id sorts closest to
    // `left`, so the deterministic order is descending replica id:
    // 6, 5, 4, 3, 2 — not merely "some" consistent order, this exact one.
    let mut base = Document::new();
    let mut bb = OpBuilder::new(r(1));
    base.apply(bb.insert_at(&base, 0, 'a').unwrap()).unwrap();
    base.apply(bb.insert_at(&base, 1, 'c').unwrap()).unwrap();
    let a_id = base.char_id_at(0).unwrap();
    let c_id = base.char_id_at(1).unwrap();

    let ops: Vec<Op> = (2u64..=6)
        .map(|replica| Op {
            id: OpId {
                counter: 1,
                replica: r(replica),
            },
            payload: OpPayload::Insert {
                left: Some(a_id),
                right: Some(c_id),
                value: char::from_digit(replica as u32, 10).unwrap(),
            },
        })
        .collect();
    assert_eq!(ops.len(), 5, "pinned replica count for this case");

    const EXPECTED: &str = "a65432c";

    let orders: [[usize; 5]; 4] = [
        [0, 1, 2, 3, 4], // ascending replica (generation order)
        [4, 3, 2, 1, 0], // descending replica
        [2, 0, 4, 1, 3], // arbitrary shuffle
        [1, 4, 0, 3, 2], // another arbitrary shuffle
    ];
    for order in orders {
        let mut doc = base.clone();
        for idx in order {
            doc.apply(ops[idx]).unwrap();
        }
        assert_eq!(
            doc.text(),
            EXPECTED,
            "delivery order {order:?} must not change the deterministic total order"
        );
    }
}

// --- Interleaved delete/insert on the same element -------------------------

#[test]
fn interleaved_delete_then_insert_on_the_same_element_converges_across_delivery_orders() {
    // Base "ac". Replica 2 deletes 'a'. Concurrently, replica 3 inserts an
    // 'X' anchored (left = a_id, right = c_id) — i.e. anchored to the very
    // element replica 2 is concurrently deleting. Replica 4 then deletes
    // that same 'X' as a third, independent concurrent operation (it only
    // depends on X's id, which is why generating it does not require
    // either of the other two operations to have been delivered anywhere
    // yet). Every causally valid delivery order (X's insert must precede
    // its own delete) must converge on the same result: 'a' tombstoned,
    // 'X' inserted then also tombstoned, 'c' untouched -> visible text "c".
    let mut base = Document::new();
    let mut bb = OpBuilder::new(r(1));
    base.apply(bb.insert_at(&base, 0, 'a').unwrap()).unwrap();
    base.apply(bb.insert_at(&base, 1, 'c').unwrap()).unwrap();
    let a_id = base.char_id_at(0).unwrap();
    let c_id = base.char_id_at(1).unwrap();

    let delete_a = Op {
        id: OpId {
            counter: 1,
            replica: r(2),
        },
        payload: OpPayload::Delete { target: a_id },
    };
    let insert_x = Op {
        id: OpId {
            counter: 1,
            replica: r(3),
        },
        payload: OpPayload::Insert {
            left: Some(a_id),
            right: Some(c_id),
            value: 'X',
        },
    };
    let delete_x = Op {
        id: OpId {
            counter: 1,
            replica: r(4),
        },
        payload: OpPayload::Delete {
            target: insert_x.id,
        },
    };

    let ops = [delete_a, insert_x, delete_x];
    // Every ordering in which `insert_x` (index 1) precedes `delete_x`
    // (index 2) is causally valid; `delete_a` (index 0) has no dependency
    // on either of the other two. That is exactly half of the 6
    // permutations of 3 elements.
    let orders: [[usize; 3]; 3] = [[0, 1, 2], [1, 0, 2], [1, 2, 0]];
    for order in orders {
        let mut doc = base.clone();
        for idx in order {
            doc.apply(ops[idx]).unwrap();
        }
        assert_eq!(
            doc.text(),
            "c",
            "order {order:?} diverged from the pinned result"
        );
        assert_eq!(doc.len_chars(), 1);
    }
}

// --- Replay of an entire op log in reverse and shuffled order --------------

/// A small mixed insert/delete op log (5 sequential inserts, 1 concurrent
/// insert, 2 deletes — 8 operations total) plus the document that results
/// from applying it forward, once, in generation order.
fn full_op_log_fixture() -> (Vec<Op>, Document) {
    let mut doc = Document::new();
    let mut b1 = OpBuilder::new(r(1));
    let mut ops = Vec::new();
    for (i, ch) in "hello".chars().enumerate() {
        let op = b1.insert_at(&doc, i, ch).unwrap();
        doc.apply(op).unwrap();
        ops.push(op);
    }
    // A concurrent insert from a second replica, anchored inside the word.
    let left = doc.char_id_at(1).unwrap(); // 'e'
    let right = doc.char_id_at(2).unwrap(); // 'l'
    let concurrent = Op {
        id: OpId {
            counter: 1,
            replica: r(2),
        },
        payload: OpPayload::Insert {
            left: Some(left),
            right: Some(right),
            value: 'X',
        },
    };
    doc.apply(concurrent).unwrap();
    ops.push(concurrent);
    let del1 = Op {
        id: OpId {
            counter: 1,
            replica: r(3),
        },
        payload: OpPayload::Delete {
            target: doc.char_id_at(0).unwrap(),
        },
    };
    doc.apply(del1).unwrap();
    ops.push(del1);
    let del2 = Op {
        id: OpId {
            counter: 2,
            replica: r(3),
        },
        payload: OpPayload::Delete {
            target: concurrent.id,
        },
    };
    doc.apply(del2).unwrap();
    ops.push(del2);
    (ops, doc)
}

#[test]
fn replaying_the_full_op_log_in_reverse_order_is_idempotent() {
    let (ops, reference) = full_op_log_fixture();
    assert_eq!(ops.len(), 8, "pinned op-log length");
    let reference_text = reference.text();

    // `reference` already has every op applied; replay the identical log,
    // in reverse, into a clone of it. Every dependency is already
    // satisfied (the whole log was already applied once), so dedup makes
    // this a pure no-op regardless of delivery order: `Document::apply`
    // checks `self.applied.contains(&op.id)` before it ever looks at
    // left/right/target.
    let mut doc = reference.clone();
    for &op in ops.iter().rev() {
        assert_eq!(doc.apply(op), Ok(ApplyOutcome::Duplicate));
    }
    assert_eq!(
        doc.text(),
        reference_text,
        "reverse replay of an already-applied log must not mutate the document"
    );
}

#[test]
fn replaying_the_full_op_log_in_shuffled_order_is_idempotent() {
    let (ops, reference) = full_op_log_fixture();
    let reference_text = reference.text();

    // A fixed, non-monotonic shuffle of the 8 op-log indices.
    const SHUFFLE: [usize; 8] = [4, 0, 7, 2, 5, 1, 6, 3];
    assert_eq!(SHUFFLE.len(), ops.len());
    let mut doc = reference.clone();
    for idx in SHUFFLE {
        assert_eq!(doc.apply(ops[idx]), Ok(ApplyOutcome::Duplicate));
    }
    assert_eq!(
        doc.text(),
        reference_text,
        "shuffled replay of an already-applied log must not mutate the document"
    );
}

// --- Pending buffer boundary: zero capacity --------------------------------

#[test]
fn pending_buffer_zero_capacity_rejects_the_very_first_blocked_operation() {
    // `max_pending = 0` is the degenerate "one past capacity, from empty"
    // bound: the buffer can never hold anything, so the very first
    // operation whose dependency is missing must be rejected immediately,
    // never buffered even momentarily.
    let mut doc = Document::new();
    let mut pending = PendingOps::new(0);
    let ghost = OpId {
        counter: 1,
        replica: r(500),
    };
    let op = Op {
        id: OpId {
            counter: 1,
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
        Err(CrdtError::PendingBufferFull { max_pending: 0 })
    );
    assert_eq!(pending.pending_len(), 0);
    assert!(pending.is_empty());
}

// --- Checkpoint entry bound: growing a document one past a fixed bound ----

#[test]
fn checkpoint_bound_fixed_document_grows_from_within_bound_to_one_past_it() {
    let mut doc = Document::new();
    let mut b = OpBuilder::new(r(1));
    for (i, ch) in "abcd".chars().enumerate() {
        doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
    }
    const MAX_ENTRIES: usize = 4;
    // At exactly 4 elements, checkpointing at the bound succeeds.
    let at_bound = doc
        .checkpoint(MAX_ENTRIES)
        .expect("4 entries at a bound of 4 must succeed");
    assert_eq!(at_bound.entry_count(), 4);

    // One more insert grows the document to exactly one past the same,
    // unchanged bound; the identical `checkpoint` call now fails typed,
    // not truncated or silently clamped.
    let fifth = b.insert_at(&doc, 4, 'e').unwrap();
    doc.apply(fifth).unwrap();
    assert_eq!(
        doc.checkpoint(MAX_ENTRIES),
        Err(CheckpointError::TooLarge {
            entries: 5,
            max_entries: MAX_ENTRIES
        })
    );
}

// --- Truncated checkpoint bytes inside the delete-op-ids section ----------

#[test]
fn checkpoint_bytes_truncated_inside_the_delete_op_ids_section_is_rejected() {
    let mut doc = Document::new();
    let mut b = OpBuilder::new(r(1));
    for (i, ch) in "abc".chars().enumerate() {
        doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
    }
    // A delete-operation id (not present among the elements) so
    // `delete_op_ids` is non-empty and its bytes appear last in the
    // encoding.
    let del = Op {
        id: OpId {
            counter: 99,
            replica: r(2),
        },
        payload: OpPayload::Delete {
            target: doc.char_id_at(0).unwrap(),
        },
    };
    doc.apply(del).unwrap();
    let cp = doc.checkpoint(100).unwrap();
    assert_eq!(cp.entry_count(), 4, "3 elements + 1 delete-op id");
    let mut bytes = cp.to_bytes();
    // Cut off the last 4 of the 16 bytes that encode the sole delete-op id
    // (8-byte counter + 8-byte replica), landing inside that field
    // specifically rather than at a field or section boundary.
    bytes.truncate(bytes.len() - 4);
    assert_eq!(
        Checkpoint::from_bytes(&bytes),
        Err(CheckpointDecodeError::Truncated)
    );
}

// --- Truncated and corrupted v2 checkpoint bytes (delete target field) ---

/// The real v2 magic prefix, read off a freshly produced checkpoint rather
/// than hard-coded, so this stays correct even if the exact magic bytes
/// ever change.
fn v2_magic() -> [u8; 8] {
    let bytes = Document::new().checkpoint(0).unwrap().to_bytes();
    bytes[..8].try_into().unwrap()
}

#[test]
fn checkpoint_bytes_truncated_inside_the_v2_delete_target_field_is_rejected() {
    // A delete with a *known* target (an ordinary applied delete, not one
    // restored from a legacy checkpoint), so the v2 encoding actually
    // contains a target to truncate into: id(16) + flag(1) + target(16).
    let mut doc = Document::new();
    let mut b = OpBuilder::new(r(1));
    for (i, ch) in "abc".chars().enumerate() {
        doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
    }
    doc.apply(b.delete_at(&doc, 0).unwrap()).unwrap();
    let cp = doc.checkpoint(100).unwrap();
    let mut bytes = cp.to_bytes();
    // Cut off the last 4 bytes: with a known target present this lands
    // inside the *target*'s bytes specifically, not the id or the flag.
    bytes.truncate(bytes.len() - 4);
    assert_eq!(
        Checkpoint::from_bytes(&bytes),
        Err(CheckpointDecodeError::Truncated)
    );
}

#[test]
fn checkpoint_bytes_v2_magic_present_with_declared_counts_far_exceeding_actual_data_is_rejected() {
    // Same shape as the v1 "huge lying element_count" case, but with the v2
    // magic prefix present, so this exercises the v2 decode path
    // specifically rather than falling back to v1.
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&v2_magic());
    bytes.extend_from_slice(&0u64.to_le_bytes()); // max_elements
    bytes.extend_from_slice(&u64::MAX.to_le_bytes()); // "element_count": huge lie
    assert_eq!(
        Checkpoint::from_bytes(&bytes),
        Err(CheckpointDecodeError::Truncated)
    );
}

#[test]
fn checkpoint_bytes_v2_corrupted_with_arbitrary_patterns_never_panics() {
    let magic = v2_magic();
    let bodies: &[&[u8]] = &[
        &[],
        &[0x00],
        &[0xFF],
        &[0xFF; 7],
        &[0xFF; 40],
        &[0x00; 40],
        &[0xAB, 0xCD, 0xEF, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07],
    ];
    for body in bodies {
        let mut pattern = magic.to_vec();
        pattern.extend_from_slice(body);
        // As with the v1 fuzz case above: the only contract under test is
        // that decoding never panics, whatever `Result` comes back.
        let result = std::panic::catch_unwind(|| Checkpoint::from_bytes(&pattern));
        assert!(result.is_ok(), "from_bytes panicked on pattern {pattern:?}");
    }
}

// --- Counter at u64::MAX across two different replicas ---------------------

#[test]
fn two_replicas_both_at_u64_max_counter_coexist_without_id_conflict() {
    // `OpId` uniqueness is the pair `(counter, replica)`, not `counter`
    // alone, so two different replicas both issuing `u64::MAX` as their
    // counter must coexist rather than colliding.
    let mut doc = Document::new();
    let op_from_1 = Op {
        id: OpId {
            counter: u64::MAX,
            replica: r(1),
        },
        payload: OpPayload::Insert {
            left: None,
            right: None,
            value: 'x',
        },
    };
    let x_id = op_from_1.id;
    let op_from_2 = Op {
        id: OpId {
            counter: u64::MAX,
            replica: r(2),
        },
        payload: OpPayload::Insert {
            left: Some(x_id),
            right: None,
            value: 'y',
        },
    };
    assert_eq!(doc.apply(op_from_1), Ok(ApplyOutcome::Applied));
    assert_eq!(doc.apply(op_from_2), Ok(ApplyOutcome::Applied));
    assert_eq!(doc.text(), "xy");
    assert_ne!(
        op_from_1.id, op_from_2.id,
        "same counter, different replica, must not collide"
    );
}

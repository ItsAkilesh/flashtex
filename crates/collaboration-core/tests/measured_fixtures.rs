//! Revision 4: expanded measured convergence fixtures.
//!
//! Every fixture below states its replica count and operation count as an
//! assertion (computed from the fixture's own data, not hand-copied), and
//! its converged result as a pinned literal. A rerun reproduces every one
//! of these numbers exactly; nothing here is described only in prose.
//!
//! These are additional to, not replacements for, `tests/convergence.rs`
//! (revision 1's two hand-derived cases plus the 120-permutation sweep) and
//! `tests/adversarial.rs`'s N-way-anchor case (revision 4).

use std::collections::BTreeSet;

use flashtex_collaboration_core::{Document, Op, OpBuilder, OpId, OpPayload, ReplicaId};

fn r(n: u64) -> ReplicaId {
    ReplicaId(n)
}

/// Distinct replica ids that authored any operation in `ops` (by each op's
/// own id, not by ids it merely references as a dependency).
fn replica_count(ops: &[Op]) -> usize {
    ops.iter()
        .map(|op| op.id.replica.0)
        .collect::<BTreeSet<u64>>()
        .len()
}

/// Fixture 1 — three-replica concurrent word merge.
///
/// - Replica count: 3 (id 1 builds the shared base `"[]"`; ids 2 and 3
///   each concurrently insert a 3-character word into the same gap).
/// - Operation count: 8 (2 base inserts + 3 + 3 word-character inserts).
/// - Converged result, pinned: `"[dogcat]"`.
///
/// Hand-derivation: both words are anchored to the identical gap
/// `(left = '[', right = ']')` at their first character, so — per the
/// documented tie-break — the word whose first character has the larger
/// `OpId` sorts closer to `'['`. Replica 3's first character (id
/// `(counter=1, replica=3)`) outranks replica 2's (id `(counter=1,
/// replica=2)`), so "dog" (replica 3) sorts before "cat" (replica 2). Each
/// word's own characters stay contiguous and internally ordered, because
/// every character after a word's first anchors to its own predecessor,
/// never to the shared gap.
#[test]
fn fixture_1_three_replica_word_merge_pinned_at_eight_ops() {
    let mut base = Document::new();
    let mut base_builder = OpBuilder::new(r(1));
    let op_open = base_builder.insert_at(&base, 0, '[').unwrap();
    base.apply(op_open).unwrap();
    let op_close = base_builder.insert_at(&base, 1, ']').unwrap();
    base.apply(op_close).unwrap();
    assert_eq!(base.text(), "[]");

    // Replica 2 builds "cat" into its own offline copy of the base.
    let mut cat_doc = base.clone();
    let mut cat_builder = OpBuilder::new(r(2));
    let mut cat_ops = Vec::new();
    for (i, ch) in "cat".chars().enumerate() {
        let op = cat_builder.insert_at(&cat_doc, 1 + i, ch).unwrap();
        cat_doc.apply(op).unwrap();
        cat_ops.push(op);
    }

    // Replica 3 builds "dog" into its own, independent offline copy.
    let mut dog_doc = base.clone();
    let mut dog_builder = OpBuilder::new(r(3));
    let mut dog_ops = Vec::new();
    for (i, ch) in "dog".chars().enumerate() {
        let op = dog_builder.insert_at(&dog_doc, 1 + i, ch).unwrap();
        dog_doc.apply(op).unwrap();
        dog_ops.push(op);
    }

    let mut ops = vec![op_open, op_close];
    ops.extend(cat_ops.iter().copied());
    ops.extend(dog_ops.iter().copied());
    assert_eq!(ops.len(), 8, "pinned operation count for fixture 1");
    assert_eq!(replica_count(&ops), 3, "pinned replica count for fixture 1");

    const EXPECTED: &str = "[dogcat]";

    // ops indices: 0=open,1=close, 2,3,4=cat's c,a,t, 5,6,7=dog's d,o,g.
    let order_a: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7]; // base, cat, dog
    let order_b: [usize; 8] = [0, 1, 5, 2, 6, 3, 7, 4]; // base, interleaved
    let order_c: [usize; 8] = [0, 1, 5, 6, 7, 2, 3, 4]; // base, dog, cat

    for (name, order) in [("A", order_a), ("B", order_b), ("C", order_c)] {
        let mut doc = Document::new();
        for idx in order {
            doc.apply(ops[idx])
                .unwrap_or_else(|e| panic!("order {name} op {idx}: {e}"));
        }
        assert_eq!(
            doc.text(),
            EXPECTED,
            "order {name} diverged from the pinned converged result"
        );
    }
}

/// Fixture 2 — five-replica concurrent single-anchor insert with a
/// follow-up delete.
///
/// - Replica count: 5 (id 1 builds the shared base `"[]"`; ids 2-5 each
///   concurrently insert one character into the same gap; id 2
///   additionally deletes its own inserted character).
/// - Operation count: 7 (2 base inserts + 4 concurrent inserts + 1 delete).
/// - Converged result, pinned: `"[srq]"`.
#[test]
fn fixture_2_five_replica_single_anchor_insert_with_delete_pinned_at_seven_ops() {
    let mut base = Document::new();
    let mut bb = OpBuilder::new(r(1));
    let op_open = bb.insert_at(&base, 0, '[').unwrap();
    base.apply(op_open).unwrap();
    let op_close = bb.insert_at(&base, 1, ']').unwrap();
    base.apply(op_close).unwrap();

    // Replicas 2..=5 each independently insert one character anchored to
    // the identical gap (left = '[', right = ']').
    let mut inserts = Vec::new();
    for (replica, ch) in [(2u64, 'p'), (3, 'q'), (4, 'r'), (5, 's')] {
        let mut local = base.clone();
        let mut b = OpBuilder::new(r(replica));
        let op = b.insert_at(&local, 1, ch).unwrap();
        local.apply(op).unwrap();
        inserts.push(op);
    }

    // Replica 2 additionally deletes its own character ('p') as a
    // follow-up operation, depending only on its own earlier insert.
    let delete_p = Op {
        id: OpId {
            counter: 2,
            replica: r(2),
        },
        payload: OpPayload::Delete {
            target: inserts[0].id,
        },
    };

    let mut ops = vec![op_open, op_close];
    ops.extend(inserts.iter().copied());
    ops.push(delete_p);
    assert_eq!(ops.len(), 7, "pinned operation count for fixture 2");
    assert_eq!(replica_count(&ops), 5, "pinned replica count for fixture 2");

    const EXPECTED: &str = "[srq]";

    // ops indices: 0=open,1=close, 2=p,3=q,4=r,5=s, 6=delete_p (depends on 2).
    let order_a: [usize; 7] = [0, 1, 2, 3, 4, 5, 6]; // natural authoring order
    let order_b: [usize; 7] = [0, 1, 3, 4, 2, 6, 5]; // q, r, p, delete p, s
    let order_c: [usize; 7] = [0, 1, 5, 4, 3, 2, 6]; // descending sibling order, delete last

    for (name, order) in [("A", order_a), ("B", order_b), ("C", order_c)] {
        let mut doc = Document::new();
        for idx in order {
            doc.apply(ops[idx])
                .unwrap_or_else(|e| panic!("order {name} op {idx}: {e}"));
        }
        assert_eq!(
            doc.text(),
            EXPECTED,
            "order {name} diverged from the pinned converged result"
        );
    }
}

/// Fixture 3 — five-replica concurrent four-word merge (larger scale).
///
/// - Replica count: 5 (id 1 builds the shared base `"[]"`; ids 2-5 each
///   concurrently insert a distinct 2-character word into the same gap).
/// - Operation count: 10 (2 base inserts + 4 * 2 word-character inserts).
/// - Converged result, pinned: `"[dwczbyax]"`.
#[test]
fn fixture_3_five_replica_four_word_merge_pinned_at_ten_ops() {
    let mut base = Document::new();
    let mut bb = OpBuilder::new(r(1));
    let op_open = bb.insert_at(&base, 0, '[').unwrap();
    base.apply(op_open).unwrap();
    let op_close = bb.insert_at(&base, 1, ']').unwrap();
    base.apply(op_close).unwrap();

    let words: [(u64, &str); 4] = [(2, "ax"), (3, "by"), (4, "cz"), (5, "dw")];
    let mut per_replica_ops: Vec<Vec<Op>> = Vec::new();
    for (replica, word) in words {
        let mut local = base.clone();
        let mut b = OpBuilder::new(r(replica));
        let mut word_ops = Vec::new();
        for (i, ch) in word.chars().enumerate() {
            let op = b.insert_at(&local, 1 + i, ch).unwrap();
            local.apply(op).unwrap();
            word_ops.push(op);
        }
        per_replica_ops.push(word_ops);
    }

    let mut ops = vec![op_open, op_close];
    for word_ops in &per_replica_ops {
        ops.extend(word_ops.iter().copied());
    }
    assert_eq!(ops.len(), 10, "pinned operation count for fixture 3");
    assert_eq!(replica_count(&ops), 5, "pinned replica count for fixture 3");

    const EXPECTED: &str = "[dwczbyax]";

    // Order A: natural authoring order (base, then ax, by, cz, dw in turn).
    let order_a: Vec<usize> = (0..ops.len()).collect();
    // Order B: base, then the four words in reverse authoring order
    // (dw, cz, by, ax) — each word's own two ops stay internally ordered.
    let order_b: Vec<usize> = {
        let mut v = vec![0, 1];
        // ops layout: [open, close, ax(2,3), by(4,5), cz(6,7), dw(8,9)]
        v.extend([8, 9, 6, 7, 4, 5, 2, 3]);
        v
    };

    for (name, order) in [("A", &order_a), ("B", &order_b)] {
        let mut doc = Document::new();
        for &idx in order.iter() {
            doc.apply(ops[idx])
                .unwrap_or_else(|e| panic!("order {name} op {idx}: {e}"));
        }
        assert_eq!(
            doc.text(),
            EXPECTED,
            "order {name} diverged from the pinned converged result"
        );
    }
}

//! Checkpoint equivalence: FT-044 revision 2's load-bearing property.
//!
//! Resuming a document from a checkpoint taken after some prefix of a
//! causally valid operation log, then applying the remaining operations,
//! must produce EXACTLY the same document as replaying the whole log from
//! scratch - not just the same visible text, but the same structural
//! elements (including tombstones, in the same order) and the same causal
//! frontier. `assert_checkpoint_resume_matches_full_replay` checks this by
//! comparing the two documents' own checkpoints (a stronger, structural
//! check) as well as their visible text.
//!
//! This is exercised over several independent operation sequences - a
//! sequential single-replica edit, a Unicode/multibyte sequence, and (most
//! thoroughly) every one of the causally valid orderings of a 5-operation
//! concurrent scenario shared in spirit with `tests/convergence.rs`, each
//! checked at every possible cut point - not one happy path.

use flashtex_collaboration_core::{
    CrdtError, Document, Op, OpBuilder, OpId, OpPayload, ReplicaId,
};

/// Applies `ops` (a single fixed, causally valid order) to completion, then
/// re-derives the same final state by checkpointing after `cut` operations,
/// resuming from that checkpoint, and applying only `ops[cut..]`. Asserts
/// the two results are identical, structurally and not just as text.
fn assert_checkpoint_resume_matches_full_replay(ops: &[Op], cut: usize) {
    assert!(cut <= ops.len());
    let bound = ops.len() + 10;

    let mut full = Document::new();
    for &op in ops {
        full.apply(op)
            .expect("fixture ops must be causally valid and well-formed");
    }

    let mut partial = Document::new();
    for &op in &ops[..cut] {
        partial.apply(op).unwrap();
    }
    let checkpoint = partial
        .checkpoint(bound)
        .expect("fixture is well within the bound");
    let mut resumed = Document::from(checkpoint);
    for &op in &ops[cut..] {
        resumed.apply(op).unwrap();
    }

    assert_eq!(
        full.text(),
        resumed.text(),
        "checkpoint-resume text diverged from full replay at cut={cut}"
    );
    assert_eq!(
        full.checkpoint(bound).unwrap(),
        resumed.checkpoint(bound).unwrap(),
        "checkpoint-resume structural state diverged from full replay at cut={cut}"
    );
}

#[test]
fn checkpoint_equivalence_sequential_build_with_deletes() {
    let mut probe = Document::new();
    let mut b = OpBuilder::new(ReplicaId(1));
    let mut ops = Vec::new();
    for (i, ch) in "hello world".chars().enumerate() {
        let op = b.insert_at(&probe, i, ch).unwrap();
        probe.apply(op).unwrap();
        ops.push(op);
    }
    for &idx in &[4usize, 0, 2] {
        let op = b.delete_at(&probe, idx).unwrap();
        probe.apply(op).unwrap();
        ops.push(op);
    }

    for cut in 0..=ops.len() {
        assert_checkpoint_resume_matches_full_replay(&ops, cut);
    }
}

#[test]
fn checkpoint_equivalence_unicode_mixed_widths() {
    let mut probe = Document::new();
    let mut b = OpBuilder::new(ReplicaId(1));
    let mut ops = Vec::new();
    let source = "a\u{00e9}\u{4e2d}\u{1f389}\u{200d}b";
    for (i, ch) in source.chars().enumerate() {
        let op = b.insert_at(&probe, i, ch).unwrap();
        probe.apply(op).unwrap();
        ops.push(op);
    }
    let del = b.delete_at(&probe, 2).unwrap(); // delete the CJK char
    probe.apply(del).unwrap();
    ops.push(del);

    for cut in 0..=ops.len() {
        assert_checkpoint_resume_matches_full_replay(&ops, cut);
    }
}

/// The same 5-operation concurrent insert/insert/delete scenario as
/// `tests/convergence.rs`'s exhaustive sweep (two concurrent absolute-start
/// inserts, two ops depending on one of them, one depending on the other),
/// but here every causally valid ordering is *also* checked at every cut
/// point for checkpoint equivalence, not just for convergence.
#[test]
fn checkpoint_equivalence_holds_across_every_valid_ordering_and_cut_point() {
    let replica_a = ReplicaId(101);
    let replica_b = ReplicaId(102);
    let replica_c = ReplicaId(103);

    let op_a = Op {
        id: OpId {
            counter: 1,
            replica: replica_a,
        },
        payload: OpPayload::Insert {
            left: None,
            right: None,
            value: 'p',
        },
    };
    let op_b = Op {
        id: OpId {
            counter: 1,
            replica: replica_b,
        },
        payload: OpPayload::Insert {
            left: None,
            right: None,
            value: 'q',
        },
    };
    let op_c = Op {
        id: OpId {
            counter: 2,
            replica: replica_a,
        },
        payload: OpPayload::Insert {
            left: Some(op_a.id),
            right: None,
            value: 'r',
        },
    };
    let op_d = Op {
        id: OpId {
            counter: 1,
            replica: replica_c,
        },
        payload: OpPayload::Delete { target: op_b.id },
    };
    let op_e = Op {
        id: OpId {
            counter: 2,
            replica: replica_b,
        },
        payload: OpPayload::Insert {
            left: Some(op_b.id),
            right: None,
            value: 's',
        },
    };

    let ops = [op_a, op_b, op_c, op_d, op_e];
    let indices: Vec<usize> = (0..ops.len()).collect();

    let mut sequences_checked = 0usize;
    let mut cut_points_checked = 0usize;
    for perm in permutations(&indices) {
        let mut probe = Document::new();
        let mut causally_valid = true;
        for &idx in &perm {
            match probe.apply(ops[idx]) {
                Ok(_) => {}
                Err(CrdtError::MissingDependency(_)) => {
                    causally_valid = false;
                    break;
                }
                Err(other) => panic!("unexpected error for a well-formed op: {other}"),
            }
        }
        if !causally_valid {
            continue;
        }

        let ordered_ops: Vec<Op> = perm.iter().map(|&idx| ops[idx]).collect();
        for cut in 0..=ordered_ops.len() {
            assert_checkpoint_resume_matches_full_replay(&ordered_ops, cut);
            cut_points_checked += 1;
        }
        sequences_checked += 1;
    }

    assert!(
        sequences_checked >= 10,
        "expected many causally valid orderings, found {sequences_checked}"
    );
    assert!(
        cut_points_checked >= 60,
        "expected many (ordering, cut point) combinations, found {cut_points_checked}"
    );
}

/// All permutations of `items` (small inputs only; mirrors the helper in
/// `tests/convergence.rs` - duplicated rather than shared because
/// integration test binaries are separate crates).
fn permutations(items: &[usize]) -> Vec<Vec<usize>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut result = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut tail in permutations(&rest) {
            tail.insert(0, head);
            result.push(tail);
        }
    }
    result
}

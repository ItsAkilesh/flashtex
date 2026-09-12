//! Convergence fixtures: concurrent operations applied in *different orders*
//! at different replicas must produce identical final text. Each required
//! case below is worked out by hand in comments before being checked in
//! code, per FT-044's acceptance criteria — convergence must be
//! demonstrated, not asserted.
//!
//! Scope note: convergence is proven here for the crate's supported
//! operation set only — single-character `Insert` (anchored to `left`/
//! `right` neighbor ids observed by the inserting replica) and `Delete`
//! (tombstone of one existing id), with causally-ordered delivery per
//! replica (an operation's referenced ids must already be applied locally
//! before it is). It is not a claim about arbitrary/out-of-causal-order
//! delivery, which `Document::apply` rejects with `MissingDependency`
//! instead of silently reordering.

use flashtex_collaboration_core::{ApplyOutcome, CrdtError, Document, Op, OpBuilder, OpId, OpPayload, ReplicaId};

/// Case 1: two replicas concurrently insert a different character at the
/// *same* position.
///
/// Setup: replica R1 sequentially builds base text "ac":
///   a = id(counter=1, replica=1), left=None,  right=None
///   c = id(counter=2, replica=1), left=Some(a), right=None
///
/// Replicas A and B each start from a clone of "ac" and, without
/// coordinating, both insert between 'a' and 'c':
///   X ('b') = id(counter=1, replica=2), left=Some(a), right=Some(c)   [from A]
///   Y ('z') = id(counter=1, replica=3), left=Some(a), right=Some(c)   [from B]
///
/// X and Y are anchored to the exact same gap (a, c), so they are true
/// siblings and must be ordered by comparing `OpId` as a tuple
/// `(counter, replica)`: counters are equal (both 1), so replica breaks the
/// tie. `ReplicaId(3) > ReplicaId(2)`, so `Y.id > X.id`, and the
/// integration rule keeps the *larger* id closer to the shared left anchor.
/// Hand-worked result: Y sits immediately after 'a', then X, then 'c':
///
///     a , Y('z') , X('b') , c   ==  "azbc"
///
/// This must hold however the two operations are interleaved at each
/// replica, so both delivery orders are exercised below.
#[test]
fn concurrent_insert_at_same_position_converges() {
    let mut base = Document::new();
    let mut base_builder = OpBuilder::new(ReplicaId(1));
    base.apply(base_builder.insert_at(&base, 0, 'a').unwrap())
        .unwrap();
    base.apply(base_builder.insert_at(&base, 1, 'c').unwrap())
        .unwrap();
    assert_eq!(base.text(), "ac");

    let a_id = base.char_id_at(0).unwrap();
    let c_id = base.char_id_at(1).unwrap();

    let op_x = Op {
        id: OpId {
            counter: 1,
            replica: ReplicaId(2),
        },
        payload: OpPayload::Insert {
            left: Some(a_id),
            right: Some(c_id),
            value: 'b',
        },
    };
    let op_y = Op {
        id: OpId {
            counter: 1,
            replica: ReplicaId(3),
        },
        payload: OpPayload::Insert {
            left: Some(a_id),
            right: Some(c_id),
            value: 'z',
        },
    };
    assert!(op_y.id > op_x.id, "fixture assumption: Y must outrank X");

    // Replica A: applies its own X first, then receives Y.
    let mut replica_a = base.clone();
    replica_a.apply(op_x).unwrap();
    replica_a.apply(op_y).unwrap();

    // Replica B: applies its own Y first, then receives X.
    let mut replica_b = base.clone();
    replica_b.apply(op_y).unwrap();
    replica_b.apply(op_x).unwrap();

    assert_eq!(replica_a.text(), "azbc");
    assert_eq!(replica_b.text(), "azbc");
    assert_eq!(
        replica_a.text(),
        replica_b.text(),
        "different application orders must converge"
    );
}

/// Case 2: a concurrent insert anchored next to a character that is
/// *concurrently deleted*.
///
/// Setup: base text "abc" built sequentially by R1 (a, b, c chained left to
/// right). Concurrently:
///   D = delete target = b.id                                    [replica A]
///   I = insert 'X', left = Some(b.id), right = Some(c.id)        [replica B]
///
/// Delete only ever sets a tombstone flag; it never removes the element or
/// changes anyone's `left`/`right` id references, and `integrate` never
/// consults the tombstone flag when placing an element. So the *structural*
/// position `I` resolves to (immediately before 'c', immediately after 'b')
/// is identical whether `D` has already run or not — only whether 'b' itself
/// is currently visible differs. Hand-worked result, both orders:
///
///     structure: a, b(tombstoned), X, c   -->  visible text "aXc"
#[test]
fn concurrent_insert_versus_delete_overlap_converges() {
    let mut base = Document::new();
    let mut base_builder = OpBuilder::new(ReplicaId(1));
    for (i, ch) in "abc".chars().enumerate() {
        base.apply(base_builder.insert_at(&base, i, ch).unwrap())
            .unwrap();
    }
    assert_eq!(base.text(), "abc");
    let b_id = base.char_id_at(1).unwrap();
    let c_id = base.char_id_at(2).unwrap();

    let delete_b = Op {
        id: OpId {
            counter: 1,
            replica: ReplicaId(2),
        },
        payload: OpPayload::Delete { target: b_id },
    };
    let insert_x = Op {
        id: OpId {
            counter: 1,
            replica: ReplicaId(3),
        },
        payload: OpPayload::Insert {
            left: Some(b_id),
            right: Some(c_id),
            value: 'X',
        },
    };

    // Replica A: deletes 'b' locally first, then receives the insert.
    let mut replica_a = base.clone();
    replica_a.apply(delete_b).unwrap();
    replica_a.apply(insert_x).unwrap();

    // Replica B: inserts 'X' locally first, then receives the delete.
    let mut replica_b = base.clone();
    replica_b.apply(insert_x).unwrap();
    replica_b.apply(delete_b).unwrap();

    assert_eq!(replica_a.text(), "aXc");
    assert_eq!(replica_b.text(), "aXc");
    assert_eq!(replica_a.text(), replica_b.text());
}

/// Replaying an already-applied operation (e.g. an at-least-once transport
/// redelivering it) must be a no-op at the receiving replica, not a second
/// insertion — checked here across a delivery order that already includes
/// concurrency, not just the single-op case covered in `src/lib.rs`.
#[test]
fn replay_of_a_concurrent_op_does_not_double_apply() {
    let mut base = Document::new();
    let mut b = OpBuilder::new(ReplicaId(1));
    base.apply(b.insert_at(&base, 0, 'a').unwrap()).unwrap();
    base.apply(b.insert_at(&base, 1, 'c').unwrap()).unwrap();
    let a_id = base.char_id_at(0).unwrap();
    let c_id = base.char_id_at(1).unwrap();

    let op_x = Op {
        id: OpId {
            counter: 1,
            replica: ReplicaId(2),
        },
        payload: OpPayload::Insert {
            left: Some(a_id),
            right: Some(c_id),
            value: 'b',
        },
    };

    let mut doc = base.clone();
    assert_eq!(doc.apply(op_x).unwrap(), ApplyOutcome::Applied);
    assert_eq!(doc.text(), "abc");
    // Redeliver X twice more, interleaved with nothing else.
    assert_eq!(doc.apply(op_x).unwrap(), ApplyOutcome::Duplicate);
    assert_eq!(doc.apply(op_x).unwrap(), ApplyOutcome::Duplicate);
    assert_eq!(doc.text(), "abc", "redelivery must never double-insert");
}

/// Beyond the two required hand-worked cases above, sweep every *causally
/// valid* application ordering of a small mixed insert/insert/delete
/// scenario and confirm they all converge to the same text. An ordering
/// that violates causal delivery (an operation applied before an id it
/// references) is skipped, matching this crate's documented contract that
/// causal delivery is the caller's responsibility.
#[test]
fn convergence_holds_across_all_causally_valid_orderings() {
    let replica_a = ReplicaId(101);
    let replica_b = ReplicaId(102);
    let replica_c = ReplicaId(103);

    // A, B: concurrent inserts at the absolute start of an empty document.
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
    // C depends on A; D and E depend on B. A and B have no dependencies, so
    // a valid ordering may interleave these five operations in many ways.
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

    let mut reference: Option<String> = None;
    let mut valid_orderings = 0usize;
    for perm in permutations(&indices) {
        let mut doc = Document::new();
        let mut causally_valid = true;
        for &idx in &perm {
            match doc.apply(ops[idx]) {
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
        valid_orderings += 1;
        let text = doc.text();
        match &reference {
            None => reference = Some(text),
            Some(expected) => assert_eq!(
                expected, &text,
                "ordering {perm:?} diverged from the reference text"
            ),
        }
    }

    assert!(
        valid_orderings >= 10,
        "expected many causally valid orderings to exercise convergence, found {valid_orderings}"
    );
}

/// All permutations of `items` (small inputs only; used for an exhaustive
/// convergence sweep over a 5-element scenario, i.e. 120 permutations).
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

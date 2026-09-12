//! Bounded recovery buffer for operations that arrive with a missing
//! causal dependency - e.g. a partial or out-of-order backlog delivered
//! after a reconnect.
//!
//! [`Document::apply`] itself is unchanged from revision 1: it still
//! rejects an operation whose dependency is missing with
//! [`CrdtError::MissingDependency`] rather than buffering, so every
//! existing revision-1 caller and test keeps its exact behavior.
//!
//! **Chosen strategy for FT-044 revision 2's interrupted-delivery
//! requirement: buffer, exposed as the additive [`PendingOps`] type.**
//! Feeding an operation through [`PendingOps::receive`] whose dependency has
//! not yet been applied does not error; the operation is held, and it (and
//! anything depending transitively on it) is automatically retried and
//! integrated once the missing dependency arrives, in any order, including
//! multi-level chains. The buffer is bounded: past `max_pending` waiting
//! operations, `receive` returns [`CrdtError::PendingBufferFull`] instead of
//! growing without limit. [`PendingOps::missing_dependencies`] additionally
//! exposes exactly which ids the buffer is still waiting on, so a caller
//! can proactively request redelivery of just those instead of only
//! waiting.

use crate::{ApplyOutcome, CrdtError, Document, Op, OpId, OpPayload};

/// A bounded buffer of operations waiting on a missing causal dependency.
/// See the module docs for the recovery strategy this implements.
#[derive(Debug, Clone, Default)]
pub struct PendingOps {
    waiting: Vec<Op>,
    max_pending: usize,
}

impl PendingOps {
    /// Create an empty buffer that holds at most `max_pending` operations
    /// at once.
    pub fn new(max_pending: usize) -> Self {
        Self {
            waiting: Vec::new(),
            max_pending,
        }
    }

    /// Number of operations currently buffered, waiting on a dependency.
    pub fn pending_len(&self) -> usize {
        self.waiting.len()
    }

    /// Whether the buffer currently holds no waiting operations.
    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }

    /// Attempt to apply `op` to `doc`. If a referenced id (`left`, `right`,
    /// or `target`) is missing, `op` is buffered (bounded by `max_pending`)
    /// instead of returning an error, and [`ApplyOutcome::Buffered`] is
    /// returned. Whenever an operation is newly applied - directly here, or
    /// while draining - every other buffered operation is retried, so a
    /// multi-level chain (an operation depending on another still-buffered
    /// operation) integrates correctly once their common root arrives, in
    /// whatever order the pieces show up.
    ///
    /// Returns [`CrdtError::PendingBufferFull`] if the buffer is already at
    /// its bound and `op` is not a duplicate of something already buffered
    /// or applied. Returns [`CrdtError::IdConflict`] exactly when
    /// [`Document::apply`] would (buffering never masks it).
    pub fn receive(&mut self, doc: &mut Document, op: Op) -> Result<ApplyOutcome, CrdtError> {
        if doc.is_applied(op.id) {
            // Already applied (e.g. redelivered, or unblocked earlier by a
            // drain from another path): defer straight to `apply`, which
            // reports the correct `Duplicate`/`IdConflict` outcome.
            return doc.apply(op);
        }
        if self.waiting.iter().any(|w| w.id == op.id) {
            // Already queued; a dependency may have arrived since. Try
            // draining, then report the operation's actual current state.
            self.drain(doc);
            return Ok(if doc.is_applied(op.id) {
                ApplyOutcome::Applied
            } else {
                ApplyOutcome::Buffered
            });
        }

        match doc.apply(op) {
            Ok(outcome) => {
                self.drain(doc);
                Ok(outcome)
            }
            Err(CrdtError::MissingDependency(_)) => {
                if self.waiting.len() >= self.max_pending {
                    return Err(CrdtError::PendingBufferFull {
                        max_pending: self.max_pending,
                    });
                }
                self.waiting.push(op);
                Ok(ApplyOutcome::Buffered)
            }
            Err(other) => Err(other),
        }
    }

    /// Drain every buffered operation whose dependencies are now satisfied,
    /// repeating full passes until one makes no further progress.
    fn drain(&mut self, doc: &mut Document) {
        loop {
            let mut progressed = false;
            let mut i = 0;
            while i < self.waiting.len() {
                match doc.apply(self.waiting[i]) {
                    Ok(_) => {
                        self.waiting.remove(i);
                        progressed = true;
                    }
                    Err(CrdtError::MissingDependency(_)) => i += 1,
                    // A pathological outcome (e.g. `DocumentFull`) leaves
                    // the operation buffered rather than silently dropping
                    // it; it will be retried on the next drain.
                    Err(_) => i += 1,
                }
            }
            if !progressed {
                break;
            }
        }
    }

    /// Ids that currently-buffered operations reference but that have not
    /// been applied to `doc` - exactly what a caller should request
    /// redelivery of to unblock the backlog. Order is unspecified; each
    /// missing id appears at most once.
    pub fn missing_dependencies(&self, doc: &Document) -> Vec<OpId> {
        let mut missing = Vec::new();
        for op in &self.waiting {
            for id in dependency_ids(op) {
                if !doc.is_applied(id) && !missing.contains(&id) {
                    missing.push(id);
                }
            }
        }
        missing
    }
}

fn dependency_ids(op: &Op) -> Vec<OpId> {
    match op.payload {
        OpPayload::Insert { left, right, .. } => left.into_iter().chain(right).collect(),
        OpPayload::Delete { target } => vec![target],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Document, OpBuilder, OpId, ReplicaId};

    fn r(n: u64) -> ReplicaId {
        ReplicaId(n)
    }

    #[test]
    fn out_of_order_single_gap_is_buffered_then_drained() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        let op_a = b.insert_at(&doc, 0, 'a').unwrap();
        doc.apply(op_a).unwrap();
        let op_b = b.insert_at(&doc, 1, 'b').unwrap(); // depends on op_a
        // Simulate op_b having been *generated* (so op_c can depend on it)
        // without yet being delivered.
        let mut preview = doc.clone();
        preview.apply(op_b).unwrap();
        let op_c = b.insert_at(&preview, 2, 'c').unwrap(); // depends on op_b

        let mut pending = PendingOps::new(10);
        // op_c arrives first: its dependency (op_b) is missing, so it is
        // buffered rather than rejected.
        assert_eq!(
            pending.receive(&mut doc, op_c).unwrap(),
            ApplyOutcome::Buffered
        );
        assert_eq!(doc.text(), "a", "buffered op must not be integrated yet");
        assert_eq!(pending.pending_len(), 1);

        // The missing dependency is exactly op_b's id.
        assert_eq!(pending.missing_dependencies(&doc), vec![op_b.id]);

        // op_b now arrives: applying it directly should drain op_c too.
        assert_eq!(pending.receive(&mut doc, op_b).unwrap(), ApplyOutcome::Applied);
        assert_eq!(doc.text(), "abc");
        assert!(pending.is_empty(), "op_c must have drained automatically");
    }

    #[test]
    fn multi_level_chain_drains_completely_once_the_root_arrives() {
        // Build a', b', c' where each depends on the previous one, then
        // feed them in the exact reverse order they were generated.
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        let mut reference = Document::new();
        let op1 = b.insert_at(&reference, 0, 'x').unwrap();
        reference.apply(op1).unwrap();
        let op2 = b.insert_at(&reference, 1, 'y').unwrap();
        reference.apply(op2).unwrap();
        let op3 = b.insert_at(&reference, 2, 'z').unwrap();
        reference.apply(op3).unwrap();
        assert_eq!(reference.text(), "xyz");

        let mut pending = PendingOps::new(10);
        assert_eq!(
            pending.receive(&mut doc, op3).unwrap(),
            ApplyOutcome::Buffered
        );
        assert_eq!(
            pending.receive(&mut doc, op2).unwrap(),
            ApplyOutcome::Buffered
        );
        assert_eq!(doc.text(), "", "nothing is applicable yet");
        assert_eq!(pending.pending_len(), 2);

        // The root arrives last; both buffered operations should cascade in.
        assert_eq!(pending.receive(&mut doc, op1).unwrap(), ApplyOutcome::Applied);
        assert_eq!(doc.text(), "xyz");
        assert!(pending.is_empty());
    }

    #[test]
    fn duplicate_and_redundant_buffering_is_idempotent() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        let op_a = b.insert_at(&doc, 0, 'a').unwrap();
        let mut preview = doc.clone();
        preview.apply(op_a).unwrap();
        let op_b = b.insert_at(&preview, 1, 'b').unwrap();

        let mut pending = PendingOps::new(10);
        assert_eq!(
            pending.receive(&mut doc, op_b).unwrap(),
            ApplyOutcome::Buffered
        );
        // Redelivering the same still-blocked operation must not duplicate
        // the buffer slot.
        assert_eq!(
            pending.receive(&mut doc, op_b).unwrap(),
            ApplyOutcome::Buffered
        );
        assert_eq!(pending.pending_len(), 1);

        pending.receive(&mut doc, op_a).unwrap();
        assert_eq!(doc.text(), "ab");
        // Redelivering an operation that has since been applied (via drain)
        // must report it as a duplicate, not re-buffer or re-integrate it.
        assert_eq!(
            pending.receive(&mut doc, op_b).unwrap(),
            ApplyOutcome::Duplicate
        );
        assert!(pending.is_empty());
    }

    #[test]
    fn buffer_bound_is_a_typed_error_not_unbounded_growth() {
        let mut doc = Document::new();
        // Two operations that will never arrive, so both stay buffered.
        let ghost1 = OpId {
            counter: 1,
            replica: r(200),
        };
        let ghost2 = OpId {
            counter: 2,
            replica: r(200),
        };
        let op_x = crate::Op {
            id: OpId {
                counter: 1,
                replica: r(1),
            },
            payload: OpPayload::Insert {
                left: Some(ghost1),
                right: None,
                value: 'x',
            },
        };
        let op_y = crate::Op {
            id: OpId {
                counter: 2,
                replica: r(1),
            },
            payload: OpPayload::Insert {
                left: Some(ghost2),
                right: None,
                value: 'y',
            },
        };

        let mut pending = PendingOps::new(1);
        assert_eq!(
            pending.receive(&mut doc, op_x).unwrap(),
            ApplyOutcome::Buffered
        );
        assert_eq!(
            pending.receive(&mut doc, op_y),
            Err(CrdtError::PendingBufferFull { max_pending: 1 })
        );
        assert_eq!(pending.pending_len(), 1);
    }

    #[test]
    fn id_conflict_still_surfaces_through_receive() {
        // Buffering must never mask a genuine `IdConflict`: an operation
        // that can be applied immediately (no missing dependency) but
        // reuses an id with different content is rejected exactly like
        // `Document::apply` would reject it directly.
        let mut doc = Document::new();
        let id = OpId {
            counter: 1,
            replica: r(1),
        };
        doc.apply(crate::Op {
            id,
            payload: OpPayload::Insert {
                left: None,
                right: None,
                value: 'a',
            },
        })
        .unwrap();

        let mut pending = PendingOps::new(10);
        let conflicting = crate::Op {
            id,
            payload: OpPayload::Insert {
                left: None,
                right: None,
                value: 'b',
            },
        };
        assert_eq!(
            pending.receive(&mut doc, conflicting),
            Err(CrdtError::IdConflict(id))
        );
    }
}

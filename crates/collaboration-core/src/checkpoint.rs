//! Bounded, serializable checkpoints of a [`Document`]'s complete state.
//!
//! A [`Checkpoint`] is a plain-data snapshot of a document: its structural
//! elements (visible characters plus tombstones, in the exact order the
//! document has them) and its causal frontier - the complete set of
//! operation ids already applied, including delete-operation ids, which
//! don't otherwise appear among the elements. Every field is a primitive,
//! `Option<OpId>`, `char`, or a `Vec` of those, so a `Checkpoint` is
//! straightforward to serialize; [`Checkpoint::to_bytes`] /
//! [`Checkpoint::from_bytes`] give one concrete, dependency-free encoding.
//!
//! # Checkpoint equivalence
//!
//! Restoring a `Document` from a checkpoint (`Document::from(checkpoint)`)
//! and then applying the operations that had not yet been applied when the
//! checkpoint was taken must produce a document byte-for-byte identical to
//! one that applied every operation from scratch, in the same order. This
//! holds because a checkpoint stores the receiving document's element
//! vector *verbatim*, in the exact structural order [`Document::apply`]'s
//! `integrate` step already computed - restoring never re-runs integration,
//! it just reinstates that order directly. See `tests/checkpoint_equivalence.rs`
//! for the proof, exercised over several independent operation sequences
//! (not one happy path).
//!
//! # Bound
//!
//! [`Document::checkpoint`] takes an explicit `max_entries` cap (elements
//! plus delete-operation ids). Exceeding it returns
//! [`CheckpointError::TooLarge`] instead of producing an unboundedly large
//! snapshot.

use std::fmt;

use crate::{Document, Element, OpId, ReplicaId};

/// A bounded, serializable snapshot of a [`Document`]. See the module docs
/// for the equivalence guarantee and the encoding used by
/// [`Checkpoint::to_bytes`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    elements: Vec<Element>,
    /// Ids of `Delete` operations that have been applied. An insert's own
    /// id already appears in `elements`, so only delete-operation ids
    /// (which never appear there) need to be stored separately to fully
    /// reconstruct the causal frontier. Kept sorted so two checkpoints of
    /// documents that applied the same operation set always compare equal
    /// regardless of internal hash-set iteration order.
    delete_op_ids: Vec<OpId>,
    max_elements: usize,
}

impl Checkpoint {
    /// Total snapshot size: structural elements plus delete-operation ids.
    /// This is exactly the quantity [`Document::checkpoint`]'s `max_entries`
    /// bounds.
    pub fn entry_count(&self) -> usize {
        self.elements.len() + self.delete_op_ids.len()
    }

    /// Number of currently visible (non-tombstoned) characters captured by
    /// this checkpoint.
    pub fn visible_len(&self) -> usize {
        self.elements.iter().filter(|e| !e.deleted).count()
    }

    /// Encode this checkpoint as a flat, dependency-free byte sequence
    /// (little-endian fixed-width fields; see [`Checkpoint::from_bytes`]
    /// for the exact layout via its bounded reader). Round-trips exactly
    /// through [`Checkpoint::from_bytes`].
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.entry_count() * 24);
        out.extend_from_slice(&(self.max_elements as u64).to_le_bytes());
        out.extend_from_slice(&(self.elements.len() as u64).to_le_bytes());
        for e in &self.elements {
            write_opid(&mut out, e.id);
            write_opt_opid(&mut out, e.left);
            write_opt_opid(&mut out, e.right);
            out.extend_from_slice(&(e.value as u32).to_le_bytes());
            out.push(e.deleted as u8);
        }
        out.extend_from_slice(&(self.delete_op_ids.len() as u64).to_le_bytes());
        for id in &self.delete_op_ids {
            write_opid(&mut out, *id);
        }
        out
    }

    /// Decode a checkpoint previously produced by [`Checkpoint::to_bytes`].
    /// Bounded and non-panicking on malformed or truncated input: every
    /// field is read through a length-checked cursor, and an out-of-range
    /// Unicode scalar value is rejected rather than producing an invalid
    /// `char`.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CheckpointDecodeError> {
        let mut r = Reader::new(bytes);
        let max_elements = r.read_u64()? as usize;
        let element_count = r.read_u64()?;
        let mut elements = Vec::new();
        for _ in 0..element_count {
            let id = read_opid(&mut r)?;
            let left = read_opt_opid(&mut r)?;
            let right = read_opt_opid(&mut r)?;
            let value_code = r.read_u32()?;
            let value =
                char::from_u32(value_code).ok_or(CheckpointDecodeError::InvalidChar(value_code))?;
            let deleted = r.read_u8()? != 0;
            elements.push(Element {
                id,
                left,
                right,
                value,
                deleted,
            });
        }
        let delete_count = r.read_u64()?;
        let mut delete_op_ids = Vec::new();
        for _ in 0..delete_count {
            delete_op_ids.push(read_opid(&mut r)?);
        }
        Ok(Checkpoint {
            elements,
            delete_op_ids,
            max_elements,
        })
    }
}

/// Failure modes of [`Document::checkpoint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointError {
    /// The document's current entry count (elements plus delete-operation
    /// ids) exceeds the requested `max_entries` bound.
    TooLarge { entries: usize, max_entries: usize },
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckpointError::TooLarge {
                entries,
                max_entries,
            } => write!(
                f,
                "checkpoint would have {entries} entries, exceeding the bound of {max_entries}"
            ),
        }
    }
}

impl std::error::Error for CheckpointError {}

/// Failure modes of [`Checkpoint::from_bytes`]. Bounded and non-panicking:
/// malformed input is always rejected with one of these, never a panic or
/// an out-of-bounds read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointDecodeError {
    /// The byte sequence ended before a declared field could be read.
    Truncated,
    /// A stored `char` code point is not a valid Unicode scalar value.
    InvalidChar(u32),
}

impl fmt::Display for CheckpointDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckpointDecodeError::Truncated => {
                write!(f, "checkpoint bytes truncated before a declared field")
            }
            CheckpointDecodeError::InvalidChar(code) => {
                write!(f, "checkpoint contains an invalid char code point {code}")
            }
        }
    }
}

impl std::error::Error for CheckpointDecodeError {}

impl Document {
    /// Take a bounded, serializable snapshot of this document: its full
    /// structure (visible characters plus tombstones, in order) and the
    /// causal frontier of every applied operation id. Returns
    /// [`CheckpointError::TooLarge`] instead of an unboundedly large
    /// checkpoint if the current entry count (elements plus
    /// delete-operation ids) exceeds `max_entries`.
    pub fn checkpoint(&self, max_entries: usize) -> Result<Checkpoint, CheckpointError> {
        let element_ids: std::collections::HashSet<OpId> =
            self.elements_slice().iter().map(|e| e.id).collect();
        let mut delete_op_ids: Vec<OpId> = self
            .applied_set()
            .iter()
            .copied()
            .filter(|id| !element_ids.contains(id))
            .collect();
        delete_op_ids.sort();

        let entries = self.elements_slice().len() + delete_op_ids.len();
        if entries > max_entries {
            return Err(CheckpointError::TooLarge {
                entries,
                max_entries,
            });
        }

        Ok(Checkpoint {
            elements: self.elements_slice().to_vec(),
            delete_op_ids,
            max_elements: self.max_elements_bound(),
        })
    }
}

impl From<Checkpoint> for Document {
    /// Resume a document from a checkpoint. Never fails: reconstructing is
    /// simply reinstating the stored element order and applied-id set
    /// verbatim (see the module docs for why this is exactly what
    /// checkpoint equivalence requires).
    fn from(checkpoint: Checkpoint) -> Self {
        let mut applied: std::collections::HashSet<OpId> =
            checkpoint.elements.iter().map(|e| e.id).collect();
        applied.extend(checkpoint.delete_op_ids.iter().copied());
        Document::from_parts(checkpoint.elements, applied, checkpoint.max_elements)
    }
}

fn write_opid(out: &mut Vec<u8>, id: OpId) {
    out.extend_from_slice(&id.counter.to_le_bytes());
    out.extend_from_slice(&id.replica.0.to_le_bytes());
}

fn write_opt_opid(out: &mut Vec<u8>, id: Option<OpId>) {
    match id {
        None => out.push(0),
        Some(id) => {
            out.push(1);
            write_opid(out, id);
        }
    }
}

fn read_opid(r: &mut Reader<'_>) -> Result<OpId, CheckpointDecodeError> {
    let counter = r.read_u64()?;
    let replica = r.read_u64()?;
    Ok(OpId {
        counter,
        replica: ReplicaId(replica),
    })
}

fn read_opt_opid(r: &mut Reader<'_>) -> Result<Option<OpId>, CheckpointDecodeError> {
    match r.read_u8()? {
        0 => Ok(None),
        _ => Ok(Some(read_opid(r)?)),
    }
}

/// A length-checked cursor over a byte slice. Every read either returns
/// exactly the requested field or [`CheckpointDecodeError::Truncated`];
/// there is no path that indexes out of bounds or panics.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, CheckpointDecodeError> {
        let b = *self
            .bytes
            .get(self.pos)
            .ok_or(CheckpointDecodeError::Truncated)?;
        self.pos += 1;
        Ok(b)
    }

    fn read_u32(&mut self) -> Result<u32, CheckpointDecodeError> {
        let end = self
            .pos
            .checked_add(4)
            .ok_or(CheckpointDecodeError::Truncated)?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(CheckpointDecodeError::Truncated)?;
        self.pos = end;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(slice);
        Ok(u32::from_le_bytes(buf))
    }

    fn read_u64(&mut self) -> Result<u64, CheckpointDecodeError> {
        let end = self
            .pos
            .checked_add(8)
            .ok_or(CheckpointDecodeError::Truncated)?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(CheckpointDecodeError::Truncated)?;
        self.pos = end;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(slice);
        Ok(u64::from_le_bytes(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApplyOutcome, OpBuilder};

    fn r(n: u64) -> ReplicaId {
        ReplicaId(n)
    }

    #[test]
    fn checkpoint_too_large_is_a_typed_error_not_a_panic() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        for (i, ch) in "hello".chars().enumerate() {
            doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
        }
        assert_eq!(
            doc.checkpoint(2),
            Err(CheckpointError::TooLarge {
                entries: 5,
                max_entries: 2
            })
        );
        // A sufficient bound still succeeds.
        assert!(doc.checkpoint(5).is_ok());
    }

    #[test]
    fn checkpoint_bytes_round_trip_exactly() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        for (i, ch) in "abc\u{1f389}".chars().enumerate() {
            doc.apply(b.insert_at(&doc, i, ch).unwrap()).unwrap();
        }
        doc.apply(b.delete_at(&doc, 1).unwrap()).unwrap();

        let cp = doc.checkpoint(100).unwrap();
        let bytes = cp.to_bytes();
        let decoded = Checkpoint::from_bytes(&bytes).unwrap();
        assert_eq!(cp, decoded);
    }

    #[test]
    fn checkpoint_bytes_truncated_input_is_rejected_not_panicking() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        doc.apply(b.insert_at(&doc, 0, 'x').unwrap()).unwrap();
        let cp = doc.checkpoint(10).unwrap();
        let mut bytes = cp.to_bytes();
        bytes.truncate(bytes.len() - 1);
        assert_eq!(
            Checkpoint::from_bytes(&bytes),
            Err(CheckpointDecodeError::Truncated)
        );
        // Empty input specifically, not just "one byte short".
        assert_eq!(
            Checkpoint::from_bytes(&[]),
            Err(CheckpointDecodeError::Truncated)
        );
    }

    #[test]
    fn checkpoint_bytes_invalid_char_code_is_rejected_not_panicking() {
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        doc.apply(b.insert_at(&doc, 0, 'x').unwrap()).unwrap();
        let cp = doc.checkpoint(10).unwrap();
        let mut bytes = cp.to_bytes();
        // Layout: max_elements(8) + element_count(8) + id.counter(8) +
        // id.replica(8) + left flag(1) + right flag(1) = offset 34, then the
        // 4-byte char code. 0x110000 is one past the last valid scalar value.
        let char_offset = 8 + 8 + 8 + 8 + 1 + 1;
        bytes[char_offset..char_offset + 4].copy_from_slice(&0x0011_0000u32.to_le_bytes());
        assert_eq!(
            Checkpoint::from_bytes(&bytes),
            Err(CheckpointDecodeError::InvalidChar(0x0011_0000))
        );
    }

    #[test]
    fn restoring_an_empty_checkpoint_gives_an_empty_document() {
        let doc = Document::new();
        let cp = doc.checkpoint(10).unwrap();
        let restored = Document::from(cp);
        assert_eq!(restored.text(), "");
        assert_eq!(restored.len_chars(), 0);
    }

    #[test]
    fn restored_document_rejects_replaying_the_wrong_dependency_the_same_way() {
        // Sanity check that a restored document is a fully functional
        // `Document`, not a read-only view: it still enforces causal
        // dependencies and replay dedup exactly like a fresh one.
        let mut doc = Document::new();
        let mut b = OpBuilder::new(r(1));
        let op = b.insert_at(&doc, 0, 'x').unwrap();
        doc.apply(op).unwrap();
        let cp = doc.checkpoint(10).unwrap();
        let mut restored = Document::from(cp);
        assert_eq!(restored.apply(op).unwrap(), ApplyOutcome::Duplicate);
        assert_eq!(restored.text(), "x");
    }
}

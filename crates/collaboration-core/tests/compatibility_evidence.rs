//! Revision 4: reproducible compatibility evidence.
//!
//! Two claims, each pinned to exact bytes (or a hash of them) inline in the
//! assertions below, so the claim is checkable by inspection and by a
//! rerun — not merely asserted as "it round-trips" or "orders agree with
//! each other":
//!
//! 1. A checkpoint encoded under this crate's stated contract
//!    (`src/checkpoint.rs`'s module docs) decodes back byte-for-byte, and
//!    its `to_bytes()` output is pinned literally, so a future change to
//!    the wire encoding is caught here as a literal mismatch, not just a
//!    passing round trip that could hide a compatible-looking change to
//!    the encoding. This is pinned for both wire format versions: v2 (what
//!    `to_bytes()` produces today) and the original v1 (what it produced
//!    before GH#23's checkpoint-target bump), the latter proving old
//!    checkpoints already on disk or held by a peer keep decoding.
//! 2. A document converged from several different, causally valid replica
//!    delivery orderings is byte-identical not only to itself across those
//!    orderings, but to a pinned literal and a pinned hash of that
//!    literal — an independent, compact, checkable invariant. Unaffected by
//!    the GH#23 checkpoint change: convergence is a property of
//!    `Document::apply`'s insert integration, not of checkpoint encoding.

use flashtex_collaboration_core::{
    ApplyOutcome, Checkpoint, Document, Op, OpBuilder, OpId, OpPayload, ReplicaId,
};

fn r(n: u64) -> ReplicaId {
    ReplicaId(n)
}

/// A tiny, dependency-free FNV-1a 64-bit hash. Used only to give the
/// cross-ordering convergence fixture below a second, compact, pinned
/// invariant alongside the literal text comparison; this is not a
/// cryptographic claim of any kind, and adding it here keeps the crate's
/// zero-dependency property intact even in test code.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// Builds the shared fixture both checkpoint-format tests below pin: replica
/// 1 inserts "ab"; replica 2 deletes 'a' as a follow-up operation (counter
/// 5, exercising a delete-op id distinct from any element id, so both
/// halves of the encoding — elements and deletes — are non-empty).
fn fixture_doc() -> Document {
    let mut doc = Document::new();
    let mut ba = OpBuilder::new(r(1));
    doc.apply(ba.insert_at(&doc, 0, 'a').unwrap()).unwrap();
    doc.apply(ba.insert_at(&doc, 1, 'b').unwrap()).unwrap();
    let target = doc.char_id_at(0).unwrap();
    doc.apply(Op {
        id: OpId {
            counter: 5,
            replica: r(2),
        },
        payload: OpPayload::Delete { target },
    })
    .unwrap();
    assert_eq!(doc.text(), "b");
    doc
}

/// Evidence #1: a checkpoint written under this exact contract is readable
/// byte-for-byte, and its wire encoding is pinned literally.
///
/// GH#23 (18:15Z) approved storing each delete's target alongside its id, to
/// close the gap where two `Delete`s could share an id but name different
/// targets and be indistinguishable from a legitimate idempotent replay.
/// That bumped the wire format to v2 and moved this literal — expected and
/// legitimate, since the byte-for-byte pin exists precisely to catch wire
/// changes, and this one was deliberate and approved. What changed and why:
///
/// - an 8-byte magic prefix now leads the encoding, letting
///   [`Checkpoint::from_bytes`] tell v2 bytes apart from the old,
///   prefix-less v1 layout,
/// - each delete entry now carries its target the same way an optional
///   `left`/`right` anchor is encoded (a presence flag, then the `OpId` if
///   present) instead of being a bare id; this fixture's delete's target is
///   known (it was produced by an ordinary `apply`, not restored from a v1
///   checkpoint), so the flag is set and the target bytes are present.
///
/// The elements section is untouched — only the delete section and the new
/// leading magic changed. v1 bytes are still decodable: see
/// `v1_checkpoint_bytes_still_decode_and_restore_correctly` below, which
/// keeps the *exact* pre-bump 102-byte literal this test used to pin, as
/// the actual proof of backward compatibility.
#[test]
fn checkpoint_bytes_for_a_fixed_fixture_are_pinned_byte_for_byte() {
    let doc = fixture_doc();
    let cp = doc.checkpoint(10).expect("3 entries within a bound of 10");
    let bytes = cp.to_bytes();

    // Layout (see `src/checkpoint.rs`): magic:[u8; 8], max_elements:u64 LE,
    // element_count:u64 LE, then per element {id.counter:u64, id.replica:u64,
    // left:1+[8+8 if present], right:1+[8+8 if present], value:u32,
    // deleted:u8}, then delete_count:u64, then per delete {id.counter:u64,
    // id.replica:u64, target:1+[8+8 if present]}.
    //
    // 'a' was inserted into an empty document, so it has left=None AND
    // right=None (there was nothing to its right yet at insertion time,
    // not "None because it's now the only element") — `right` records what
    // the inserting replica observed, not the current document.
    #[rustfmt::skip]
    const PINNED_V2: &[u8] = &[
        // magic = "FTCPKV2\0"
        0x46, 0x54, 0x43, 0x50, 0x4B, 0x56, 0x32, 0x00,
        // max_elements = 200_000 (DEFAULT_MAX_ELEMENTS)
        0x40, 0x0D, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
        // element_count = 2
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // element 0 ('a'): id = (counter=1, replica=1)
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // left = None, right = None
        0x00, 0x00,
        // value = 'a' (0x61), deleted = true
        0x61, 0x00, 0x00, 0x00, 0x01,
        // element 1 ('b'): id = (counter=2, replica=1)
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // left = Some((counter=1, replica=1))
        0x01,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // right = None
        0x00,
        // value = 'b' (0x62), deleted = false
        0x62, 0x00, 0x00, 0x00, 0x00,
        // delete_count = 1
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // delete 0: id = (counter=5, replica=2)
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // target = Some((counter=1, replica=1)) - the id of 'a'
        0x01,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    assert_eq!(
        bytes, PINNED_V2,
        "checkpoint wire encoding changed; see this test's doc comment for what changed and why"
    );

    // "Readable under the stated contract": decoding the pinned bytes
    // reproduces a checkpoint identical to the one written.
    let decoded = Checkpoint::from_bytes(PINNED_V2)
        .expect("pinned bytes must decode under the documented contract");
    assert_eq!(
        decoded, cp,
        "decoding the pinned bytes must reproduce the written checkpoint exactly"
    );
}

/// Evidence #1 (compatibility half): the *exact* 102-byte v1 literal this
/// crate produced for the same fixture before GH#23's checkpoint-target
/// bump still decodes and restores correctly. This — not merely "the new
/// code round-trips its own new format" — is the actual proof that
/// existing checkpoints (on disk, or held by a peer that hasn't upgraded
/// yet) are not broken by the wire-format change. The literal is untouched
/// from the pre-bump version of this test.
#[test]
fn v1_checkpoint_bytes_still_decode_and_restore_correctly() {
    #[rustfmt::skip]
    const PINNED_V1: &[u8] = &[
        // max_elements = 200_000 (DEFAULT_MAX_ELEMENTS)
        0x40, 0x0D, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
        // element_count = 2
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // element 0 ('a'): id = (counter=1, replica=1)
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // left = None, right = None
        0x00, 0x00,
        // value = 'a' (0x61), deleted = true
        0x61, 0x00, 0x00, 0x00, 0x01,
        // element 1 ('b'): id = (counter=2, replica=1)
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // left = Some((counter=1, replica=1))
        0x01,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // right = None
        0x00,
        // value = 'b' (0x62), deleted = false
        0x62, 0x00, 0x00, 0x00, 0x00,
        // delete_count = 1
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // delete-op id 0: (counter=5, replica=2) - bare, no target field
        0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    assert_eq!(PINNED_V1.len(), 102, "this is the exact pre-bump literal");

    let decoded = Checkpoint::from_bytes(PINNED_V1)
        .expect("a v1 checkpoint produced before the GH#23 wire-format bump must still decode");
    assert_eq!(decoded.entry_count(), 3, "2 elements + 1 delete-op id");
    assert_eq!(decoded.visible_len(), 1);

    let restored = Document::from(decoded);
    assert_eq!(
        restored.text(),
        "b",
        "a legacy v1 checkpoint must still restore to exactly the document it snapshotted"
    );

    // And the restored document must still be a fully functional `Document`
    // afterwards, not a read-only or degraded view.
    let mut restored = restored;
    let mut b2 = OpBuilder::new(r(3));
    let op = b2.insert_at(&restored, 1, 'X').unwrap();
    assert_eq!(restored.apply(op), Ok(ApplyOutcome::Applied));
    assert_eq!(restored.text(), "bX");
}

/// Evidence #2: a converged document is byte-identical across every one of
/// several distinct, causally valid replica delivery orderings — checked
/// against a pinned literal and a pinned hash, not only against each
/// other.
///
/// Fixture: base "ac"; replicas 2, 3, 4 concurrently insert into the
/// identical gap (left = a, right = c). Per the documented tie-break,
/// descending replica id sorts closest to `left`: 4, 3, 2.
#[test]
fn converged_document_is_byte_identical_across_replica_orderings_pinned() {
    let mut base = Document::new();
    let mut bb = OpBuilder::new(r(1));
    base.apply(bb.insert_at(&base, 0, 'a').unwrap()).unwrap();
    base.apply(bb.insert_at(&base, 1, 'c').unwrap()).unwrap();
    let a_id = base.char_id_at(0).unwrap();
    let c_id = base.char_id_at(1).unwrap();

    let ops: Vec<Op> = [(2u64, 'p'), (3, 'q'), (4, 'r')]
        .into_iter()
        .map(|(replica, ch)| Op {
            id: OpId {
                counter: 1,
                replica: r(replica),
            },
            payload: OpPayload::Insert {
                left: Some(a_id),
                right: Some(c_id),
                value: ch,
            },
        })
        .collect();

    const EXPECTED_TEXT: &str = "arqpc";
    const EXPECTED_HASH: u64 = 7_532_153_511_834_115_664;

    let orders: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for order in orders {
        let mut doc = base.clone();
        for idx in order {
            doc.apply(ops[idx]).unwrap();
        }
        let text = doc.text();
        assert_eq!(
            text.as_bytes(),
            EXPECTED_TEXT.as_bytes(),
            "order {order:?} produced different bytes than the pinned result"
        );
        assert_eq!(
            fnv1a_64(text.as_bytes()),
            EXPECTED_HASH,
            "order {order:?} hash mismatch against the pinned value"
        );
    }
}

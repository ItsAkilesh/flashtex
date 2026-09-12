//! Synthetic fixtures for [`collection_layout`]: bounded validation of TTC
//! face/table-directory ranges, duplicate-tag and overlap rejection, and the
//! legitimate-sharing exception a real `ttcf` collection depends on.
//!
//! These are built by hand rather than loaded from disk so the malformed
//! and adversarial cases (duplicate tags, overlapping ranges, a hostile
//! header) are exercised deterministically without depending on any font
//! file being present on the machine running the suite.

use flashtex_font_engine::{Error, collection_layout};

/// Appends one table directory record (tag, offset, length) in the format
/// both the real parser and `collection_layout` read: 4-byte tag, checksum
/// (unused by either), then u32 offset and u32 length.
fn push_record(buf: &mut Vec<u8>, tag: &[u8; 4], offset: u32, length: u32) {
    buf.extend_from_slice(tag);
    buf.extend_from_slice(&0u32.to_be_bytes()); // checksum, not validated
    buf.extend_from_slice(&offset.to_be_bytes());
    buf.extend_from_slice(&length.to_be_bytes());
}

/// Builds one `sfnt` table directory (header + records) at the given file
/// offset, with a payload of `payload_len` zero bytes placed right after the
/// directory for the records to point into. Returns the whole directory+
/// payload bytes; the caller places them at `sfnt_offset` in the final file.
fn sfnt_dir(records: &[(&[u8; 4], u32, u32)]) -> Vec<u8> {
    let mut dir = Vec::new();
    dir.extend_from_slice(&0x0001_0000u32.to_be_bytes()); // sfnt version 1.0
    dir.extend_from_slice(&(records.len() as u16).to_be_bytes()); // numTables
    dir.extend_from_slice(&0u16.to_be_bytes()); // searchRange
    dir.extend_from_slice(&0u16.to_be_bytes()); // entrySelector
    dir.extend_from_slice(&0u16.to_be_bytes()); // rangeShift
    for (tag, offset, length) in records {
        push_record(&mut dir, tag, *offset, *length);
    }
    dir
}

/// A minimal well-formed two-face TTC where both faces share one table
/// (`cmap`) byte-for-byte at the same offset/length — the normal, legitimate
/// space-saving a real `ttcf` collection relies on — and each also has one
/// table of its own that does not overlap anything.
///
/// Layout (byte offsets are chosen generously so nothing is adjacent by
/// accident):
///   0..12   ttcf header (tag, version, numFonts=2)
///   12..20  offset table: face0 sfnt offset, face1 sfnt offset
///   100     face0's sfnt directory (2 tables: shared cmap, own head0)
///   200     face1's sfnt directory (2 tables: shared cmap, own head1)
///   1000    shared cmap table data (32 bytes)
///   2000    face0's own head0 data (16 bytes)
///   3000    face1's own head1 data (16 bytes)
fn build_ttc(face0_dir_offset: u32, face1_dir_offset: u32) -> Vec<u8> {
    let mut f = vec![0u8; 4000];
    f[0..4].copy_from_slice(b"ttcf");
    f[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    f[8..12].copy_from_slice(&2u32.to_be_bytes()); // numFonts
    f[12..16].copy_from_slice(&face0_dir_offset.to_be_bytes());
    f[16..20].copy_from_slice(&face1_dir_offset.to_be_bytes());

    let dir0 = sfnt_dir(&[(b"cmap", 1000, 32), (b"head", 2000, 16)]);
    let dir1 = sfnt_dir(&[(b"cmap", 1000, 32), (b"head", 3000, 16)]);
    f[face0_dir_offset as usize..face0_dir_offset as usize + dir0.len()].copy_from_slice(&dir0);
    f[face1_dir_offset as usize..face1_dir_offset as usize + dir1.len()].copy_from_slice(&dir1);
    f
}

#[test]
fn identical_shared_table_across_faces_is_accepted() {
    let data = build_ttc(100, 200);
    let layout = collection_layout(&data).expect("well-formed TTC with legitimate sharing");
    assert_eq!(layout.faces.len(), 2);
    assert_eq!(layout.faces[0].tables.len(), 2);
    assert_eq!(layout.faces[1].tables.len(), 2);
    let shared0 = layout.faces[0]
        .tables
        .iter()
        .find(|t| &t.tag == b"cmap")
        .unwrap();
    let shared1 = layout.faces[1]
        .tables
        .iter()
        .find(|t| &t.tag == b"cmap")
        .unwrap();
    assert_eq!((shared0.offset, shared0.length), (1000, 32));
    assert_eq!(
        (shared0.offset, shared0.length),
        (shared1.offset, shared1.length)
    );
}

#[test]
fn partially_overlapping_cross_face_tables_are_rejected() {
    let mut data = build_ttc(100, 200);
    // Corrupt face1's cmap record to overlap face0's cmap by 16 bytes
    // instead of matching it exactly: same tag, inconsistent extent.
    let dir1_cmap_length_at = 200 + 12 + 12; // dir header(12) + record0(tag,cksum,off) then length field
    data[dir1_cmap_length_at..dir1_cmap_length_at + 4].copy_from_slice(&48u32.to_be_bytes());
    let err = collection_layout(&data).unwrap_err();
    assert!(
        matches!(err, Error::Malformed(_)),
        "expected a typed Malformed error, got {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("overlap"), "expected overlap in error: {msg}");
}

#[test]
fn duplicate_table_tag_within_one_face_is_rejected() {
    let mut f = vec![0u8; 2000];
    f[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes()); // plain sfnt, face 0
    let dir = sfnt_dir(&[(b"cmap", 500, 16), (b"cmap", 700, 16)]);
    f[0..dir.len()].copy_from_slice(&dir);
    let err = collection_layout(&f).unwrap_err();
    assert!(matches!(err, Error::Malformed(_)));
    assert!(err.to_string().contains("duplicate"), "{err}");
}

#[test]
fn partially_overlapping_tables_within_one_face_are_rejected() {
    let mut f = vec![0u8; 2000];
    f[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    // head at 500..516, hhea at 510..526: partial overlap, different tags.
    let dir = sfnt_dir(&[(b"head", 500, 16), (b"hhea", 510, 16)]);
    f[0..dir.len()].copy_from_slice(&dir);
    let err = collection_layout(&f).unwrap_err();
    assert!(matches!(err, Error::Malformed(_)));
    assert!(err.to_string().contains("overlap"), "{err}");
}

#[test]
fn well_formed_single_face_non_overlapping_tables_are_accepted() {
    let mut f = vec![0u8; 2000];
    f[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    let dir = sfnt_dir(&[(b"head", 500, 16), (b"hhea", 600, 16), (b"maxp", 700, 6)]);
    f[0..dir.len()].copy_from_slice(&dir);
    let layout = collection_layout(&f).expect("distinct, non-overlapping tables are fine");
    assert_eq!(layout.faces.len(), 1);
    assert_eq!(layout.faces[0].tables.len(), 3);
}

#[test]
fn hostile_ttc_face_count_fails_without_allocating() {
    // A 12-byte header claiming ~4 billion faces. The offset table alone
    // would need 16 GiB; collection_layout must reject this by comparing
    // the claimed size against the real (tiny) buffer, not by attempting
    // any allocation sized by the claim.
    let mut f = vec![0u8; 12];
    f[0..4].copy_from_slice(b"ttcf");
    f[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    f[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
    let err = collection_layout(&f).unwrap_err();
    assert!(matches!(err, Error::Malformed(_)));
}

#[test]
fn hostile_table_count_fails_without_allocating() {
    // A plain sfnt header claiming ~65535 tables (the u16 max) in a file
    // that is nowhere near big enough to hold that many 16-byte records.
    let mut f = vec![0u8; 12];
    f[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    f[4..6].copy_from_slice(&u16::MAX.to_be_bytes()); // numTables
    let err = collection_layout(&f).unwrap_err();
    assert!(matches!(err, Error::Malformed(_)));
}

#[test]
fn empty_and_truncated_inputs_are_errors_not_panics() {
    assert!(collection_layout(&[]).is_err());
    assert!(collection_layout(&[0u8; 4]).is_err());
    let mut ttc_header_only = vec![0u8; 8];
    ttc_header_only[0..4].copy_from_slice(b"ttcf");
    assert!(collection_layout(&ttc_header_only).is_err());
}

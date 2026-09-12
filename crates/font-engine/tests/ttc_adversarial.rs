//! Adversarial bounds suite for [`collection_layout`] (rev 4).
//!
//! Attacks the rev-3 TTC face/table descriptor with deliberately hostile
//! byte buffers and proves each one is rejected as a typed [`Error`] —
//! never a panic, a hang, or an allocation sized by an attacker-controlled
//! count. Cases are data (`name`, buffer) pairs run through one loop, so
//! adding a new hostile case is a one-line addition to `CASES`, not a new
//! test function.
//!
//! Every buffer here is deliberately malformed; none is expected to parse.
//! Well-formed acceptance (including legitimate cross-face table sharing)
//! is covered separately in `tests/ttc_layout.rs`, and exact-value pinning
//! of well-formed output is covered in `tests/ttc_identity.rs`.

use std::panic::{self, AssertUnwindSafe};

use flashtex_font_engine::{Error, collection_layout};

/// One table-directory record: tag, offset, length. Checksum is zero and
/// unchecked by the parser, matching the real format's byte layout.
fn push_record(buf: &mut Vec<u8>, tag: &[u8; 4], offset: u32, length: u32) {
    buf.extend_from_slice(tag);
    buf.extend_from_slice(&0u32.to_be_bytes());
    buf.extend_from_slice(&offset.to_be_bytes());
    buf.extend_from_slice(&length.to_be_bytes());
}

/// A plain `sfnt` header (12 bytes) followed by `records`, each 16 bytes,
/// padded out to `total_len` zero bytes so record offsets can point past
/// the directory into "table data" that is never actually read by
/// `collection_layout` (it validates ranges, not contents).
fn sfnt(records: &[(&[u8; 4], u32, u32)], total_len: usize) -> Vec<u8> {
    let mut f = vec![0u8; total_len];
    f[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    f[4..6].copy_from_slice(&(records.len() as u16).to_be_bytes());
    let mut dir = Vec::new();
    for (tag, offset, length) in records {
        push_record(&mut dir, tag, *offset, *length);
    }
    let end = 12 + dir.len();
    assert!(end <= f.len(), "test fixture: directory does not fit");
    f[12..end].copy_from_slice(&dir);
    f
}

/// A `ttcf` header claiming `num_fonts` faces, with `offsets` (as many as
/// fit) written into the offset table, padded to `total_len`.
fn ttcf(num_fonts: u32, offsets: &[u32], total_len: usize) -> Vec<u8> {
    let mut f = vec![0u8; total_len];
    f[0..4].copy_from_slice(b"ttcf");
    f[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    f[8..12].copy_from_slice(&num_fonts.to_be_bytes());
    for (i, off) in offsets.iter().enumerate() {
        let at = 12 + 4 * i;
        if at + 4 <= f.len() {
            f[at..at + 4].copy_from_slice(&off.to_be_bytes());
        }
    }
    f
}

fn case_all_zero_bytes() -> Vec<u8> {
    vec![0u8; 64]
}

fn case_truncated_mid_header_tag_only() -> Vec<u8> {
    b"ttcf".to_vec()
}

fn case_truncated_mid_header_no_numfonts() -> Vec<u8> {
    // 'ttcf' + version, but the numFonts field itself is cut off.
    let mut f = b"ttcf".to_vec();
    f.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    f
}

fn case_truncated_mid_directory() -> Vec<u8> {
    // Claims 2 faces (needs a 20-byte offset table: 12-byte header + 2*4)
    // but the file stops after the first offset entry.
    ttcf(2, &[100], 16)
}

fn case_num_fonts_u32_max() -> Vec<u8> {
    ttcf(u32::MAX, &[], 12)
}

fn case_num_tables_u16_max() -> Vec<u8> {
    let mut f = vec![0u8; 12];
    f[0..4].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    f[4..6].copy_from_slice(&u16::MAX.to_be_bytes());
    f
}

fn case_face_offset_past_end_of_file() -> Vec<u8> {
    // One face whose offset table fits (16 bytes total), but the face's
    // sfnt offset (1_000_000) points nowhere near the actual file.
    ttcf(1, &[1_000_000], 16)
}

fn case_table_offset_plus_length_overflow() -> Vec<u8> {
    // offset and length are both near u32::MAX; on a 64-bit host their sum
    // does not overflow `usize`, but it must still be rejected against the
    // real (tiny) file length rather than accepted because the addition
    // "succeeded". This proves the bounds check, not just the checked_add
    // guard, is load-bearing.
    sfnt(&[(b"cmap", u32::MAX - 10, u32::MAX - 10)], 64)
}

fn case_table_length_zero() -> Vec<u8> {
    sfnt(&[(b"cmap", 20, 0)], 64)
}

fn case_misaligned_table_offset() -> Vec<u8> {
    sfnt(&[(b"cmap", 21, 16)], 64)
}

fn case_misaligned_face_offset_in_ttc() -> Vec<u8> {
    // Face 0's sfnt directory would otherwise be well-formed at offset 21,
    // but 21 is not 4-byte aligned.
    let mut f = ttcf(1, &[21], 64);
    // Write a well-formed single-table directory at offset 21 so the only
    // thing wrong with this file is the alignment of the face offset.
    let dir = {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0001_0000u32.to_be_bytes());
        d.extend_from_slice(&1u16.to_be_bytes());
        d.extend_from_slice(&0u16.to_be_bytes());
        d.extend_from_slice(&0u16.to_be_bytes());
        d.extend_from_slice(&0u16.to_be_bytes());
        push_record(&mut d, b"cmap", 40, 16);
        d
    };
    f[21..21 + dir.len()].copy_from_slice(&dir);
    f
}

fn case_directory_non_ascending_order() -> Vec<u8> {
    // 'zzzz' before 'aaaa': both ranges are valid, aligned, non-overlapping
    // and non-zero-length, so ascending-tag order is the only thing wrong.
    sfnt(&[(b"zzzz", 40, 4), (b"aaaa", 60, 4)], 128)
}

fn case_single_zero_byte() -> Vec<u8> {
    vec![0u8]
}

fn case_empty() -> Vec<u8> {
    Vec::new()
}

#[test]
fn hostile_inputs_never_panic_and_always_error() {
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty file", case_empty()),
        ("single zero byte", case_single_zero_byte()),
        ("entirely zero bytes", case_all_zero_bytes()),
        (
            "truncated mid-header: tag only",
            case_truncated_mid_header_tag_only(),
        ),
        (
            "truncated mid-header: no numFonts",
            case_truncated_mid_header_no_numfonts(),
        ),
        (
            "truncated mid-directory: offset table cut short",
            case_truncated_mid_directory(),
        ),
        ("numFonts at u32::MAX", case_num_fonts_u32_max()),
        ("numTables at u16::MAX", case_num_tables_u16_max()),
        (
            "face offset past end of file",
            case_face_offset_past_end_of_file(),
        ),
        (
            "table offset+length overflow/overrun",
            case_table_offset_plus_length_overflow(),
        ),
        ("table length of zero", case_table_length_zero()),
        ("misaligned table offset", case_misaligned_table_offset()),
        (
            "misaligned face offset in a ttc",
            case_misaligned_face_offset_in_ttc(),
        ),
        (
            "directory entries not in ascending order",
            case_directory_non_ascending_order(),
        ),
    ];

    let mut failures = Vec::new();
    for (name, data) in cases {
        let result = panic::catch_unwind(AssertUnwindSafe(|| collection_layout(&data)));
        match result {
            Ok(Ok(layout)) => failures.push(format!(
                "{name}: expected an error, got Ok({} faces)",
                layout.faces.len()
            )),
            Ok(Err(e)) => {
                // Must be a typed error variant, not some ad-hoc signal.
                if !matches!(e, Error::Malformed(_)) {
                    failures.push(format!("{name}: expected Error::Malformed, got {e:?}"));
                }
            }
            Err(panic_payload) => {
                let msg = panic_payload
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "<non-string panic payload>".to_string());
                failures.push(format!("{name}: PANICKED: {msg}"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "adversarial cases did not fail cleanly:\n{}",
        failures.join("\n")
    );
}

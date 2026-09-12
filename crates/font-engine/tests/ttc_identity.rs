//! Exact-identity regressions for [`collection_layout`] (rev 4).
//!
//! Each case pins the *entire* [`CollectionLayout`] — every face's
//! `sfnt_offset` and the tag/offset/length of every [`TableRange`] — as a
//! literal `assert_eq!` against a hand-built expected value, not a shape
//! check (`.len()`, `is_ok()`, etc). A future change that alters a computed
//! range, a tag, or a face count must fail this test even if the change
//! still "looks reasonable" by weaker assertions.
//!
//! Inputs are synthetic, hand-built buffers rather than real system fonts:
//! a real `.ttc`/`.ttf`'s exact byte offsets can shift across OS/font
//! updates for reasons that have nothing to do with this crate, which would
//! make an identity pin against one flaky for the wrong reason. A synthetic
//! buffer's bytes are fully under this test's control, so its expected
//! output is exactly known and stable.

use flashtex_font_engine::{CollectionLayout, FaceLayout, TableRange, collection_layout};

fn push_record(buf: &mut Vec<u8>, tag: &[u8; 4], offset: u32, length: u32) {
    buf.extend_from_slice(tag);
    buf.extend_from_slice(&0u32.to_be_bytes()); // checksum, unchecked
    buf.extend_from_slice(&offset.to_be_bytes());
    buf.extend_from_slice(&length.to_be_bytes());
}

fn sfnt_header(records: &[(&[u8; 4], u32, u32)]) -> Vec<u8> {
    let mut dir = Vec::new();
    dir.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    dir.extend_from_slice(&(records.len() as u16).to_be_bytes());
    dir.extend_from_slice(&0u16.to_be_bytes());
    dir.extend_from_slice(&0u16.to_be_bytes());
    dir.extend_from_slice(&0u16.to_be_bytes());
    for (tag, offset, length) in records {
        push_record(&mut dir, tag, *offset, *length);
    }
    dir
}

fn table(tag: &[u8; 4], offset: usize, length: usize) -> TableRange {
    TableRange {
        tag: *tag,
        offset,
        length,
    }
}

/// A single-face `OTTO` (CFF) program with four tables, in ascending tag
/// order, at well-separated, 4-byte-aligned offsets.
#[test]
fn single_face_otto_directory_is_pinned_exactly() {
    let mut f = vec![0u8; 2000];
    let dir = {
        let mut d = Vec::new();
        d.extend_from_slice(&0x4F54_544Fu32.to_be_bytes()); // 'OTTO'
        d.extend_from_slice(&4u16.to_be_bytes());
        d.extend_from_slice(&0u16.to_be_bytes());
        d.extend_from_slice(&0u16.to_be_bytes());
        d.extend_from_slice(&0u16.to_be_bytes());
        push_record(&mut d, b"CFF ", 800, 500);
        push_record(&mut d, b"cmap", 400, 64);
        push_record(&mut d, b"head", 300, 54);
        push_record(&mut d, b"maxp", 360, 6);
        d
    };
    f[0..dir.len()].copy_from_slice(&dir);

    let got = collection_layout(&f).expect("well-formed OTTO directory");
    let want = CollectionLayout {
        faces: vec![FaceLayout {
            face_index: 0,
            sfnt_offset: 0,
            tables: vec![
                table(b"CFF ", 800, 500),
                table(b"cmap", 400, 64),
                table(b"head", 300, 54),
                table(b"maxp", 360, 6),
            ],
        }],
    };
    assert_eq!(got, want);
}

/// A three-face `ttcf` collection: face 0 and face 2 share one `cmap` table
/// byte-for-byte; face 1 has an entirely disjoint set of tables. Pins face
/// count, every `sfnt_offset`, and every table's exact tag/offset/length —
/// including that the shared-table optimization produces byte-identical
/// `TableRange`s on both sharing faces.
#[test]
fn three_face_ttc_with_one_shared_table_is_pinned_exactly() {
    let mut f = vec![0u8; 6000];
    f[0..4].copy_from_slice(b"ttcf");
    f[4..8].copy_from_slice(&0x0001_0000u32.to_be_bytes());
    f[8..12].copy_from_slice(&3u32.to_be_bytes());
    f[12..16].copy_from_slice(&100u32.to_be_bytes()); // face 0 dir
    f[16..20].copy_from_slice(&200u32.to_be_bytes()); // face 1 dir
    f[20..24].copy_from_slice(&300u32.to_be_bytes()); // face 2 dir

    let dir0 = sfnt_header(&[(b"cmap", 2000, 128), (b"head", 3000, 54)]);
    let dir1 = sfnt_header(&[(b"glyf", 4000, 800), (b"loca", 4800, 40)]);
    let dir2 = sfnt_header(&[(b"cmap", 2000, 128), (b"head", 3100, 54)]);
    f[100..100 + dir0.len()].copy_from_slice(&dir0);
    f[200..200 + dir1.len()].copy_from_slice(&dir1);
    f[300..300 + dir2.len()].copy_from_slice(&dir2);

    let got = collection_layout(&f).expect("well-formed 3-face ttc with legitimate sharing");
    let want = CollectionLayout {
        faces: vec![
            FaceLayout {
                face_index: 0,
                sfnt_offset: 100,
                tables: vec![table(b"cmap", 2000, 128), table(b"head", 3000, 54)],
            },
            FaceLayout {
                face_index: 1,
                sfnt_offset: 200,
                tables: vec![table(b"glyf", 4000, 800), table(b"loca", 4800, 40)],
            },
            FaceLayout {
                face_index: 2,
                sfnt_offset: 300,
                tables: vec![table(b"cmap", 2000, 128), table(b"head", 3100, 54)],
            },
        ],
    };
    assert_eq!(got, want);
    assert_eq!(got.faces.len(), 3);
    // The shared table really is the identical range on both sides, not
    // merely equal by coincidence of hand-typed literals.
    assert_eq!(got.faces[0].tables[0], got.faces[2].tables[0]);
}

/// A plain single-table `sfnt` (the minimum a face can carry) at a
/// non-zero, non-trivial offset, to pin the smallest possible shape too.
#[test]
fn single_table_sfnt_is_pinned_exactly() {
    let mut f = vec![0u8; 200];
    let dir = sfnt_header(&[(b"head", 100, 54)]);
    f[0..dir.len()].copy_from_slice(&dir);

    let got = collection_layout(&f).expect("single-table sfnt");
    let want = CollectionLayout {
        faces: vec![FaceLayout {
            face_index: 0,
            sfnt_offset: 0,
            tables: vec![table(b"head", 100, 54)],
        }],
    };
    assert_eq!(got, want);
}

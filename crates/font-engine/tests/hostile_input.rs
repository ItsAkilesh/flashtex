//! Bounded, deterministic hostile-input tests against font-engine's public
//! byte-ingesting entry points: `TrueTypeFace::parse` (untrusted font bytes)
//! and `subset::verify_checksums` (untrusted sfnt table directory). No
//! `cargo fuzz`/proptest dependency; everything here is a plain `#[test]`
//! that runs in well under a second.

use std::panic::{self, AssertUnwindSafe};

use flashtex_font_engine::subset::verify_checksums;
use flashtex_font_engine::{Face, GlyphId, TrueTypeFace};

fn push_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}
fn push_i16(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// A minimal but structurally valid CFF-outlines ('OTTO') sfnt carrying a
/// `GSUB` table whose sole `liga` lookup has a format-2 (Ranges) Coverage
/// table declaring `startCoverageIndex = 9999` against a ligature-set array
/// of length 1 -- the exact shape described in
/// `coordination/daniel-parent-panic-audit.md` for `gsub.rs`'s unchecked
/// `sub.sets[usize::from(ci)]` indexing. `TrueTypeFace::parse` itself must
/// succeed (the out-of-range index is only *used*, not detected, at parse
/// time); the panic (pre-fix) happens on the `longest_ligature` call below.
fn hostile_gsub_font() -> Vec<u8> {
    // --- GSUB table body, offsets relative to its own start ---
    let mut gsub = Vec::new();
    push_u16(&mut gsub, 1); // majorVersion
    push_u16(&mut gsub, 0); // minorVersion
    push_u16(&mut gsub, 10); // scriptListOffset
    push_u16(&mut gsub, 30); // featureListOffset
    push_u16(&mut gsub, 44); // lookupListOffset
    assert_eq!(gsub.len(), 10);

    // ScriptList @10: 1 record "DFLT" -> Script table @18
    push_u16(&mut gsub, 1); // scriptCount
    gsub.extend_from_slice(b"DFLT");
    push_u16(&mut gsub, 8); // offset from ScriptList start (10) -> 18
    assert_eq!(gsub.len(), 18);

    // Script table @18: defaultLangSys -> @22, langSysCount=0
    push_u16(&mut gsub, 4); // offset from Script start (18) -> 22
    push_u16(&mut gsub, 0);
    assert_eq!(gsub.len(), 22);

    // Default LangSys @22
    push_u16(&mut gsub, 0); // lookupOrder (reserved)
    push_u16(&mut gsub, 0xFFFF); // requiredFeatureIndex (none)
    push_u16(&mut gsub, 1); // featureIndexCount
    push_u16(&mut gsub, 0); // featureIndices[0]
    assert_eq!(gsub.len(), 30);

    // FeatureList @30: 1 record "liga" -> Feature table @38
    push_u16(&mut gsub, 1); // featureCount
    gsub.extend_from_slice(b"liga");
    push_u16(&mut gsub, 8); // offset from FeatureList start (30) -> 38
    assert_eq!(gsub.len(), 38);

    // Feature table @38
    push_u16(&mut gsub, 0); // featureParamsOffset
    push_u16(&mut gsub, 1); // lookupIndexCount
    push_u16(&mut gsub, 0); // lookupListIndices[0]
    assert_eq!(gsub.len(), 44);

    // LookupList @44: 1 lookup -> Lookup table @48
    push_u16(&mut gsub, 1); // lookupCount
    push_u16(&mut gsub, 4); // offset from LookupList start (44) -> 48
    assert_eq!(gsub.len(), 48);

    // Lookup table @48: type 4 (LigatureSubst), 1 subtable -> @56
    push_u16(&mut gsub, 4); // lookupType
    push_u16(&mut gsub, 0); // lookupFlag
    push_u16(&mut gsub, 1); // subTableCount
    push_u16(&mut gsub, 8); // offset from Lookup start (48) -> 56
    assert_eq!(gsub.len(), 56);

    // LigatureSubstFormat1 subtable @56
    push_u16(&mut gsub, 1); // substFormat
    push_u16(&mut gsub, 8); // coverageOffset from subtable start (56) -> 64
    push_u16(&mut gsub, 1); // ligSetCount
    push_u16(&mut gsub, 18); // ligSetOffsets[0] from subtable start (56) -> 74
    assert_eq!(gsub.len(), 64);

    // Coverage table (format 2) @64: glyph 5 alone, maliciously claiming
    // startCoverageIndex 9999 (the crate's own len-1 ligature-set array
    // never has index 9999).
    push_u16(&mut gsub, 2); // coverageFormat
    push_u16(&mut gsub, 1); // rangeCount
    push_u16(&mut gsub, 5); // startGlyphID
    push_u16(&mut gsub, 5); // endGlyphID
    push_u16(&mut gsub, 9999); // startCoverageIndex (malicious)
    assert_eq!(gsub.len(), 74);

    // LigatureSet table @74: no ligatures (count 0) is enough -- the
    // fabricated coverage index is never expected to resolve to it.
    push_u16(&mut gsub, 0); // ligatureCount
    assert_eq!(gsub.len(), 76);

    // --- wrap in a minimal CFF-outlines sfnt: head, hhea, hmtx, maxp, CFF ---
    let mut head = vec![0u8; 54];
    head[12..16].copy_from_slice(&0x5F0F_3CF5u32.to_be_bytes()); // magic
    head[18..20].copy_from_slice(&1000u16.to_be_bytes()); // unitsPerEm

    let mut hhea = vec![0u8; 36];
    hhea[4..6].copy_from_slice(&700i16.to_be_bytes()); // ascender
    hhea[6..8].copy_from_slice(&(-300i16).to_be_bytes()); // descender
    hhea[34..36].copy_from_slice(&2u16.to_be_bytes()); // numberOfHMetrics

    let mut hmtx = Vec::new();
    push_u16(&mut hmtx, 500);
    push_i16(&mut hmtx, 0);
    push_u16(&mut hmtx, 500);
    push_i16(&mut hmtx, 0);

    let mut maxp = Vec::new();
    push_u32(&mut maxp, 0x0000_5000); // version 0.5
    push_u16(&mut maxp, 2); // numGlyphs

    let cff = vec![0u8; 4]; // never parsed by TrueTypeFace::parse

    let tables: [(&[u8; 4], &[u8]); 5] = [
        (b"head", &head),
        (b"hhea", &hhea),
        (b"hmtx", &hmtx),
        (b"maxp", &maxp),
        (b"CFF ", &cff),
    ];
    // GSUB goes last so it can be pushed after the loop below.
    let mut font = Vec::new();
    push_u32(&mut font, 0x4F54_544F); // 'OTTO'
    push_u16(&mut font, 6); // numTables
    push_u16(&mut font, 0);
    push_u16(&mut font, 0);
    push_u16(&mut font, 0);
    let dir_start = font.len();
    font.resize(dir_start + 16 * 6, 0);
    let mut data_start = dir_start + 16 * 6;
    let mut write_record = |i: usize, font: &mut Vec<u8>, tag: &[u8; 4], body: &[u8]| {
        let rec = dir_start + 16 * i;
        font[rec..rec + 4].copy_from_slice(tag);
        // checksum left as 0: TrueTypeFace::parse never verifies it.
        font[rec + 8..rec + 12].copy_from_slice(&(data_start as u32).to_be_bytes());
        font[rec + 12..rec + 16].copy_from_slice(&(body.len() as u32).to_be_bytes());
        font.extend_from_slice(body);
        data_start += body.len();
    };
    for (i, (tag, body)) in tables.iter().enumerate() {
        write_record(i, &mut font, tag, body);
    }
    write_record(5, &mut font, b"GSUB", &gsub);
    font
}

#[test]
fn gsub_out_of_range_coverage_index_does_not_panic() {
    let data = hostile_gsub_font();
    let face = TrueTypeFace::parse(data).expect("hostile-but-structurally-valid font parses");
    assert!(face.has_gsub_ligatures());
    // Pre-fix this panicked: "index out of bounds: the len is 1 but the
    // index is 9999" (gsub.rs, `sub.sets[usize::from(ci)]`). Post-fix it
    // must report "no ligature" rather than crash the process.
    let result = panic::catch_unwind(AssertUnwindSafe(|| face.longest_ligature(0, &[GlyphId(5)])));
    match result {
        Ok(v) => assert_eq!(
            v, None,
            "an out-of-range coverage index is not a real match"
        ),
        Err(e) => panic!("longest_ligature panicked on a hostile coverage index: {e:?}"),
    }
}

#[test]
fn verify_checksums_short_directory_does_not_panic() {
    // numTables (bytes 4..6) declares 1 table, but the buffer is far too
    // short to hold even one 16-byte directory record starting at byte 12.
    // Pre-fix: "range start index 12 out of range for slice of length 6".
    let data: [u8; 6] = [0, 0, 0, 0, 0, 1];
    let result = panic::catch_unwind(AssertUnwindSafe(|| verify_checksums(&data)));
    match result {
        Ok(r) => assert!(
            r.is_err(),
            "a truncated directory must be rejected, not silently accepted"
        ),
        Err(e) => panic!("verify_checksums panicked on a truncated table directory: {e:?}"),
    }
}

/// Small, dependency-free xorshift64* PRNG so the hostile-input loop below
/// is deterministic across runs (same seed -> same bytes -> a reported
/// failure is always reproducible from the seed alone).
struct Xorshift64(u64);
impl Xorshift64 {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn fill(&mut self, len: usize) -> Vec<u8> {
        let mut v = Vec::with_capacity(len);
        while v.len() < len {
            v.extend_from_slice(&self.next_u64().to_le_bytes());
        }
        v.truncate(len);
        v
    }
}

/// Bounded (a few thousand iterations, all-`#[test]`, no external fuzzer)
/// hostile-input sweep over `TrueTypeFace::parse` and `verify_checksums`
/// with arbitrary and lightly-structured garbage. Any panic (index
/// out-of-bounds, overflow, unbounded recursion/allocation) is a defect;
/// a returned `Err` is the expected, correct outcome for malformed input.
#[test]
fn hostile_bytes_never_panic_truetype_parse_or_verify_checksums() {
    let mut rng = Xorshift64(0x9E37_79B9_7F4A_7C15);
    for round in 0..4000u32 {
        // Mix of pure noise and small buffers (small lengths exercise the
        // directory/table-count edge cases more than large ones do).
        let len = (rng.next_u64() % 600) as usize;
        let mut buf = rng.fill(len);
        // Occasionally force a recognizable sfnt/ttc tag or table count so
        // the parser gets further into its logic before rejecting the rest.
        if !buf.is_empty() && rng.next_u64().is_multiple_of(3) {
            let tags: [[u8; 4]; 3] = [[0x00, 0x01, 0x00, 0x00], *b"OTTO", *b"ttcf"];
            let tag = tags[(rng.next_u64() as usize) % tags.len()];
            for (i, b) in tag.iter().enumerate() {
                if i < buf.len() {
                    buf[i] = *b;
                }
            }
        }

        let buf_for_parse = buf.clone();
        let r1 = panic::catch_unwind(AssertUnwindSafe(|| TrueTypeFace::parse(buf_for_parse)));
        assert!(
            r1.is_ok(),
            "round {round}: TrueTypeFace::parse panicked on {} random bytes (seed-derived, len {len})",
            buf.len()
        );

        let r2 = panic::catch_unwind(AssertUnwindSafe(|| verify_checksums(&buf)));
        assert!(
            r2.is_ok(),
            "round {round}: verify_checksums panicked on {} random bytes (seed-derived, len {len})",
            buf.len()
        );
    }
}

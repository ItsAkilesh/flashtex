use flashtex_rendering_core::{font_adapter::*, *};
use std::collections::BTreeMap;
// Original synthetic empty-outline sfnt from font-resources16dab88 tests; no visual claim.
fn be16(data: &mut [u8], at: usize, n: u16) {
    data[at..at + 2].copy_from_slice(&n.to_be_bytes());
}
fn be32(data: &mut [u8], at: usize, n: u32) {
    data[at..at + 4].copy_from_slice(&n.to_be_bytes());
}
fn fixture() -> Vec<u8> {
    let mut tables = BTreeMap::new();
    let mut head = vec![0; 54];
    be32(&mut head, 0, 0x00010000);
    be32(&mut head, 12, 0x5f0f3cf5);
    be16(&mut head, 18, 1000);
    tables.insert(*b"head", head);
    let mut maxp = vec![0; 32];
    be32(&mut maxp, 0, 0x00010000);
    be16(&mut maxp, 4, 3);
    tables.insert(*b"maxp", maxp);
    tables.insert(*b"loca", vec![0; 8]);
    tables.insert(*b"glyf", vec![]);
    let mut hhea = vec![0; 36];
    be32(&mut hhea, 0, 0x00010000);
    be16(&mut hhea, 34, 2);
    tables.insert(*b"hhea", hhea);
    tables.insert(*b"hmtx", vec![0; 10]);
    let mut name = vec![0; 18];
    be16(&mut name, 2, 1);
    be16(&mut name, 4, 18);
    be16(&mut name, 6, 3);
    be16(&mut name, 8, 1);
    be16(&mut name, 10, 0x0409);
    be16(&mut name, 12, 6);
    be16(&mut name, 14, 12);
    for ch in "FTTest".encode_utf16() {
        name.extend_from_slice(&ch.to_be_bytes());
    }
    tables.insert(*b"name", name);
    let mut bytes = vec![0; 12 + tables.len() * 16];
    be32(&mut bytes, 0, 0x00010000);
    be16(&mut bytes, 4, tables.len() as u16);
    for (index, (tag, data)) in tables.into_iter().enumerate() {
        while !bytes.len().is_multiple_of(4) {
            bytes.push(0);
        }
        let start = bytes.len();
        let at = 12 + index * 16;
        bytes[at..at + 4].copy_from_slice(&tag);
        be32(&mut bytes, at + 8, start as u32);
        be32(&mut bytes, at + 12, data.len() as u32);
        bytes.extend_from_slice(&data);
    }
    bytes
}

#[test]
fn real_loader_validates_original_sfnt_metadata_through_adapter() {
    let bytes = fixture();
    let metadata = StaticTrueTypeLoader
        .validate_static_truetype(&bytes)
        .unwrap();
    assert_eq!(metadata.units_per_em, 1000);
    assert_eq!(metadata.glyph_count, 3);
}
#[test]
fn adapter_rejects_malformed_and_non_truetype_without_fallback() {
    for bytes in [
        b"not a font".as_slice(),
        b"ttcf".as_slice(),
        b"OTTO".as_slice(),
    ] {
        assert!(StaticTrueTypeLoader
            .validate_static_truetype(bytes)
            .is_err());
    }
    let mut bytes = fixture();
    bytes[20..24].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(StaticTrueTypeLoader
        .validate_static_truetype(&bytes)
        .is_err());
}
#[test]
fn display_resources_are_verified_by_actual_loader_not_fake_callback() {
    let mut list = match parse(include_bytes!("fixtures/synthetic-display-list.json"))
        .unwrap()
        .message
    {
        Message::DisplayList(list) => list,
        _ => unreachable!(),
    };
    let capabilities = match parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    {
        Message::Offer(caps) => caps,
        _ => unreachable!(),
    };
    let bytes = fixture();
    list.fonts[0].sha256 = digest(&bytes);
    list.fonts[0].byte_length = bytes.len() as u64;
    let fonts = BTreeMap::from([("synthetic".into(), bytes)]);
    let documents = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: "office e\u{301}".into(),
        },
    )]);
    let result = list
        .validate_resources(&capabilities, &documents, &fonts, &StaticTrueTypeLoader)
        .unwrap();
    assert!(result.font_resources_verified);
    assert!(!result.paintable);
    list.fonts[0].glyph_count = 4;
    assert!(list
        .validate_resources(&capabilities, &documents, &fonts, &StaticTrueTypeLoader)
        .is_err());
}

#[test]
fn immutable_collection_matches_exact_descriptor_and_license_record() {
    use flashtex_font_resources::{
        EmbeddingPermission, FontCollection, LicenseMetadata, Manifest, ManifestEntry,
    };
    let root = tempfile::tempdir().unwrap();
    let bytes = fixture();
    let license = b"Original synthetic fixture; no real font license claimed";
    std::fs::write(root.path().join("font.ttf"), &bytes).unwrap();
    std::fs::write(root.path().join("LICENSE.txt"), license).unwrap();
    let mut list = match parse(include_bytes!("fixtures/synthetic-display-list.json"))
        .unwrap()
        .message
    {
        Message::DisplayList(list) => list,
        _ => unreachable!(),
    };
    let capabilities = match parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    {
        Message::Offer(caps) => caps,
        _ => unreachable!(),
    };
    list.fonts[0].sha256 = digest(&bytes);
    list.fonts[0].byte_length = bytes.len() as u64;
    list.fonts[0].postscript_name = "FTTest".into();
    let manifest = Manifest {
        schema_version: 1,
        resources: vec![ManifestEntry {
            font: descriptor(&list.fonts[0]),
            path: "font.ttf".into(),
            license: LicenseMetadata {
                identifier: "LicenseRef-Synthetic-Test".into(),
                copyright: "Synthetic fixture".into(),
                source: "Original local validation fixture".into(),
                text_path: "LICENSE.txt".into(),
                text_sha256: digest(license),
                embedding_permission: EmbeddingPermission::Unknown,
            },
        }],
    };
    let collection = FontCollection::load(root.path(), &manifest).unwrap();
    let documents = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: "office e\u{301}".into(),
        },
    )]);
    let evidence = validate_with_collection(&list, &capabilities, &documents, &collection).unwrap();
    assert!(evidence.font_resources_verified);
    assert!(!evidence.paintable);
    assert_eq!(
        collection
            .get("synthetic")
            .unwrap()
            .license()
            .embedding_permission,
        EmbeddingPermission::Unknown
    );
    list.fonts[0].postscript_name = "different".into();
    assert!(validate_with_collection(&list, &capabilities, &documents, &collection).is_err());
}

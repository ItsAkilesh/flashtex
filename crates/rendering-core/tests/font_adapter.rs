use flashtex_rendering_core::{font_adapter::*, *};
use std::collections::BTreeMap;
// Original synthetic empty-outline sfnt from font-resources16dab88 tests; no visual claim.
#[path = "support/font_fixture.rs"]
mod font_fixture;
use font_fixture::fixture;

#[test]
fn real_loader_validates_original_sfnt_metadata_through_adapter() {
    for bytes in [fixture(), font_fixture::triangle_fixture()] {
        let metadata = StaticTrueTypeLoader
            .validate_static_truetype(&bytes)
            .unwrap();
        assert_eq!(metadata.units_per_em, 1000);
        assert_eq!(metadata.glyph_count, 3);
    }
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
    let prepared =
        flashtex_rendering_core::outlines::PreparedOutlines::new(&list, &capabilities, &collection)
            .unwrap();
    let glyph = prepared.glyph(1, 0, 0).unwrap();
    assert_eq!(glyph.original_gid, 1);
    assert_eq!(glyph.sources[0].end_byte, 10);
    assert!(!glyph.hinting_applied);
    assert!(glyph.commands.is_empty()); // Synthetic fixture has empty glyph outlines.
    assert!(prepared.glyph(1, 0, 99).is_err());
    let mut cache = flashtex_rendering_core::glyph_cache::GlyphPathCache::new(8, 10000).unwrap();
    let first = prepared.glyph_cached(1, 0, 0, &mut cache).unwrap();
    let second = prepared.glyph_cached(1, 0, 0, &mut cache).unwrap();
    assert_eq!(first.commands, second.commands);
    assert_eq!(first.original_gid, glyph.original_gid);
    assert_eq!(cache.stats().hits, 1);
    list.fonts[0].postscript_name = "different".into();
    assert!(validate_with_collection(&list, &capabilities, &documents, &collection).is_err());
}

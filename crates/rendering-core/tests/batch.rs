use flashtex_font_resources::{
    EmbeddingPermission, FontCollection, LicenseMetadata, Manifest, ManifestEntry,
};
use flashtex_rendering_core::{batch::*, font_adapter::descriptor, glyph_cache::GlyphPathCache, *};
use std::collections::BTreeMap;
#[path = "support/font_fixture.rs"]
mod font_fixture;
struct Fixture {
    list: DisplayList,
    caps: Capabilities,
    docs: BTreeMap<String, SourceSnapshot>,
    fonts: FontCollection,
}
fn setup(triangle: bool) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let bytes = if triangle {
        font_fixture::triangle_fixture()
    } else {
        font_fixture::fixture()
    };
    std::fs::write(root.path().join("font.ttf"), &bytes).unwrap();
    std::fs::write(root.path().join("LICENSE"), b"synthetic").unwrap();
    let mut list = match parse(include_bytes!("fixtures/synthetic-display-list.json"))
        .unwrap()
        .message
    {
        Message::DisplayList(list) => list,
        _ => unreachable!(),
    };
    let caps = match parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    {
        Message::Offer(caps) => caps,
        _ => unreachable!(),
    };
    list.fonts[0].sha256 = digest(&bytes);
    list.fonts[0].byte_length = bytes.len() as u64;
    list.fonts[0].postscript_name = "FTTest".into();
    let entry = ManifestEntry {
        font: descriptor(&list.fonts[0]),
        path: "font.ttf".into(),
        license: LicenseMetadata {
            identifier: "LicenseRef-Synthetic".into(),
            copyright: "Original synthetic".into(),
            source: "test".into(),
            text_path: "LICENSE".into(),
            text_sha256: digest(b"synthetic"),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    };
    let fonts = FontCollection::load(
        root.path(),
        &Manifest {
            schema_version: 1,
            resources: vec![entry],
        },
    )
    .unwrap();
    Fixture {
        list,
        caps,
        docs: BTreeMap::from([(
            "main.tex".into(),
            SourceSnapshot {
                revision: 1,
                text: "office e\u{301}".into(),
            },
        )]),
        fonts,
    }
}
#[test]
fn paths_rules_paint_order_and_source_provenance_survive_batching() {
    let mut f = setup(true);
    f.list.pages[0].items.push(Item::Rule(Rule {
        x: Tick(5),
        top: Tick(5),
        width: Tick(20),
        height: Tick(2),
        paint: Paint {
            r: 1.,
            g: 0.,
            b: 0.,
            a: 0.5,
        },
        sources: None,
        synthetic_reason: Some("fraction rule".into()),
    }));
    let prepared = PreparedBatchSource::new(&f.list, &f.caps, &f.docs, &f.fonts).unwrap();
    let mut cache = GlyphPathCache::new(16, 100000).unwrap();
    let batch = prepared
        .page("p", 2, 1, None, BatchLimits::default(), &mut cache)
        .unwrap();
    assert_eq!(batch.operations.len(), 3);
    assert_eq!(batch.path_commands, 4);
    assert!(!batch.hinting_applied);
    if let DrawOperation::Glyph { path, .. } = &batch.operations[0] {
        assert_eq!(path.original_gid, 1);
        assert_eq!(path.sources[0].end_byte, 10);
        assert_eq!(path.commands.len(), 4);
    } else {
        panic!()
    }
    if let DrawOperation::Rule {
        paint,
        synthetic_reason,
        ..
    } = &batch.operations[2]
    {
        assert_eq!(paint.a, 0.5);
        assert_eq!(synthetic_reason.as_deref(), Some("fraction rule"));
    } else {
        panic!()
    }
}
#[test]
fn empty_clip_avoids_expansion_and_does_not_mean_unbounded_render() {
    let f = setup(false);
    let prepared = PreparedBatchSource::new(&f.list, &f.caps, &f.docs, &f.fonts).unwrap();
    let mut cache = GlyphPathCache::new(16, 100000).unwrap();
    let clip = HitRect {
        x: Tick(-100),
        top: Tick(0),
        width: Tick(100),
        height: Tick(100),
    };
    let batch = prepared
        .page("p", 2, 1, Some(&clip), BatchLimits::default(), &mut cache)
        .unwrap();
    assert!(batch.visible_clip.is_none());
    assert!(batch.operations.is_empty());
    assert_eq!(cache.stats().expansions, 0);
}
#[test]
fn clip_is_exact_and_glyphs_are_not_culled_by_source_hit_boxes() {
    let f = setup(true);
    let prepared = PreparedBatchSource::new(&f.list, &f.caps, &f.docs, &f.fonts).unwrap();
    let mut cache = GlyphPathCache::new(16, 100000).unwrap();
    let clip = HitRect {
        x: Tick(200),
        top: Tick(200),
        width: Tick(50),
        height: Tick(50),
    };
    let batch = prepared
        .page("p", 2, 1, Some(&clip), BatchLimits::default(), &mut cache)
        .unwrap();
    assert_eq!(batch.operations.len(), 2);
    assert_eq!(batch.visible_clip.unwrap().x, Tick(200));
}
#[test]
fn exceeded_budget_returns_no_partial_batch_and_retry_reuses_expansion() {
    let f = setup(true);
    let prepared = PreparedBatchSource::new(&f.list, &f.caps, &f.docs, &f.fonts).unwrap();
    let mut cache = GlyphPathCache::new(16, 100000).unwrap();
    assert!(prepared
        .page(
            "p",
            2,
            1,
            None,
            BatchLimits {
                max_operations: 1,
                max_path_commands: 100
            },
            &mut cache
        )
        .is_err());
    assert!(prepared
        .page(
            "p",
            2,
            1,
            None,
            BatchLimits {
                max_operations: 10,
                max_path_commands: 2
            },
            &mut cache
        )
        .is_err());
    let batch = prepared
        .page("p", 2, 1, None, BatchLimits::default(), &mut cache)
        .unwrap();
    assert_eq!(batch.operations.len(), 2);
    assert!(cache.stats().hits >= 2);
}
#[test]
fn stale_revision_and_changed_source_snapshots_fail_closed() {
    let mut f = setup(true);
    let prepared = PreparedBatchSource::new(&f.list, &f.caps, &f.docs, &f.fonts).unwrap();
    let mut cache = GlyphPathCache::new(16, 100000).unwrap();
    assert!(prepared
        .page("p", 3, 1, None, BatchLimits::default(), &mut cache)
        .is_err());
    f.docs.get_mut("main.tex").unwrap().text = "changed".into();
    assert!(PreparedBatchSource::new(&f.list, &f.caps, &f.docs, &f.fonts).is_err());
}

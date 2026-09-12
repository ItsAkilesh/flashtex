use flashtex_font_resources::{
    EmbeddingPermission, FontDescriptor, FontResource, LicenseMetadata, ManifestEntry,
};
use flashtex_rendering_core::{device_grid::*, digest, geometry_diff::*, outlines::*};
#[path = "support/font_fixture.rs"]
mod font_fixture;
fn font() -> FontResource {
    let bytes = font_fixture::grid_fixture();
    let entry = ManifestEntry {
        font: FontDescriptor {
            font_id: "test".into(),
            sha256: digest(&bytes),
            byte_length: bytes.len() as u64,
            format: "static-truetype".into(),
            face_index: 0,
            units_per_em: 1000,
            glyph_count: 3,
            postscript_name: "FTTest".into(),
        },
        path: "font.ttf".into(),
        license: LicenseMetadata {
            identifier: "LicenseRef-Synthetic".into(),
            copyright: "Original".into(),
            source: "test".into(),
            text_path: "LICENSE".into(),
            text_sha256: digest(b"synthetic"),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    };
    FontResource::from_bytes(&entry, &bytes, b"synthetic").unwrap()
}
fn context(ppem: u32, tie: TieRule) -> DeviceContext {
    DeviceContext::new(ppem, ppem, tie, &"1".repeat(64)).unwrap()
}
fn r(n: i128, d: u128) -> OutlineCoordinate {
    OutlineCoordinate::from_fraction(n, d).unwrap()
}
fn placed(path: &DevicePath) -> PositionedDevicePath {
    path.place(
        r(1000, 1),
        OutlinePoint {
            x: r(1, 2),
            y: r(7, 4),
        },
    )
    .unwrap()
}
#[test]
fn direct_cached_exact_paths_match_and_default_still_requires_context() {
    let font = font();
    assert!(font.expanded_outline(2).is_err());
    let ctx = context(10, TieRule::AwayFromZero);
    let direct = expand_device(&font, 2, &ctx).unwrap();
    let mut cache = DevicePathCache::new(4, 100000).unwrap();
    assert!(cache.lookup(&font, 2, None).is_err());
    let DeviceOutcome::Ready(cached) = cache.lookup(&font, 2, Some(&ctx)).unwrap().outcome else {
        panic!()
    };
    assert_eq!(direct.commands, cached.commands);
    assert_eq!(placed(&direct).commands, placed(&cached).commands);
    let hit = cache.lookup(&font, 2, Some(&ctx)).unwrap();
    assert!(hit.cache_hit);
    assert_eq!(cache.stats().expansions, 1);
    assert!(font.expanded_outline(2).is_err());
    assert_ne!(font_fixture::fixture(), font_fixture::triangle_fixture());
}
#[test]
fn ppem_tie_and_build_context_switches_invalidate_device_entries() {
    let font = font();
    let mut cache = DevicePathCache::new(4, 100000).unwrap();
    let a = context(10, TieRule::AwayFromZero);
    let b = context(10, TieRule::TowardPositive);
    let DeviceOutcome::Ready(old) = cache.lookup(&font, 2, Some(&a)).unwrap().outcome else {
        panic!()
    };
    let DeviceOutcome::Ready(new) = cache.lookup(&font, 2, Some(&b)).unwrap().outcome else {
        panic!()
    };
    assert_ne!(old.commands, new.commands);
    assert_eq!(cache.stats().context_invalidations, 1);
    assert_eq!(cache.stats().entries, 1);
    let c = context(20, TieRule::AwayFromZero);
    assert!(!cache.lookup(&font, 2, Some(&c)).unwrap().cache_hit);
    let changed = DeviceContext::new(20, 20, TieRule::AwayFromZero, &"2".repeat(64)).unwrap();
    assert!(!cache.lookup(&font, 2, Some(&changed)).unwrap().cache_hit);
    assert_eq!(cache.stats().context_invalidations, 3);
    assert_eq!(old.key.context, a);
}
#[test]
fn device_fixture_diff_reports_policy_and_exact_positions_without_equality_claim() {
    let font = font();
    let a = placed(&expand_device(&font, 2, &context(10, TieRule::AwayFromZero)).unwrap())
        .comparison_fixture(100000)
        .unwrap();
    let b = placed(&expand_device(&font, 2, &context(10, TieRule::TowardPositive)).unwrap())
        .comparison_fixture(100000)
        .unwrap();
    let report = compare(
        &ValidatedGeometry::device(&a).unwrap(),
        &ValidatedGeometry::device(&b).unwrap(),
        DiffLimits::default(),
    )
    .unwrap();
    assert_eq!(report.equal, Some(false));
    assert!(report
        .differences
        .iter()
        .any(|d| d.category == Category::FontResourceOrGid && d.path.contains("tie_rule")));
    assert!(report
        .differences
        .iter()
        .any(|d| d.category == Category::AdvanceOrPosition && d.delta.is_some()));
    let mut value: serde_json::Value = serde_json::from_slice(&a).unwrap();
    value.as_object_mut().unwrap().remove("device_context");
    assert!(ValidatedGeometry::device(&serde_json::to_vec(&value).unwrap()).is_err());
}
#[test]
fn device_cache_budgets_and_non_dyadic_context_remain_explicit() {
    let font = font();
    let mut cache = DevicePathCache::new(1, 512).unwrap();
    let DeviceOutcome::Unavailable(error) = cache
        .lookup(&font, 2, Some(&context(10, TieRule::AwayFromZero)))
        .unwrap()
        .outcome
    else {
        panic!()
    };
    assert!(matches!(*error, DeviceFailure::PayloadBudget { .. }));
    assert!(
        cache
            .lookup(&font, 2, Some(&context(10, TieRule::AwayFromZero)))
            .unwrap()
            .cache_hit
    );
    assert!(DeviceContext::new(0, 10, TieRule::AwayFromZero, &"1".repeat(64)).is_err());
    assert!(expand_device(&font, 2, &context(12, TieRule::AwayFromZero)).is_err());
}

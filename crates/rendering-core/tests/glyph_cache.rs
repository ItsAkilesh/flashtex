use flashtex_font_resources::{
    EmbeddingPermission, FontDescriptor, FontResource, LicenseMetadata, ManifestEntry,
};
use flashtex_rendering_core::{glyph_cache::*, *};
#[path = "support/font_fixture.rs"]
mod font_fixture;
fn font(alias: &str, change_metrics: bool) -> FontResource {
    let mut bytes = if alias == "triangle-fixture" {
        font_fixture::triangle_fixture()
    } else {
        font_fixture::fixture()
    };
    if change_metrics {
        let record = (0..u16::from_be_bytes([bytes[4], bytes[5]]) as usize)
            .map(|i| 12 + i * 16)
            .find(|i| &bytes[*i..*i + 4] == b"hmtx")
            .unwrap();
        let offset =
            u32::from_be_bytes(bytes[record + 8..record + 12].try_into().unwrap()) as usize;
        bytes[offset] = 1;
    }
    let entry = ManifestEntry {
        font: FontDescriptor {
            font_id: alias.into(),
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
            copyright: "Synthetic test".into(),
            source: "Original fixture".into(),
            text_path: "license.txt".into(),
            text_sha256: digest(b"fixture"),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    };
    FontResource::from_bytes(&entry, &bytes, b"fixture").unwrap()
}
#[test]
fn immutable_paths_reuse_by_content_not_font_alias() {
    let a = font("a", false);
    let b = font("b", false);
    let mut cache = GlyphPathCache::new(10, 10000).unwrap();
    let first = cache.lookup(&a, 1).unwrap();
    assert!(!first.cache_hit);
    let second = cache.lookup(&b, 1).unwrap();
    assert!(second.cache_hit);
    if let (PathOutcome::Ready(a), PathOutcome::Ready(b)) = (first.outcome, second.outcome) {
        assert!(std::sync::Arc::ptr_eq(&a, &b));
        assert_eq!(a.key.original_gid, 1);
    } else {
        panic!()
    }
    assert_eq!(cache.stats().expansions, 1);
}
#[test]
fn same_name_new_font_bytes_never_reuses_old_glyph() {
    let a = font("same", false);
    let b = font("same", true);
    let mut cache = GlyphPathCache::new(10, 10000).unwrap();
    cache.lookup(&a, 1).unwrap();
    assert!(!cache.lookup(&b, 1).unwrap().cache_hit);
    assert_eq!(cache.stats().expansions, 2);
}
#[test]
fn invalid_gid_outcome_is_explicit_and_cached_without_repeated_expansion() {
    let a = font("a", false);
    let mut cache = GlyphPathCache::new(10, 10000).unwrap();
    assert!(matches!(
        cache.lookup(&a, 4).unwrap().outcome,
        PathOutcome::Unavailable(_)
    ));
    assert!(cache.lookup(&a, 4).unwrap().cache_hit);
    assert_eq!(cache.stats().expansions, 1);
}
#[test]
fn lru_capacity_and_clear_are_bounded() {
    let a = font("a", false);
    let mut cache = GlyphPathCache::new(2, 10000).unwrap();
    cache.lookup(&a, 0).unwrap();
    cache.lookup(&a, 1).unwrap();
    cache.lookup(&a, 0).unwrap();
    cache.lookup(&a, 2).unwrap();
    assert_eq!(cache.stats().evictions, 1);
    assert!(cache.lookup(&a, 0).unwrap().cache_hit);
    assert!(!cache.lookup(&a, 1).unwrap().cache_hit);
    cache.clear();
    assert_eq!(cache.stats().entries, 0);
    assert_eq!(cache.stats().retained_payload_bytes, 0);
    assert!(!cache.lookup(&a, 0).unwrap().cache_hit);
}
#[test]
fn payload_budget_evicts_before_exceeding_cap() {
    let a = font("a", false);
    let mut cache = GlyphPathCache::new(100, 256).unwrap();
    for gid in 0..3 {
        cache.lookup(&a, gid).unwrap();
        assert!(cache.stats().retained_payload_bytes <= 256);
    }
    assert!(cache.stats().evictions > 0);
}

#[test]
fn oversized_expansion_reports_explicit_budget_and_negative_caches_it() {
    let font = font("triangle-fixture", false);
    let mut cache = GlyphPathCache::new(8, 256).unwrap();
    let first = cache.lookup(&font, 1).unwrap();
    assert!(
        matches!(first.outcome,PathOutcome::Unavailable(ref error) if matches!(**error,PathFailure::PayloadBudget{..}))
    );
    assert!(cache.lookup(&font, 1).unwrap().cache_hit);
    assert_eq!(cache.stats().expansions, 1);
    assert!(cache.stats().retained_payload_bytes <= 256);
}

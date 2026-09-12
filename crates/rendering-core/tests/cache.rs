use flashtex_rendering_core::{cache::*, *};
use std::{collections::BTreeMap, sync::Arc};
struct FakeFont;
impl FontValidator for FakeFont {
    fn validate_static_truetype(&self, _: &[u8]) -> Result<FontMetadata> {
        Ok(FontMetadata {
            units_per_em: 1000,
            glyph_count: 3,
        })
    }
}
fn setup() -> (
    DisplayList,
    Capabilities,
    BTreeMap<String, SourceSnapshot>,
    BTreeMap<String, Vec<u8>>,
) {
    let mut list = match parse(include_bytes!("fixtures/synthetic-display-list.json"))
        .unwrap()
        .message
    {
        Message::DisplayList(l) => l,
        _ => unreachable!(),
    };
    let caps = match parse(include_bytes!("fixtures/capabilities.json"))
        .unwrap()
        .message
    {
        Message::Offer(c) => c,
        _ => unreachable!(),
    };
    let bytes = b"synthetic-only".to_vec();
    list.fonts[0].sha256 = digest(&bytes);
    list.fonts[0].byte_length = bytes.len() as u64;
    (
        list,
        caps,
        BTreeMap::from([(
            "main.tex".into(),
            SourceSnapshot {
                revision: 1,
                text: "office e\u{301}".into(),
            },
        )]),
        BTreeMap::from([("synthetic".into(), bytes)]),
    )
}
fn identity(list: &DisplayList) -> RenderIdentity {
    RenderIdentity::new(
        list.project_id.clone(),
        list.revision,
        "b".repeat(64),
        list.documents.clone(),
        list.fonts.clone(),
    )
    .unwrap()
}
#[test]
fn identical_frame_is_reused_immutably_with_verified_font_bytes() {
    let (list, caps, docs, fonts) = setup();
    let mut cache = DisplayCache::new(4, 100000).unwrap();
    let ticket = cache.begin(identity(&list)).unwrap();
    let first = cache
        .install(&ticket, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .unwrap();
    let second = cache
        .install(&ticket, list, &caps, &docs, &fonts, &FakeFont)
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(
        first.font_bytes("synthetic"),
        Some(b"synthetic-only".as_slice())
    );
    assert!(!first.paintable());
    assert!(cache.retained_payload_bytes() > 0);
}
#[test]
fn new_revision_rejects_late_old_result_and_clears_current_frame() {
    let (mut list, caps, docs, fonts) = setup();
    let mut cache = DisplayCache::new(4, 100000).unwrap();
    let old = cache.begin(identity(&list)).unwrap();
    let previous = cache
        .install(&old, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .unwrap();
    list.revision = 3;
    let next = cache.begin(identity(&list)).unwrap();
    assert!(cache.current(&next).unwrap().is_none());
    assert!(cache.current(&old).is_err());
    assert!(cache
        .install(&old, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .is_err());
    assert_eq!(previous.display().revision, 2);
    cache
        .install(&next, list, &caps, &docs, &fonts, &FakeFont)
        .unwrap();
}
#[test]
fn same_font_id_with_new_bytes_invalidates_even_at_same_revision() {
    let (mut list, caps, docs, mut fonts) = setup();
    let mut cache = DisplayCache::new(4, 100000).unwrap();
    let old = cache.begin(identity(&list)).unwrap();
    cache
        .install(&old, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .unwrap();
    let bytes = b"replacement-font".to_vec();
    list.fonts[0].sha256 = digest(&bytes);
    list.fonts[0].byte_length = bytes.len() as u64;
    fonts.insert("synthetic".into(), bytes);
    let next = cache.begin(identity(&list)).unwrap();
    assert!(cache.current(&old).is_err());
    assert!(cache.current(&next).unwrap().is_none());
    cache
        .install(&next, list, &caps, &docs, &fonts, &FakeFont)
        .unwrap();
}
#[test]
fn source_hash_and_configuration_changes_invalidate_tickets() {
    let (mut list, _, _, _) = setup();
    let mut cache = DisplayCache::new(4, 100000).unwrap();
    let old = cache.begin(identity(&list)).unwrap();
    list.documents[0].sha256 = "c".repeat(64);
    let next = cache.begin(identity(&list)).unwrap();
    assert!(cache.current(&old).is_err());
    let changed = RenderIdentity::new(
        "p".into(),
        2,
        "d".repeat(64),
        list.documents.clone(),
        list.fonts.clone(),
    )
    .unwrap();
    cache.begin(changed).unwrap();
    assert!(cache.current(&next).is_err());
}
#[test]
fn stale_manifest_or_conflicting_same_identity_cannot_replace_frame() {
    let (list, caps, docs, fonts) = setup();
    let mut cache = DisplayCache::new(4, 100000).unwrap();
    let ticket = cache.begin(identity(&list)).unwrap();
    let first = cache
        .install(&ticket, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .unwrap();
    let mut changed = list.clone();
    changed.pages[0].width = Tick(999);
    assert!(cache
        .install(&ticket, changed, &caps, &docs, &fonts, &FakeFont)
        .is_err());
    assert!(Arc::ptr_eq(
        &first,
        &cache.current(&ticket).unwrap().unwrap()
    ));
    let mut changed = list;
    changed.revision = 3;
    assert!(cache
        .install(&ticket, changed, &caps, &docs, &fonts, &FakeFont)
        .is_err());
}
#[test]
fn invalidation_blocks_inflight_jobs_without_losing_revision_high_water() {
    let (mut list, caps, docs, fonts) = setup();
    let mut cache = DisplayCache::new(4, 100000).unwrap();
    let old = cache.begin(identity(&list)).unwrap();
    cache.invalidate("p").unwrap();
    assert!(cache
        .install(&old, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .is_err());
    let next = cache.begin(identity(&list)).unwrap();
    cache
        .install(&next, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .unwrap();
    list.revision = 1;
    assert!(cache.begin(identity(&list)).is_err());
}
#[test]
fn resource_or_capacity_failures_leave_cache_empty() {
    let (list, caps, docs, mut fonts) = setup();
    let mut cache = DisplayCache::new(1, 100000).unwrap();
    let ticket = cache.begin(identity(&list)).unwrap();
    fonts.insert("synthetic".into(), b"wrong".to_vec());
    assert!(cache
        .install(&ticket, list.clone(), &caps, &docs, &fonts, &FakeFont)
        .is_err());
    assert!(cache.current(&ticket).unwrap().is_none());
    let (list, caps, docs, fonts) = setup();
    let mut tiny = DisplayCache::new(1, 1).unwrap();
    let ticket = tiny.begin(identity(&list)).unwrap();
    assert!(tiny
        .install(&ticket, list, &caps, &docs, &fonts, &FakeFont)
        .is_err());
    assert_eq!(tiny.retained_payload_bytes(), 0);
}

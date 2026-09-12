use flashtex_font_resources::cff::HintPolicy;
use flashtex_rendering_core::{batch::ExactClip, cubic::*, digest, outlines::*};
fn index(objects: &[&[u8]]) -> Vec<u8> {
    let mut b = (objects.len() as u16).to_be_bytes().to_vec();
    if objects.is_empty() {
        return b;
    }
    b.push(1);
    let mut off = 1;
    b.push(off);
    for object in objects {
        off += object.len() as u8;
        b.push(off);
    }
    for object in objects {
        b.extend(*object);
    }
    b
}
fn fixture(program: &[u8], matrix: bool) -> Vec<u8> {
    let mut top = vec![29, 0, 0, 0, 0, 17];
    if matrix {
        top.extend([
            30, 0x0a, 0x00, 0x2f, 139, 139, 30, 0x0a, 0x00, 0x3f, 140, 141, 12, 7,
        ]);
    }
    let offset = 4 + index(&[b"F"]).len() + index(&[&top]).len() + 4;
    top[1..5].copy_from_slice(&(offset as u32).to_be_bytes());
    let mut b = vec![1, 0, 4, 4];
    b.extend(index(&[b"F"]));
    b.extend(index(&[&top]));
    b.extend(index(&[]));
    b.extend(index(&[]));
    b.extend(index(&[&[14], program]));
    b
}
fn r(n: i128, d: u128) -> OutlineCoordinate {
    OutlineCoordinate::from_fraction(n, d).unwrap()
}
fn origin() -> OutlinePoint {
    OutlinePoint {
        x: r(1, 2),
        y: r(7, 4),
    }
}
fn program() -> Vec<u8> {
    vec![139, 139, 21, 239, 139, 139, 239, 39, 139, 8, 14]
}
#[test]
fn default_matrix_keeps_cubic_controls_exact_and_flips_baseline() {
    let bytes = fixture(&program(), false);
    let resource = CffConsumer::from_table(&bytes, &digest(&bytes)).unwrap();
    let path = resource
        .place_glyph(1, HintPolicy::Unhinted, r(1, 3), origin(), 100)
        .unwrap();
    assert_eq!(path.original_gid, 1);
    assert_eq!(path.font_matrix[0].numerator(), 1);
    assert_eq!(path.font_matrix[0].denominator(), 1000);
    assert_eq!(path.commands.len(), 3);
    let CubicPathCommand::CurveTo {
        control1,
        control2,
        end,
    } = path.commands[1]
    else {
        panic!()
    };
    assert_eq!(
        control1,
        OutlinePoint {
            x: r(8, 15),
            y: r(7, 4)
        }
    );
    assert_eq!(
        control2,
        OutlinePoint {
            x: r(8, 15),
            y: r(103, 60)
        }
    );
    assert_eq!(
        end,
        OutlinePoint {
            x: r(1, 2),
            y: r(103, 60)
        }
    );
    assert!(!path.hinting_applied);
    assert_eq!(path.hints.policy, HintPolicy::Unhinted);
}
#[test]
fn custom_font_matrix_translation_is_scaled_once() {
    let bytes = fixture(&program(), true);
    let resource = CffConsumer::from_table(&bytes, &digest(&bytes)).unwrap();
    let path = resource
        .place_glyph(1, HintPolicy::Unhinted, r(2, 1), origin(), 100)
        .unwrap();
    let CubicPathCommand::MoveTo(p) = path.commands[0] else {
        panic!()
    };
    assert_eq!(p.x, r(5, 2));
    assert_eq!(p.y, r(-9, 4));
    let CubicPathCommand::CurveTo { control1, .. } = path.commands[1] else {
        panic!()
    };
    assert_eq!(control1.x, r(29, 10));
    assert_eq!(
        path.advance,
        OutlinePoint {
            x: r(0, 1),
            y: r(0, 1)
        }
    );
}
#[test]
fn hints_require_explicit_policy_and_metadata_survives() {
    let mut program = vec![139, 149, 1, 19, 128];
    program.extend([139, 139, 21, 239, 139, 5, 14]);
    let bytes = fixture(&program, false);
    let resource = CffConsumer::from_table(&bytes, &digest(&bytes)).unwrap();
    assert!(matches!(
        resource.place_glyph(1, HintPolicy::Reject, r(1, 1), origin(), 100),
        Err(CubicError::Resource(_))
    ));
    let path = resource
        .place_glyph(1, HintPolicy::Unhinted, r(1, 1), origin(), 100)
        .unwrap();
    assert_eq!(path.hints.stems.len(), 1);
    assert_eq!(path.hints.masks[0].bytes, [128]);
    assert!(!path.hinting_applied);
}
#[test]
fn digest_gid_budget_and_precision_fail_closed() {
    let bytes = fixture(&program(), false);
    assert!(matches!(
        CffConsumer::from_table(&bytes, &"0".repeat(64)),
        Err(CubicError::DigestMismatch)
    ));
    let resource = CffConsumer::from_table(&bytes, &digest(&bytes)).unwrap();
    assert!(matches!(
        resource.place_glyph(0, HintPolicy::Unhinted, r(1, 1), origin(), 100),
        Err(CubicError::Notdef)
    ));
    assert!(matches!(
        resource.place_glyph(1, HintPolicy::Unhinted, r(1, 1), origin(), 1),
        Err(CubicError::Budget)
    ));
    assert!(resource
        .place_glyph(2, HintPolicy::Unhinted, r(1, 1), origin(), 100)
        .is_err());
    assert!(resource
        .place_glyph(1, HintPolicy::Unhinted, r(0, 1), origin(), 100)
        .is_err());
    assert!(resource
        .place_glyph(1, HintPolicy::Unhinted, r(1, 1u128 << 126), origin(), 100)
        .is_err());
}
#[test]
fn fractional_clip_is_attached_without_curve_approximation() {
    let bytes = fixture(&program(), false);
    let resource = CffConsumer::from_table(&bytes, &digest(&bytes)).unwrap();
    let path = resource
        .place_glyph(1, HintPolicy::Unhinted, r(1, 3), origin(), 100)
        .unwrap();
    let original = path.commands.clone();
    let clip = ExactClip {
        left: r(1, 2),
        top: r(1, 1),
        right: r(3, 4),
        bottom: r(2, 1),
    };
    let clipped = path.with_clip(clip).unwrap();
    assert_eq!(clipped.clip, clip);
    assert_eq!(clipped.outline.commands, original);
}
#[test]
fn full_font_cache_identity_and_exact_geometry_survive_shared_hits() {
    use flashtex_font_resources::cff::{CacheLimits, CacheStatus, CffOutlineCache};
    let bytes = fixture(&program(), false);
    let mut full = b"OTTO-original-fixture".to_vec();
    let start = full.len();
    full.extend(&bytes);
    let cache = CffOutlineCache::from_font_table(
        &full,
        0,
        start..full.len(),
        CacheLimits {
            max_entries: 4,
            max_bytes: 100000,
        },
    )
    .unwrap();
    let consumer = CachedCffConsumer::new(cache);
    let a = consumer
        .place_cached(1, HintPolicy::Unhinted, r(1, 3), origin(), 100)
        .unwrap();
    assert_eq!(a.cache_status, CacheStatus::Stored);
    let b = consumer
        .clone()
        .place_cached(1, HintPolicy::Unhinted, r(1, 3), origin(), 100)
        .unwrap();
    assert_eq!(b.cache_status, CacheStatus::Hit);
    assert_eq!(a.outline.commands, b.outline.commands);
    assert_eq!(
        b.outline.full_font_identity.as_ref().unwrap().font_sha256,
        digest(&full)
    );
    assert_eq!(
        b.outline.full_font_identity.as_ref().unwrap().table_range,
        start..full.len()
    );
    let direct = CffConsumer::from_table(&bytes, &digest(&bytes))
        .unwrap()
        .place_glyph(1, HintPolicy::Unhinted, r(1, 3), origin(), 100)
        .unwrap();
    assert_eq!(direct.commands, b.outline.commands);
    assert_eq!(direct.advance, b.outline.advance);
}
#[test]
fn oversize_cache_bypass_is_explicit_without_geometry_loss() {
    use flashtex_font_resources::cff::{CacheLimits, CacheStatus, CffOutlineCache};
    let bytes = fixture(&program(), false);
    let mut full = b"OTTO".to_vec();
    full.extend(bytes);
    let cache = CffOutlineCache::from_font_table(
        &full,
        0,
        4..full.len(),
        CacheLimits {
            max_entries: 0,
            max_bytes: 0,
        },
    )
    .unwrap();
    let consumer = CachedCffConsumer::new(cache);
    let placed = consumer
        .place_cached(1, HintPolicy::Unhinted, r(1, 3), origin(), 100)
        .unwrap();
    assert_eq!(placed.cache_status, CacheStatus::BypassedOversize);
    assert_eq!(placed.outline.commands.len(), 3);
}

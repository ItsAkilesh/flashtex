use flashtex_font_resources::{
    cff::HintPolicy,
    vf_graph::{ResourceKey, SourceStep},
};
use flashtex_rendering_core::{
    batch::*,
    cubic::CffConsumer,
    mixed::*,
    outlines::*,
    tex_adapter::{TracedBatch, TracedPrimitive},
    *,
};
fn r(n: i128, d: u128) -> OutlineCoordinate {
    OutlineCoordinate::from_fraction(n, d).unwrap()
}
fn clip() -> ExactClip {
    ExactClip {
        left: r(0, 1),
        top: r(0, 1),
        right: r(100, 1),
        bottom: r(100, 1),
    }
}
fn paint() -> Paint {
    Paint {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    }
}
fn source(index: usize) -> Vec<SourceStep> {
    vec![SourceStep {
        resource: ResourceKey::Physical {
            font_sha256: "1".repeat(64),
            tfm_sha256: "2".repeat(64),
            face_index: 0,
        },
        character: 65 + index as u8,
        command_index: Some(index),
    }]
}
fn fixture() -> TracedBatch {
    let mut primitives = Vec::new();
    for index in 0..2 {
        let id = PrimitiveId {
            item_index: index,
            glyph_index: Some(0),
        };
        // Explicit synthetic upstream geometry, not an installed-font outline claim.
        let path = PositionedGlyph {
            project_id: "test".into(),
            revision: 1,
            page: 1,
            item_index: index,
            glyph_index: 0,
            font_id: "synthetic".into(),
            font_sha256: "1".repeat(64),
            original_gid: 1,
            cluster_index: index as u32,
            logical_start_byte: index as u64,
            logical_end_byte: index as u64 + 1,
            sources: vec![],
            synthetic_reason: Some("original synthetic upstream fixture".into()),
            instances: vec![],
            commands: vec![
                PlacedPathCommand::MoveTo(OutlinePoint {
                    x: r(1, 3),
                    y: r(1, 2),
                }),
                PlacedPathCommand::QuadTo {
                    control: OutlinePoint {
                        x: r(2, 3),
                        y: r(3, 4),
                    },
                    end: OutlinePoint {
                        x: r(1, 1),
                        y: r(2, 1),
                    },
                },
            ],
            hinting_applied: false,
        };
        primitives.push(TracedPrimitive {
            identity: id,
            operation: DrawOperation::Glyph {
                path: Box::new(path),
                paint: paint(),
            },
            source_chain: source(index),
        });
    }
    let id = PrimitiveId {
        item_index: 2,
        glyph_index: None,
    };
    primitives.push(TracedPrimitive {
        identity: id,
        operation: DrawOperation::ExactRule {
            primitive_id: id,
            geometry: ExactClip {
                left: r(1, 2),
                top: r(1, 3),
                right: r(3, 4),
                bottom: r(2, 3),
            },
            paint: paint(),
            sources: vec![],
            synthetic_reason: Some("fixture rule".into()),
        },
        source_chain: source(2),
    });
    TracedBatch {
        project_id: "test".into(),
        revision: 1,
        page: 1,
        page_width: Tick(100),
        page_height: Tick(100),
        visible_clip: Some(clip()),
        primitives,
        path_commands: 4,
        color_space: "srgb",
        compositing: "source-over",
        hinting_applied: false,
    }
}
fn cff_bytes() -> Vec<u8> {
    let program = [139, 139, 21, 239, 139, 139, 239, 39, 139, 8, 14];
    let mut b = vec![
        1, 0, 4, 4, 0, 1, 1, 1, 2, b'F', 0, 1, 1, 1, 3, 160, 17, 0, 0, 0, 0, 0, 2, 1, 1, 2, 13, 14,
    ];
    b.extend(program);
    b
}
fn cff() -> CffConsumer {
    let b = cff_bytes();
    CffConsumer::from_table(&b, &digest(&b)).unwrap()
}
fn context() -> MixedContext<'static> {
    MixedContext {
        project_id: "test",
        revision: 1,
        page: 1,
        page_width: Tick(100),
        page_height: Tick(100),
        clip: clip(),
    }
}
fn inputs<'a>(
    s: &'a TracedBatch,
    c: &'a dyn flashtex_rendering_core::cubic::CubicProvider,
) -> Vec<MixedInput<'a>> {
    vec![
        MixedInput {
            source: s,
            primitive_index: 0,
            cubic: None,
        },
        MixedInput {
            source: s,
            primitive_index: 1,
            cubic: Some(CubicReplacement {
                resource: c,
                original_gid: 1,
                policy: HintPolicy::Unhinted,
                size: r(1, 3),
                origin: OutlinePoint {
                    x: r(1, 2),
                    y: r(7, 4),
                },
            }),
        },
        MixedInput {
            source: s,
            primitive_index: 2,
            cubic: None,
        },
    ]
}
#[test]
fn mixed_fonts_preserve_order_cubic_controls_and_source_identity() {
    let s = fixture();
    let c = cff();
    let batch = MixedBatch::build(context(), &inputs(&s, &c), MixedLimits::default()).unwrap();
    assert_eq!(batch.primitives().len(), 3);
    assert!(matches!(
        batch.primitives()[0].geometry,
        MixedGeometry::Quadratic(_)
    ));
    let MixedGeometry::Cubic(path) = &batch.primitives()[1].geometry else {
        panic!()
    };
    assert_eq!(path.original_gid, 1);
    assert!(!path.hinting_applied);
    assert_eq!(batch.primitives()[1].source_chain[0].character, 66);
    assert_eq!(batch.primitives()[1].identity.item_index, 1);
    assert!(matches!(
        batch.primitives()[2].geometry,
        MixedGeometry::Rule(_)
    ));
    assert_eq!(batch.command_count(), 5);
    let json: serde_json::Value = serde_json::from_slice(batch.fixture_bytes()).unwrap();
    assert_eq!(json["format"], "flashtex-internal-mixed-v1");
    assert_eq!(
        json["primitives"][0]["geometry"]["commands"][0][1][0],
        serde_json::json!(["1", "3"])
    );
}
#[test]
fn strict_serialized_byte_boundary_and_command_budget_are_atomic() {
    let s = fixture();
    let c = cff();
    let batch = MixedBatch::build(context(), &inputs(&s, &c), MixedLimits::default()).unwrap();
    let size = batch.fixture_bytes().len();
    let exact = MixedLimits {
        max_serialized_bytes: size,
        ..MixedLimits::default()
    };
    assert!(MixedBatch::build(context(), &inputs(&s, &c), exact).is_ok());
    assert!(matches!(
        MixedBatch::build(
            context(),
            &inputs(&s, &c),
            MixedLimits {
                max_serialized_bytes: size - 1,
                ..exact
            }
        ),
        Err(MixedError::Budget)
    ));
    assert!(MixedBatch::build(
        context(),
        &inputs(&s, &c),
        MixedLimits {
            max_commands: 4,
            ..exact
        }
    )
    .is_err());
    assert_eq!(s.primitives.len(), 3);
    assert!(MixedBatch::build(context(), &inputs(&s, &c), exact).is_ok());
}
#[test]
fn mismatched_revision_duplicate_identity_and_rule_replacement_fail() {
    let mut s = fixture();
    let c = cff();
    let mut context = context();
    context.revision = 2;
    assert!(matches!(
        MixedBatch::build(context, &inputs(&s, &c), MixedLimits::default()),
        Err(MixedError::Identity)
    ));
    let mut duplicate = inputs(&s, &c);
    duplicate[2].primitive_index = 0;
    assert!(matches!(
        MixedBatch::build(super_context(), &duplicate, MixedLimits::default()),
        Err(MixedError::Identity)
    ));
    drop(duplicate);
    let mut rule_replacement = inputs(&s, &c);
    rule_replacement[1].primitive_index = 2;
    assert!(matches!(
        MixedBatch::build(
            super_context(),
            &rule_replacement[..2],
            MixedLimits::default()
        ),
        Err(MixedError::Unsupported(_))
    ));
    drop(rule_replacement);
    s.primitives[1].identity.item_index = 99;
    assert!(MixedBatch::build(super_context(), &inputs(&s, &c), MixedLimits::default()).is_err());
}
fn super_context() -> MixedContext<'static> {
    context()
}
#[test]
fn fractional_rule_clip_culling_never_relabels_the_remaining_primitives() {
    let s = fixture();
    let c = cff();
    let mut ctx = context();
    ctx.clip = ExactClip {
        left: r(1, 1),
        top: r(0, 1),
        right: r(2, 1),
        bottom: r(100, 1),
    };
    let batch = MixedBatch::build(ctx, &inputs(&s, &c), MixedLimits::default()).unwrap();
    assert_eq!(batch.primitives().len(), 2);
    assert_eq!(batch.primitives()[1].source_chain[0].character, 66);
    assert_eq!(batch.primitives()[1].identity.item_index, 1);
}
#[test]
fn replay_keeps_large_rational_coordinates_and_metadata_exact() {
    use flashtex_rendering_core::mixed_replay::*;
    let mut source = fixture();
    let c = cff();
    let exact = r((1i128 << 100) + 1, (1u128 << 90) + 1);
    let DrawOperation::Glyph { path, .. } = &mut source.primitives[0].operation else {
        panic!()
    };
    path.commands[0] = PlacedPathCommand::MoveTo(OutlinePoint {
        x: exact,
        y: r(1, 7),
    });
    let batch = MixedBatch::build(context(), &inputs(&source, &c), MixedLimits::default()).unwrap();
    let replay = ReplayBatch::parse(batch.fixture_bytes(), MixedLimits::default()).unwrap();
    assert_eq!(replay.command_count(), batch.command_count());
    let ReplayGeometry::Quadratic(commands) = &replay.primitives()[0].geometry else {
        panic!()
    };
    assert_eq!(
        commands[0],
        ReplayCommand::Move(OutlinePoint {
            x: exact,
            y: r(1, 7)
        })
    );
    let canonical = replay.canonical_bytes().unwrap();
    let again = ReplayBatch::parse(&canonical, MixedLimits::default()).unwrap();
    assert_eq!(again.primitives(), replay.primitives());
    assert_eq!(again.metadata(), replay.metadata());
    assert_eq!(again.canonical_bytes().unwrap(), canonical);
}
#[test]
fn replay_rejects_unknown_primitives_commands_duplicate_keys_and_noncanonical_rationals() {
    use flashtex_rendering_core::mixed_replay::ReplayBatch;
    let source = fixture();
    let c = cff();
    let batch = MixedBatch::build(context(), &inputs(&source, &c), MixedLimits::default()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(batch.fixture_bytes()).unwrap();
    for mode in 0..5 {
        let mut value = original.clone();
        match mode {
            0 => value["primitives"][0]["geometry"]["kind"] = "image".into(),
            1 => value["primitives"][0]["geometry"]["commands"][0][0] = "cubic".into(),
            2 => {
                value["primitives"][0]["geometry"]["commands"][0][1][0] =
                    serde_json::json!(["2", "6"])
            }
            3 => value["primitives"][0]["extra"] = true.into(),
            _ => value["commands"] = 0.into(),
        };
        assert!(
            ReplayBatch::parse(&serde_json::to_vec(&value).unwrap(), MixedLimits::default())
                .is_err(),
            "{mode}"
        );
    }
    let duplicate = String::from_utf8(batch.fixture_bytes().to_vec())
        .unwrap()
        .replacen("\"page\":1", "\"page\":2,\"page\":1", 1);
    assert!(ReplayBatch::parse(duplicate.as_bytes(), MixedLimits::default()).is_err());
    assert!(matches!(
        ReplayBatch::parse(
            batch.fixture_bytes(),
            MixedLimits {
                max_serialized_bytes: batch.fixture_bytes().len() - 1,
                ..MixedLimits::default()
            }
        ),
        Err(MixedError::Budget)
    ));
}
#[test]
fn checked_in_illustrative_mixed_fixture_replays_all_primitive_types() {
    use flashtex_rendering_core::mixed_replay::*;
    let replay = ReplayBatch::parse(
        include_bytes!("fixtures/synthetic-mixed.json"),
        MixedLimits::default(),
    )
    .unwrap();
    assert_eq!(replay.primitives().len(), 3);
    assert_eq!(replay.command_count(), 5);
    let canonical = replay.canonical_bytes().unwrap();
    let next = ReplayBatch::parse(&canonical, MixedLimits::default()).unwrap();
    assert_eq!(replay.primitives(), next.primitives());
    assert_eq!(replay.metadata(), next.metadata());
}
#[test]
fn cached_cubic_mixed_replay_retains_full_font_and_table_identity() {
    use flashtex_font_resources::cff::{CacheLimits, CffOutlineCache};
    use flashtex_rendering_core::{cubic::CachedCffConsumer, mixed_replay::ReplayBatch};
    let mut bytes = b"OTTO".to_vec();
    bytes.extend(cff_bytes());
    let cache = CffOutlineCache::from_font_table(
        &bytes,
        0,
        4..bytes.len(),
        CacheLimits {
            max_entries: 4,
            max_bytes: 100000,
        },
    )
    .unwrap();
    let c = CachedCffConsumer::new(cache);
    let s = fixture();
    let batch = MixedBatch::build(context(), &inputs(&s, &c), MixedLimits::default()).unwrap();
    let replay = ReplayBatch::parse(batch.fixture_bytes(), MixedLimits::default()).unwrap();
    let identity = &replay.metadata()["primitives"][1]["geometry"]["full_font_identity"];
    assert_eq!(identity["font_sha256"], digest(&bytes));
    assert_eq!(identity["cff_sha256"], digest(&bytes[4..]));
    assert_eq!(identity["table_range"], serde_json::json!([4, bytes.len()]));
}
#[test]
fn provider_cannot_silently_change_hint_policy_or_claim_grid_fitting() {
    use flashtex_rendering_core::cubic::*;
    struct Wrong(CffConsumer);
    impl CubicProvider for Wrong {
        fn place_glyph(
            &self,
            gid: u16,
            policy: HintPolicy,
            size: OutlineCoordinate,
            origin: OutlinePoint,
            max_commands: usize,
        ) -> CubicResult<PositionedCubic> {
            let mut result = self
                .0
                .place_glyph(gid, policy, size, origin, max_commands)?;
            result.hinting_applied = true;
            Ok(result)
        }
    }
    let c = Wrong(cff());
    let s = fixture();
    assert!(matches!(
        MixedBatch::build(context(), &inputs(&s, &c), MixedLimits::default()),
        Err(MixedError::Identity)
    ));
}
fn residency_inputs(
    text: &str,
    config: &str,
) -> flashtex_rendering_core::residency::ResidencyInputs {
    use flashtex_rendering_core::residency::*;
    use std::collections::{BTreeMap, BTreeSet};
    ResidencyInputs::new(
        "test",
        1,
        config,
        vec![DocumentResource {
            path: "main.tex".into(),
            revision: 1,
            sha256: digest(text.as_bytes()),
            byte_length: text.len() as u64,
        }],
        BTreeMap::from([(
            "main.tex".into(),
            SourceSnapshot {
                revision: 1,
                text: text.into(),
            },
        )]),
        BTreeSet::from([
            ResourceIdentity::TrueType {
                sha256: "1".repeat(64),
            },
            ResourceIdentity::CffTable {
                sha256: digest(&cff_bytes()),
            },
        ]),
    )
    .unwrap()
}
fn residency_limits() -> flashtex_rendering_core::residency::ResidencyLimits {
    flashtex_rendering_core::residency::ResidencyLimits {
        max_pages: 2,
        max_encoded_bytes: 100000,
        max_commands: 100,
    }
}
#[test]
fn source_or_configuration_change_rejects_already_prepared_completion() {
    use flashtex_rendering_core::residency::*;
    let source = fixture();
    let c = cff();
    let mut cache = MixedResidency::new("test", residency_limits()).unwrap();
    let first = cache
        .begin(residency_inputs("old", &"3".repeat(64)))
        .unwrap();
    let pending = first
        .build(context(), &inputs(&source, &c), MixedLimits::default())
        .unwrap();
    let next = cache
        .begin(residency_inputs("new", &"3".repeat(64)))
        .unwrap();
    assert_eq!(next.snapshots()["main.tex"].text, "new");
    assert!(cache.install(pending).is_err());
    assert!(cache.page(&first, 1).is_err());
    let ready = next
        .build(context(), &inputs(&source, &c), MixedLimits::default())
        .unwrap();
    let held = cache.install(ready).unwrap();
    assert_eq!(cache.stats().pages, 1);
    let changed = cache
        .begin(residency_inputs("new", &"4".repeat(64)))
        .unwrap();
    assert_eq!(cache.stats().pages, 0);
    assert!(cache.page(&next, 1).is_err());
    assert!(cache.page(&changed, 1).unwrap().is_none());
    assert_eq!(held.primitives().len(), 3);
}
#[test]
fn residency_checks_source_utf8_and_resource_binding_before_publication() {
    use flashtex_rendering_core::residency::*;
    use std::collections::{BTreeMap, BTreeSet};
    let mut source = fixture();
    let c = cff();
    let DrawOperation::Glyph { path, .. } = &mut source.primitives[0].operation else {
        panic!()
    };
    path.sources = vec![SourceRange {
        path: "main.tex".into(),
        start_byte: 0,
        end_byte: 1,
    }];
    let mut cache = MixedResidency::new("test", residency_limits()).unwrap();
    let lease = cache.begin(residency_inputs("é", &"3".repeat(64))).unwrap();
    assert!(lease
        .build(context(), &inputs(&source, &c), MixedLimits::default())
        .is_err());
    let empty = ResidencyInputs::new(
        "test",
        1,
        &"3".repeat(64),
        vec![],
        BTreeMap::new(),
        BTreeSet::new(),
    )
    .unwrap();
    let empty = cache.begin(empty).unwrap();
    assert!(empty
        .build(context(), &inputs(&fixture(), &c), MixedLimits::default())
        .is_err());
    assert!(cache.page(&lease, 1).is_err());
    let wrong = ResidencyInputs::new(
        "test",
        1,
        &"3".repeat(64),
        vec![DocumentResource {
            path: "main.tex".into(),
            revision: 1,
            sha256: "0".repeat(64),
            byte_length: 1,
        }],
        BTreeMap::from([(
            "main.tex".into(),
            SourceSnapshot {
                revision: 1,
                text: "x".into(),
            },
        )]),
        BTreeSet::new(),
    );
    assert!(wrong.is_err());
}
#[test]
fn residency_lru_eviction_keeps_external_immutable_frame_alive() {
    use flashtex_rendering_core::residency::*;
    let source = fixture();
    let c = cff();
    let mut cache = MixedResidency::new(
        "test",
        ResidencyLimits {
            max_pages: 1,
            ..residency_limits()
        },
    )
    .unwrap();
    let lease = cache.begin(residency_inputs("x", &"3".repeat(64))).unwrap();
    let first = cache
        .install(
            lease
                .build(context(), &inputs(&source, &c), MixedLimits::default())
                .unwrap(),
        )
        .unwrap();
    let mut second = fixture();
    second.page = 2;
    for primitive in &mut second.primitives {
        if let DrawOperation::Glyph { path, .. } = &mut primitive.operation {
            path.page = 2;
        }
    }
    let mut ctx = context();
    ctx.page = 2;
    cache
        .install(
            lease
                .build(ctx, &inputs(&second, &c), MixedLimits::default())
                .unwrap(),
        )
        .unwrap();
    assert!(cache.page(&lease, 1).unwrap().is_none());
    assert!(cache.page(&lease, 2).unwrap().is_some());
    assert_eq!(cache.stats().evictions, 1);
    assert_eq!(cache.stats().pages, 1);
    assert_eq!(first.identity(), ("test", 1, 1));
    assert_eq!(first.command_count(), 5);
}
#[test]
fn residency_rejects_conflicting_same_identity_and_oversize_without_replacing_frame() {
    use flashtex_rendering_core::residency::*;
    let source = fixture();
    let c = cff();
    let mut cache = MixedResidency::new("test", residency_limits()).unwrap();
    let lease = cache.begin(residency_inputs("x", &"3".repeat(64))).unwrap();
    let original = cache
        .install(
            lease
                .build(context(), &inputs(&source, &c), MixedLimits::default())
                .unwrap(),
        )
        .unwrap();
    let mut reordered = inputs(&source, &c);
    reordered.swap(0, 1);
    assert!(cache
        .install(
            lease
                .build(context(), &reordered, MixedLimits::default())
                .unwrap()
        )
        .is_err());
    assert!(std::sync::Arc::ptr_eq(
        &original,
        &cache.page(&lease, 1).unwrap().unwrap()
    ));
    let mut tiny = MixedResidency::new(
        "test",
        ResidencyLimits {
            max_encoded_bytes: 1,
            ..residency_limits()
        },
    )
    .unwrap();
    let lease = tiny.begin(residency_inputs("x", &"3".repeat(64))).unwrap();
    assert!(tiny
        .install(
            lease
                .build(context(), &inputs(&source, &c), MixedLimits::default())
                .unwrap()
        )
        .is_err());
    assert_eq!(tiny.stats().pages, 0);
}
#[test]
fn resource_only_generation_change_and_revision_rollback_are_rejected() {
    use flashtex_rendering_core::residency::*;
    use std::collections::BTreeSet;
    let source = fixture();
    let c = cff();
    let mut cache = MixedResidency::new("test", residency_limits()).unwrap();
    let old = cache.begin(residency_inputs("x", &"3".repeat(64))).unwrap();
    let pending = old
        .build(context(), &inputs(&source, &c), MixedLimits::default())
        .unwrap();
    let make = |revision| {
        ResidencyInputs::new(
            "test",
            revision,
            &"3".repeat(64),
            vec![DocumentResource {
                path: "main.tex".into(),
                revision: 1,
                sha256: digest(b"x"),
                byte_length: 1,
            }],
            old.snapshots().clone(),
            BTreeSet::from([
                ResourceIdentity::TrueType {
                    sha256: "9".repeat(64),
                },
                ResourceIdentity::CffTable {
                    sha256: digest(&cff_bytes()),
                },
            ]),
        )
        .unwrap()
    };
    let fresh = cache.begin(make(1)).unwrap();
    assert!(cache.install(pending).is_err());
    assert!(fresh
        .build(context(), &inputs(&source, &c), MixedLimits::default())
        .is_err());
    assert!(cache.begin(make(0)).is_err());
}

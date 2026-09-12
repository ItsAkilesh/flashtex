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
fn cff() -> CffConsumer {
    let program = [139, 139, 21, 239, 139, 139, 239, 39, 139, 8, 14];
    let mut b = vec![
        1, 0, 4, 4, 0, 1, 1, 1, 2, b'F', 0, 1, 1, 1, 3, 160, 17, 0, 0, 0, 0, 0, 2, 1, 1, 2, 13, 14,
    ];
    b.extend(program);
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
fn inputs<'a>(s: &'a TracedBatch, c: &'a CffConsumer) -> Vec<MixedInput<'a>> {
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

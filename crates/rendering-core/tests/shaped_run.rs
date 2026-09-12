use flashtex_font_engine::ShapeOptions;
use flashtex_font_resources::{
    engine_adapter::*, EmbeddingPermission, FontDescriptor, FontResource, LicenseMetadata,
    ManifestEntry,
};
use flashtex_rendering_core::{
    batch::ExactClip, digest, glyph_cache::GlyphPathCache, outlines::*, shaped_run::*,
    SourceSnapshot,
};
#[allow(dead_code)]
#[path = "support/font_fixture.rs"]
mod font_fixture;
fn font() -> FontResource {
    font_bytes(font_fixture::shaping_fixture())
}
fn font_bytes(bytes: Vec<u8>) -> FontResource {
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

fn r(n: i128, d: u128) -> OutlineCoordinate {
    OutlineCoordinate::from_fraction(n, d).unwrap()
}
fn placement() -> Placement {
    Placement {
        item_index: 7,
        size: r(1000, 3),
        origin: OutlinePoint {
            x: r(1, 2),
            y: r(7, 4),
        },
        clip: ExactClip {
            left: r(-1000, 1),
            top: r(-1000, 1),
            right: r(1000, 1),
            bottom: r(1000, 1),
        },
    }
}
fn limits() -> PlacementLimits {
    PlacementLimits {
        max_glyphs: 100,
        max_commands: 1000,
        max_payload_bytes: 100000,
    }
}
fn run(font: &FontResource) -> (BoundShapedRun, SourceSnapshot) {
    let snapshot = SourceSnapshot {
        revision: 9,
        text: "xAé\u{200b}A!".into(),
    };
    let adapter = EngineFontAdapter::from_resource(font).unwrap();
    let run = adapter
        .shape(ShapeRequest {
            source: &snapshot.text,
            source_sha256: &digest(snapshot.text.as_bytes()),
            path: "main.tex",
            revision: 9,
            range: 1..8,
            font_sha256: &font.descriptor().sha256,
            face_index: 0,
            encoding: InputEncoding::Unicode,
            variation_coordinates: &[],
            options: ShapeOptions::PLAIN,
        })
        .unwrap();
    (run, snapshot)
}
#[test]
fn exact_cluster_mapping_fractional_origins_and_cached_geometry() {
    let font = font();
    let (run, snapshot) = run(&font);
    let mut cache = GlyphPathCache::new(10, 100000).unwrap();
    let make = |cache: &mut GlyphPathCache| {
        PlacedShapedRun::prepare(
            &run,
            &snapshot,
            OutlineSource::TrueType {
                resource: &font,
                cache,
            },
            placement(),
            limits(),
        )
        .unwrap()
    };
    let a = make(&mut cache);
    let b = make(&mut cache);
    assert_eq!(a.run(), &run);
    assert_eq!(a.run().shaped().clusters.len(), 4);
    assert_eq!(a.glyphs().len(), 3);
    assert_eq!(a.glyphs()[0].source_range, 1..2);
    assert_eq!(a.glyphs()[1].source_range, 2..4);
    assert_eq!(a.glyphs()[2].source_range, 7..8);
    assert_eq!(a.glyphs()[2].cluster_index, 3);
    assert_eq!(a.glyphs()[2].primitive_id.glyph_index, Some(2));
    assert_eq!(a.glyphs()[0].primitive_id.item_index, 7);
    assert_eq!(a.advance(), r(500, 1));
    assert_eq!(a.glyphs()[1].origin.x, r(1003, 6));
    for (g, h) in a.glyphs().iter().zip(b.glyphs()) {
        assert_eq!(g.original_gid, 1);
        if let (ShapedGeometry::Quadratic(x), ShapedGeometry::Quadratic(y)) =
            (&g.geometry, &h.geometry)
        {
            assert_eq!(x, y);
            assert_eq!(x[0], PlacedPathCommand::MoveTo(g.origin));
        } else {
            panic!()
        }
    }
    assert_eq!(cache.stats().expansions, 1);
    assert_ne!(
        run.identity().font_sha256,
        run.identity().engine_font_id.content_hex()
    );
}
#[test]
fn stale_source_and_atomic_budget_failures() {
    let font = font();
    let (run, mut snapshot) = run(&font);
    let mut cache = GlyphPathCache::new(10, 100000).unwrap();
    snapshot.revision += 1;
    assert!(matches!(
        PlacedShapedRun::prepare(
            &run,
            &snapshot,
            OutlineSource::TrueType {
                resource: &font,
                cache: &mut cache
            },
            placement(),
            limits()
        ),
        Err(ShapePlacementError::StaleSource)
    ));
    snapshot.revision -= 1;
    for budget in [
        PlacementLimits {
            max_glyphs: 2,
            ..limits()
        },
        PlacementLimits {
            max_commands: 1,
            ..limits()
        },
        PlacementLimits {
            max_payload_bytes: 1,
            ..limits()
        },
    ] {
        assert!(matches!(
            PlacedShapedRun::prepare(
                &run,
                &snapshot,
                OutlineSource::TrueType {
                    resource: &font,
                    cache: &mut cache
                },
                placement(),
                budget
            ),
            Err(ShapePlacementError::Budget)
        ));
    }
    snapshot.text.push('x');
    assert!(matches!(
        PlacedShapedRun::prepare(
            &run,
            &snapshot,
            OutlineSource::TrueType {
                resource: &font,
                cache: &mut cache
            },
            placement(),
            limits()
        ),
        Err(ShapePlacementError::StaleSource)
    ));
}

#[test]
fn different_resource_rejects_before_expansion() {
    let font = font();
    let (run, snapshot) = run(&font);
    let wrong = font_bytes(font_fixture::triangle_fixture());
    let mut cache = GlyphPathCache::new(10, 100000).unwrap();
    assert!(matches!(
        PlacedShapedRun::prepare(
            &run,
            &snapshot,
            OutlineSource::TrueType {
                resource: &wrong,
                cache: &mut cache
            },
            placement(),
            limits()
        ),
        Err(ShapePlacementError::Identity)
    ));
    assert_eq!(cache.stats().expansions, 0);
}
fn replay_fixture() -> (Vec<u8>, SourceSnapshot) {
    let font = font();
    let (run, snapshot) = run(&font);
    let mut cache = GlyphPathCache::new(10, 100000).unwrap();
    let placed = PlacedShapedRun::prepare(
        &run,
        &snapshot,
        OutlineSource::TrueType {
            resource: &font,
            cache: &mut cache,
        },
        placement(),
        limits(),
    )
    .unwrap();
    (placed.replay_bytes(100000).unwrap(), snapshot)
}
#[test]
fn replay_roundtrip_preserves_empty_clusters_and_identity() {
    use flashtex_rendering_core::shaped_replay::*;
    let (bytes, snapshot) = replay_fixture();
    let replay = ShapedReplay::parse(&bytes, ReplayLimits::default()).unwrap();
    assert_eq!(replay.cluster_count(), 4);
    assert_eq!(replay.glyph_count(), 3);
    assert_eq!(replay.command_count(), 12);
    assert_eq!(replay.raw_sha256(), digest(&bytes));
    assert_eq!(replay.canonical_bytes(100000).unwrap(), bytes);
    assert_eq!(
        replay.metadata()["clusters"][2]["glyphs"],
        serde_json::json!([])
    );
    replay.verify_source("main.tex", &snapshot).unwrap();
    assert!(replay.verify_source("other.tex", &snapshot).is_err());
    let stale = SourceSnapshot {
        revision: snapshot.revision + 1,
        ..snapshot.clone()
    };
    assert!(replay.verify_source("main.tex", &stale).is_err());
    let stale = SourceSnapshot {
        revision: snapshot.revision,
        text: "same bytes length!".into(),
    };
    assert!(replay.verify_source("main.tex", &stale).is_err());
    assert!(replay.canonical_bytes(16).is_err());
    if std::env::var_os("FLASHTEX_RECORD_SHAPED_FIXTURE").is_some() {
        std::fs::write("tests/fixtures/synthetic-shaped.json", &bytes).unwrap();
    }
}
#[test]
fn malformed_shaped_replays_fail_closed() {
    use flashtex_rendering_core::shaped_replay::*;
    use serde_json::json;
    let (bytes, _) = replay_fixture();
    let valid: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let mutations: Vec<(&str, serde_json::Value)> = vec![
        ("/format", json!("unknown")),
        ("/identity/font_sha256", json!("bad")),
        ("/identity/face_index", json!(1)),
        ("/primitives/0/kind", json!("raster")),
        ("/primitives/0/commands/0", json!({"unsupported":[]})),
        ("/primitives/0/origin/0", json!(["02", "1"])),
        ("/primitives/0/origin/0", json!(["1", "0"])),
        ("/primitives/0/original_gid", json!(2)),
        ("/primitives/0/cluster_index", json!(1)),
        ("/primitives/1/identity/glyph_index", json!(0)),
        ("/clusters/1/source_range", json!([1, 4])),
        ("/clusters/0/text", json!("é")),
        ("/clusters/0/glyphs/0/advance_units", json!(499)),
        ("/command_count", json!(999)),
        ("/advance", json!(["501", "1"])),
        ("/hinting_applied", json!(true)),
    ];
    for (pointer, value) in mutations {
        let mut v = valid.clone();
        *v.pointer_mut(pointer).unwrap() = value;
        assert!(
            ShapedReplay::parse(&serde_json::to_vec(&v).unwrap(), ReplayLimits::default()).is_err(),
            "{pointer}"
        );
    }
    let duplicate = String::from_utf8(bytes.clone()).unwrap().replacen(
        "\"format\":",
        "\"format\":\"duplicate\",\"format\":",
        1,
    );
    assert!(ShapedReplay::parse(duplicate.as_bytes(), ReplayLimits::default()).is_err());
    let mut extra = valid;
    extra["extra"] = json!(true);
    assert!(ShapedReplay::parse(
        &serde_json::to_vec(&extra).unwrap(),
        ReplayLimits::default()
    )
    .is_err());
    for limits in [
        ReplayLimits {
            max_bytes: 10,
            ..ReplayLimits::default()
        },
        ReplayLimits {
            max_clusters: 3,
            ..ReplayLimits::default()
        },
        ReplayLimits {
            max_glyphs: 2,
            ..ReplayLimits::default()
        },
        ReplayLimits {
            max_commands: 11,
            ..ReplayLimits::default()
        },
    ] {
        assert!(ShapedReplay::parse(&bytes, limits).is_err());
    }
}
#[test]
fn shaped_diff_keeps_raw_hashes_and_exact_command_deltas() {
    use flashtex_rendering_core::geometry_diff::*;
    use serde_json::json;
    let (bytes, _) = replay_fixture();
    let left = ValidatedGeometry::shaped(&bytes).unwrap();
    let equal = compare(&left, &left, DiffLimits::default()).unwrap();
    assert_eq!(equal.equal, Some(true));
    assert!(!equal.resources_verified);
    let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["primitives"][0]["commands"][0]["move"][0] = json!(["2", "3"]);
    let right_bytes = serde_json::to_vec(&v).unwrap();
    let right = ValidatedGeometry::shaped(&right_bytes).unwrap();
    let report = compare(&left, &right, DiffLimits::default()).unwrap();
    assert_eq!(report.equal, Some(false));
    assert_eq!(report.left_sha256, digest(&bytes));
    assert_eq!(report.right_sha256, digest(&right_bytes));
    let delta = report
        .differences
        .iter()
        .find_map(|d| d.delta.as_ref())
        .unwrap();
    assert_eq!((&*delta.numerator, &*delta.denominator), ("1", "6"));
    let incomplete = compare(
        &left,
        &right,
        DiffLimits {
            max_visited_nodes: 1,
            ..DiffLimits::default()
        },
    )
    .unwrap();
    assert_eq!(incomplete.equal, None);
    assert!(incomplete.truncated);
}

#[test]
fn frozen_synthetic_replay_is_bounded_and_source_verifiable() {
    use flashtex_rendering_core::shaped_replay::*;
    let bytes = include_bytes!("fixtures/synthetic-shaped.json");
    let replay = ShapedReplay::parse(bytes, ReplayLimits::default()).unwrap();
    assert_eq!(replay.canonical_bytes(100000).unwrap().as_slice(), bytes);
    replay
        .verify_source(
            "main.tex",
            &SourceSnapshot {
                revision: 9,
                text: "xAé\u{200b}A!".into(),
            },
        )
        .unwrap();
}

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

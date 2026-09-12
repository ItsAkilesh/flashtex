//! Pinned STIX original-engine -> exact cubic consumer smoke; no visual oracle.
use flashtex_font_engine::ShapeOptions;
use flashtex_font_resources::{
    cff::{CacheLimits, CffOutlineCache, HintPolicy},
    engine_adapter::*,
};
use flashtex_rendering_core::{
    batch::ExactClip, cubic::CachedCffConsumer, digest, outlines::*, shaped_run::*, SourceSnapshot,
};
use std::{error::Error, fs};
fn r(n: i128, d: u128) -> OutlineCoordinate {
    OutlineCoordinate::from_fraction(n, d).unwrap()
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: shaped_run_probe PINNED_STIX_OTF LICENSE".into());
    }
    let spec: serde_json::Value =
        serde_json::from_str(include_str!("../tests/fixtures/stix-cff-tfm.json"))?;
    let bytes = fs::read(&args[0])?;
    let license = fs::read(&args[1])?;
    if digest(&bytes) != spec["font_sha256"].as_str().unwrap()
        || digest(&license) != spec["license_sha256"].as_str().unwrap()
    {
        return Err("font/license pin mismatch".into());
    }
    let cache = CffOutlineCache::from_font_table(
        &bytes,
        0,
        5644..177326,
        CacheLimits {
            max_entries: 64,
            max_bytes: 1024 * 1024,
        },
    )?;
    let adapter = EngineFontAdapter::from_cff(&bytes, &cache)?;
    let resource = CachedCffConsumer::new(cache);
    let snapshot = SourceSnapshot {
        revision: 4,
        text: "prefix AV office é end".into(),
    };
    let run = adapter.shape(ShapeRequest {
        source: &snapshot.text,
        source_sha256: &digest(snapshot.text.as_bytes()),
        path: "main.tex",
        revision: 4,
        range: 7..19,
        font_sha256: &digest(&bytes),
        face_index: 0,
        encoding: InputEncoding::Unicode,
        variation_coordinates: &[],
        options: ShapeOptions::PLAIN,
    })?;
    let placement = Placement {
        item_index: 9,
        size: r(10485761, 3),
        origin: OutlinePoint {
            x: r(1, 2),
            y: r(7, 4),
        },
        clip: ExactClip {
            left: r(-100000000, 1),
            top: r(-100000000, 1),
            right: r(100000000, 1),
            bottom: r(100000000, 1),
        },
    };
    let prepare = || {
        PlacedShapedRun::prepare(
            &run,
            &snapshot,
            OutlineSource::Cff {
                resource: &resource,
                policy: HintPolicy::Unhinted,
            },
            placement,
            PlacementLimits {
                max_glyphs: 100,
                max_commands: 10000,
                max_payload_bytes: 1024 * 1024,
            },
        )
        .map_err(|e| format!("{e:?}"))
    };
    let started = std::time::Instant::now();
    let a = prepare()?;
    let cold_ns = started.elapsed().as_nanos();
    let started = std::time::Instant::now();
    let b = prepare()?;
    let warm_ns = started.elapsed().as_nanos();
    let a_bytes = a.replay_bytes(1024 * 1024)?;
    let b_bytes = b.replay_bytes(1024 * 1024)?;
    assert_eq!(a_bytes, b_bytes);
    let replay =
        flashtex_rendering_core::shaped_replay::ShapedReplay::parse(&a_bytes, Default::default())?;
    replay.verify_source("main.tex", &snapshot)?;
    assert_eq!(replay.canonical_bytes(1024 * 1024)?, a_bytes);
    let left = flashtex_rendering_core::geometry_diff::ValidatedGeometry::shaped(&a_bytes)?;
    let right = flashtex_rendering_core::geometry_diff::ValidatedGeometry::shaped(&b_bytes)?;
    assert_eq!(
        flashtex_rendering_core::geometry_diff::compare(&left, &right, Default::default())?.equal,
        Some(true)
    );
    println!("replay_sha256={} replay_bytes={} cold_placement_ns={} warm_placement_ns={} measurement=debug_single_run_not_paint_latency",digest(&a_bytes),a_bytes.len(),cold_ns,warm_ns);
    let mut verified_warm_hits = 0;
    for g in a.glyphs() {
        let result = resource
            .place_cached(
                g.original_gid,
                HintPolicy::Unhinted,
                placement.size,
                g.origin,
                10000,
            )
            .map_err(|e| format!("{e:?}"))?;
        assert_eq!(
            result.cache_status,
            flashtex_font_resources::cff::CacheStatus::Hit
        );
        let ShapedGeometry::Cubic(original) = &g.geometry else {
            panic!()
        };
        assert_eq!(result.outline.commands, original.commands);
        verified_warm_hits += 1;
    }
    let started = std::time::Instant::now();
    for _ in 0..100 {
        assert_eq!(prepare()?.replay_bytes(1024 * 1024)?, a_bytes);
    }
    println!("verified_warm_hits={} replay_iterations=100 placement_plus_serialization_ns={} exact_replay_equal=true",verified_warm_hits,started.elapsed().as_nanos());
    let expected = placement.size.checked_multiply(r(
        run.shaped().advance_units() as i128,
        run.shaped().units_per_em as u128,
    ))?;
    assert_eq!(a.advance(), expected);
    for (x, y) in a.glyphs().iter().zip(b.glyphs()) {
        let (ShapedGeometry::Cubic(x), ShapedGeometry::Cubic(y)) = (&x.geometry, &y.geometry)
        else {
            panic!()
        };
        assert_eq!(x.commands, y.commands);
        assert_eq!(x.full_font_identity, y.full_font_identity);
        assert!(!x.hinting_applied);
    }
    println!("glyphs={} clusters={} commands={} advance={}/{} source={} original_engine_id={} font_sha256={} cached_direct_equal=true hinting=false native_painted=false",a.glyphs().len(),run.shaped().clusters.len(),a.command_count(),a.advance().numerator(),a.advance().denominator(),run.source().source_sha256,run.identity().engine_font_id.content_hex(),run.identity().font_sha256);
    Ok(())
}

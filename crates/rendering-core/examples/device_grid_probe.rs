//! Pinned explicit composite-grid geometry replay; not a hinted raster oracle.
use flashtex_font_resources::{
    EmbeddingPermission, Error, FontDescriptor, FontResource, LicenseMetadata, ManifestEntry,
};
use flashtex_rendering_core::{
    device_grid::*,
    digest,
    outlines::{OutlineCoordinate, OutlinePoint},
};
use sha2::{Digest, Sha256};
use std::{error::Error as StdError, fs::File, io::Read};
fn read(path: &str, limit: u64) -> Result<Vec<u8>, Box<dyn StdError>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("input byte limit".into());
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn StdError>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: device_grid_probe FONT.ttf LICENSE [--record]".into());
    }
    let spec: serde_json::Value = serde_json::from_str(include_str!(
        "../tests/fixtures/liberation-device-grid.json"
    ))?;
    let bytes = read(&args[0], 64 * 1024 * 1024)?;
    let license = read(&args[1], 1024 * 1024)?;
    if digest(&bytes) != spec["font_sha256"].as_str().unwrap()
        || digest(&license) != spec["license_sha256"].as_str().unwrap()
    {
        return Err("pinned resource mismatch".into());
    }
    let metadata = flashtex_font_resources::inspect_static_truetype(&bytes)?;
    let entry = ManifestEntry {
        font: FontDescriptor {
            font_id: "pinned-liberation".into(),
            sha256: digest(&bytes),
            byte_length: bytes.len() as u64,
            format: "static-truetype".into(),
            face_index: 0,
            units_per_em: metadata.units_per_em,
            glyph_count: metadata.glyph_count,
            postscript_name: metadata.postscript_names[0].clone(),
        },
        path: "font.ttf".into(),
        license: LicenseMetadata {
            identifier: "LicenseRef-Pinned-System-Resource".into(),
            copyright: "See pinned license".into(),
            source: "Explicit pinned probe".into(),
            text_path: "LICENSE".into(),
            text_sha256: digest(&license),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    };
    let font = FontResource::from_bytes(&entry, &bytes, &license)?;
    let context = DeviceContext::new(
        16,
        16,
        TieRule::AwayFromZero,
        spec["policy_sha256"].as_str().unwrap(),
    )?;
    let mut cache = DevicePathCache::new(8, 2 * 1024 * 1024)?;
    let mut hash = Sha256::new();
    hash.update(b"flashtex-device-grid-placement-v1\0");
    let mut supported = 0;
    let mut unsupported = 0;
    let mut commands = 0usize;
    let scalar = |n, d| OutlineCoordinate::from_fraction(n, d).unwrap();
    for gid in 1..metadata.glyph_count as u16 {
        match font.expanded_outline(gid) {
            Ok(_) => supported += 1,
            Err(Error::UnsupportedFont(_)) => unsupported += 1,
            Err(e) => return Err(e.into()),
        }
        let direct = expand_device(&font, gid, &context)?;
        let DeviceOutcome::Ready(cached) = cache.lookup(&font, gid, Some(&context))?.outcome else {
            return Err("device cache unexpectedly unavailable".into());
        };
        if direct.commands != cached.commands || direct.instances != cached.instances {
            return Err("direct/cache grid mismatch".into());
        }
        if !cache.lookup(&font, gid, Some(&context))?.cache_hit {
            return Err("warm grid miss".into());
        }
        let placed = cached.place(
            scalar(10485761, 3),
            OutlinePoint {
                x: scalar(1, 2),
                y: scalar(7, 4),
            },
        )?;
        commands += placed.commands.len();
        let encoded = placed.comparison_fixture(2 * 1024 * 1024)?;
        hash.update((encoded.len() as u64).to_be_bytes());
        hash.update(encoded);
    }
    let sha = format!("{:x}", hash.finalize());
    if supported != spec["expected_default_supported"].as_u64().unwrap()
        || unsupported != spec["expected_default_unsupported"].as_u64().unwrap()
    {
        return Err("default expansion support changed".into());
    }
    if args.get(2).map(String::as_str) != Some("--record")
        && sha != spec["expected_geometry_sha256"].as_str().unwrap()
    {
        return Err("device geometry regression mismatch".into());
    }
    println!("geometry_sha256={sha} device_non_notdef={} default_supported={supported} default_unsupported={unsupported} commands={commands} expansions={} hits={} hinting_applied=false native_painted=false",metadata.glyph_count-1,cache.stats().expansions,cache.stats().hits);
    Ok(())
}

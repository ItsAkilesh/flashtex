//! Developer-only original glyph path probe. Does not run TeX or paint pixels.
use flashtex_font_resources::{
    EmbeddingPermission, FontDescriptor, FontResource, LicenseMetadata, ManifestEntry,
};
use flashtex_rendering_core::{
    digest, hit_test::Point, outlines::place_outline, Tick, TICKS_PER_BP,
};
use std::io::Read;
fn read(path: &str, limit: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("input size limit".into());
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: outline_probe FONT.ttf LICENSE.txt CHARACTER".into());
    }
    let mut chars = args[3].chars();
    let character = chars.next().ok_or("empty character")?;
    if chars.next().is_some() {
        return Err("one Unicode scalar required".into());
    }
    let bytes = read(&args[1], 64 * 1024 * 1024)?;
    let license = read(&args[2], 1024 * 1024)?;
    let metadata = flashtex_font_resources::inspect_static_truetype(&bytes)?;
    let entry = ManifestEntry {
        font: FontDescriptor {
            font_id: "probe-font".into(),
            sha256: digest(&bytes),
            byte_length: bytes.len() as u64,
            format: "static-truetype".into(),
            face_index: 0,
            units_per_em: metadata.units_per_em,
            glyph_count: metadata.glyph_count,
            postscript_name: metadata
                .postscript_names
                .first()
                .cloned()
                .ok_or("no PostScript name")?,
        },
        path: "font.ttf".into(),
        license: LicenseMetadata {
            identifier: "LicenseRef-Explicit-CLI-Input".into(),
            copyright: "See supplied license bytes".into(),
            source: "Explicit developer probe input".into(),
            text_path: "LICENSE.txt".into(),
            text_sha256: digest(&license),
            embedding_permission: EmbeddingPermission::Unknown,
        },
    };
    let font = FontResource::from_bytes(&entry, &bytes, &license)?;
    let gid = font.glyph_id(character)?.ok_or("missing glyph")?;
    let outline = font.expanded_outline(gid)?;
    let path = place_outline(
        &outline,
        Tick(12 * TICKS_PER_BP),
        metadata.units_per_em,
        Point {
            x: Tick(72 * TICKS_PER_BP),
            y: Tick(84 * TICKS_PER_BP),
        },
    )?;
    let mut cache = flashtex_rendering_core::glyph_cache::GlyphPathCache::new(128, 1024 * 1024)?;
    let start = std::time::Instant::now();
    cache.lookup(&font, gid)?;
    let cold_ns = start.elapsed().as_nanos();
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        cache.lookup(&font, gid)?;
    }
    let warm_total_ns = start.elapsed().as_nanos();
    eprintln!(
        "{}",
        serde_json::json!({"scope":"font expansion cache only, not native paint latency","cold_expansion_ns":cold_ns,"warm_lookup_samples":1000,"warm_lookup_total_ns":warm_total_ns,"expansions":cache.stats().expansions,"hits":cache.stats().hits})
    );
    println!(
        "{}",
        serde_json::json!({"font_sha256":entry.font.sha256,"license_sha256":entry.license.text_sha256,"character":character.to_string(),"original_gid":gid,"contours":outline.contour_ends.len(),"points":outline.points.len(),"path_commands":path.len(),"hinting_applied":false,"painted":false})
    );
    Ok(())
}

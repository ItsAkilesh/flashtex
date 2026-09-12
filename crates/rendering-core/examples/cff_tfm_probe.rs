//! Pinned installed STIX resource + original synthetic TFM, never a TeX oracle.
use flashtex_font_resources::{
    cff::{BoundCffTfmFont, CacheLimits, CffEncodingManifest, CffOutlineCache, HintPolicy},
    encoding::EncodingEntry,
    tfm::Tfm,
};
use flashtex_rendering_core::{
    cff_run::{CffRun, RunLimits, RunPlacement},
    cubic::CachedCffConsumer,
    digest,
    outlines::{OutlineCoordinate, OutlinePoint},
    tex_adapter::{MetricPolicy, RunScale},
    MAX_MESSAGE_BYTES,
};
use std::{error::Error, fs::File, io::Read};
fn bytes(path: &str, cap: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut b = Vec::new();
    File::open(path)?.take(cap + 1).read_to_end(&mut b)?;
    if b.len() as u64 > cap {
        return Err("resource byte cap".into());
    }
    Ok(b)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: cff_tfm_probe FONT_OTF LICENSE [--record]".into());
    }
    let spec: serde_json::Value =
        serde_json::from_str(include_str!("../tests/fixtures/stix-cff-tfm.json"))?;
    let font = bytes(&args[0], 64 * 1024 * 1024)?;
    let license = bytes(&args[1], 1024 * 1024)?;
    if digest(&font) != spec["font_sha256"].as_str().unwrap()
        || font.len() as u64 != spec["font_byte_length"].as_u64().unwrap()
        || digest(&license) != spec["license_sha256"].as_str().unwrap()
    {
        return Err("pinned font/license mismatch".into());
    }
    let hex = spec["tfm_hex"].as_str().unwrap();
    let tfm_bytes = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()?;
    if digest(&tfm_bytes) != spec["tfm_sha256"].as_str().unwrap() {
        return Err("synthetic TFM pin mismatch".into());
    }
    let tfm = Tfm::parse(&tfm_bytes)?;
    // Range is pinned to the cited existing OpenType reader and exact font hash;
    // this harness does not implement another sfnt selector.
    let range = spec["cff_range"][0].as_u64().unwrap() as usize
        ..spec["cff_range"][1].as_u64().unwrap() as usize;
    let cache = CffOutlineCache::from_font_table(
        &font,
        0,
        range,
        CacheLimits {
            max_entries: 8,
            max_bytes: 1024 * 1024,
        },
    )?;
    if cache.identity().cff_sha256 != spec["cff_sha256"].as_str().unwrap() {
        return Err("CFF table pin mismatch".into());
    }
    let manifest = CffEncodingManifest {
        font_sha256: cache.identity().font_sha256.clone(),
        cff_sha256: cache.identity().cff_sha256.clone(),
        tfm_sha256: tfm.source_sha256.clone(),
        face_index: 0,
        encoding: vec![EncodingEntry {
            code: 65,
            glyph_name: spec["declared_name"].as_str().unwrap().into(),
        }],
    };
    let binding = BoundCffTfmFont::new(&tfm, &cache, &manifest)?;
    if binding.map_code(65)?.0
        != flashtex_font_resources::encoding::GlyphIdentity::Original(
            spec["expected_original_gid"].as_u64().unwrap() as u16,
        )
    {
        return Err("named glyph identity mismatch".into());
    }
    let consumer = CachedCffConsumer::new(cache);
    let placement = RunPlacement {
        scale: RunScale::design_size(tfm.design_size, MetricPolicy::ExactRationalNoTexRounding)
            .map_err(|e| format!("{e:?}"))?,
        origin: OutlinePoint {
            x: OutlineCoordinate::from_fraction(1, 2)?,
            y: OutlineCoordinate::from_fraction(7, 4)?,
        },
        hints: HintPolicy::Unhinted,
    };
    let run = CffRun::prepare(&binding, &consumer, b"AA", placement, RunLimits::default())
        .map_err(|e| format!("{e:?}"))?;
    let encoded = run
        .fixture_bytes(MAX_MESSAGE_BYTES)
        .map_err(|e| format!("{e:?}"))?;
    let warm = CffRun::prepare(&binding, &consumer, b"AA", placement, RunLimits::default())
        .map_err(|e| format!("{e:?}"))?;
    if encoded
        != warm
            .fixture_bytes(MAX_MESSAGE_BYTES)
            .map_err(|e| format!("{e:?}"))?
    {
        return Err("warm exact run mismatch".into());
    }
    let sha = digest(&encoded);
    if args.get(2).map(String::as_str) != Some("--record")
        && sha != spec["expected_run_sha256"].as_str().unwrap()
    {
        return Err("run regression digest mismatch".into());
    }
    println!("run_sha256={sha} bytes={} commands={} original_gid=3 input_slots=65,65 synthetic_tfm=true exact_warm_replay=true native_painted=false tex_oracle=false",encoded.len(),run.command_count());
    Ok(())
}

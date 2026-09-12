//! Offline exact internal-geometry replay; never activates a native renderer.
use flashtex_rendering_core::{
    digest, mixed::MixedLimits, mixed_replay::ReplayBatch, MAX_MESSAGE_BYTES,
};
use std::{error::Error, fs::File, io::Read};
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: replay_mixed INTERNAL_MIXED_FIXTURE.json")?;
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_MESSAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    let replay =
        ReplayBatch::parse(&bytes, MixedLimits::default()).map_err(|e| format!("{e:?}"))?;
    let canonical = replay.canonical_bytes().map_err(|e| format!("{e:?}"))?;
    let again =
        ReplayBatch::parse(&canonical, MixedLimits::default()).map_err(|e| format!("{e:?}"))?;
    if replay.primitives() != again.primitives() || replay.metadata() != again.metadata() {
        return Err("exact replay mismatch".into());
    }
    println!("primitives={} commands={} canonical_bytes={} canonical_sha256={} exact_replay=true resources_verified=false native_painted=false",replay.primitives().len(),replay.command_count(),canonical.len(),digest(&canonical));
    Ok(())
}

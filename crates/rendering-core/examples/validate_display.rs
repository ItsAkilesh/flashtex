//! Offline experimental display-list roundtrip gate; no font or renderer claims.
use flashtex_rendering_core::{wire::*, *};
use std::io::Read;
fn read(path: &str) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = vec![];
    std::fs::File::open(path)?
        .take(MAX_MESSAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(WireError::TooLarge {
            limit: MAX_MESSAGE_BYTES,
        }
        .into());
    }
    Ok(bytes)
}
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        return Err("usage: validate_display MESSAGE.json [OFFER.json]".into());
    }
    let offer = if args.len() == 3 {
        Some(parse_validated(&read(&args[2])?, None)?)
    } else {
        None
    };
    let envelope = parse_validated(&read(&args[1])?, offer.as_ref())?;
    let encoded = serialize_validated(&envelope, offer.as_ref(), MAX_MESSAGE_BYTES)?;
    let decoded = parse_validated(&encoded, offer.as_ref())?;
    if encoded != serialize_validated(&decoded, offer.as_ref(), MAX_MESSAGE_BYTES)? {
        return Err("noncanonical roundtrip".into());
    }
    println!(
        "{}",
        serde_json::json!({"status":"valid_experimental_roundtrip","canonical_bytes":encoded.len(),"sha256":digest(&encoded),"paintable":false})
    );
    Ok(())
}

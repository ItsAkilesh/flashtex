//! Compare exact validated mixed fixtures or display-list-v2 envelopes offline.
use flashtex_rendering_core::{
    geometry_diff::{compare, DiffLimits, ValidatedGeometry},
    MAX_MESSAGE_BYTES,
};
use std::{error::Error, fs::File, io::Read};
fn read(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut b = Vec::new();
    File::open(path)?
        .take(MAX_MESSAGE_BYTES as u64 + 1)
        .read_to_end(&mut b)?;
    if b.len() > MAX_MESSAGE_BYTES {
        return Err("comparison input byte cap".into());
    }
    Ok(b)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 && (args.len() != 4 || args[2] != "--offer") {
        return Err("usage: geometry_diff LEFT.json RIGHT.json [--offer CAPABILITIES.json]".into());
    }
    let left = read(&args[0])?;
    let right = read(&args[1])?;
    let offer = if args.len() == 4 {
        Some(read(&args[3])?)
    } else {
        None
    };
    let parse = |bytes: &[u8]| -> Result<ValidatedGeometry, Box<dyn Error>> {
        let value: serde_json::Value = serde_json::from_slice(bytes)?;
        if value.get("format").and_then(|v| v.as_str()) == Some("flashtex-internal-device-v1") {
            Ok(ValidatedGeometry::device(bytes)?)
        } else if value.get("format").is_some() {
            Ok(ValidatedGeometry::mixed(bytes)?)
        } else {
            Ok(ValidatedGeometry::display(
                bytes,
                offer
                    .as_deref()
                    .ok_or("display comparison requires explicit --offer")?,
            )?)
        }
    };
    let report = compare(&parse(&left)?, &parse(&right)?, DiffLimits::default())?;
    println!("{}", String::from_utf8(report.json_bytes()?)?);
    match report.equal {
        Some(true) => Ok(()),
        Some(false) => std::process::exit(1),
        None => std::process::exit(2),
    }
}

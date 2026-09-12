//! Compare actual PDFs; never re-emit a reference as a compilation result.
use flashtex_rendering_core::pdf_compare::*;
use std::{error::Error, fs::File, io::Read};
fn read(path: &str, limit: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("input byte cap".into());
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 && args.len() != 4 {
        return Err(
            "usage: pdf_compare CANDIDATE.pdf REFERENCE.pdf REPORT.json [CANDIDATE_EVIDENCE.json]"
                .into(),
        );
    }
    if args[2] == args[0] || args[2] == args[1] || args.get(3) == Some(&args[2]) {
        return Err("report path must differ from inputs".into());
    }
    let limits = CompareLimits::default();
    let a = read(&args[0], limits.max_pdf_bytes)?;
    let b = read(&args[1], limits.max_pdf_bytes)?;
    let evidence = args
        .get(3)
        .map(|p| read(p, flashtex_rendering_core::MAX_MESSAGE_BYTES))
        .transpose()?;
    let report = compare(&a, &b, evidence.as_deref(), limits)
        .map_err(|e| format!("PDF comparison refused: {e:?}"))?;
    std::fs::write(&args[2], report.bytes())?;
    println!(
        "raw_bytes_equal={} parsed_operators_equal={} truncated={} visual_equal=unknown",
        report.value()["raw_bytes_equal"],
        report.value()["parsed_operators_equal"],
        report.value()["truncated"]
    );
    if report.value()["parsed_operators_equal"].is_null() || report.value()["truncated"] == true {
        std::process::exit(4);
    }
    if report.value()["raw_bytes_equal"] != true {
        std::process::exit(3);
    }
    Ok(())
}

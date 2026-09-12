//! Export validated exact operators and evidence; this does not emit a PDF file.
use flashtex_rendering_core::{pdf_stream::*, Paint, Tick, MAX_MESSAGE_BYTES};
use std::{
    error::Error,
    fs::{self, File},
    io::Read,
};
fn main() -> Result<(), Box<dyn Error>> {
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 3 && a.len() != 5 {
        return Err("usage: pdf_stream INPUT.json OUTPUT.content OUTPUT.evidence.json [WIDTH_TICKS HEIGHT_TICKS]".into());
    }
    if a[0] == a[1] || a[0] == a[2] || a[1] == a[2] {
        return Err("input and output paths must differ".into());
    }
    let mut bytes = Vec::new();
    File::open(&a[0])?
        .take(MAX_MESSAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err("input byte cap".into());
    }
    let stream = if a.len() == 5 {
        PdfCommandStream::from_shaped_replay(
            &bytes,
            PdfPage {
                width: Tick(a[3].parse()?),
                height: Tick(a[4].parse()?),
            },
            Paint {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            StreamLimits::default(),
        )
    } else {
        PdfCommandStream::from_mixed_replay(&bytes, StreamLimits::default())
    }
    .map_err(|e| format!("{e:?}"))?;
    let content = stream
        .content_bytes()
        .map_err(|e| format!("exact PDF operand conversion refused: {e:?}"))?;
    let evidence = stream.evidence_bytes().map_err(|e| format!("{e:?}"))?;
    fs::write(&a[1], &content)?;
    fs::write(&a[2], evidence)?;
    println!(
        "operators={} content_bytes={} source_sha256={} standalone_pdf=false",
        stream.operators().len(),
        content.len(),
        stream.source_sha256()
    );
    Ok(())
}

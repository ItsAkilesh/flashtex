//! Offline diagnostic for an explicitly supplied raw CFF table; not a paint test.
use flashtex_font_resources::cff::HintPolicy;
use flashtex_rendering_core::{
    cubic::CffConsumer,
    digest,
    outlines::{OutlineCoordinate, OutlinePoint},
};
use std::{error::Error, fs::File, io::Read};
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: cff_probe RAW_CFF_TABLE")?;
    let mut bytes = Vec::new();
    File::open(path)?
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    let sha = digest(&bytes);
    let font = CffConsumer::from_table(&bytes, &sha).map_err(|e| format!("{e:?}"))?;
    let scalar = |n, d| OutlineCoordinate::from_fraction(n, d).unwrap();
    let mut accepted = 0;
    let mut rejected = 0;
    let mut commands = 0;
    for gid in 1..font.glyph_count() {
        match font.place_glyph(
            gid as u16,
            HintPolicy::Unhinted,
            scalar(10485761, 3),
            OutlinePoint {
                x: scalar(1, 2),
                y: scalar(7, 4),
            },
            2_000_000,
        ) {
            Ok(out) => {
                accepted += 1;
                commands += out.commands.len();
            }
            Err(_) => rejected += 1,
        }
    }
    println!("cff_table_sha256={sha} glyphs={} accepted={accepted} rejected={rejected} skipped_notdef=1 commands={commands} hinting_applied=false native_painted=false",font.glyph_count());
    Ok(())
}

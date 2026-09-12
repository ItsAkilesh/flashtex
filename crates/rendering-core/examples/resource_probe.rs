//! Developer probe: validate supplied real font bytes against a synthetic display
//! fixture. It proves resource integration, not glyph shaping/visual correctness.
use flashtex_rendering_core::{font_adapter::StaticTrueTypeLoader, *};
use std::collections::BTreeMap;
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: resource_probe FONT.ttf")?;
    if std::fs::metadata(&path)?.len() > 64 * 1024 * 1024 {
        return Err("font exceeds 64 MiB".into());
    }
    let bytes = std::fs::read(path)?;
    let metadata = flashtex_font_resources::inspect_static_truetype(&bytes)?;
    let mut list = match parse(include_bytes!(
        "../tests/fixtures/synthetic-display-list.json"
    ))?
    .message
    {
        Message::DisplayList(list) => list,
        _ => unreachable!(),
    };
    let capabilities = match parse(include_bytes!("../tests/fixtures/capabilities.json"))?.message {
        Message::Offer(caps) => caps,
        _ => unreachable!(),
    };
    list.fonts[0].sha256 = digest(&bytes);
    list.fonts[0].byte_length = bytes.len() as u64;
    list.fonts[0].units_per_em = metadata.units_per_em;
    list.fonts[0].glyph_count = metadata.glyph_count;
    list.fonts[0].postscript_name = metadata
        .postscript_names
        .first()
        .cloned()
        .unwrap_or_else(|| "unnamed".into());
    let fonts = BTreeMap::from([("synthetic".into(), bytes)]);
    let documents = BTreeMap::from([(
        "main.tex".into(),
        SourceSnapshot {
            revision: 1,
            text: "office e\u{301}".into(),
        },
    )]);
    let result =
        list.validate_resources(&capabilities, &documents, &fonts, &StaticTrueTypeLoader)?;
    println!(
        "{}",
        serde_json::json!({"font_sha256":list.fonts[0].sha256,"units_per_em":metadata.units_per_em,"glyph_count":metadata.glyph_count,"font_resources_verified":result.font_resources_verified,"paintable":result.paintable,"fixture":"synthetic glyph IDs, no shaping or visual claim"})
    );
    Ok(())
}

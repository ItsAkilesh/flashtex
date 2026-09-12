//! Bounded fixed-fixture subset comparison using the PDF owner's parser/replay.
//! This is an offline fixture probe, not an untrusted PDF validator or rasterizer.
use flashtex_pdf::{cff::CffFont, compare::font_from_dict, exact::*, reader::PdfFile};
use flashtex_rendering_core::digest;
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    io::Read,
};
fn read(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("fixture byte cap".into());
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("OLD.pdf NEW.pdf REPORT.json".into());
    }
    let old = read(&args[0])?;
    let new = read(&args[1])?;
    let report = compare(&old, &new)?;
    std::fs::write(&args[2], serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}
pub fn compare(old: &[u8], new: &[u8]) -> Result<serde_json::Value, Box<dyn Error>> {
    if old.len() > 4 * 1024 * 1024 || new.len() > 4 * 1024 * 1024 {
        return Err("fixture byte cap".into());
    }
    let a = PdfFile::parse(old)?;
    let b = PdfFile::parse(new)?;
    let ap = a.pages()?;
    let bp = b.pages()?;
    if ap.len() != 1 || bp.len() != 1 {
        return Err("one-page fixture required".into());
    }
    let mut frames = Vec::new();
    let mut resources = Vec::new();
    for (pdf, page) in [(&a, ap[0]), (&b, bp[0])] {
        let fonts: BTreeMap<_, _> = pdf
            .page_fonts(page)
            .into_iter()
            .map(|(name, dict)| Ok((name, font_from_dict(pdf, dict)?)))
            .collect::<Result<_, String>>()?;
        let ops = flashtex_pdf::exact::parse(&pdf.page_content(page)?)?;
        if ops.len() > 10000 {
            return Err("fixture operator cap".into());
        }
        let positions = glyph_positions(&ops, &|_| true, &|name, gid| match fonts.get(name) {
            Some(ExactFont::CidCff(font)) => font.widths.get(&gid).map(Ratio::from_decimal),
            _ => None,
        })?;
        frames.push(positions);
        resources.push(fonts);
    }
    if frames[0] != frames[1] {
        return Err("original GID/font-resource/position changed".into());
    }
    let mut used: BTreeMap<String, BTreeSet<u16>> = BTreeMap::new();
    for g in &frames[0] {
        used.entry(g.font.clone()).or_default().insert(g.code);
    }
    let mut fonts_report = Vec::new();
    for (name, gids) in used {
        let (Some(ExactFont::CidCff(oldfont)), Some(ExactFont::CidCff(newfont))) =
            (resources[0].get(&name), resources[1].get(&name))
        else {
            return Err("CID CFF fixture required".into());
        };
        let cmap = |f: &CidFont| -> Result<_, String> {
            match &f.to_unicode_verbatim {
                Some(bytes) => parse_to_unicode(bytes),
                None => Ok(f.to_unicode.clone()),
            }
        };
        if cmap(oldfont)? != cmap(newfont)? || oldfont.widths != newfont.widths {
            return Err("Unicode/width map changed".into());
        }
        let oldcff = CffFont::parse(oldfont.program.bytes()).map_err(|e| format!("{e:?}"))?;
        let newcff = CffFont::parse(newfont.program.bytes()).map_err(|e| format!("{e:?}"))?;
        for gid in &gids {
            if oldcff
                .expanded_charstring(
                    (0..oldcff.glyph_count())
                        .find(|i| oldcff.charset_entry(*i) == Some(*gid))
                        .ok_or("missing original CID in old subset charset")?,
                )
                .map_err(|e| format!("{e:?}"))?
                != newcff
                    .expanded_charstring(
                        (0..newcff.glyph_count())
                            .find(|i| newcff.charset_entry(*i) == Some(*gid))
                            .ok_or("missing original CID in new subset charset")?,
                    )
                    .map_err(|e| format!("{e:?}"))?
            {
                return Err("expanded original glyph charstring changed".into());
            }
        }
        fonts_report.push(json!({"resource":name,"original_gids":gids,"old_program_bytes":oldfont.program.bytes().len(),"new_program_bytes":newfont.program.bytes().len(),"old_program_sha256":digest(oldfont.program.bytes()),"new_program_sha256":digest(newfont.program.bytes()),"expanded_charstrings_equal":true,"unicode_and_widths_equal":true}));
    }
    let report = json!({"format":"flashtex-subset-fixture-comparison-v1","old_pdf_sha256":digest(old),"new_pdf_sha256":digest(new),"old_pdf_bytes":old.len(),"new_pdf_bytes":new.len(),"glyphs":frames[0].len(),"exact_original_gid_and_positions_equal":true,"fonts":fonts_report,"raster_equal":null,"scope":"fixed one-page CID-CFF fixtures; no native/general font safety or oracle parity claim"});
    Ok(report)
}

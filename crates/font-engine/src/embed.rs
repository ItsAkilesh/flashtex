//! Data a PDF writer needs to embed a shaped TrueType face as a
//! `Type0` / `CIDFontType2` font with `Identity-H` encoding: the subset
//! program, `/W` widths, `/CIDToGIDMap`, descriptor metrics and a
//! `ToUnicode` CMap. No PDF objects are written here.
//!
//! Glyph ids in the output are SUBSET ids; the [`PdfFontProgram::subset`]
//! carries the explicit old-to-new map so a writer can translate shaping
//! output (original ids) to the two-byte CIDs it emits.

use std::collections::BTreeMap;

use crate::shape::Shaped;
use crate::subset::{Subset, subset};
use crate::truetype::TrueTypeFace;
use crate::{Error, Face, GlyphId};

/// Accumulates the glyphs and Unicode text used by one document font.
#[derive(Debug, Clone, Default)]
pub struct EmbedPlan {
    /// Original glyph id -> the Unicode strings it has rendered (a ligature
    /// glyph maps to "fi"; a mark-composed glyph maps to "e\u{301}" and, if
    /// also reached through the precomposed character, to "é").
    used: BTreeMap<u16, Vec<String>>,
}

impl EmbedPlan {
    pub fn new() -> EmbedPlan {
        EmbedPlan::default()
    }

    /// Records every glyph of `shaped`. Missing glyphs (`.notdef`) are
    /// recorded too so the subset contains what the page will draw.
    pub fn add_shaped(&mut self, shaped: &Shaped) {
        for cluster in &shaped.clusters {
            match cluster.glyphs.len() {
                0 => {}
                1 => self.add(cluster.glyphs[0].gid, &cluster.text),
                _ => {
                    // Base + marks or a mark-only cluster: attribute the whole
                    // text to the first glyph and nothing to the marks (their
                    // text is already covered), mirroring what ActualText on
                    // the cluster will say.
                    self.add(cluster.glyphs[0].gid, &cluster.text);
                    for g in &cluster.glyphs[1..] {
                        self.used.entry(g.gid.0).or_default();
                    }
                }
            }
        }
    }

    pub fn add(&mut self, gid: GlyphId, text: &str) {
        let entry = self.used.entry(gid.0).or_default();
        if !entry.iter().any(|t| t == text) {
            entry.push(text.to_string());
        }
    }

    pub fn glyph_ids(&self) -> Vec<GlyphId> {
        self.used.keys().map(|g| GlyphId(*g)).collect()
    }

    /// Subsets `face` to the recorded glyphs and assembles the PDF data.
    pub fn finish(&self, face: &TrueTypeFace) -> Result<PdfFontProgram, Error> {
        let gids = self.glyph_ids();
        let sub = subset(face, &gids)?;
        let scale = |v: i16| -> i32 { face.to_pdf_units(i64::from(v)) };
        let vm = face.vertical_metrics();
        let bbox = face.bbox();
        let mut flags = 0u32;
        if face.is_fixed_pitch() {
            flags |= 1 << 0;
        }
        flags |= 1 << 5; // Nonsymbolic
        if face.italic_angle() != 0.0 {
            flags |= 1 << 6;
        }
        let widths: Vec<i32> = sub
            .advances
            .iter()
            .map(|a| face.to_pdf_units(i64::from(*a)))
            .collect();
        let mut to_unicode: BTreeMap<u16, String> = BTreeMap::new();
        for (old, texts) in &self.used {
            let new = sub.old_to_new[old];
            // First recorded text wins; ToUnicode is one-to-one per CID.
            // Ambiguities are still recoverable through per-cluster text.
            if let Some(t) = texts.first() {
                to_unicode.insert(new, t.clone());
            }
        }
        let base_font = format!("{}+{}", sub.tag, face.postscript_name());
        let cmap = to_unicode_cmap(&to_unicode);
        Ok(PdfFontProgram {
            base_font,
            descriptor: Descriptor {
                flags,
                bbox: [scale(bbox[0]), scale(bbox[1]), scale(bbox[2]), scale(bbox[3])],
                italic_angle: face.italic_angle(),
                ascent: scale(vm.ascender),
                descent: scale(vm.descender),
                cap_height: scale(vm.cap_height),
                x_height: scale(vm.x_height),
                stem_v: 80,
            },
            widths,
            cid_to_gid: CidToGid::Identity,
            to_unicode: to_unicode.clone(),
            to_unicode_cmap: cmap,
            fs_type: face.fs_type,
            subset: sub,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CidToGid {
    /// CIDs are subset glyph ids.
    Identity,
}

/// `/FontDescriptor` values in PDF glyph space (1/1000 em).
#[derive(Debug, Clone, PartialEq)]
pub struct Descriptor {
    pub flags: u32,
    pub bbox: [i32; 4],
    pub italic_angle: f64,
    pub ascent: i32,
    pub descent: i32,
    pub cap_height: i32,
    pub x_height: i32,
    /// Not derivable from TrueType data; a conventional placeholder.
    pub stem_v: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PdfFontProgram {
    /// `/BaseFont` with the six-letter subset tag.
    pub base_font: String,
    pub descriptor: Descriptor,
    /// Width of every subset glyph in 1/1000 em, indexed by CID.
    pub widths: Vec<i32>,
    pub cid_to_gid: CidToGid,
    /// CID -> Unicode text.
    pub to_unicode: BTreeMap<u16, String>,
    /// The `ToUnicode` CMap stream body.
    pub to_unicode_cmap: Vec<u8>,
    /// OS/2 `fsType`; 0x0002 means restricted licence embedding.
    pub fs_type: u16,
    /// The subset program (`/FontFile2` bytes are `subset.program`) and maps.
    pub subset: Subset,
}

impl PdfFontProgram {
    /// `/W [ 0 [ w0 w1 ... ] ]`.
    pub fn w_array(&self) -> String {
        let w: Vec<String> = self.widths.iter().map(ToString::to_string).collect();
        format!("[ 0 [ {} ] ]", w.join(" "))
    }

    /// Subset CID for an original glyph id from shaping output.
    pub fn cid(&self, original: GlyphId) -> Option<u16> {
        self.subset.old_to_new.get(&original.0).copied()
    }

    /// Whether OS/2 `fsType` forbids embedding outright (bit 1 set and no
    /// other permission bits). Consumers must still honour the font licence.
    pub fn embedding_restricted(&self) -> bool {
        self.fs_type & 0x000F == 0x0002
    }
}

/// Writes a `ToUnicode` CMap with `bfchar` entries (multi-character
/// destinations allowed, as the PDF specification permits).
pub fn to_unicode_cmap(map: &BTreeMap<u16, String>) -> Vec<u8> {
    let mut s = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(&u16, &String)> = map.iter().collect();
    for chunk in entries.chunks(100) {
        s.push_str(&format!("{} beginbfchar\n", chunk.len()));
        for (cid, text) in chunk {
            let hex: String = text.encode_utf16().map(|u| format!("{u:04X}")).collect();
            s.push_str(&format!("<{cid:04X}> <{hex}>\n"));
        }
        s.push_str("endbfchar\n");
    }
    s.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    s.into_bytes()
}

/// Parses a CMap written by [`to_unicode_cmap`] back to CID -> text.
pub fn parse_to_unicode(cmap: &[u8]) -> Result<BTreeMap<u16, String>, Error> {
    let text = std::str::from_utf8(cmap).map_err(|_| Error::Malformed("CMap not UTF-8".into()))?;
    let mut map = BTreeMap::new();
    let mut in_bfchar = false;
    for line in text.lines() {
        if line.ends_with("beginbfchar") {
            in_bfchar = true;
            continue;
        }
        if line == "endbfchar" {
            in_bfchar = false;
            continue;
        }
        if !in_bfchar {
            continue;
        }
        let (src, dst) = line
            .split_once("> <")
            .ok_or_else(|| Error::Malformed(format!("bfchar line {line:?}")))?;
        let cid = u16::from_str_radix(src.trim_start_matches('<'), 16)
            .map_err(|_| Error::Malformed("bad CID".into()))?;
        let dst = dst.trim_end_matches('>');
        let units: Vec<u16> = dst
            .as_bytes()
            .chunks(4)
            .map(|c| u16::from_str_radix(std::str::from_utf8(c).unwrap_or("zz"), 16))
            .collect::<Result<_, _>>()
            .map_err(|_| Error::Malformed("bad UTF-16".into()))?;
        let s = String::from_utf16(&units).map_err(|_| Error::Malformed("bad UTF-16".into()))?;
        map.insert(cid, s);
    }
    Ok(map)
}

/// For the standard 14 fonts (not embedded): the widths a `/Widths` array
/// needs, in 1/1000 em, for each requested character that the face has.
pub fn standard_font_widths(face: &dyn Face, chars: impl IntoIterator<Item = char>) -> Vec<(char, i32)> {
    chars
        .into_iter()
        .filter_map(|c| {
            let g = face.glyph_id(c)?;
            let a = face.advance(g).ok()?;
            Some((c, face.to_pdf_units(i64::from(a))))
        })
        .collect()
}

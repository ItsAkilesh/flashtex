//! Opt-in embedding of a Unicode OpenType font for characters that neither
//! WinAnsi Times-Roman nor Symbol can show.
//!
//! Two font programs are handled, both under a Type0 font with `Identity-H`
//! encoding (two bytes per glyph), a `/W` widths array, and a `ToUnicode`
//! CMap so text extraction and search return the original characters:
//!
//! - **TrueType** (`glyf`): subset to the glyphs used and embedded as
//!   `/FontFile2` under a `CIDFontType2` with `/CIDToGIDMap /Identity`.
//! - **CFF OpenType** (`OTTO`, e.g. Latin Modern): the raw `CFF ` table is
//!   embedded **whole** as `/FontFile3` `/Subtype /CIDFontType0C` under a
//!   `CIDFontType0`. Latin Modern's CFF is not CID-keyed; CoreGraphics
//!   (Preview, PDFKit, `sips`) selects glyphs by CID = GID for such a
//!   program, so the OpenType `cmap` lookup gives the codes to write and the
//!   raster and text extraction were verified. PDF 32000 §9.7.4.2 words the
//!   non-CID-keyed case in terms of the CFF charset, so other viewers may
//!   differ; converting the program to a CID-keyed CFF (as dvipdfmx does) is
//!   the robust follow-up and will come with CFF subsetting. `/Type1C` was
//!   tried first and CoreGraphics rejects it for a CIDFontType0
//!   ("unsupported CIDFontType0 subtype"); `/OpenType` with the whole file
//!   also works but is 50 KB larger per document. No subsetting yet: every
//!   document carries the full CFF table.
//!
//! Nothing here is on by default. Callers opt in with a path or with
//! [`EmbedFont::discover`], which honours `FLASHTEX_UNICODE_FONT`, then looks
//! for Latin Modern (`FLASHTEX_LM_DIR`, TeX Live), then, on macOS,
//! Apple-supplied system fonts. Embedding a system font into a PDF that is
//! redistributed has licence implications the user must weigh; the README
//! says so, and this crate does not decide for them.

use crate::truetype::{Outlines, Subset, TrueTypeFont};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const ENV_VAR: &str = "FLASHTEX_UNICODE_FONT";
/// Directory containing `lmroman10-regular.otf` (Latin Modern), checked
/// before the TeX Live locations below.
pub const LM_DIR_ENV_VAR: &str = "FLASHTEX_LM_DIR";
/// The Latin Modern face `auto` prefers: LaTeX's default text font, GUST
/// Font License.
pub const LATIN_MODERN_FILE: &str = "lmroman10-regular.otf";
/// Roots under which `<root>/<release>/texmf-dist/fonts/opentype/public/lm/`
/// is searched, newest release first.
pub const TEXLIVE_ROOTS: [&str; 3] = ["/usr/local/texlive", "/opt/texlive", "/usr/share/texlive"];
/// Fixed Latin Modern locations tried after the TeX Live roots.
pub const LATIN_MODERN_FIXED: [&str; 2] = [
    "/usr/share/texlive/texmf-dist/fonts/opentype/public/lm",
    "/usr/share/texmf/fonts/opentype/public/lm",
];

/// macOS system fonts tried, in order, after Latin Modern. These are
/// Apple-supplied; see the module docs about redistribution.
pub const MACOS_FALLBACKS: [&str; 2] = [
    "/System/Library/Fonts/Supplemental/Times New Roman.ttf",
    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
];

/// A parsed font ready to be subset per document.
#[derive(Debug, Clone)]
pub struct EmbedFont {
    pub font: TrueTypeFont,
    pub source: PathBuf,
}

impl EmbedFont {
    pub fn load(path: &Path) -> Result<Self, String> {
        Ok(EmbedFont {
            font: TrueTypeFont::load(path)?,
            source: path.to_path_buf(),
        })
    }

    /// `FLASHTEX_UNICODE_FONT` if set, else the first existing entry of
    /// [`candidate_paths`] (Latin Modern, then macOS system fonts).
    /// `Ok(None)` means no candidate exists; a candidate that exists but fails
    /// to parse is an error, not silently skipped.
    pub fn discover() -> Result<Option<Self>, String> {
        if let Some(p) = std::env::var_os(ENV_VAR) {
            let path = PathBuf::from(p);
            return Self::load(&path)
                .map(Some)
                .map_err(|e| format!("{ENV_VAR}: {e}"));
        }
        for candidate in candidate_paths() {
            if candidate.is_file() {
                return Self::load(&candidate).map(Some);
            }
        }
        Ok(None)
    }
}

/// The search list for `auto`, in order: `$FLASHTEX_LM_DIR/lmroman10-regular.otf`,
/// Latin Modern under each TeX Live root (newest release directory first),
/// the fixed Linux TeX locations, then on macOS the Apple system fonts.
/// Paths are listed whether or not they exist; [`EmbedFont::discover`]
/// takes the first that does.
pub fn candidate_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(dir) = std::env::var_os(LM_DIR_ENV_VAR) {
        out.push(PathBuf::from(dir).join(LATIN_MODERN_FILE));
    }
    for root in TEXLIVE_ROOTS {
        if let Ok(entries) = std::fs::read_dir(root) {
            let mut releases: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            releases.sort();
            releases.reverse();
            for release in releases {
                out.push(
                    release
                        .join("texmf-dist/fonts/opentype/public/lm")
                        .join(LATIN_MODERN_FILE),
                );
            }
        }
    }
    for dir in LATIN_MODERN_FIXED {
        out.push(PathBuf::from(dir).join(LATIN_MODERN_FILE));
    }
    if cfg!(target_os = "macos") {
        out.extend(MACOS_FALLBACKS.iter().map(PathBuf::from));
    }
    out
}

/// The glyph program to embed.
#[derive(Debug, Clone)]
pub enum Program {
    /// A subset TrueType font: `/FontFile2`, `CIDFontType2`.
    TrueType(Subset),
    /// The font's whole `CFF ` table: `/FontFile3` `/Subtype /CIDFontType0C`,
    /// `CIDFontType0`. Glyph ids are the original font's.
    Cff {
        bytes: Vec<u8>,
        /// Advance widths in font units for every glyph id used, for `/W`.
        used_advances: BTreeMap<u16, u16>,
    },
}

/// The per-document embedded program: which characters map to which glyph
/// ids in it.
#[derive(Debug, Clone)]
pub struct EmbeddedSubset {
    pub program: Program,
    pub units_per_em: u16,
    /// Character to subset glyph id, for every character the font covers
    /// among those requested.
    pub chars: BTreeMap<char, u16>,
    /// `/BaseFont` name, with the six-letter subset tag when subset.
    pub base_font: String,
    pub descriptor: Descriptor,
}

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub bbox: [i32; 4],
    pub ascent: i32,
    pub descent: i32,
    pub cap_height: i32,
    pub italic_angle: f64,
}

impl EmbedFont {
    /// Subsets the font to the characters it covers among `wanted`.
    /// Characters the font lacks are simply absent from `chars`.
    pub fn subset_for(
        &self,
        wanted: impl IntoIterator<Item = char>,
    ) -> Result<EmbeddedSubset, String> {
        let mut old_gids: BTreeMap<char, u16> = BTreeMap::new();
        for c in wanted {
            if let Some(g) = self.font.glyph_id(c) {
                old_gids.insert(c, g);
            }
        }
        let gids: Vec<u16> = old_gids.values().copied().collect();
        let (program, chars, base_font): (Program, BTreeMap<char, u16>, String) =
            match self.font.outlines {
                Outlines::TrueType => {
                    let subset = self.font.subset(&gids)?;
                    let chars = old_gids
                        .iter()
                        .map(|(&c, old)| (c, subset.glyph_map[old]))
                        .collect();
                    let name = format!("{}+{}", subset_tag(&gids), self.font.postscript_name);
                    (Program::TrueType(subset), chars, name)
                }
                Outlines::Cff => {
                    let bytes = self
                        .font
                        .cff_table()
                        .ok_or("OTTO font without a CFF table")?
                        .to_vec();
                    let used_advances = gids.iter().map(|&g| (g, self.font.advance(g))).collect();
                    (
                        Program::Cff {
                            bytes,
                            used_advances,
                        },
                        old_gids,
                        self.font.postscript_name.clone(),
                    )
                }
            };
        let scale =
            |v: i16| -> i32 { (v as f64 * 1000.0 / self.font.units_per_em as f64).round() as i32 };
        let f = &self.font;
        let descriptor = Descriptor {
            bbox: [
                scale(f.bbox[0]),
                scale(f.bbox[1]),
                scale(f.bbox[2]),
                scale(f.bbox[3]),
            ],
            ascent: scale(f.ascender),
            descent: scale(f.descender),
            cap_height: scale(f.cap_height.unwrap_or(f.ascender)),
            italic_angle: f.italic_angle,
        };
        Ok(EmbeddedSubset {
            units_per_em: f.units_per_em,
            program,
            chars,
            base_font,
            descriptor,
        })
    }
}

/// Six uppercase letters derived deterministically from the glyph set, as the
/// PDF spec requires for subset fonts.
fn subset_tag(gids: &[u16]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for g in gids {
        h ^= *g as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (0..6)
        .map(|i| (b'A' + ((h >> (8 * i)) % 26) as u8) as char)
        .collect()
}

impl EmbeddedSubset {
    /// `/W` array in 1000/em units: one dense run from glyph 0 for a subset,
    /// or one entry per used glyph id for a whole CFF.
    pub fn widths_array(&self) -> String {
        let scale = 1000.0 / self.units_per_em as f64;
        let w = |a: u16| ((a as f64 * scale).round() as i64).to_string();
        match &self.program {
            Program::TrueType(subset) => {
                let widths: Vec<String> = subset.advances.iter().map(|&a| w(a)).collect();
                format!("[ 0 [ {} ] ]", widths.join(" "))
            }
            Program::Cff { used_advances, .. } => {
                let entries: Vec<String> = used_advances
                    .iter()
                    .map(|(&gid, &a)| format!("{gid} [ {} ]", w(a)))
                    .collect();
                if entries.is_empty() {
                    "[ ]".to_string()
                } else {
                    format!("[ {} ]", entries.join(" "))
                }
            }
        }
    }

    /// Bytes of the program that will be written to the font file stream.
    pub fn program_bytes(&self) -> &[u8] {
        match &self.program {
            Program::TrueType(subset) => &subset.bytes,
            Program::Cff { bytes, .. } => bytes,
        }
    }

    /// The ToUnicode CMap stream body.
    pub fn to_unicode_cmap(&self) -> Vec<u8> {
        let mut s = String::from(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
             /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
             /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
             1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
        );
        let entries: Vec<(u16, char)> = self.chars.iter().map(|(&c, &g)| (g, c)).collect();
        for chunk in entries.chunks(100) {
            s.push_str(&format!("{} beginbfchar\n", chunk.len()));
            for (gid, c) in chunk {
                let mut units = [0u16; 2];
                let utf16 = c.encode_utf16(&mut units);
                let hex: String = utf16.iter().map(|u| format!("{u:04X}")).collect();
                s.push_str(&format!("<{gid:04X}> <{hex}>\n"));
            }
            s.push_str("endbfchar\n");
        }
        s.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
        s.into_bytes()
    }
}

/// Parses a ToUnicode CMap written by [`EmbeddedSubset::to_unicode_cmap`]
/// back into glyph id to character, for tests and self-checks.
pub fn parse_to_unicode(cmap: &[u8]) -> Result<BTreeMap<u16, char>, String> {
    let text = std::str::from_utf8(cmap).map_err(|_| "CMap is not UTF-8")?;
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
            .ok_or_else(|| format!("malformed bfchar line {line:?}"))?;
        let gid = u16::from_str_radix(src.trim_start_matches('<'), 16).map_err(|_| "bad gid")?;
        let dst = dst.trim_end_matches('>');
        let units: Vec<u16> = dst
            .as_bytes()
            .chunks(4)
            .map(|c| u16::from_str_radix(std::str::from_utf8(c).unwrap_or("zz"), 16))
            .collect::<Result<_, _>>()
            .map_err(|_| "bad UTF-16")?;
        let s = String::from_utf16(&units).map_err(|_| "bad UTF-16 sequence")?;
        let mut chars = s.chars();
        let c = chars.next().ok_or("empty mapping")?;
        if chars.next().is_some() {
            return Err(format!("multi-character mapping for {gid}"));
        }
        map.insert(gid, c);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subset_tag_is_six_uppercase_letters_and_deterministic() {
        let a = subset_tag(&[1, 2, 3]);
        assert_eq!(a.len(), 6);
        assert!(a.chars().all(|c| c.is_ascii_uppercase()));
        assert_eq!(a, subset_tag(&[1, 2, 3]));
        assert_ne!(a, subset_tag(&[1, 2, 4]));
    }
}

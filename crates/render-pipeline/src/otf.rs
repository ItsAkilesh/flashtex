//! CFF-flavoured OpenType (`OTTO`) face implementing font-engine's `Face`
//! trait, so font-engine's shaper (kerning + ligatures + clusters) runs
//! unchanged on Latin Modern.
//!
//! TEMPORARY SHIM: font-engine 07fa9fe rejects `OTTO`. Tables read: `head`,
//! `hhea`, `hmtx`, `maxp`, `cmap` (4/12), `OS/2`, `post`, `name`, `GPOS`
//! (kern PairPos), `GSUB` (liga), `CFF ` (glyph bounds via `cff.rs`) and
//! `MATH` (constants and vertical variants). Whole-program bytes are kept for
//! content-addressed identity (SHA-256 of program + face index) and for
//! whole-font PDF embedding.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use flashtex_font_engine::sha256;
use flashtex_font_engine::{
    Error, Face, FontId, FontSource, GlyphId, KerningSource, Style, Unsupported, VerticalMetrics,
};

use crate::cff::{self, Cff};
use crate::otl::{i16_at, i32_at, slice, u16_at, u32_at, GposKerning, GsubLigatures};

/// Glyph extents in font units: `[x_min, y_min, x_max, y_max]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bounds {
    pub x_min: i32,
    pub y_min: i32,
    pub x_max: i32,
    pub y_max: i32,
    pub empty: bool,
}

/// One vertical glyph variant from the MATH table.
#[derive(Debug, Clone, Copy)]
pub struct VertVariant {
    pub gid: GlyphId,
    /// Total height (ascent + descent) in font units, as declared.
    pub advance: u16,
}

#[derive(Debug, Clone, Default)]
pub struct MathTable {
    /// Constant index (OpenType MathConstants order) -> value in font units.
    constants: Vec<i32>,
    vert_variants: BTreeMap<u16, Vec<VertVariant>>,
}

impl MathTable {
    pub fn constant(&self, index: usize) -> Option<i32> {
        self.constants.get(index).copied()
    }

    pub fn vertical_variants(&self, gid: GlyphId) -> &[VertVariant] {
        self.vert_variants.get(&gid.0).map_or(&[], |v| v.as_slice())
    }
}

#[derive(Debug)]
pub struct OtfFace {
    data: Vec<u8>,
    tables: BTreeMap<[u8; 4], (usize, usize)>,
    id: FontId,
    units_per_em: u16,
    num_glyphs: u16,
    bbox: [i16; 4],
    advances: Vec<u16>,
    cmap: BTreeMap<u32, u16>,
    metrics: VerticalMetrics,
    italic_angle: f64,
    is_fixed_pitch: bool,
    postscript_name: String,
    gpos: GposKerning,
    gsub: GsubLigatures,
    cff: Cff,
    math: Option<MathTable>,
    unsupported: Vec<Unsupported>,
    bounds_cache: RefCell<BTreeMap<u16, Bounds>>,
    /// Stable human-readable name (file stem), e.g. `lmroman12-regular`.
    pub name: String,
}

impl OtfFace {
    pub fn load(path: &Path) -> Result<OtfFace, Error> {
        let bytes =
            std::fs::read(path).map_err(|e| Error::Io(format!("{}: {e}", path.display())))?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "font".into());
        OtfFace::parse(
            bytes,
            FontSource::File {
                path: path.to_path_buf(),
                face_index: 0,
            },
            name,
        )
    }

    pub fn parse(data: Vec<u8>, source: FontSource, name: String) -> Result<OtfFace, Error> {
        let b = &data[..];
        let sfnt = u32_at(b, 0)?;
        if sfnt != 0x4F54_544F {
            return Err(Error::Unsupported(format!(
                "not a CFF OpenType font (sfnt 0x{sfnt:08X}); use font-engine's TrueTypeFace"
            )));
        }
        let n_tables = usize::from(u16_at(b, 4)?);
        let mut tables = BTreeMap::new();
        for i in 0..n_tables {
            let rec = 12 + 16 * i;
            let tag: [u8; 4] = slice(b, rec, 4)?.try_into().unwrap();
            let off = u32_at(b, rec + 8)? as usize;
            let len = u32_at(b, rec + 12)? as usize;
            if off.checked_add(len).map_or(true, |end| end > b.len()) {
                return Err(Error::Malformed(format!(
                    "table {} overruns file",
                    String::from_utf8_lossy(&tag)
                )));
            }
            tables.insert(tag, (off, len));
        }
        for t in [b"head", b"hhea", b"hmtx", b"maxp", b"cmap", b"CFF "] {
            if !tables.contains_key(t) {
                return Err(Error::MissingTable(String::from_utf8_lossy(t).into_owned()));
            }
        }
        let table = |tag: &[u8; 4]| -> Option<&[u8]> { tables.get(tag).map(|(o, l)| &b[*o..*o + *l]) };

        let head = table(b"head").unwrap();
        if u32_at(head, 12)? != 0x5F0F_3CF5 {
            return Err(Error::Malformed("head magic".into()));
        }
        let units_per_em = u16_at(head, 18)?;
        if units_per_em == 0 {
            return Err(Error::Malformed("unitsPerEm is 0".into()));
        }
        let bbox = [i16_at(head, 36)?, i16_at(head, 38)?, i16_at(head, 40)?, i16_at(head, 42)?];
        let mac_style = u16_at(head, 44)?;
        let maxp = table(b"maxp").unwrap();
        let num_glyphs = u16_at(maxp, 4)?;
        let hhea = table(b"hhea").unwrap();
        let mut metrics = VerticalMetrics {
            ascender: i16_at(hhea, 4)?,
            descender: i16_at(hhea, 6)?,
            line_gap: i16_at(hhea, 8)?,
            cap_height: 0,
            x_height: 0,
            cap_height_declared: false,
            x_height_declared: false,
        };
        let num_hmetrics = usize::from(u16_at(hhea, 34)?);
        let hmtx = table(b"hmtx").unwrap();
        let mut advances = Vec::with_capacity(usize::from(num_glyphs));
        let mut last = 0u16;
        for g in 0..usize::from(num_glyphs) {
            if g < num_hmetrics {
                last = u16_at(hmtx, 4 * g)?;
            }
            advances.push(last);
        }
        let cmap = parse_cmap(table(b"cmap").unwrap())?;
        let mut weight = 400u16;
        let mut os2_italic = false;
        let mut os2_oblique = false;
        if let Some(os2) = table(b"OS/2") {
            let version = u16_at(os2, 0)?;
            weight = u16_at(os2, 4)?;
            let fs_selection = u16_at(os2, 62)?;
            os2_italic = fs_selection & 0x0001 != 0;
            os2_oblique = fs_selection & 0x0200 != 0;
            if fs_selection & 0x0080 != 0 {
                metrics.ascender = i16_at(os2, 68)?;
                metrics.descender = i16_at(os2, 70)?;
                metrics.line_gap = i16_at(os2, 72)?;
            }
            if version >= 2 && os2.len() >= 90 {
                metrics.x_height = i16_at(os2, 86)?;
                metrics.cap_height = i16_at(os2, 88)?;
                metrics.x_height_declared = metrics.x_height != 0;
                metrics.cap_height_declared = metrics.cap_height != 0;
            }
        }
        let mut italic_angle = 0.0;
        let mut is_fixed_pitch = false;
        if let Some(post) = table(b"post") {
            italic_angle = f64::from(i32_at(post, 4)?) / 65536.0;
            is_fixed_pitch = u32_at(post, 12)? != 0;
        }
        let (family, ps) = match table(b"name") {
            Some(t) => parse_names(t)?,
            None => (None, None),
        };
        let family = family.unwrap_or_else(|| name.clone());
        let postscript_name = ps.unwrap_or_else(|| family.replace(' ', ""));
        let style = if os2_oblique {
            Style::Oblique
        } else if os2_italic || mac_style & 0x0002 != 0 || italic_angle != 0.0 {
            Style::Italic
        } else {
            Style::Upright
        };
        let mut unsupported = Vec::new();
        let gpos = match table(b"GPOS") {
            Some(t) => GposKerning::parse(t)?,
            None => GposKerning::default(),
        };
        let gsub = match table(b"GSUB") {
            Some(t) => GsubLigatures::parse(t)?,
            None => GsubLigatures::default(),
        };
        unsupported.extend(gpos.unsupported.iter().cloned());
        unsupported.extend(gsub.unsupported.iter().cloned());
        let cff = Cff::parse(table(b"CFF ").unwrap())?;
        if cff.num_glyphs() != usize::from(num_glyphs) {
            return Err(Error::Malformed(format!(
                "CFF has {} charstrings but maxp says {num_glyphs}",
                cff.num_glyphs()
            )));
        }
        let math = match table(b"MATH") {
            Some(t) => Some(parse_math(t)?),
            None => None,
        };
        let mut hash_input = data.clone();
        hash_input.extend_from_slice(&0u32.to_be_bytes());
        let content_sha256 = sha256::digest(&hash_input);
        Ok(OtfFace {
            id: FontId {
                family,
                weight,
                style,
                source,
                content_sha256,
            },
            data,
            tables,
            units_per_em,
            num_glyphs,
            bbox,
            advances,
            cmap,
            metrics,
            italic_angle,
            is_fixed_pitch,
            postscript_name,
            gpos,
            gsub,
            cff,
            math,
            unsupported,
            bounds_cache: RefCell::new(BTreeMap::new()),
            name,
        })
    }

    /// Whole font program (for hashing and embedding).
    pub fn program(&self) -> &[u8] {
        &self.data
    }

    pub fn table(&self, tag: &[u8; 4]) -> Option<&[u8]> {
        self.tables.get(tag).map(|(o, l)| &self.data[*o..*o + *l])
    }

    pub fn math(&self) -> Option<&MathTable> {
        self.math.as_ref()
    }

    /// Glyph bounds in font units (cached).
    pub fn bounds(&self, gid: GlyphId) -> Result<Bounds, Error> {
        if let Some(b) = self.bounds_cache.borrow().get(&gid.0) {
            return Ok(*b);
        }
        let (o, l) = self.tables[b"CFF "];
        let (bb, _) = self.cff.glyph_bbox(&self.data[o..o + l], gid.0)?;
        let bounds = match cff::round_bbox(bb.map(|b| b)) {
            Some([x0, y0, x1, y1]) => Bounds {
                x_min: x0,
                y_min: y0,
                x_max: x1,
                y_max: y1,
                empty: false,
            },
            None => Bounds {
                empty: true,
                ..Bounds::default()
            },
        };
        self.bounds_cache.borrow_mut().insert(gid.0, bounds);
        Ok(bounds)
    }

    pub fn char_map(&self) -> &BTreeMap<u32, u16> {
        &self.cmap
    }
}

impl Face for OtfFace {
    fn id(&self) -> &FontId {
        &self.id
    }
    fn units_per_em(&self) -> u16 {
        self.units_per_em
    }
    fn num_glyphs(&self) -> u16 {
        self.num_glyphs
    }
    fn vertical_metrics(&self) -> VerticalMetrics {
        self.metrics
    }
    fn bbox(&self) -> [i16; 4] {
        self.bbox
    }
    fn italic_angle(&self) -> f64 {
        self.italic_angle
    }
    fn advance(&self, gid: GlyphId) -> Result<u16, Error> {
        self.advances
            .get(usize::from(gid.0))
            .copied()
            .ok_or(Error::GlyphOutOfRange(gid.0))
    }
    fn glyph_id(&self, ch: char) -> Option<GlyphId> {
        self.cmap.get(&(ch as u32)).copied().filter(|g| *g != 0).map(GlyphId)
    }
    fn kerning(&self, left: GlyphId, right: GlyphId) -> (i16, KerningSource) {
        if self.gpos.is_empty() {
            return (0, KerningSource::None);
        }
        (self.gpos.kerning(left, right).unwrap_or(0), KerningSource::Gpos)
    }
    fn ligature(&self, components: &[GlyphId]) -> Option<GlyphId> {
        self.gsub.ligature(components)
    }
    fn longest_ligature(&self, glyphs: &[GlyphId]) -> Option<(GlyphId, usize)> {
        self.gsub.longest(glyphs)
    }
    fn unsupported(&self) -> &[Unsupported] {
        &self.unsupported
    }
    fn postscript_name(&self) -> &str {
        &self.postscript_name
    }
    fn is_fixed_pitch(&self) -> bool {
        self.is_fixed_pitch
    }
}

fn parse_cmap(cmap: &[u8]) -> Result<BTreeMap<u32, u16>, Error> {
    let n = usize::from(u16_at(cmap, 2)?);
    let mut best: Option<(u8, usize)> = None;
    for i in 0..n {
        let rec = 4 + 8 * i;
        let platform = u16_at(cmap, rec)?;
        let encoding = u16_at(cmap, rec + 2)?;
        let off = u32_at(cmap, rec + 4)? as usize;
        let format = u16_at(cmap, off)?;
        let rank = match (platform, encoding, format) {
            (3, 10, 12) => 5,
            (0, _, 12) => 4,
            (3, 1, 4) => 3,
            (0, _, 4) => 2,
            _ => continue,
        };
        if best.map_or(true, |(r, _)| rank > r) {
            best = Some((rank, off));
        }
    }
    let Some((_, off)) = best else {
        return Err(Error::Unsupported("cmap has no Unicode format 4 or 12 subtable".into()));
    };
    let mut map = BTreeMap::new();
    match u16_at(cmap, off)? {
        4 => {
            let seg_x2 = usize::from(u16_at(cmap, off + 6)?);
            let segs = seg_x2 / 2;
            let ends = off + 14;
            let starts = ends + seg_x2 + 2;
            let deltas = starts + seg_x2;
            let range_offsets = deltas + seg_x2;
            for s in 0..segs {
                let end = u16_at(cmap, ends + 2 * s)?;
                let start = u16_at(cmap, starts + 2 * s)?;
                let delta = u16_at(cmap, deltas + 2 * s)?;
                let ro = u16_at(cmap, range_offsets + 2 * s)?;
                if start > end {
                    return Err(Error::Malformed("cmap segment start > end".into()));
                }
                for c in start..=end {
                    if c == 0xFFFF {
                        break;
                    }
                    let g = if ro == 0 {
                        c.wrapping_add(delta)
                    } else {
                        let addr = range_offsets + 2 * s + usize::from(ro) + 2 * usize::from(c - start);
                        let raw = u16_at(cmap, addr)?;
                        if raw == 0 {
                            0
                        } else {
                            raw.wrapping_add(delta)
                        }
                    };
                    if g != 0 {
                        map.entry(u32::from(c)).or_insert(g);
                    }
                }
            }
        }
        12 => {
            let n_groups = u32_at(cmap, off + 12)? as usize;
            for i in 0..n_groups {
                let rec = off + 16 + 12 * i;
                let start = u32_at(cmap, rec)?;
                let end = u32_at(cmap, rec + 4)?;
                let start_gid = u32_at(cmap, rec + 8)?;
                if start > end || end > 0x10FFFF {
                    return Err(Error::Malformed("cmap group range".into()));
                }
                for c in start..=end {
                    let g = start_gid + (c - start);
                    if g != 0 && g <= 0xFFFF {
                        map.entry(c).or_insert(g as u16);
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(map)
}

fn parse_names(name: &[u8]) -> Result<(Option<String>, Option<String>), Error> {
    let count = usize::from(u16_at(name, 2)?);
    let storage = usize::from(u16_at(name, 4)?);
    let mut family: Option<(u8, String)> = None;
    let mut ps: Option<(u8, String)> = None;
    for i in 0..count {
        let rec = 6 + 12 * i;
        let platform = u16_at(name, rec)?;
        let encoding = u16_at(name, rec + 2)?;
        let id = u16_at(name, rec + 6)?;
        let len = usize::from(u16_at(name, rec + 8)?);
        let off = storage + usize::from(u16_at(name, rec + 10)?);
        if id != 1 && id != 6 {
            continue;
        }
        let Ok(bytes) = slice(name, off, len) else {
            continue;
        };
        let (rank, text) = match (platform, encoding) {
            (3, 1) | (3, 10) | (0, _) => {
                let units: Vec<u16> =
                    bytes.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
                (2u8, String::from_utf16_lossy(&units))
            }
            (1, 0) => (1u8, bytes.iter().map(|b| *b as char).collect()),
            _ => continue,
        };
        let slot = if id == 1 { &mut family } else { &mut ps };
        if slot.as_ref().map_or(true, |(r, _)| rank > *r) {
            *slot = Some((rank, text));
        }
    }
    Ok((family.map(|(_, s)| s), ps.map(|(_, s)| s)))
}

/// Parses MathConstants (all 56, as font units) and the vertical variants of
/// MathVariants. Glyph assemblies (extensible pieces) are not read.
fn parse_math(m: &[u8]) -> Result<MathTable, Error> {
    let version = u32_at(m, 0)?;
    if version >> 16 != 1 {
        return Err(Error::Unsupported(format!("MATH version {version:#x}")));
    }
    let constants_off = usize::from(u16_at(m, 4)?);
    let variants_off = usize::from(u16_at(m, 8)?);
    let mut constants = Vec::with_capacity(56);
    let c = constants_off;
    constants.push(i32::from(i16_at(m, c)?));
    constants.push(i32::from(i16_at(m, c + 2)?));
    constants.push(i32::from(u16_at(m, c + 4)?));
    constants.push(i32::from(u16_at(m, c + 6)?));
    for k in 4..55 {
        constants.push(i32::from(i16_at(m, c + 8 + 4 * (k - 4))?));
    }
    constants.push(i32::from(i16_at(m, c + 8 + 4 * 51)?));
    let mut vert_variants = BTreeMap::new();
    if variants_off != 0 {
        let v = variants_off;
        let vert_cov = usize::from(u16_at(m, v + 2)?);
        let vert_count = usize::from(u16_at(m, v + 6)?);
        if vert_cov != 0 {
            let coverage = crate::otl::Coverage::parse(m, v + vert_cov)?;
            let gids: Vec<u16> = match &coverage {
                crate::otl::Coverage::Glyphs(g) => g.clone(),
                crate::otl::Coverage::Ranges(r) => {
                    r.iter().flat_map(|(s, e, _)| *s..=*e).collect()
                }
            };
            for (i, gid) in gids.iter().enumerate().take(vert_count) {
                let cons = v + usize::from(u16_at(m, v + 10 + 2 * i)?);
                let n = usize::from(u16_at(m, cons + 2)?);
                let mut list = Vec::with_capacity(n);
                for j in 0..n {
                    let rec = cons + 4 + 4 * j;
                    list.push(VertVariant {
                        gid: GlyphId(u16_at(m, rec)?),
                        advance: u16_at(m, rec + 2)?,
                    });
                }
                vert_variants.insert(*gid, list);
            }
        }
    }
    Ok(MathTable {
        constants,
        vert_variants,
    })
}

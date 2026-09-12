//! OpenType Layout structures needed by the CFF face shim: coverage tables,
//! class definitions, the FeatureList/LookupList walk, `GPOS` pair kerning
//! and `GSUB` ligatures.
//!
//! TEMPORARY SHIM. This is a verbatim-in-spirit copy of the *private* modules
//! `otl.rs`, `gpos.rs` and `gsub.rs` of `crates/font-engine` at 07fa9fe, which
//! only exposes them through `TrueTypeFace` (a `glyf` parser that rejects
//! CFF-flavoured `OTTO` fonts). Latin Modern is CFF, so this crate needs the
//! same layout tables on an `OTTO` face. Delete this file as soon as
//! font-engine either accepts `OTTO` in `TrueTypeFace` or exports these
//! modules (requested in docs/proposals/rendering-abi.md).

use flashtex_font_engine::{Error, GlyphId, Unsupported};

pub fn u16_at(b: &[u8], at: usize) -> Result<u16, Error> {
    b.get(at..at + 2)
        .map(|s| u16::from_be_bytes([s[0], s[1]]))
        .ok_or_else(|| Error::Malformed(format!("read u16 at {at} past end")))
}

pub fn i16_at(b: &[u8], at: usize) -> Result<i16, Error> {
    u16_at(b, at).map(|v| v as i16)
}

pub fn u32_at(b: &[u8], at: usize) -> Result<u32, Error> {
    b.get(at..at + 4)
        .map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| Error::Malformed(format!("read u32 at {at} past end")))
}

pub fn i32_at(b: &[u8], at: usize) -> Result<i32, Error> {
    u32_at(b, at).map(|v| v as i32)
}

pub fn slice(b: &[u8], at: usize, len: usize) -> Result<&[u8], Error> {
    b.get(at..at + len)
        .ok_or_else(|| Error::Malformed(format!("slice {at}+{len} past end ({})", b.len())))
}

/// A parsed Coverage table: glyph id -> coverage index.
#[derive(Debug, Clone)]
pub enum Coverage {
    Glyphs(Vec<u16>),
    Ranges(Vec<(u16, u16, u16)>),
}

impl Coverage {
    pub fn parse(b: &[u8], at: usize) -> Result<Coverage, Error> {
        match u16_at(b, at)? {
            1 => {
                let n = usize::from(u16_at(b, at + 2)?);
                let mut v = Vec::with_capacity(n);
                for i in 0..n {
                    v.push(u16_at(b, at + 4 + 2 * i)?);
                }
                Ok(Coverage::Glyphs(v))
            }
            2 => {
                let n = usize::from(u16_at(b, at + 2)?);
                let mut v = Vec::with_capacity(n);
                for i in 0..n {
                    let r = at + 4 + 6 * i;
                    v.push((u16_at(b, r)?, u16_at(b, r + 2)?, u16_at(b, r + 4)?));
                }
                Ok(Coverage::Ranges(v))
            }
            f => Err(Error::Malformed(format!("coverage format {f}"))),
        }
    }

    pub fn index(&self, gid: u16) -> Option<u16> {
        match self {
            Coverage::Glyphs(v) => v.binary_search(&gid).ok().map(|i| i as u16),
            Coverage::Ranges(v) => {
                let i = v.partition_point(|(start, _, _)| *start <= gid);
                if i == 0 {
                    return None;
                }
                let (start, end, base) = v[i - 1];
                (gid <= end).then(|| base + (gid - start))
            }
        }
    }
}

/// A parsed ClassDef table: glyph id -> class (0 when unlisted).
#[derive(Debug, Clone)]
pub enum ClassDef {
    Array { start: u16, classes: Vec<u16> },
    Ranges(Vec<(u16, u16, u16)>),
}

impl ClassDef {
    pub fn parse(b: &[u8], at: usize) -> Result<ClassDef, Error> {
        match u16_at(b, at)? {
            1 => {
                let start = u16_at(b, at + 2)?;
                let n = usize::from(u16_at(b, at + 4)?);
                let mut classes = Vec::with_capacity(n);
                for i in 0..n {
                    classes.push(u16_at(b, at + 6 + 2 * i)?);
                }
                Ok(ClassDef::Array { start, classes })
            }
            2 => {
                let n = usize::from(u16_at(b, at + 2)?);
                let mut v = Vec::with_capacity(n);
                for i in 0..n {
                    let r = at + 4 + 6 * i;
                    v.push((u16_at(b, r)?, u16_at(b, r + 2)?, u16_at(b, r + 4)?));
                }
                Ok(ClassDef::Ranges(v))
            }
            f => Err(Error::Malformed(format!("class def format {f}"))),
        }
    }

    pub fn class(&self, gid: u16) -> u16 {
        match self {
            ClassDef::Array { start, classes } => {
                if gid < *start {
                    return 0;
                }
                classes.get(usize::from(gid - start)).copied().unwrap_or(0)
            }
            ClassDef::Ranges(v) => {
                let i = v.partition_point(|(start, _, _)| *start <= gid);
                if i == 0 {
                    return 0;
                }
                let (_, end, class) = v[i - 1];
                if gid <= end {
                    class
                } else {
                    0
                }
            }
        }
    }
}

/// Lookup indices referenced by every FeatureRecord with `tag`, deduplicated.
/// Walks the FeatureList directly (no script/language selection), as the
/// font-engine original does.
pub fn lookups_for_feature(b: &[u8], tag: &[u8; 4]) -> Result<Vec<u16>, Error> {
    let version_major = u16_at(b, 0)?;
    if version_major != 1 {
        return Err(Error::Unsupported(format!(
            "layout table version {version_major}"
        )));
    }
    let feature_list = usize::from(u16_at(b, 6)?);
    let n = usize::from(u16_at(b, feature_list)?);
    let mut out: Vec<u16> = Vec::new();
    for i in 0..n {
        let rec = feature_list + 2 + 6 * i;
        if slice(b, rec, 4)? != tag {
            continue;
        }
        let feat = feature_list + usize::from(u16_at(b, rec + 4)?);
        let count = usize::from(u16_at(b, feat + 2)?);
        for j in 0..count {
            out.push(u16_at(b, feat + 4 + 2 * j)?);
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct Lookup {
    pub index: u16,
    pub lookup_type: u16,
    pub subtables: Vec<usize>,
}

pub fn lookup(b: &[u8], index: u16, ext_type: u16) -> Result<Lookup, Error> {
    let lookup_list = usize::from(u16_at(b, 8)?);
    let count = u16_at(b, lookup_list)?;
    if index >= count {
        return Err(Error::Malformed(format!("lookup index {index} >= {count}")));
    }
    let at = lookup_list + usize::from(u16_at(b, lookup_list + 2 + 2 * usize::from(index))?);
    let mut lookup_type = u16_at(b, at)?;
    let n = usize::from(u16_at(b, at + 4)?);
    let mut subtables = Vec::with_capacity(n);
    for i in 0..n {
        let mut st = at + usize::from(u16_at(b, at + 6 + 2 * i)?);
        if lookup_type == ext_type {
            let inner_type = u16_at(b, st + 2)?;
            st += u32_at(b, st + 4)? as usize;
            lookup_type = inner_type;
        }
        subtables.push(st);
    }
    Ok(Lookup {
        index,
        lookup_type,
        subtables,
    })
}

// ---------------------------------------------------------------- GPOS kern

#[derive(Debug, Clone)]
enum PairSubtable {
    Format1 {
        coverage: Coverage,
        pair_sets: Vec<Vec<(u16, i16)>>,
    },
    Format2 {
        coverage: Coverage,
        class1: ClassDef,
        class2: ClassDef,
        class2_count: u16,
        values: Vec<i16>,
    },
}

#[derive(Debug, Clone, Default)]
pub struct GposKerning {
    subtables: Vec<PairSubtable>,
    pub unsupported: Vec<Unsupported>,
}

fn value_record_len(format: u16) -> usize {
    2 * usize::from(format.count_ones() as u16)
}

fn x_advance(b: &[u8], at: usize, format: u16) -> Result<i16, Error> {
    if format & 0x0004 == 0 {
        return Ok(0);
    }
    let mut off = at;
    if format & 0x0001 != 0 {
        off += 2;
    }
    if format & 0x0002 != 0 {
        off += 2;
    }
    i16_at(b, off)
}

impl GposKerning {
    pub fn parse(gpos: &[u8]) -> Result<GposKerning, Error> {
        let mut out = GposKerning::default();
        for index in lookups_for_feature(gpos, b"kern")? {
            let lk = lookup(gpos, index, 9)?;
            if lk.lookup_type != 2 {
                out.unsupported.push(Unsupported {
                    table: "GPOS",
                    detail: format!(
                        "kern lookup {} is type {} (only PairPos type 2 is applied)",
                        lk.index, lk.lookup_type
                    ),
                });
                continue;
            }
            for st in lk.subtables {
                match parse_pair_subtable(gpos, st) {
                    Ok(sub) => out.subtables.push(sub),
                    Err(Error::Unsupported(detail)) => out.unsupported.push(Unsupported {
                        table: "GPOS",
                        detail,
                    }),
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(out)
    }

    pub fn is_empty(&self) -> bool {
        self.subtables.is_empty()
    }

    pub fn kerning(&self, left: GlyphId, right: GlyphId) -> Option<i16> {
        for sub in &self.subtables {
            match sub {
                PairSubtable::Format1 {
                    coverage,
                    pair_sets,
                } => {
                    let Some(ci) = coverage.index(left.0) else {
                        continue;
                    };
                    let set = pair_sets.get(usize::from(ci))?;
                    if let Ok(i) = set.binary_search_by_key(&right.0, |(g, _)| *g) {
                        return Some(set[i].1);
                    }
                }
                PairSubtable::Format2 {
                    coverage,
                    class1,
                    class2,
                    class2_count,
                    values,
                } => {
                    if coverage.index(left.0).is_none() {
                        continue;
                    }
                    let c1 = usize::from(class1.class(left.0));
                    let c2 = usize::from(class2.class(right.0));
                    return values.get(c1 * usize::from(*class2_count) + c2).copied();
                }
            }
        }
        None
    }
}

fn parse_pair_subtable(b: &[u8], at: usize) -> Result<PairSubtable, Error> {
    let format = u16_at(b, at)?;
    let coverage = Coverage::parse(b, at + usize::from(u16_at(b, at + 2)?))?;
    let vf1 = u16_at(b, at + 4)?;
    let vf2 = u16_at(b, at + 6)?;
    let rec_len = value_record_len(vf1) + value_record_len(vf2);
    match format {
        1 => {
            let n = usize::from(u16_at(b, at + 8)?);
            let mut pair_sets = Vec::with_capacity(n);
            for i in 0..n {
                let ps = at + usize::from(u16_at(b, at + 10 + 2 * i)?);
                let count = usize::from(u16_at(b, ps)?);
                let mut set = Vec::with_capacity(count);
                for j in 0..count {
                    let rec = ps + 2 + j * (2 + rec_len);
                    set.push((u16_at(b, rec)?, x_advance(b, rec + 2, vf1)?));
                }
                set.sort_by_key(|(g, _)| *g);
                pair_sets.push(set);
            }
            Ok(PairSubtable::Format1 {
                coverage,
                pair_sets,
            })
        }
        2 => {
            let class1 = ClassDef::parse(b, at + usize::from(u16_at(b, at + 8)?))?;
            let class2 = ClassDef::parse(b, at + usize::from(u16_at(b, at + 10)?))?;
            let class1_count = usize::from(u16_at(b, at + 12)?);
            let class2_count = u16_at(b, at + 14)?;
            let mut values = Vec::with_capacity(class1_count * usize::from(class2_count));
            let mut rec = at + 16;
            for _ in 0..class1_count * usize::from(class2_count) {
                values.push(x_advance(b, rec, vf1)?);
                rec += rec_len;
            }
            Ok(PairSubtable::Format2 {
                coverage,
                class1,
                class2,
                class2_count,
                values,
            })
        }
        f => Err(Error::Unsupported(format!("PairPos format {f}"))),
    }
}

// ---------------------------------------------------------------- GSUB liga

#[derive(Debug, Clone)]
struct LigatureSubtable {
    coverage: Coverage,
    sets: Vec<Vec<(u16, Vec<u16>)>>,
}

#[derive(Debug, Clone, Default)]
pub struct GsubLigatures {
    subtables: Vec<LigatureSubtable>,
    pub unsupported: Vec<Unsupported>,
}

impl GsubLigatures {
    pub fn parse(gsub: &[u8]) -> Result<GsubLigatures, Error> {
        let mut out = GsubLigatures::default();
        for index in lookups_for_feature(gsub, b"liga")? {
            let lk = lookup(gsub, index, 7)?;
            if lk.lookup_type != 4 {
                out.unsupported.push(Unsupported {
                    table: "GSUB",
                    detail: format!(
                        "liga lookup {} is type {} (only LigatureSubst type 4 is applied)",
                        lk.index, lk.lookup_type
                    ),
                });
                continue;
            }
            for st in lk.subtables {
                out.subtables.push(parse_ligature_subtable(gsub, st)?);
            }
        }
        Ok(out)
    }

    pub fn is_empty(&self) -> bool {
        self.subtables.is_empty()
    }

    pub fn ligature(&self, components: &[GlyphId]) -> Option<GlyphId> {
        let first = components.first()?;
        for sub in &self.subtables {
            let Some(ci) = sub.coverage.index(first.0) else {
                continue;
            };
            for (lig, rest) in &sub.sets[usize::from(ci)] {
                if rest.len() == components.len() - 1
                    && rest.iter().zip(&components[1..]).all(|(a, b)| *a == b.0)
                {
                    return Some(GlyphId(*lig));
                }
            }
        }
        None
    }

    pub fn longest(&self, glyphs: &[GlyphId]) -> Option<(GlyphId, usize)> {
        let first = glyphs.first()?;
        let mut best: Option<(GlyphId, usize)> = None;
        for sub in &self.subtables {
            let Some(ci) = sub.coverage.index(first.0) else {
                continue;
            };
            for (lig, rest) in &sub.sets[usize::from(ci)] {
                let len = rest.len() + 1;
                if len <= glyphs.len()
                    && rest.iter().zip(&glyphs[1..]).all(|(a, b)| *a == b.0)
                    && best.map_or(true, |(_, l)| len > l)
                {
                    best = Some((GlyphId(*lig), len));
                }
            }
            if best.is_some() {
                return best;
            }
        }
        best
    }
}

fn parse_ligature_subtable(b: &[u8], at: usize) -> Result<LigatureSubtable, Error> {
    let format = u16_at(b, at)?;
    if format != 1 {
        return Err(Error::Malformed(format!("LigatureSubst format {format}")));
    }
    let coverage = Coverage::parse(b, at + usize::from(u16_at(b, at + 2)?))?;
    let n = usize::from(u16_at(b, at + 4)?);
    let mut sets = Vec::with_capacity(n);
    for i in 0..n {
        let set = at + usize::from(u16_at(b, at + 6 + 2 * i)?);
        let count = usize::from(u16_at(b, set)?);
        let mut ligs = Vec::with_capacity(count);
        for j in 0..count {
            let lig = set + usize::from(u16_at(b, set + 2 + 2 * j)?);
            let lig_glyph = u16_at(b, lig)?;
            let comp_count = usize::from(u16_at(b, lig + 2)?);
            if comp_count == 0 {
                return Err(Error::Malformed("ligature with zero components".into()));
            }
            let mut rest = Vec::with_capacity(comp_count - 1);
            for k in 0..comp_count - 1 {
                rest.push(u16_at(b, lig + 4 + 2 * k)?);
            }
            ligs.push((lig_glyph, rest));
        }
        sets.push(ligs);
    }
    Ok(LigatureSubtable { coverage, sets })
}

//! Shared OpenType Layout structures: coverage tables, class definitions and
//! the FeatureList/LookupList walk used by both `GPOS` and `GSUB`.

use crate::Error;
use crate::reader::{u16_at, u32_at};

/// A parsed Coverage table: glyph id -> coverage index.
#[derive(Debug, Clone)]
pub enum Coverage {
    /// Format 1: sorted glyph ids; index is the position.
    Glyphs(Vec<u16>),
    /// Format 2: (start, end, start_coverage_index) ranges.
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
                if gid <= end { class } else { 0 }
            }
        }
    }
}

/// Lookup indices referenced by every FeatureRecord with `tag`, in the order
/// the LookupList orders them, deduplicated.
///
/// This walks the FeatureList directly rather than ScriptList/LangSys: the
/// engine has no script/language selection yet, so every script's `kern` or
/// `liga` feature contributes. That is documented as a simplification.
pub fn lookups_for_feature(b: &[u8], tag: &[u8; 4]) -> Result<Vec<u16>, Error> {
    let version_major = u16_at(b, 0)?;
    if version_major != 1 {
        return Err(Error::Unsupported(format!("layout table version {version_major}")));
    }
    let feature_list = usize::from(u16_at(b, 6)?);
    let n = usize::from(u16_at(b, feature_list)?);
    let mut out: Vec<u16> = Vec::new();
    for i in 0..n {
        let rec = feature_list + 2 + 6 * i;
        if &b[rec..rec + 4] != tag {
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

/// One lookup's type and subtable offsets (absolute into the table),
/// with Extension lookups (`ext_type`) already unwrapped.
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
            // Extension: format 1, extensionLookupType, extensionOffset (u32).
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

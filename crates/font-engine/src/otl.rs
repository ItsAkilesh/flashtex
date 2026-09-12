//! Shared OpenType Layout structures: coverage tables, class definitions and
//! the FeatureList/LookupList walk used by both `GPOS` and `GSUB`.

use crate::Error;
use crate::reader::{slice, u16_at, u32_at};

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
                if gid > end {
                    return None;
                }
                // `base` (startCoverageIndex) is an unvalidated font value;
                // a hostile or corrupt table can set it near `u16::MAX` so
                // this addition overflows. Treat overflow as "not covered"
                // rather than panicking (debug) or wrapping to a bogus, but
                // in-range, coverage index (release) that would then address
                // the wrong entry in whatever table this coverage indexes.
                base.checked_add(gid - start)
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

/// Lookup indices of feature `tag` for the default language system of the
/// `DFLT` script (falling back to `latn`, then the first script), in
/// LookupList order, deduplicated.
///
/// Going through ScriptList/LangSys matters: fonts such as Latin Modern
/// register language-specific `liga`/`kern` FeatureRecords (Turkish without
/// `fi`, Polish alternates, ...) and a naive union of every record with the
/// tag would apply them all. Only the default language system is selected;
/// there is no per-run language tag yet, and that is documented.
pub fn lookups_for_feature(b: &[u8], tag: &[u8; 4]) -> Result<Vec<u16>, Error> {
    let version_major = u16_at(b, 0)?;
    if version_major != 1 {
        return Err(Error::Unsupported(format!(
            "layout table version {version_major}"
        )));
    }
    let script_list = usize::from(u16_at(b, 4)?);
    let feature_list = usize::from(u16_at(b, 6)?);

    // Pick the script: DFLT, else latn, else the first one listed.
    let n_scripts = usize::from(u16_at(b, script_list)?);
    let mut chosen: Option<usize> = None;
    let mut first: Option<usize> = None;
    let mut latn: Option<usize> = None;
    for i in 0..n_scripts {
        let rec = script_list + 2 + 6 * i;
        let script_tag = slice(b, rec, 4)?;
        let off = script_list + usize::from(u16_at(b, rec + 4)?);
        if first.is_none() {
            first = Some(off);
        }
        match script_tag {
            b"DFLT" => chosen = Some(off),
            b"latn" => latn = Some(off),
            _ => {}
        }
    }
    let Some(script) = chosen.or(latn).or(first) else {
        return Ok(Vec::new());
    };
    // Default LangSys, else the first LangSysRecord.
    let default_off = usize::from(u16_at(b, script)?);
    let lang_sys = if default_off != 0 {
        script + default_off
    } else {
        let n = usize::from(u16_at(b, script + 2)?);
        if n == 0 {
            return Ok(Vec::new());
        }
        script + usize::from(u16_at(b, script + 4 + 4)?)
    };
    // LangSys: lookupOrderOffset, requiredFeatureIndex, featureIndexCount, [indices]
    let required = u16_at(b, lang_sys + 2)?;
    let n_feat = usize::from(u16_at(b, lang_sys + 4)?);
    let mut feature_indices: Vec<u16> = Vec::with_capacity(n_feat + 1);
    if required != 0xFFFF {
        feature_indices.push(required);
    }
    for i in 0..n_feat {
        feature_indices.push(u16_at(b, lang_sys + 6 + 2 * i)?);
    }

    let n_features = usize::from(u16_at(b, feature_list)?);
    let mut out: Vec<u16> = Vec::new();
    for fi in feature_indices {
        let fi = usize::from(fi);
        if fi >= n_features {
            return Err(Error::Malformed(format!(
                "feature index {fi} >= {n_features}"
            )));
        }
        let rec = feature_list + 2 + 6 * fi;
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
    let declared_type = u16_at(b, at)?;
    let mut lookup_type = declared_type;
    let n = usize::from(u16_at(b, at + 4)?);
    let mut subtables = Vec::with_capacity(n);
    for i in 0..n {
        let mut st = at + usize::from(u16_at(b, at + 6 + 2 * i)?);
        if declared_type == ext_type {
            // Extension: format 1, extensionLookupType, extensionOffset (u32),
            // relative to the extension subtable. Every subtable of one
            // extension lookup must name the same inner type.
            if u16_at(b, st)? != 1 {
                return Err(Error::Malformed("extension subtable format".into()));
            }
            let inner_type = u16_at(b, st + 2)?;
            if i > 0 && inner_type != lookup_type {
                return Err(Error::Malformed(
                    "extension subtables disagree on lookup type".into(),
                ));
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A Coverage format 2 range whose `startCoverageIndex` sits right at
    /// `u16::MAX` overflows `base + (gid - start)` for any `gid` past
    /// `start`: debug panicked ("attempt to add with overflow"), release
    /// silently wrapped to a small, wrong-but-in-range coverage index
    /// (`Some(0)`) that would then address the wrong entry in whatever
    /// table (PairSet, LigatureSet, MarkArray, ...) this coverage indexes.
    /// `index` must now report "not covered" instead, identically in both
    /// profiles.
    #[test]
    fn coverage_format2_range_index_overflow_is_not_covered_not_wrapped() {
        let mut b = vec![0u8; 10];
        b[0..2].copy_from_slice(&2u16.to_be_bytes());
        b[2..4].copy_from_slice(&1u16.to_be_bytes());
        b[4..6].copy_from_slice(&0u16.to_be_bytes()); // start
        b[6..8].copy_from_slice(&10u16.to_be_bytes()); // end
        b[8..10].copy_from_slice(&65535u16.to_be_bytes()); // base (u16::MAX)
        let cov = Coverage::parse(&b, 0).unwrap();
        // gid=0: base+0 = 65535, no overflow, still reported.
        assert_eq!(cov.index(0), Some(65535));
        // gid=1: base+1 would overflow u16 -> "not covered", not a wrapped
        // Some(0) and not a panic.
        assert_eq!(cov.index(1), None);
        // A glyph outside the range entirely is still "not covered".
        assert_eq!(cov.index(11), None);
    }
}

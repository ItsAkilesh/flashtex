//! `GSUB` ligatures: the `liga` feature's lookups of type 4 (LigatureSubst),
//! including type 7 Extension wrappers. Other lookup types under `liga`
//! (contextual, chained, single, multiple, alternate) are recorded as
//! unsupported; no other feature (`dlig`, `clig`, `calt`, `smcp`, ...) is
//! read at all.

use crate::Unsupported;
use crate::otl::{self, Coverage};
use crate::reader::u16_at;
use crate::{Error, GlyphId};

#[derive(Debug, Clone)]
struct LigatureSubtable {
    coverage: Coverage,
    /// Per coverage index: (ligature glyph, component glyphs after the first),
    /// in font order (the font lists longer ligatures first when it wants
    /// them preferred; we honour font order).
    sets: Vec<Vec<(u16, Vec<u16>)>>,
}

/// Ligature lookups in LookupList order. Each lookup is one shaping pass
/// over the whole run (OpenType semantics), so a font can build `ffi` as
/// `f`+`f` -> `ff` in one lookup and `ff`+`i` -> `ffi` in the next.
#[derive(Debug, Clone, Default)]
pub struct GsubLigatures {
    lookups: Vec<Vec<LigatureSubtable>>,
    pub unsupported: Vec<Unsupported>,
}

impl GsubLigatures {
    pub fn parse(gsub: &[u8]) -> Result<GsubLigatures, Error> {
        let mut out = GsubLigatures::default();
        for index in otl::lookups_for_feature(gsub, b"liga")? {
            let lk = otl::lookup(gsub, index, 7)?;
            if lk.lookup_type != 4 {
                out.unsupported.push(Unsupported {
                    table: "GSUB",
                    feature: "liga",
                    detail: format!(
                        "liga lookup {} is type {} (only LigatureSubst type 4 is applied)",
                        lk.index, lk.lookup_type
                    ),
                });
                continue;
            }
            let mut subtables = Vec::with_capacity(lk.subtables.len());
            for st in lk.subtables {
                subtables.push(parse_ligature_subtable(gsub, st)?);
            }
            out.lookups.push(subtables);
        }
        Ok(out)
    }

    pub fn is_empty(&self) -> bool {
        self.lookups.is_empty()
    }

    /// Number of passes (lookups).
    pub fn passes(&self) -> usize {
        self.lookups.len()
    }

    /// Result of running every pass over exactly `components`; `Some` only
    /// when they collapse to a single glyph.
    pub fn ligature(&self, components: &[GlyphId]) -> Option<GlyphId> {
        let mut run: Vec<GlyphId> = components.to_vec();
        for pass in 0..self.passes() {
            let mut i = 0;
            while i < run.len() {
                if let Some((lig, len)) = self.longest_in_pass(pass, &run[i..]) {
                    run.splice(i..i + len, [lig]);
                }
                i += 1;
            }
        }
        (run.len() == 1 && components.len() > 1).then(|| run[0])
    }

    /// Longest ligature of pass `pass` starting at `glyphs[0]`; returns
    /// (ligature, component count).
    pub fn longest_in_pass(&self, pass: usize, glyphs: &[GlyphId]) -> Option<(GlyphId, usize)> {
        let first = glyphs.first()?;
        let mut best: Option<(GlyphId, usize)> = None;
        for sub in self.lookups.get(pass)? {
            let Some(ci) = sub.coverage.index(first.0) else {
                continue;
            };
            for (lig, rest) in &sub.sets[usize::from(ci)] {
                let len = rest.len() + 1;
                if len <= glyphs.len()
                    && len >= 2
                    && rest.iter().zip(&glyphs[1..]).all(|(a, b)| *a == b.0)
                    && best.is_none_or(|(_, l)| len > l)
                {
                    best = Some((GlyphId(*lig), len));
                }
            }
            if best.is_some() {
                // First subtable covering the glyph decides within a lookup.
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
    // The Coverage table assigns every covered glyph an index that directly
    // addresses `sets` (one LigatureSet per covered glyph, in coverage
    // order). A format 2 Coverage's `startCoverageIndex` is an unvalidated
    // font value with no relationship to `n` enforced by `Coverage::parse`
    // (which parses GPOS coverage the same way, against a differently sized
    // array); a hostile or corrupt font can set it past `n`, or list more
    // glyphs than `n` in a format 1 Coverage. Unchecked, `longest_in_pass`
    // below would then index `sets` out of bounds. Reject that here, at
    // parse time, rather than clamping the index (which would silently
    // substitute the wrong LigatureSet) or panicking during shaping.
    let covered = match &coverage {
        Coverage::Glyphs(v) => v.len(),
        Coverage::Ranges(v) => v
            .iter()
            .map(|&(start, end, base)| {
                usize::from(base) + usize::from(end.saturating_sub(start)) + 1
            })
            .max()
            .unwrap_or(0),
    };
    if covered > n {
        return Err(Error::Malformed(format!(
            "LigatureSubst coverage reaches index {} but only {n} ligature sets are present",
            covered - 1
        )));
    }
    Ok(LigatureSubtable { coverage, sets })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A LigatureSubst format 1 subtable whose Coverage is format 2 with a
    /// single range (glyphs 10..=12) and `startCoverageIndex = 5`, while the
    /// subtable declares only one LigatureSet (index 0). `Coverage::index`
    /// happily reports covered glyphs at indices 5, 6, 7 (`base + offset`,
    /// no overflow, so `otl`'s overflow guard does not fire) — indices that
    /// do not exist in `sets`. Before the bounds check above, parsing this
    /// subtable succeeded and the out-of-range index only surfaced later, as
    /// an index-out-of-bounds panic at `sub.sets[usize::from(ci)]` in
    /// `longest_in_pass` the first time shaping looked up glyph 10, 11 or 12
    /// — i.e. ordinary text shaping panicked on a font carrying this table,
    /// via the public `shape::shape()`.
    fn out_of_range_coverage_bytes() -> Vec<u8> {
        let mut b = vec![0u8; 28];
        b[0..2].copy_from_slice(&1u16.to_be_bytes()); // format
        b[2..4].copy_from_slice(&8u16.to_be_bytes()); // coverageOffset
        b[4..6].copy_from_slice(&1u16.to_be_bytes()); // ligatureSetCount
        b[6..8].copy_from_slice(&18u16.to_be_bytes()); // ligatureSetOffsets[0]

        // Coverage (format 2) @ 8: one range, glyphs 10..=12, base 5.
        b[8..10].copy_from_slice(&2u16.to_be_bytes());
        b[10..12].copy_from_slice(&1u16.to_be_bytes());
        b[12..14].copy_from_slice(&10u16.to_be_bytes()); // startGlyphID
        b[14..16].copy_from_slice(&12u16.to_be_bytes()); // endGlyphID
        b[16..18].copy_from_slice(&5u16.to_be_bytes()); // startCoverageIndex

        // LigatureSet[0] @ 18: one ligature.
        b[18..20].copy_from_slice(&1u16.to_be_bytes()); // ligatureCount
        b[20..22].copy_from_slice(&4u16.to_be_bytes()); // ligatureOffsets[0] (-> 22)

        // Ligature @ 22: glyph 999, 2 components, second component glyph 3.
        b[22..24].copy_from_slice(&999u16.to_be_bytes());
        b[24..26].copy_from_slice(&2u16.to_be_bytes());
        b[26..28].copy_from_slice(&3u16.to_be_bytes());
        b
    }

    #[test]
    fn ligature_coverage_index_past_ligature_set_count_is_rejected_not_panicking() {
        let bytes = out_of_range_coverage_bytes();
        // Sanity check: the Coverage table alone parses fine (it is
        // internally well-formed) and does report the out-of-range index,
        // confirming this exercises the same overflow-free path as the
        // panic, not `otl`'s separate checked-add overflow guard.
        let coverage = Coverage::parse(&bytes, 8).unwrap();
        assert_eq!(coverage.index(10), Some(5));

        let err = parse_ligature_subtable(&bytes, 0)
            .expect_err("coverage index 5 exceeds the single declared ligature set");
        assert!(
            matches!(err, Error::Malformed(_)),
            "expected Error::Malformed, got {err:?}"
        );
    }
}

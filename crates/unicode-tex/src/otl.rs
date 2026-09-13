//! Minimal OpenType Layout (GSUB/GPOS) with explicit script, language and
//! feature selection.
//!
//! `crates/font-engine` applies `liga` and `kern` under a fixed script
//! choice (`DFLT`, else `latn`). XeTeX/LuaTeX fonts need arbitrary features
//! (`smcp`, `onum`, `dlig`, `tnum`, ...) under the script fontspec passes
//! (`latn`), so this module reads the raw tables itself. Implemented:
//!
//! * GSUB 1 (single, formats 1–2), 4 (ligature), 7 (extension);
//! * GPOS 1 (single adjustment, formats 1–2), 2 (pair, formats 1–2), 9 (extension);
//! * lookup flags IgnoreBaseGlyphs/IgnoreLigatures/IgnoreMarks via GDEF classes;
//! * required features of the selected language system.
//!
//! Every other lookup type met under an active feature is reported in
//! [`Applied::skipped`] (never silently ignored): multiple/alternate/
//! contextual/chaining/reverse substitution and cursive/mark/contextual
//! positioning. Device tables and placements are ignored (advances only).

use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub [u8; 4]);

impl Tag {
    /// Up to four ASCII bytes, space padded.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Tag {
        let mut b = [b' '; 4];
        for (i, c) in s.bytes().take(4).enumerate() {
            b[i] = c;
        }
        Tag(b)
    }
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap_or("????")
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", self.as_str())
    }
}

fn u16_at(b: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*b.get(o)?, *b.get(o + 1)?]))
}
fn i16_at(b: &[u8], o: usize) -> Option<i16> {
    u16_at(b, o).map(|v| v as i16)
}
fn u32_at(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *b.get(o)?,
        *b.get(o + 1)?,
        *b.get(o + 2)?,
        *b.get(o + 3)?,
    ]))
}
fn tag_at(b: &[u8], o: usize) -> Option<Tag> {
    Some(Tag(b.get(o..o + 4)?.try_into().ok()?))
}

/// Coverage table lookup: coverage index of `gid`.
fn coverage(b: &[u8], at: usize, gid: u16) -> Option<u16> {
    match u16_at(b, at)? {
        1 => {
            let n = usize::from(u16_at(b, at + 2)?);
            let (mut lo, mut hi) = (0usize, n);
            while lo < hi {
                let mid = (lo + hi) / 2;
                let g = u16_at(b, at + 4 + 2 * mid)?;
                match g.cmp(&gid) {
                    std::cmp::Ordering::Equal => return Some(mid as u16),
                    std::cmp::Ordering::Less => lo = mid + 1,
                    std::cmp::Ordering::Greater => hi = mid,
                }
            }
            None
        }
        2 => {
            let n = usize::from(u16_at(b, at + 2)?);
            let (mut lo, mut hi) = (0usize, n);
            while lo < hi {
                let mid = (lo + hi) / 2;
                let r = at + 4 + 6 * mid;
                let (s, e) = (u16_at(b, r)?, u16_at(b, r + 2)?);
                if gid < s {
                    hi = mid;
                } else if gid > e {
                    lo = mid + 1;
                } else {
                    return Some(u16_at(b, r + 4)? + (gid - s));
                }
            }
            None
        }
        _ => None,
    }
}

/// ClassDef lookup (class 0 when absent).
fn class_def(b: &[u8], at: usize, gid: u16) -> u16 {
    let f = || -> Option<u16> {
        match u16_at(b, at)? {
            1 => {
                let start = u16_at(b, at + 2)?;
                let n = u16_at(b, at + 4)?;
                if gid >= start && gid - start < n {
                    u16_at(b, at + 6 + 2 * usize::from(gid - start))
                } else {
                    Some(0)
                }
            }
            2 => {
                let n = usize::from(u16_at(b, at + 2)?);
                let (mut lo, mut hi) = (0usize, n);
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    let r = at + 4 + 6 * mid;
                    let (s, e) = (u16_at(b, r)?, u16_at(b, r + 2)?);
                    if gid < s {
                        hi = mid;
                    } else if gid > e {
                        lo = mid + 1;
                    } else {
                        return u16_at(b, r + 4);
                    }
                }
                Some(0)
            }
            _ => Some(0),
        }
    };
    f().unwrap_or(0)
}

/// GDEF glyph classes: 1 base, 2 ligature, 3 mark, 4 component.
#[derive(Clone, Copy)]
pub struct Gdef<'a> {
    data: &'a [u8],
    class_def: Option<usize>,
}

impl<'a> Gdef<'a> {
    pub fn new(data: Option<&'a [u8]>) -> Gdef<'a> {
        let data = data.unwrap_or(&[]);
        let class_def = u16_at(data, 4).filter(|&o| o != 0).map(usize::from);
        Gdef { data, class_def }
    }
    pub fn glyph_class(&self, gid: u16) -> u16 {
        self.class_def
            .map(|at| class_def(self.data, at, gid))
            .unwrap_or(0)
    }
    pub fn is_mark(&self, gid: u16) -> bool {
        self.glyph_class(gid) == 3
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    Gsub,
    Gpos,
}

/// A GSUB or GPOS table.
#[derive(Clone, Copy)]
pub struct Layout<'a> {
    data: &'a [u8],
    kind: TableKind,
}

/// One lookup to run: index in LookupList and the feature that selected it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectedLookup {
    pub index: u16,
    pub feature: Tag,
}

impl<'a> Layout<'a> {
    pub fn new(data: &'a [u8], kind: TableKind) -> Option<Layout<'a>> {
        if data.len() < 10 {
            return None;
        }
        Some(Layout { data, kind })
    }

    fn script_list(&self) -> usize {
        usize::from(u16_at(self.data, 4).unwrap_or(0))
    }
    fn feature_list(&self) -> usize {
        usize::from(u16_at(self.data, 6).unwrap_or(0))
    }
    fn lookup_list(&self) -> usize {
        usize::from(u16_at(self.data, 8).unwrap_or(0))
    }

    /// Script tags present.
    pub fn scripts(&self) -> Vec<Tag> {
        let sl = self.script_list();
        let n = u16_at(self.data, sl).unwrap_or(0);
        (0..usize::from(n))
            .filter_map(|i| tag_at(self.data, sl + 2 + 6 * i))
            .collect()
    }

    /// Language-system table for `script` (falling back to `DFLT`, `dflt`,
    /// `latn` like HarfBuzz) and `lang` (falling back to the default
    /// LangSys). Returns the script actually used and the LangSys offset.
    pub fn lang_sys(&self, script: Tag, lang: Option<Tag>) -> Option<(Tag, usize)> {
        let sl = self.script_list();
        let n = usize::from(u16_at(self.data, sl)?);
        let find = |t: Tag| -> Option<usize> {
            (0..n).find_map(|i| {
                (tag_at(self.data, sl + 2 + 6 * i)? == t)
                    .then(|| sl + usize::from(u16_at(self.data, sl + 6 + 6 * i).unwrap_or(0)))
            })
        };
        let (used, script_at) = [
            script,
            Tag::from_str("DFLT"),
            Tag::from_str("dflt"),
            Tag::from_str("latn"),
        ]
        .into_iter()
        .find_map(|t| find(t).map(|o| (t, o)))?;
        if let Some(l) = lang {
            let ln = usize::from(u16_at(self.data, script_at + 2)?);
            for i in 0..ln {
                let r = script_at + 4 + 6 * i;
                if tag_at(self.data, r)? == l {
                    return Some((used, script_at + usize::from(u16_at(self.data, r + 4)?)));
                }
            }
        }
        let default = u16_at(self.data, script_at)?;
        if default != 0 {
            return Some((used, script_at + usize::from(default)));
        }
        None
    }

    /// Lookups for `features` (plus the LangSys required feature), sorted by
    /// lookup index and deduplicated, as HarfBuzz and luaotfload apply them.
    pub fn select(
        &self,
        script: Tag,
        lang: Option<Tag>,
        features: &BTreeSet<Tag>,
    ) -> (Option<Tag>, Vec<SelectedLookup>) {
        let Some((used, ls)) = self.lang_sys(script, lang) else {
            return (None, Vec::new());
        };
        let fl = self.feature_list();
        let mut out: Vec<SelectedLookup> = Vec::new();
        let mut add_feature = |fi: u16, required: bool| {
            let rec = fl + 2 + 6 * usize::from(fi);
            let (Some(t), Some(off)) = (tag_at(self.data, rec), u16_at(self.data, rec + 4)) else {
                return;
            };
            if !required && !features.contains(&t) {
                return;
            }
            let ft = fl + usize::from(off);
            let n = u16_at(self.data, ft + 2).unwrap_or(0);
            for k in 0..usize::from(n) {
                if let Some(li) = u16_at(self.data, ft + 4 + 2 * k) {
                    out.push(SelectedLookup {
                        index: li,
                        feature: t,
                    });
                }
            }
        };
        let required = u16_at(self.data, ls + 2).unwrap_or(0xFFFF);
        if required != 0xFFFF {
            add_feature(required, true);
        }
        let n = u16_at(self.data, ls + 4).unwrap_or(0);
        for i in 0..usize::from(n) {
            if let Some(fi) = u16_at(self.data, ls + 6 + 2 * i) {
                add_feature(fi, false);
            }
        }
        out.sort();
        out.dedup_by_key(|l| l.index);
        (Some(used), out)
    }

    /// Whether the selected LangSys has `feature`.
    pub fn has_feature(&self, script: Tag, lang: Option<Tag>, feature: Tag) -> bool {
        let set: BTreeSet<Tag> = [feature].into_iter().collect();
        !self
            .select(script, lang, &set)
            .1
            .iter()
            .all(|l| l.feature != feature)
    }

    /// (lookup type, flag, subtable offsets) with extension lookups resolved.
    fn lookup(&self, index: u16) -> Option<(u16, u16, Vec<usize>)> {
        let ll = self.lookup_list();
        let n = u16_at(self.data, ll)?;
        if index >= n {
            return None;
        }
        let lt = ll + usize::from(u16_at(self.data, ll + 2 + 2 * usize::from(index))?);
        let mut ty = u16_at(self.data, lt)?;
        let flag = u16_at(self.data, lt + 2)?;
        let count = usize::from(u16_at(self.data, lt + 4)?);
        let ext = match self.kind {
            TableKind::Gsub => 7,
            TableKind::Gpos => 9,
        };
        let mut subs = Vec::with_capacity(count);
        for i in 0..count {
            let st = lt + usize::from(u16_at(self.data, lt + 6 + 2 * i)?);
            if ty == ext {
                ty = u16_at(self.data, st + 2)?;
                subs.push(st + u32_at(self.data, st + 4)? as usize);
            } else {
                subs.push(st);
            }
        }
        Some((ty, flag, subs))
    }
}

/// A glyph in the shaping buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufGlyph {
    pub gid: u16,
    /// Byte range of the ORIGINAL text this glyph represents.
    pub source: std::ops::Range<usize>,
    /// Advance in font units (hmtx + positioning).
    pub advance: i32,
    pub is_mark: bool,
}

/// What a lookup pass did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Applied {
    pub lookups_run: Vec<SelectedLookup>,
    /// `(feature, lookup index, lookup type)` not implemented here.
    pub skipped: Vec<(Tag, u16, u16)>,
}

fn skip(flag: u16, g: &BufGlyph, gdef: &Gdef) -> bool {
    let class = gdef.glyph_class(g.gid);
    (flag & 0x0008 != 0 && (class == 3 || g.is_mark))
        || (flag & 0x0002 != 0 && class == 1)
        || (flag & 0x0004 != 0 && class == 2)
}

impl<'a> Layout<'a> {
    /// Runs GSUB lookups over `buf`. `advance` recomputes an advance for a
    /// substituted glyph (hmtx).
    pub fn apply_gsub(
        &self,
        lookups: &[SelectedLookup],
        buf: &mut Vec<BufGlyph>,
        gdef: &Gdef,
        advance: &dyn Fn(u16) -> i32,
        applied: &mut Applied,
    ) {
        for sel in lookups {
            let Some((ty, flag, subs)) = self.lookup(sel.index) else {
                continue;
            };
            match ty {
                1 | 4 => {}
                other => {
                    applied.skipped.push((sel.feature, sel.index, other));
                    continue;
                }
            }
            applied.lookups_run.push(*sel);
            let mut i = 0;
            while i < buf.len() {
                if skip(flag, &buf[i], gdef) {
                    i += 1;
                    continue;
                }
                let mut done = false;
                for &st in &subs {
                    let ok = match ty {
                        1 => self.single_subst(st, &mut buf[i], advance),
                        _ => self.ligature_subst(st, flag, buf, i, gdef, advance),
                    };
                    if ok == Some(true) {
                        done = true;
                        break;
                    }
                }
                let _ = done;
                i += 1;
            }
        }
    }

    fn single_subst(
        &self,
        st: usize,
        g: &mut BufGlyph,
        advance: &dyn Fn(u16) -> i32,
    ) -> Option<bool> {
        let b = self.data;
        let fmt = u16_at(b, st)?;
        let cov = st + usize::from(u16_at(b, st + 2)?);
        let Some(ci) = coverage(b, cov, g.gid) else {
            return Some(false);
        };
        let new = match fmt {
            1 => (g.gid as i32 + i32::from(i16_at(b, st + 4)?)).rem_euclid(65536) as u16,
            2 => u16_at(b, st + 6 + 2 * usize::from(ci))?,
            _ => return Some(false),
        };
        g.gid = new;
        g.advance = advance(new);
        Some(true)
    }

    fn ligature_subst(
        &self,
        st: usize,
        flag: u16,
        buf: &mut Vec<BufGlyph>,
        i: usize,
        gdef: &Gdef,
        advance: &dyn Fn(u16) -> i32,
    ) -> Option<bool> {
        let b = self.data;
        let cov = st + usize::from(u16_at(b, st + 2)?);
        let Some(ci) = coverage(b, cov, buf[i].gid) else {
            return Some(false);
        };
        let set = st + usize::from(u16_at(b, st + 6 + 2 * usize::from(ci))?);
        let n = usize::from(u16_at(b, set)?);
        for k in 0..n {
            let lig = set + usize::from(u16_at(b, set + 2 + 2 * k)?);
            let glyph = u16_at(b, lig)?;
            let comps = usize::from(u16_at(b, lig + 2)?);
            // Match components after buf[i], skipping ignorable glyphs.
            let mut positions = Vec::with_capacity(comps);
            let mut j = i + 1;
            let mut ok = true;
            for c in 1..comps {
                while j < buf.len() && skip(flag, &buf[j], gdef) {
                    j += 1;
                }
                if j >= buf.len() || buf[j].gid != u16_at(b, lig + 4 + 2 * (c - 1))? {
                    ok = false;
                    break;
                }
                positions.push(j);
                j += 1;
            }
            if !ok {
                continue;
            }
            let end = positions
                .last()
                .map(|&p| buf[p].source.end)
                .unwrap_or(buf[i].source.end);
            let start = buf[i].source.start;
            buf[i].gid = glyph;
            buf[i].advance = advance(glyph);
            buf[i].source = start.min(end)..end.max(buf[i].source.end);
            for &p in positions.iter().rev() {
                buf.remove(p);
            }
            return Some(true);
        }
        Some(false)
    }

    /// Runs GPOS lookups (advance adjustments only).
    pub fn apply_gpos(
        &self,
        lookups: &[SelectedLookup],
        buf: &mut [BufGlyph],
        gdef: &Gdef,
        applied: &mut Applied,
    ) {
        for sel in lookups {
            let Some((ty, flag, subs)) = self.lookup(sel.index) else {
                continue;
            };
            match ty {
                1 | 2 => {}
                other => {
                    applied.skipped.push((sel.feature, sel.index, other));
                    continue;
                }
            }
            applied.lookups_run.push(*sel);
            let mut i = 0;
            while i < buf.len() {
                if skip(flag, &buf[i], gdef) {
                    i += 1;
                    continue;
                }
                let mut advance_by = 1;
                for &st in &subs {
                    if ty == 1 {
                        if let Some(dx) = self.single_pos(st, buf[i].gid) {
                            buf[i].advance += dx;
                            break;
                        }
                    } else {
                        let mut j = i + 1;
                        while j < buf.len() && skip(flag, &buf[j], gdef) {
                            j += 1;
                        }
                        if j >= buf.len() {
                            break;
                        }
                        if let Some((d1, d2, fmt2)) = self.pair_pos(st, buf[i].gid, buf[j].gid) {
                            buf[i].advance += d1;
                            buf[j].advance += d2;
                            if fmt2 != 0 {
                                advance_by = j - i + 1;
                            }
                            break;
                        }
                    }
                }
                i += advance_by;
            }
        }
    }

    fn value_x_advance(&self, at: usize, format: u16) -> Option<i32> {
        if format & 0x0004 == 0 {
            return Some(0);
        }
        let off = 2 * (format & 0x0003).count_ones() as usize;
        i16_at(self.data, at + off).map(i32::from)
    }

    fn single_pos(&self, st: usize, gid: u16) -> Option<i32> {
        let b = self.data;
        let fmt = u16_at(b, st)?;
        let ci = coverage(b, st + usize::from(u16_at(b, st + 2)?), gid)?;
        let vf = u16_at(b, st + 4)?;
        match fmt {
            1 => self.value_x_advance(st + 6, vf),
            2 => {
                let size = 2 * vf.count_ones() as usize;
                self.value_x_advance(st + 8 + size * usize::from(ci), vf)
            }
            _ => None,
        }
    }

    /// (first advance delta, second advance delta, second value format).
    fn pair_pos(&self, st: usize, first: u16, second: u16) -> Option<(i32, i32, u16)> {
        let b = self.data;
        let fmt = u16_at(b, st)?;
        let ci = coverage(b, st + usize::from(u16_at(b, st + 2)?), first)?;
        let vf1 = u16_at(b, st + 4)?;
        let vf2 = u16_at(b, st + 6)?;
        let s1 = 2 * vf1.count_ones() as usize;
        let s2 = 2 * vf2.count_ones() as usize;
        match fmt {
            1 => {
                let set = st + usize::from(u16_at(b, st + 10 + 2 * usize::from(ci))?);
                let n = usize::from(u16_at(b, set)?);
                let rec = 2 + s1 + s2;
                let (mut lo, mut hi) = (0usize, n);
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    let r = set + 2 + rec * mid;
                    let g = u16_at(b, r)?;
                    match g.cmp(&second) {
                        std::cmp::Ordering::Equal => {
                            return Some((
                                self.value_x_advance(r + 2, vf1)?,
                                self.value_x_advance(r + 2 + s1, vf2)?,
                                vf2,
                            ));
                        }
                        std::cmp::Ordering::Less => lo = mid + 1,
                        std::cmp::Ordering::Greater => hi = mid,
                    }
                }
                None
            }
            2 => {
                let cd1 = st + usize::from(u16_at(b, st + 8)?);
                let cd2 = st + usize::from(u16_at(b, st + 10)?);
                let c1n = u16_at(b, st + 12)?;
                let c2n = u16_at(b, st + 14)?;
                let c1 = class_def(b, cd1, first);
                let c2 = class_def(b, cd2, second);
                if c1 >= c1n || c2 >= c2n {
                    return None;
                }
                let r =
                    st + 16 + (usize::from(c1) * usize::from(c2n) + usize::from(c2)) * (s1 + s2);
                let d1 = self.value_x_advance(r, vf1)?;
                let d2 = self.value_x_advance(r + s1, vf2)?;
                if d1 == 0 && d2 == 0 && vf2 == 0 {
                    // Class-0 zero records do not stop other subtables.
                    return None;
                }
                Some((d1, d2, vf2))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags() {
        assert_eq!(Tag::from_str("TRK").as_str(), "TRK ");
        assert_eq!(format!("{:?}", Tag::from_str("liga")), "'liga'");
    }

    #[test]
    fn coverage_formats() {
        // format 1: glyphs 3, 7, 9
        let c1 = [0, 1, 0, 3, 0, 3, 0, 7, 0, 9];
        assert_eq!(coverage(&c1, 0, 7), Some(1));
        assert_eq!(coverage(&c1, 0, 8), None);
        // format 2: range 10..=20 starting at index 5
        let c2 = [0, 2, 0, 1, 0, 10, 0, 20, 0, 5];
        assert_eq!(coverage(&c2, 0, 12), Some(7));
        assert_eq!(coverage(&c2, 0, 21), None);
    }
}

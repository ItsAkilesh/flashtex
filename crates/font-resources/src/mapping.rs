use crate::{invalid, u16_at, u32_at, Error, FontResource, Result};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorizontalMetrics {
    pub advance_width: u16,
    pub left_side_bearing: i16,
}
impl FontResource {
    /// Original font GID; None denotes .notdef. Does not perform shaping.
    pub fn glyph_id(&self, character: char) -> Result<Option<u16>> {
        lookup(
            self.table(b"cmap")
                .ok_or_else(|| Error::UnsupportedFont("cmap missing".into()))?,
            character as u32,
            self.descriptor().glyph_count,
        )
    }
    pub fn horizontal_metrics(&self, gid: u16) -> Result<HorizontalMetrics> {
        if gid as u32 >= self.descriptor().glyph_count {
            return Err(invalid("glyph ID outside font"));
        }
        let hhea = self.table(b"hhea").ok_or_else(|| invalid("hhea missing"))?;
        let hmtx = self.table(b"hmtx").ok_or_else(|| invalid("hmtx missing"))?;
        let count = u16_at(hhea, 34)? as usize;
        let index = gid as usize;
        let advance_width = u16_at(hmtx, index.min(count - 1) * 4)?;
        let bearing_offset = if index < count {
            index * 4 + 2
        } else {
            count * 4 + (index - count) * 2
        };
        Ok(HorizontalMetrics {
            advance_width,
            left_side_bearing: u16_at(hmtx, bearing_offset)? as i16,
        })
    }
}
fn lookup(cmap: &[u8], cp: u32, glyphs: u32) -> Result<Option<u16>> {
    if u16_at(cmap, 0)? != 0 {
        return Err(invalid("cmap version"));
    }
    let count = u16_at(cmap, 2)? as usize;
    if count > 4096 || 4 + count * 8 > cmap.len() {
        return Err(invalid("cmap directory bounds"));
    }
    let mut chosen = None;
    for i in 0..count {
        let at = 4 + i * 8;
        let platform = u16_at(cmap, at)?;
        let encoding = u16_at(cmap, at + 2)?;
        if platform != 0 && !(platform == 3 && (encoding == 1 || encoding == 10)) {
            continue;
        }
        let offset = u32_at(cmap, at + 4)? as usize;
        if offset < 4 + count * 8 {
            return Err(invalid("cmap overlaps directory"));
        }
        let data = cmap
            .get(offset..)
            .ok_or_else(|| invalid("cmap subtable offset"))?;
        let format = u16_at(data, 0)?;
        let priority = match format {
            12 => 2,
            4 => 1,
            _ => continue,
        };
        if chosen.as_ref().is_none_or(|(p, _)| priority > *p) {
            chosen = Some((priority, data));
        }
    }
    let (_, data) =
        chosen.ok_or_else(|| Error::UnsupportedFont("Unicode cmap format 4/12 required".into()))?;
    let gid = if u16_at(data, 0)? == 12 {
        format12(data, cp, glyphs)?
    } else {
        format4(data, cp, glyphs)?
    };
    if gid >= glyphs {
        return Err(invalid("cmap glyph ID outside font"));
    }
    Ok(if gid == 0 { None } else { Some(gid as u16) })
}
fn format12(data: &[u8], cp: u32, glyphs: u32) -> Result<u32> {
    if u16_at(data, 2)? != 0 {
        return Err(invalid("cmap12 reserved"));
    }
    let length = u32_at(data, 4)? as usize;
    let data = data.get(..length).ok_or_else(|| invalid("cmap12 length"))?;
    let count = u32_at(data, 12)? as usize;
    if count > 1_114_112 || 16usize.checked_add(count * 12) != Some(length) {
        return Err(invalid("cmap12 group bounds"));
    }
    let mut previous = None;
    let mut found = 0;
    for i in 0..count {
        let at = 16 + i * 12;
        let start = u32_at(data, at)?;
        let end = u32_at(data, at + 4)?;
        let gid = u32_at(data, at + 8)?;
        if start > end
            || end > 0x10ffff
            || previous.is_some_and(|p| start <= p)
            || gid.checked_add(end - start).is_none_or(|g| g >= glyphs)
        {
            return Err(invalid("cmap12 invalid group"));
        }
        if cp >= start && cp <= end {
            found = gid + cp - start;
        }
        previous = Some(end);
    }
    Ok(found)
}
fn format4(data: &[u8], cp: u32, glyphs: u32) -> Result<u32> {
    let length = u16_at(data, 2)? as usize;
    let data = data.get(..length).ok_or_else(|| invalid("cmap4 length"))?;
    let twice = u16_at(data, 6)? as usize;
    if twice == 0 || !twice.is_multiple_of(2) {
        return Err(invalid("cmap4 segment count"));
    }
    let n = twice / 2;
    let starts = 16 + n * 2;
    let deltas = starts + n * 2;
    let ranges = deltas + n * 2;
    if ranges + n * 2 > length || u16_at(data, 14 + n * 2)? != 0 {
        return Err(invalid("cmap4 arrays"));
    }
    let mut previous = None;
    let mut found = 0;
    for i in 0..n {
        let end = u16_at(data, 14 + i * 2)? as u32;
        let start = u16_at(data, starts + i * 2)? as u32;
        let delta = u16_at(data, deltas + i * 2)?;
        let range = u16_at(data, ranges + i * 2)? as usize;
        if start > end || previous.is_some_and(|p| start <= p) || !range.is_multiple_of(2) {
            return Err(invalid("cmap4 ordering/range"));
        }
        for code in start..=end {
            let raw = if range == 0 {
                (code as u16).wrapping_add(delta)
            } else {
                let at = ranges + i * 2 + range + (code - start) as usize * 2;
                if at < ranges + n * 2 {
                    return Err(invalid("cmap4 glyph array overlaps segments"));
                }
                let g = u16_at(data, at)?;
                if g == 0 {
                    0
                } else {
                    g.wrapping_add(delta)
                }
            } as u32;
            if raw >= glyphs {
                return Err(invalid("cmap4 glyph ID outside font"));
            }
            if code == cp {
                found = raw;
            }
        }
        previous = Some(end);
    }
    if previous != Some(65535) {
        return Err(invalid("cmap4 terminal segment missing"));
    }
    Ok(found)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn wrap(data: Vec<u8>) -> Vec<u8> {
        let mut c = vec![0, 0, 0, 1, 0, 3, 0, 10, 0, 0, 0, 12];
        c.extend(data);
        c
    }
    fn cmap12() -> Vec<u8> {
        let mut d = vec![0, 12, 0, 0];
        for n in [28u32, 0, 1, 0x1f600, 0x1f601, 1] {
            d.extend(n.to_be_bytes());
        }
        wrap(d)
    }
    fn cmap4() -> Vec<u8> {
        let words = [
            4u16, 32, 0, 4, 4, 1, 0, 65, 65535, 0, 65, 65535, 65472, 1, 0, 0,
        ];
        wrap(words.into_iter().flat_map(u16::to_be_bytes).collect())
    }
    #[test]
    fn format4_indirect_glyph_array_applies_delta_except_zero() {
        let mut c = cmap4();
        c[14..16].copy_from_slice(&34u16.to_be_bytes());
        c[36..38].copy_from_slice(&1u16.to_be_bytes());
        c[40..42].copy_from_slice(&4u16.to_be_bytes());
        c.extend(1u16.to_be_bytes());
        assert_eq!(lookup(&c, 65, 4).unwrap(), Some(2));
        c[44..46].copy_from_slice(&0u16.to_be_bytes());
        assert_eq!(lookup(&c, 65, 4).unwrap(), None);
    }
    #[test]
    fn unicode12_original_gids() {
        let c = cmap12();
        assert_eq!(lookup(&c, 0x1f601, 3).unwrap(), Some(2));
        assert_eq!(lookup(&c, 65, 3).unwrap(), None);
        assert!(lookup(&c, 0x1f601, 2).is_err());
    }
    #[test]
    fn bmp4_and_supplementary_missing() {
        let c = cmap4();
        assert_eq!(lookup(&c, 65, 3).unwrap(), Some(1));
        assert_eq!(lookup(&c, 0x1f600, 3).unwrap(), None);
    }
    #[test]
    fn all_truncations_rejected() {
        for c in [cmap4(), cmap12()] {
            for i in 0..c.len() {
                assert!(lookup(&c[..i], 65, 3).is_err(), "{i}");
            }
        }
    }
    #[test]
    fn adversarial_ranges_and_lengths() {
        let mut c = cmap4();
        c[40..42].copy_from_slice(&2u16.to_be_bytes());
        assert!(lookup(&c, 65, 3).is_err());
        let mut c = cmap12();
        c[32..36].copy_from_slice(&0x110000u32.to_be_bytes());
        assert!(lookup(&c, 65, 3).is_err());
    }
}

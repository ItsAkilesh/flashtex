//! Original bounded TFM decoding; 8-bit TeX encoding is not Unicode or a TrueType GID map.
use crate::{invalid, u16_at, u32_at, Result};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixWord(pub i32);
impl FixWord {
    /// Exact product in TeX points: numerator / 2^40. No rounding to scaled points.
    pub fn at_design_size(self, design: FixWord) -> i64 {
        self.0 as i64 * design.0 as i64
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterMetrics {
    pub width: FixWord,
    pub height: FixWord,
    pub depth: FixWord,
    pub italic: FixWord,
    pub tag: u8,
    pub remainder: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairAction {
    Kern(FixWord),
    Ligature {
        replacement: u8,
        retain_left: bool,
        retain_right: bool,
        advance: u8,
    },
}
#[derive(Debug, Clone)]
pub struct Tfm {
    pub checksum: u32,
    pub design_size: FixWord,
    pub source_sha256: String,
    chars: Vec<Option<CharacterMetrics>>,
    program: Vec<[u8; 4]>,
    kerns: Vec<FixWord>,
    parameters: Vec<FixWord>,
    pub boundary_character: Option<u8>,
    pub left_boundary_program: Option<usize>,
}
impl Tfm {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut n = [0usize; 12];
        for (i, n) in n.iter_mut().enumerate() {
            *n = u16_at(bytes, i * 2)? as usize;
            if *n >= 32768 {
                return Err(invalid("TFM length high bit"));
            }
        }
        let [lf, lh, bc, ec, nw, nh, nd, ni, nl, nk, ne, np] = n;
        if lh < 2
            || ec > 255
            || bc > ec + 1
            || nw == 0
            || nh == 0
            || nd == 0
            || ni == 0
            || nw > 256
            || nh > 16
            || nd > 16
            || ni > 64
            || ne > 256
        {
            return Err(invalid("TFM header counts"));
        }
        let count = ec + 1 - bc;
        if lf != 6 + lh + count + nw + nh + nd + ni + nl + nk + ne + np || bytes.len() != lf * 4 {
            return Err(invalid("TFM file/table length mismatch"));
        }
        let checksum = u32_at(bytes, 24)?;
        let design_size = FixWord(u32_at(bytes, 28)? as i32);
        if design_size.0 < 1 << 20 {
            return Err(invalid("TFM design size below one point"));
        }
        let mut at = (6 + lh) * 4;
        let infos = bytes[at..at + count * 4].as_chunks::<4>().0.to_vec();
        at += count * 4;
        let mut fixes = |count: usize, slant: bool| -> Result<Vec<FixWord>> {
            let mut out = Vec::with_capacity(count);
            for i in 0..count {
                let v = u32_at(bytes, at)? as i32;
                at += 4;
                if !(slant && i == 0) && !(-16777216..16777216).contains(&v) {
                    return Err(invalid("TFM dimension outside fix_word metric range"));
                }
                out.push(FixWord(v));
            }
            Ok(out)
        };
        let widths = fixes(nw, false)?;
        let heights = fixes(nh, false)?;
        let depths = fixes(nd, false)?;
        let italics = fixes(ni, false)?;
        if [widths[0], heights[0], depths[0], italics[0]]
            .iter()
            .any(|v| v.0 != 0)
        {
            return Err(invalid("TFM metric zero entry"));
        }
        let program = bytes[at..at + nl * 4].as_chunks::<4>().0.to_vec();
        at += nl * 4;
        let mut kerns = Vec::new();
        for _ in 0..nk {
            let v = u32_at(bytes, at)? as i32;
            at += 4;
            if !(-16777216..16777216).contains(&v) {
                return Err(invalid("TFM kern range"));
            }
            kerns.push(FixWord(v));
        }
        let recipes = bytes[at..at + ne * 4].as_chunks::<4>().0.to_vec();
        at += ne * 4;
        let mut parameters = Vec::new();
        for i in 0..np {
            let v = u32_at(bytes, at)? as i32;
            at += 4;
            if i != 0 && !(-16777216..16777216).contains(&v) {
                return Err(invalid("TFM parameter range"));
            }
            parameters.push(FixWord(v));
        }
        let mut chars = vec![None; 256];
        for (i, info) in infos.iter().enumerate() {
            let [w, hd, it, rem] = *info;
            let h = (hd >> 4) as usize;
            let d = (hd & 15) as usize;
            let italic = (it >> 2) as usize;
            if w as usize >= nw || h >= nh || d >= nd || italic >= ni {
                return Err(invalid("TFM character metric index"));
            }
            if w != 0 {
                chars[bc + i] = Some(CharacterMetrics {
                    width: widths[w as usize],
                    height: heights[h],
                    depth: depths[d],
                    italic: italics[italic],
                    tag: it & 3,
                    remainder: rem,
                });
            }
        }
        let exists = |c: u8| chars[c as usize].is_some();
        let boundary_character = program.first().filter(|p| p[0] == 255).map(|p| p[1]);
        let left_boundary_program = program
            .last()
            .filter(|p| p[0] == 255)
            .map(|p| p[2] as usize * 256 + p[3] as usize);
        for (i, p) in program.iter().enumerate() {
            let [skip, next, op, rem] = *p;
            if skip > 128 {
                if op as usize * 256 + rem as usize >= nl {
                    return Err(invalid("TFM ligature restart index"));
                }
                continue;
            }
            if skip < 128 && i + skip as usize + 1 >= nl {
                return Err(invalid("TFM ligature skip outside program"));
            }
            if !exists(next) && Some(next) != boundary_character {
                return Err(invalid("TFM ligature next character absent"));
            }
            if op >= 128 {
                if (op as usize - 128) * 256 + rem as usize >= nk {
                    return Err(invalid("TFM kern index"));
                }
            } else if !exists(rem) || op / 4 > ((op / 2) & 1) + (op & 1) {
                return Err(invalid("TFM ligature opcode/replacement"));
            }
        }
        for (code, ch) in chars.iter().enumerate() {
            if let Some(ch) = ch {
                match ch.tag {
                    1 => {
                        if ch.remainder as usize >= nl {
                            return Err(invalid("TFM character program index"));
                        }
                    }
                    2 => {
                        let mut seen = [false; 256];
                        let mut current = code;
                        loop {
                            if seen[current] {
                                return Err(invalid("TFM next-larger cycle"));
                            }
                            seen[current] = true;
                            let c = chars[current]
                                .ok_or_else(|| invalid("TFM next-larger character absent"))?;
                            if c.tag != 2 {
                                break;
                            }
                            current = c.remainder as usize;
                        }
                    }
                    3 if ch.remainder as usize >= ne => {
                        return Err(invalid("TFM extensible recipe index"));
                    }
                    _ => {}
                }
            }
        }
        for recipe in recipes {
            for (i, c) in recipe.into_iter().enumerate() {
                if (i == 3 || c != 0) && !exists(c) {
                    return Err(invalid("TFM extensible piece absent"));
                }
            }
        }
        Ok(Self {
            checksum,
            design_size,
            source_sha256: crate::sha256(bytes),
            chars,
            program,
            kerns,
            parameters,
            boundary_character,
            left_boundary_program,
        })
    }
    pub fn char_metrics(&self, code: u8) -> Option<CharacterMetrics> {
        self.chars[code as usize]
    }
    pub fn parameter(&self, one_based: usize) -> Option<FixWord> {
        one_based
            .checked_sub(1)
            .and_then(|i| self.parameters.get(i).copied())
    }
    pub fn pair_action(&self, left: u8, right: u8) -> Result<Option<PairAction>> {
        let ch = self
            .char_metrics(left)
            .ok_or_else(|| invalid("TFM left character missing"))?;
        if self.char_metrics(right).is_none() && Some(right) != self.boundary_character {
            return Err(invalid("TFM right character missing"));
        }
        if ch.tag != 1 {
            return Ok(None);
        }
        let mut index = ch.remainder as usize;
        let first = self.program[index];
        if first[0] > 128 {
            index = first[2] as usize * 256 + first[3] as usize;
        }
        for _ in 0..self.program.len() {
            let [skip, next, op, rem] = self.program[index];
            if skip > 128 {
                return Ok(None);
            }
            if next == right {
                return Ok(Some(if op >= 128 {
                    PairAction::Kern(self.kerns[(op as usize - 128) * 256 + rem as usize])
                } else {
                    PairAction::Ligature {
                        replacement: rem,
                        retain_left: op & 2 != 0,
                        retain_right: op & 1 != 0,
                        advance: op / 4,
                    }
                }));
            }
            if skip >= 128 {
                return Ok(None);
            }
            index += skip as usize + 1;
        }
        Err(invalid("TFM pair program budget"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        let counts = [17u16, 2, 65, 66, 2, 1, 1, 1, 1, 1, 0, 0];
        let mut b = counts
            .into_iter()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<_>>();
        for word in [
            0x12345678u32,
            10 << 20,
            0x01000100,
            0x01000000,
            0,
            1 << 19,
            0,
            0,
            0,
            0x80428000,
            (-131072i32) as u32,
        ] {
            b.extend(word.to_be_bytes());
        }
        b
    }
    #[test]
    fn exact_metrics_and_signed_kern() {
        let t = Tfm::parse(&fixture()).unwrap();
        assert_eq!(t.checksum, 0x12345678);
        assert_eq!(t.char_metrics(65).unwrap().width, FixWord(1 << 19));
        assert_eq!(
            t.pair_action(65, 66).unwrap(),
            Some(PairAction::Kern(FixWord(-131072)))
        );
        assert_eq!(FixWord(1 << 19).at_design_size(t.design_size), 5i64 << 40);
    }
    #[test]
    fn all_truncations_and_extra_bytes_fail() {
        let b = fixture();
        for i in 0..b.len() {
            assert!(Tfm::parse(&b[..i]).is_err(), "{i}");
        }
        let mut b = b;
        b.push(0);
        assert!(Tfm::parse(&b).is_err());
    }
    #[test]
    fn invalid_metric_program_indices() {
        for (at, value) in [(32, 2), (34, 5), (60, 0), (62, 129)] {
            let mut b = fixture();
            b[at] = value;
            assert!(Tfm::parse(&b).is_err(), "{at}");
        }
    }
    #[test]
    fn ligature_action_retains_semantics() {
        let mut b = fixture();
        b[62] = 0;
        b[63] = 66;
        let t = Tfm::parse(&b).unwrap();
        assert_eq!(
            t.pair_action(65, 66).unwrap(),
            Some(PairAction::Ligature {
                replacement: 66,
                retain_left: false,
                retain_right: false,
                advance: 0
            })
        );
        b[62] = 4;
        assert!(Tfm::parse(&b).is_err());
    }
}

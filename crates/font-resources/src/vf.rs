//! Original bounded virtual-font packet parser. Dimensions are raw signed VF fix units.
use crate::{invalid, tfm::FixWord, Result};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDefinition {
    pub id: i32,
    pub checksum: u32,
    pub scale: FixWord,
    pub design_size: FixWord,
    pub area: Vec<u8>,
    pub name: Vec<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    W,
    X,
    Y,
    Z,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Glyph {
        code: u32,
        advance: bool,
    },
    Rule {
        height: i32,
        width: i32,
        advance: bool,
    },
    Nop,
    Push,
    Pop,
    Right(i32),
    Down(i32),
    Register {
        register: Register,
        value: Option<i32>,
    },
    Font(i32),
    Special(Vec<u8>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub code: u32,
    pub width: FixWord,
    pub commands: Vec<Command>,
}
#[derive(Debug, Clone)]
pub struct VirtualFont {
    pub checksum: u32,
    pub design_size: FixWord,
    pub comment: Vec<u8>,
    pub source_sha256: String,
    fonts: BTreeMap<i32, FontDefinition>,
    packets: BTreeMap<u32, Packet>,
    first_font: Option<i32>,
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| invalid("VF length overflow"))?;
        let b = self
            .bytes
            .get(self.at..end)
            .ok_or_else(|| invalid("VF truncated data"))?;
        self.at = end;
        Ok(b)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn unsigned(&mut self, n: usize) -> Result<u32> {
        let mut v = 0u32;
        for b in self.take(n)? {
            v = (v << 8) | *b as u32;
        }
        Ok(v)
    }
    fn signed(&mut self, n: usize) -> Result<i32> {
        let v = self.unsigned(n)?;
        Ok(if n == 4 {
            v as i32
        } else {
            ((v << (32 - n * 8)) as i32) >> (32 - n * 8)
        })
    }
    fn dimension(&mut self, n: usize) -> Result<i32> {
        let v = self.signed(n)?;
        if !(-16777215..16777216).contains(&v) {
            return Err(invalid("VF dimension range"));
        }
        Ok(v)
    }
}
impl VirtualFont {
    pub fn fonts(&self) -> &BTreeMap<i32, FontDefinition> {
        &self.fonts
    }
    pub fn packet(&self, code: u32) -> Option<&Packet> {
        self.packets.get(&code)
    }
    pub fn first_font(&self) -> Option<i32> {
        self.first_font
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid("VF file budget"));
        }
        let mut r = Reader { bytes, at: 0 };
        if r.byte()? != 247 || r.byte()? != 202 {
            return Err(invalid("VF preamble/version"));
        }
        let length = r.byte()? as usize;
        let comment = r.take(length)?.to_vec();
        let checksum = r.unsigned(4)?;
        let design_size = FixWord(r.signed(4)?);
        if design_size.0 < 1 << 20 {
            return Err(invalid("VF design size"));
        }
        let mut fonts = BTreeMap::new();
        let mut packets = BTreeMap::new();
        let mut first_font = None;
        let mut total = 0;
        loop {
            let op = r.byte()?;
            match op {
                243..=246 => {
                    if !packets.is_empty() || fonts.len() >= 4096 {
                        return Err(invalid("VF late font definition/budget"));
                    }
                    let id = r.unsigned((op - 242) as usize)? as i32;
                    let checksum = r.unsigned(4)?;
                    let scale = FixWord(r.dimension(4)?);
                    let design_size = FixWord(r.signed(4)?);
                    if scale.0 <= 0 || design_size.0 < 1 << 20 {
                        return Err(invalid("VF font size"));
                    }
                    let a = r.byte()? as usize;
                    let n = r.byte()? as usize;
                    let area = r.take(a)?.to_vec();
                    let name = r.take(n)?.to_vec();
                    if name.is_empty() {
                        return Err(invalid("VF empty font name"));
                    }
                    if fonts
                        .insert(
                            id,
                            FontDefinition {
                                id,
                                checksum,
                                scale,
                                design_size,
                                area,
                                name,
                            },
                        )
                        .is_some()
                    {
                        return Err(invalid("VF duplicate font ID"));
                    }
                    first_font.get_or_insert(id);
                }
                0..=242 => {
                    if packets.len() >= 65536 {
                        return Err(invalid("VF packet count budget"));
                    }
                    let (length, code, width) = if op == 242 {
                        (
                            r.unsigned(4)? as usize,
                            r.unsigned(4)?,
                            FixWord(r.dimension(4)?),
                        )
                    } else {
                        (
                            op as usize,
                            r.byte()? as u32,
                            FixWord(r.unsigned(3)? as i32),
                        )
                    };
                    if length > 1024 * 1024 {
                        return Err(invalid("VF packet byte budget"));
                    }
                    let commands = parse_commands(r.take(length)?, &fonts, first_font)?;
                    total += commands.len();
                    if total > 1_000_000 {
                        return Err(invalid("VF total command budget"));
                    }
                    if packets
                        .insert(
                            code,
                            Packet {
                                code,
                                width,
                                commands,
                            },
                        )
                        .is_some()
                    {
                        return Err(invalid("VF duplicate packet"));
                    }
                }
                248 => {
                    if r.take(bytes.len() - r.at)?.iter().any(|b| *b != 248) {
                        return Err(invalid("VF postamble padding"));
                    }
                    break;
                }
                _ => return Err(invalid("VF outer opcode")),
            }
        }
        Ok(Self {
            checksum,
            design_size,
            comment,
            source_sha256: crate::sha256(bytes),
            fonts,
            packets,
            first_font,
        })
    }
}
fn parse_commands(
    bytes: &[u8],
    fonts: &BTreeMap<i32, FontDefinition>,
    mut font: Option<i32>,
) -> Result<Vec<Command>> {
    let mut r = Reader { bytes, at: 0 };
    let mut commands = Vec::new();
    let mut depth = 0;
    while r.at < bytes.len() {
        if commands.len() >= 100000 {
            return Err(invalid("VF packet command budget"));
        }
        let op = r.byte()?;
        let command = match op {
            0..=127 => Command::Glyph {
                code: op as u32,
                advance: true,
            },
            128..=131 => Command::Glyph {
                code: r.unsigned((op - 127) as usize)?,
                advance: true,
            },
            133..=136 => Command::Glyph {
                code: r.unsigned((op - 132) as usize)?,
                advance: false,
            },
            132 | 137 => Command::Rule {
                height: r.dimension(4)?,
                width: r.dimension(4)?,
                advance: op == 132,
            },
            138 => Command::Nop,
            141 => {
                depth += 1;
                if depth > 64 {
                    return Err(invalid("VF stack depth"));
                }
                Command::Push
            }
            142 => {
                if depth == 0 {
                    return Err(invalid("VF stack underflow"));
                }
                depth -= 1;
                Command::Pop
            }
            143..=146 => Command::Right(r.dimension((op - 142) as usize)?),
            147..=151 => Command::Register {
                register: Register::W,
                value: if op == 147 {
                    None
                } else {
                    Some(r.dimension((op - 147) as usize)?)
                },
            },
            152..=156 => Command::Register {
                register: Register::X,
                value: if op == 152 {
                    None
                } else {
                    Some(r.dimension((op - 152) as usize)?)
                },
            },
            157..=160 => Command::Down(r.dimension((op - 156) as usize)?),
            161..=165 => Command::Register {
                register: Register::Y,
                value: if op == 161 {
                    None
                } else {
                    Some(r.dimension((op - 161) as usize)?)
                },
            },
            166..=170 => Command::Register {
                register: Register::Z,
                value: if op == 166 {
                    None
                } else {
                    Some(r.dimension((op - 166) as usize)?)
                },
            },
            171..=234 => Command::Font((op - 171) as i32),
            235..=238 => Command::Font(r.unsigned((op - 234) as usize)? as i32),
            239..=242 => {
                let n = r.unsigned((op - 238) as usize)? as usize;
                Command::Special(r.take(n)?.to_vec())
            }
            _ => return Err(invalid("VF prohibited packet opcode")),
        };
        match &command {
            Command::Font(id) => {
                if !fonts.contains_key(id) {
                    return Err(invalid("VF missing font definition"));
                }
                font = Some(*id);
            }
            Command::Glyph { .. } if font.is_none() => {
                return Err(invalid("VF glyph without font"))
            }
            _ => {}
        }
        commands.push(command);
    }
    if depth != 0 {
        return Err(invalid("VF unbalanced stack"));
    }
    Ok(commands)
}
#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn fixture(commands: &[u8], long: bool) -> Vec<u8> {
        let mut b = vec![247, 202, 0];
        for n in [0u32, 10 << 20] {
            b.extend(n.to_be_bytes());
        }
        b.extend([243, 0]);
        for n in [0u32, 1 << 20, 10 << 20] {
            b.extend(n.to_be_bytes());
        }
        b.extend([0, 1, b'f']);
        if long {
            b.push(242);
            for n in [commands.len() as u32, 65, 1 << 19] {
                b.extend(n.to_be_bytes());
            }
        } else {
            b.extend([commands.len() as u8, 65, 8, 0, 0]);
        }
        b.extend(commands);
        b.push(248);
        b
    }
    #[test]
    fn short_long_and_exact_commands() {
        for long in [false, true] {
            let v =
                VirtualFont::parse(&fixture(&[65, 143, 255, 141, 148, 2, 147, 142], long)).unwrap();
            assert_eq!(v.packet(65).unwrap().width, FixWord(1 << 19));
            assert_eq!(v.packet(65).unwrap().commands[1], Command::Right(-1));
        }
    }
    #[test]
    fn all_truncations_fail() {
        let b = fixture(&[65], true);
        for n in 0..b.len() {
            assert!(VirtualFont::parse(&b[..n]).is_err(), "{n}");
        }
    }
    #[test]
    fn invalid_opcodes_fonts_stack() {
        for commands in [
            vec![139],
            vec![172],
            vec![142],
            vec![141],
            vec![141; 65],
            vec![239, 255],
        ] {
            assert!(VirtualFont::parse(&fixture(&commands, true)).is_err());
        }
    }
    #[test]
    fn specials_preserved_not_ignored() {
        let v = VirtualFont::parse(&fixture(&[239, 2, b'p', b's'], false)).unwrap();
        assert_eq!(
            v.packet(65).unwrap().commands,
            vec![Command::Special(b"ps".to_vec())]
        );
    }
}

//! Original bounded Type1 charstring subset. No hints, PostScript or FontMatrix application.
use crate::{
    cff::{CubicCommand, CubicPoint},
    pfb::Identity,
    sha256,
    type1_records::Records,
    Coordinate,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Record(crate::type1_records::Error),
    Truncated,
    Stack,
    Width,
    Path,
    Call,
    Trailing,
    Budget,
    Arithmetic,
    Unsupported(u16),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSource {
    pub subroutine_chain: Vec<usize>,
    pub byte_offset: usize,
}
pub struct Outline {
    pub identity: Identity,
    pub glyph_name: String,
    pub charstring_sha256: String,
    pub sidebearing: CubicPoint,
    pub advance: CubicPoint,
    pub commands: Vec<CubicCommand>,
    pub sources: Vec<CommandSource>,
}
fn point(x: i32, y: i32) -> CubicPoint {
    CubicPoint {
        x: Coordinate::from_integer(x),
        y: Coordinate::from_integer(y),
    }
}
struct Decoder<F> {
    lookup: F,
    stack: Vec<i32>,
    at: CubicPoint,
    bearing: CubicPoint,
    advance: CubicPoint,
    width: bool,
    open: bool,
    commands: Vec<CubicCommand>,
    sources: Vec<CommandSource>,
    calls: Vec<usize>,
    steps: usize,
    bytes: usize,
}
#[derive(PartialEq)]
enum Flow {
    End,
    Return,
}
impl<F: FnMut(usize) -> Result<Vec<u8>, Error>> Decoder<F> {
    fn args(&mut self, n: usize) -> Result<Vec<i32>, Error> {
        if self.stack.len() != n {
            return Err(Error::Stack);
        }
        Ok(std::mem::take(&mut self.stack))
    }
    fn delta(&mut self, x: i32, y: i32) -> Result<CubicPoint, Error> {
        self.at.x = self
            .at
            .x
            .add(Coordinate::from_integer(x))
            .map_err(|_| Error::Arithmetic)?;
        self.at.y = self
            .at
            .y
            .add(Coordinate::from_integer(y))
            .map_err(|_| Error::Arithmetic)?;
        Ok(self.at)
    }
    fn emit(&mut self, command: CubicCommand, offset: usize) -> Result<(), Error> {
        if self.commands.len() == 16384 {
            return Err(Error::Budget);
        }
        self.commands.push(command);
        self.sources.push(CommandSource {
            subroutine_chain: self.calls.clone(),
            byte_offset: offset,
        });
        Ok(())
    }
    fn run(&mut self, b: &[u8], nested: bool) -> Result<Flow, Error> {
        self.bytes = self.bytes.checked_add(b.len()).ok_or(Error::Budget)?;
        if self.bytes > 16 * 1024 * 1024 {
            return Err(Error::Budget);
        }
        let mut i = 0;
        while i < b.len() {
            self.steps += 1;
            if self.steps > 65536 {
                return Err(Error::Budget);
            }
            let offset = i;
            let op = b[i];
            i += 1;
            let mut byte = || {
                let v = *b.get(i).ok_or(Error::Truncated)?;
                i += 1;
                Ok::<_, Error>(v)
            };
            if op >= 32 {
                let n = match op {
                    32..=246 => i32::from(op) - 139,
                    247..=250 => (i32::from(op) - 247) * 256 + i32::from(byte()?) + 108,
                    251..=254 => -(i32::from(op) - 251) * 256 - i32::from(byte()?) - 108,
                    255 => i32::from_be_bytes([byte()?, byte()?, byte()?, byte()?]),
                    _ => unreachable!(),
                };
                if self.stack.len() == 24 {
                    return Err(Error::Stack);
                }
                self.stack.push(n);
                continue;
            }
            let op = if op == 12 {
                0x100 + u16::from(byte()?)
            } else {
                u16::from(op)
            };
            if !self.width && matches!(op, 4..=9 | 14 | 21 | 22 | 30 | 31) {
                return Err(Error::Width);
            }
            match op {
                13 | 0x107 => {
                    if self.width {
                        return Err(Error::Width);
                    }
                    let a = self.args(if op == 13 { 2 } else { 4 })?;
                    self.bearing = if op == 13 {
                        point(a[0], 0)
                    } else {
                        point(a[0], a[1])
                    };
                    self.advance = if op == 13 {
                        point(a[1], 0)
                    } else {
                        point(a[2], a[3])
                    };
                    self.at = self.bearing;
                    self.width = true;
                }
                4 | 21 | 22 => {
                    if self.open {
                        return Err(Error::Path);
                    }
                    let a = self.args(if op == 21 { 2 } else { 1 })?;
                    let (x, y) = match op {
                        4 => (0, a[0]),
                        22 => (a[0], 0),
                        _ => (a[0], a[1]),
                    };
                    let p = self.delta(x, y)?;
                    self.emit(CubicCommand::MoveTo(p), offset)?;
                    self.open = true;
                }
                5..=7 => {
                    if !self.open {
                        return Err(Error::Path);
                    }
                    let a = self.args(if op == 5 { 2 } else { 1 })?;
                    let (x, y) = match op {
                        6 => (a[0], 0),
                        7 => (0, a[0]),
                        _ => (a[0], a[1]),
                    };
                    let p = self.delta(x, y)?;
                    self.emit(CubicCommand::LineTo(p), offset)?;
                }
                8 | 30 | 31 => {
                    if !self.open {
                        return Err(Error::Path);
                    }
                    let a = self.args(if op == 8 { 6 } else { 4 })?;
                    let d = match op {
                        30 => [0, a[0], a[1], a[2], a[3], 0],
                        31 => [a[0], 0, a[1], a[2], 0, a[3]],
                        _ => [a[0], a[1], a[2], a[3], a[4], a[5]],
                    };
                    let control1 = self.delta(d[0], d[1])?;
                    let control2 = self.delta(d[2], d[3])?;
                    let end = self.delta(d[4], d[5])?;
                    self.emit(
                        CubicCommand::CurveTo {
                            control1,
                            control2,
                            end,
                        },
                        offset,
                    )?;
                }
                9 => {
                    self.args(0)?;
                    if !self.open {
                        return Err(Error::Path);
                    }
                    self.emit(CubicCommand::Close, offset)?;
                    self.open = false;
                }
                10 => {
                    let index = usize::try_from(self.stack.pop().ok_or(Error::Stack)?)
                        .map_err(|_| Error::Call)?;
                    if self.calls.len() == 16 || self.calls.contains(&index) {
                        return Err(Error::Call);
                    }
                    let bytes = (self.lookup)(index)?;
                    self.calls.push(index);
                    let flow = self.run(&bytes, true)?;
                    self.calls.pop();
                    if flow == Flow::End {
                        if i != b.len() {
                            return Err(Error::Trailing);
                        }
                        return Ok(flow);
                    }
                }
                11 => {
                    if !nested {
                        return Err(Error::Call);
                    }
                    if i != b.len() {
                        return Err(Error::Trailing);
                    }
                    return Ok(Flow::Return);
                }
                14 => {
                    self.args(0)?;
                    if self.open {
                        return Err(Error::Path);
                    }
                    if i != b.len() {
                        return Err(Error::Trailing);
                    }
                    return Ok(Flow::End);
                }
                other => return Err(Error::Unsupported(other)),
            }
        }
        Err(Error::Truncated)
    }
}
pub fn interpret(records: &Records<'_>, glyph_name: &str) -> Result<Outline, Error> {
    let bytes = records.decrypted_glyph(glyph_name).map_err(Error::Record)?;
    let mut d = decoder(|i| records.decrypted_subr(i).map_err(Error::Record));
    if d.run(&bytes, false)? != Flow::End {
        return Err(Error::Call);
    }
    Ok(Outline {
        identity: records.identity().clone(),
        glyph_name: glyph_name.into(),
        charstring_sha256: sha256(&bytes),
        sidebearing: d.bearing,
        advance: d.advance,
        commands: d.commands,
        sources: d.sources,
    })
}
fn decoder<F: FnMut(usize) -> Result<Vec<u8>, Error>>(lookup: F) -> Decoder<F> {
    Decoder {
        lookup,
        stack: vec![],
        at: point(0, 0),
        bearing: point(0, 0),
        advance: point(0, 0),
        width: false,
        open: false,
        commands: vec![],
        sources: vec![],
        calls: vec![],
        steps: 0,
        bytes: 0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn num(n: i32, b: &mut Vec<u8>) {
        b.push(255);
        b.extend(n.to_be_bytes())
    }
    fn base() -> Vec<u8> {
        let mut b = vec![];
        num(10, &mut b);
        num(500, &mut b);
        b.push(13);
        b
    }
    #[test]
    fn exact_lines_curves_closepoint_and_subroutine_provenance() {
        let mut b = base();
        num(0, &mut b);
        num(0, &mut b);
        b.push(21);
        num(0, &mut b);
        b.push(10);
        b.push(9);
        num(1, &mut b);
        b.push(22);
        b.push(9);
        b.push(14);
        let mut sub = vec![];
        for n in [1, 2, 3, 4, 5, 6] {
            num(n, &mut sub)
        }
        sub.extend([8, 11]);
        let mut d = decoder(|_| Ok(sub.clone()));
        assert!(d.run(&b, false) == Ok(Flow::End));
        assert_eq!(d.advance, point(500, 0));
        assert_eq!(
            d.commands[1],
            CubicCommand::CurveTo {
                control1: point(11, 2),
                control2: point(14, 6),
                end: point(19, 12)
            }
        );
        assert_eq!(d.commands[3], CubicCommand::MoveTo(point(20, 12)));
        assert_eq!(d.sources[1].subroutine_chain, vec![0]);
    }
    #[test]
    fn unsupported_stack_cycles_and_invalid_control_flow() {
        for op in [1, 3, 0x100, 0x106, 0x10c, 0x110, 0x111] {
            let mut b = base();
            if op >= 256 {
                b.extend([12, (op - 256) as u8])
            } else {
                b.push(op as u8)
            }
            let mut d = decoder(|_| Err(Error::Call));
            assert_eq!(d.run(&b, false).err(), Some(Error::Unsupported(op)));
        }
        let mut b = base();
        for _ in 0..25 {
            num(0, &mut b)
        }
        assert_eq!(
            decoder(|_| Err(Error::Call)).run(&b, false).err(),
            Some(Error::Stack)
        );
        let mut b = base();
        num(0, &mut b);
        b.push(10);
        let mut sub = vec![];
        num(0, &mut sub);
        sub.extend([10, 11]);
        assert_eq!(
            decoder(|_| Ok(sub.clone())).run(&b, false).err(),
            Some(Error::Call)
        );
        assert_eq!(
            decoder(|_| Err(Error::Call)).run(&[255, 1], false).err(),
            Some(Error::Truncated)
        );
    }
    #[test]
    fn width_subroutine_and_hard_execution_output_limits() {
        let mut b = vec![];
        num(0, &mut b);
        b.extend([10, 14]);
        let mut width = base();
        width.push(11);
        let mut d = decoder(|_| Ok(width.clone()));
        assert!(d.run(&b, false) == Ok(Flow::End));
        assert_eq!(d.advance, point(500, 0));
        let mut b = base();
        num(0, &mut b);
        num(0, &mut b);
        b.push(21);
        for _ in 0..16384 {
            num(i32::MAX, &mut b);
            b.push(6);
        }
        b.extend([9, 14]);
        assert_eq!(
            decoder(|_| Err(Error::Call)).run(&b, false).err(),
            Some(Error::Budget)
        );
        let mut b = base();
        for _ in 0..22000 {
            num(0, &mut b);
            b.push(10);
        }
        b.push(14);
        assert_eq!(
            decoder(|_| Ok(vec![11])).run(&b, false).err(),
            Some(Error::Budget)
        );
    }
}

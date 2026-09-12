use crate::{invalid, u16_at, u32_at, Error, FontResource, Result};
/// Exact numerator / 2^shift in unscaled font units. Normalized after operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate {
    numerator: i128,
    shift: u32,
}
impl Coordinate {
    pub fn numerator(&self) -> i128 {
        self.numerator
    }
    pub fn shift(&self) -> u32 {
        self.shift
    }
    pub(crate) fn new(mut n: i128, mut shift: u32) -> Result<Self> {
        if shift > 96 {
            return Err(invalid("coordinate precision budget"));
        }
        if n == 0 {
            return Ok(Self {
                numerator: 0,
                shift: 0,
            });
        }
        while shift > 0 && n % 2 == 0 {
            n /= 2;
            shift -= 1;
        }
        Ok(Self {
            numerator: n,
            shift,
        })
    }
    pub fn from_integer(n: i32) -> Self {
        Self {
            numerator: n as i128,
            shift: 0,
        }
    }
    pub(crate) fn add(self, rhs: Self) -> Result<Self> {
        let shift = self.shift.max(rhs.shift);
        let a = self
            .numerator
            .checked_mul(1i128 << (shift - self.shift))
            .ok_or_else(|| invalid("coordinate overflow"))?;
        let b = rhs
            .numerator
            .checked_mul(1i128 << (shift - rhs.shift))
            .ok_or_else(|| invalid("coordinate overflow"))?;
        Self::new(
            a.checked_add(b)
                .ok_or_else(|| invalid("coordinate overflow"))?,
            shift,
        )
    }
    fn subtract(self, rhs: Self) -> Result<Self> {
        self.add(Self::new(
            rhs.numerator
                .checked_neg()
                .ok_or_else(|| invalid("coordinate negation overflow"))?,
            rhs.shift,
        )?)
    }
    fn scale(self, n: i16) -> Result<Self> {
        Self::new(
            self.numerator
                .checked_mul(n as i128)
                .ok_or_else(|| invalid("coordinate overflow"))?,
            self.shift + 14,
        )
    }
    pub fn midpoint(self, rhs: Self) -> Result<Self> {
        let sum = self.add(rhs)?;
        Self::new(sum.numerator, sum.shift + 1)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactPoint {
    pub x: Coordinate,
    pub y: Coordinate,
    pub on_curve: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphInstance {
    pub glyph_id: u16,
    pub point_start: u32,
    pub point_count: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedOutline {
    pub font_id: String,
    pub font_sha256: String,
    pub face_index: u32,
    pub glyph_id: u16,
    pub points: Vec<ExactPoint>,
    pub contour_ends: Vec<u32>,
    pub instances: Vec<GlyphInstance>,
}
const MAX_NODES: usize = 4096;
const MAX_POINTS: usize = 1_000_000;
impl FontResource {
    pub fn expanded_outline(&self, gid: u16) -> Result<ExpandedOutline> {
        let fetch = |id: u16| -> Result<&[u8]> {
            if id as u32 >= self.descriptor().glyph_count {
                return Err(invalid("glyph ID outside font"));
            }
            let head = self.table(b"head").ok_or_else(|| invalid("head missing"))?;
            let loca = self.table(b"loca").ok_or_else(|| invalid("loca missing"))?;
            let glyf = self.table(b"glyf").ok_or_else(|| invalid("glyf missing"))?;
            let offset = |i| -> Result<usize> {
                Ok(if u16_at(head, 50)? == 1 {
                    u32_at(loca, i * 4)? as usize
                } else {
                    u16_at(loca, i * 2)? as usize * 2
                })
            };
            glyf.get(offset(id as usize)?..offset(id as usize + 1)?)
                .ok_or_else(|| invalid("glyph extent"))
        };
        let raw = expand(gid, &fetch, &mut Vec::new(), &mut 0, &mut 0)?;
        Ok(ExpandedOutline {
            font_id: self.descriptor().font_id.clone(),
            font_sha256: self.descriptor().sha256.clone(),
            face_index: self.descriptor().face_index,
            glyph_id: gid,
            points: raw.points,
            contour_ends: raw.ends,
            instances: raw.instances,
        })
    }
}
#[derive(Default)]
struct Raw {
    points: Vec<ExactPoint>,
    ends: Vec<u32>,
    instances: Vec<GlyphInstance>,
}
fn expand<'a>(
    gid: u16,
    fetch: &impl Fn(u16) -> Result<&'a [u8]>,
    stack: &mut Vec<u16>,
    nodes: &mut usize,
    total_points: &mut usize,
) -> Result<Raw> {
    if stack.len() > crate::MAX_COMPOSITE_DEPTH || stack.contains(&gid) {
        return Err(invalid("composite cycle/depth"));
    }
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err(invalid("expanded node budget"));
    }
    let data = fetch(gid)?;
    if data.is_empty() || (u16_at(data, 0)? as i16) >= 0 {
        let simple = crate::outline::decode(data)?;
        *total_points += simple.points.len();
        if *total_points > MAX_POINTS {
            return Err(invalid("expanded point budget"));
        }
        let points = simple
            .points
            .iter()
            .map(|p| {
                Ok(ExactPoint {
                    x: Coordinate::new(p.x as i128, 16)?,
                    y: Coordinate::new(p.y as i128, 16)?,
                    on_curve: p.on_curve,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        return Ok(Raw {
            instances: vec![GlyphInstance {
                glyph_id: gid,
                point_start: 0,
                point_count: simple.points.len() as u32,
            }],
            points,
            ends: simple.contour_ends.iter().map(|e| *e as u32).collect(),
        });
    }
    stack.push(gid);
    let mut out = Raw::default();
    let mut at = 10;
    loop {
        let flags = u16_at(data, at)?;
        let child = u16_at(data, at + 2)?;
        at += 4;
        let xy = flags & 2 != 0;
        let (x, y) = if flags & 1 != 0 {
            let a = u16_at(data, at)?;
            let b = u16_at(data, at + 2)?;
            at += 4;
            if xy {
                (a as i16 as i32, b as i16 as i32)
            } else {
                (a as i32, b as i32)
            }
        } else {
            let args = data
                .get(at..at + 2)
                .ok_or_else(|| invalid("composite arguments"))?;
            at += 2;
            if xy {
                (args[0] as i8 as i32, args[1] as i8 as i32)
            } else {
                (args[0] as i32, args[1] as i32)
            }
        };
        let mut m = [16384i16, 0, 0, 16384];
        if flags & 8 != 0 {
            m[0] = u16_at(data, at)? as i16;
            m[3] = m[0];
            at += 2;
        } else if flags & 64 != 0 {
            m[0] = u16_at(data, at)? as i16;
            m[3] = u16_at(data, at + 2)? as i16;
            at += 4;
        } else if flags & 128 != 0 {
            for coefficient in &mut m {
                *coefficient = u16_at(data, at)? as i16;
                at += 2;
            }
        }
        let transform = |p: ExactPoint| -> Result<ExactPoint> {
            Ok(ExactPoint {
                x: p.x.scale(m[0])?.add(p.y.scale(m[2])?)?,
                y: p.x.scale(m[1])?.add(p.y.scale(m[3])?)?,
                on_curve: p.on_curve,
            })
        };
        let child = expand(child, fetch, stack, nodes, total_points)?;
        let offset = if xy {
            let mut offset = ExactPoint {
                x: Coordinate::from_integer(x),
                y: Coordinate::from_integer(y),
                on_curve: true,
            };
            if flags & 4 != 0 && (x != 0 || y != 0) {
                return Err(Error::UnsupportedFont(
                    "grid-rounded composite offsets require hinting policy".into(),
                ));
            }
            if flags & 0x800 != 0 {
                offset = transform(offset)?;
            } else if flags & 0x1000 == 0 && m != [16384, 0, 0, 16384] && (x != 0 || y != 0) {
                return Err(Error::UnsupportedFont(
                    "ambiguous default scaled composite offset".into(),
                ));
            }
            offset
        } else {
            // Only actual contour points exist in this unhinted design-space API.
            let parent = out.points.get(x as usize).ok_or_else(|| {
                invalid("composite parent attachment index outside existing outline")
            })?;
            let attached = child.points.get(y as usize).ok_or_else(|| {
                invalid(
                    "composite child attachment index outside outline; phantom points unavailable",
                )
            })?;
            let attached = transform(*attached)?;
            ExactPoint {
                x: parent.x.subtract(attached.x)?,
                y: parent.y.subtract(attached.y)?,
                on_curve: true,
            }
        };
        let start = out.points.len() as u32;
        for p in child.points {
            let mut p = transform(p)?;
            p.x = p.x.add(offset.x)?;
            p.y = p.y.add(offset.y)?;
            out.points.push(p);
        }
        out.ends.extend(child.ends.into_iter().map(|e| e + start));
        out.instances
            .extend(child.instances.into_iter().map(|mut instance| {
                instance.point_start += start;
                instance
            }));
        if flags & 32 == 0 {
            break;
        }
    }
    stack.pop();
    out.instances.insert(
        0,
        GlyphInstance {
            glyph_id: gid,
            point_start: 0,
            point_count: out.points.len() as u32,
        },
    );
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn simple() -> Vec<u8> {
        vec![0, 1, 0, 1, 0, 2, 0, 1, 0, 2, 0, 0, 0, 0, 0x37, 1, 2]
    }
    fn component(child: u16, flags: u16, x: i16, y: i16, m: &[i16]) -> Vec<u8> {
        let mut d = vec![255, 255, 0, 0, 0, 0, 0, 0, 0, 0];
        for word in [flags, child, x as u16, y as u16] {
            d.extend(word.to_be_bytes());
        }
        for c in m {
            d.extend(c.to_be_bytes());
        }
        d
    }
    fn run(glyphs: &[Vec<u8>], id: u16) -> Result<Raw> {
        expand(
            id,
            &|id| {
                glyphs
                    .get(id as usize)
                    .map(Vec::as_slice)
                    .ok_or_else(|| invalid("missing"))
            },
            &mut Vec::new(),
            &mut 0,
            &mut 0,
        )
    }
    #[test]
    #[ignore = "explicit installed font inspection, no visual assertion"]
    fn installed_expansion_smoke() {
        let bytes = std::fs::read(std::env::var("FLASHTEX_SMOKE_FONT").unwrap()).unwrap();
        let parsed = crate::parse(&bytes).unwrap();
        let table = |tag: &[u8; 4]| &bytes[parsed.tables[tag].clone()];
        let loca = table(b"loca");
        let glyf = table(b"glyf");
        let long = u16_at(table(b"head"), 50).unwrap() == 1;
        let offset = |i| {
            if long {
                u32_at(loca, i * 4).unwrap() as usize
            } else {
                u16_at(loca, i * 2).unwrap() as usize * 2
            }
        };
        let fetch =
            |id: u16| -> Result<&[u8]> { Ok(&glyf[offset(id as usize)..offset(id as usize + 1)]) };
        let mut accepted = 0;
        let mut unsupported = std::collections::BTreeMap::new();
        for gid in 0..parsed.glyphs as u16 {
            match expand(gid, &fetch, &mut Vec::new(), &mut 0, &mut 0) {
                Ok(_) => accepted += 1,
                Err(Error::UnsupportedFont(reason)) => *unsupported.entry(reason).or_insert(0) += 1,
                Err(e) => panic!("gid{gid}: {e}"),
            }
        }
        eprintln!(
            "fontSHA={} expanded={accepted} unsupported={unsupported:?}",
            crate::sha256(&bytes)
        );
    }
    #[test]
    fn nested_exact_affine() {
        let g = vec![
            simple(),
            component(0, 0x1083, 10, 20, &[8192, 0, 0, 16384]),
            component(1, 0x1003, -1, 3, &[]),
        ];
        let out = run(&g, 2).unwrap();
        assert_eq!(out.points[0].x, Coordinate::new(19, 1).unwrap());
        assert_eq!(out.points[0].y, Coordinate::from_integer(25));
        assert_eq!(
            out.instances.iter().map(|i| i.glyph_id).collect::<Vec<_>>(),
            vec![2, 1, 0]
        );
    }
    #[test]
    fn expansion_depth_and_node_budgets() {
        let mut glyphs = vec![simple()];
        for id in 0..34 {
            glyphs.push(component(id, 3, 0, 0, &[]));
        }
        assert!(run(&glyphs, 34).is_err());
        let mut glyphs = vec![simple()];
        for id in 0..13 {
            let mut d = component(id, 35, 0, 0, &[]);
            d.extend(&component(id, 3, 0, 0, &[])[10..]);
            glyphs.push(d);
        }
        assert!(run(&glyphs, 13).is_err());
    }
    #[test]
    fn point_attachment_after_affine_transform_is_exact() {
        let two = vec![
            0, 1, 0, 1, 0, 2, 0, 3, 0, 2, 0, 1, 0, 0, 0x37, 0x33, 1, 2, 2,
        ];
        let mut composite = component(0, 35, 10, 20, &[]);
        composite.extend(&component(1, 9, 0, 1, &[8192])[10..]);
        let out = run(&[simple(), two, composite], 2).unwrap();
        assert_eq!(out.points[2].x, out.points[0].x);
        assert_eq!(out.points[2].y, out.points[0].y);
        assert_eq!(out.points[1].x, Coordinate::from_integer(10));
        assert_eq!(out.instances[2].glyph_id, 1);
    }
    #[test]
    fn invalid_parent_child_attachment_indices_fail() {
        for (parent, child) in [(1, 0), (0, 1), (0, -1)] {
            let mut composite = component(0, 35, 0, 0, &[]);
            composite.extend(&component(0, 1, parent, child, &[])[10..]);
            assert!(run(&[simple(), composite], 1).is_err());
        }
        let mut composite = component(0, 35, 0, 0, &[]);
        let mut byte_attachment = vec![0, 0, 0, 0, 0, 0];
        composite.append(&mut byte_attachment);
        let out = run(&[simple(), composite], 1).unwrap();
        assert_eq!(out.points[0], out.points[1]);
    }
    #[test]
    fn explicit_offset_policy_is_retained_and_rounding_not_guessed() {
        let scaled = run(&[simple(), component(0, 0x80b, 10, 0, &[8192])], 1).unwrap();
        let unscaled = run(&[simple(), component(0, 0x100b, 10, 0, &[8192])], 1).unwrap();
        assert_eq!(scaled.points[0].x, Coordinate::new(11, 1).unwrap());
        assert_eq!(unscaled.points[0].x, Coordinate::new(21, 1).unwrap());
        assert!(matches!(
            run(&[simple(), component(0, 7, 10, 0, &[])], 1),
            Err(Error::UnsupportedFont(_))
        ));
    }
    #[test]
    fn cycles_and_attachment_explicit() {
        assert!(run(&[component(0, 3, 0, 0, &[])], 0).is_err());
        assert!(run(&[simple(), component(0, 1, 0, 0, &[])], 1).is_err());
    }
    #[test]
    fn checked_overflow_and_precision() {
        assert!(Coordinate {
            numerator: i128::MAX,
            shift: 0
        }
        .add(Coordinate::from_integer(1))
        .is_err());
        assert!(Coordinate {
            numerator: 1,
            shift: 96
        }
        .scale(1)
        .is_err());
    }
    #[test]
    fn scaled_offset_and_ambiguous_default() {
        let g = vec![simple(), component(0, 0x80b, 10, 0, &[8192])];
        assert_eq!(
            run(&g, 1).unwrap().points[0].x,
            Coordinate::new(11, 1).unwrap()
        );
        assert!(run(&[simple(), component(0, 11, 10, 0, &[8192])], 1).is_err());
    }
}

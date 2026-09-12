//! Strict literal clear-header matrix/paint context. No dictionary execution.
use crate::{
    cff::{CubicCommand, CubicPoint, DictNumber, MatrixCommand, Rational, RationalPoint},
    pfb::{Identity, Resource, SegmentKind},
    sha256,
    type1_outline::Outline,
    type1_records::{number, Parser},
};
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Header(crate::type1_records::Error),
    UnsupportedHeader,
    MissingContext,
    Paint,
    Identity,
    Arithmetic,
    Budget,
}
impl From<crate::type1_records::Error> for Error {
    fn from(e: crate::type1_records::Error) -> Self {
        Self::Header(e)
    }
}
#[derive(Clone)]
pub struct Context {
    identity: Identity,
    font_name: String,
    matrix: [Rational; 6],
    header_sha256: String,
}
impl Context {
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    pub fn font_name(&self) -> &str {
        &self.font_name
    }
    pub fn matrix(&self) -> [Rational; 6] {
        self.matrix
    }
    pub fn header_sha256(&self) -> &str {
        &self.header_sha256
    }
    pub fn from_resource(resource: &Resource) -> Result<Self, Error> {
        let mut header = Vec::new();
        for s in resource.segments() {
            if s.kind == SegmentKind::Binary {
                break;
            }
            if header.len() + s.payload.len() > 65536 {
                return Err(Error::Budget);
            }
            header.extend(&resource.bytes()[s.payload.clone()]);
        }
        let (font_name, matrix) = parse(&header)?;
        let mut trailer = Vec::new();
        let mut binary_seen = false;
        for segment in resource.segments() {
            if segment.kind == SegmentKind::Binary {
                binary_seen = true;
                continue;
            }
            if binary_seen {
                if trailer.len() + segment.payload.len() > 65536 {
                    return Err(Error::Budget);
                }
                trailer.extend(&resource.bytes()[segment.payload.clone()]);
            }
        }
        validate_trailer(&trailer)?;
        Ok(Self {
            identity: resource.identity().clone(),
            font_name,
            matrix,
            header_sha256: sha256(&header),
        })
    }
}
fn validate_trailer(bytes: &[u8]) -> Result<(), Error> {
    let mut at = 0;
    let mut zeros = 0;
    while let Some(c) = bytes.get(at) {
        if c.is_ascii_whitespace() {
            at += 1
        } else if *c == b'0' {
            zeros += 1;
            at += 1
        } else {
            break;
        }
    }
    if zeros != 512 {
        return Err(Error::UnsupportedHeader);
    }
    let tail = bytes.get(at..).ok_or(Error::UnsupportedHeader)?;
    if !tail.starts_with(b"cleartomark") || tail[11..].iter().any(|c| !c.is_ascii_whitespace()) {
        return Err(Error::UnsupportedHeader);
    }
    Ok(())
}
fn decimal(p: &mut Parser<'_>) -> Result<Rational, Error> {
    let t = p.token()?;
    if !number(t) {
        return Err(Error::UnsupportedHeader);
    }
    let s = std::str::from_utf8(t).map_err(|_| Error::UnsupportedHeader)?;
    Rational::operand(&DictNumber::Decimal(s.into())).map_err(|_| Error::Arithmetic)
}
fn parse(bytes: &[u8]) -> Result<(String, [Rational; 6]), Error> {
    let mut p = Parser {
        bytes,
        at: 0,
        start: 0,
        tokens: 0,
    };
    p.integer(64)?;
    p.words(&[b"dict", b"begin"])?;
    let mut seen = BTreeSet::new();
    let mut name = None;
    let mut matrix = None;
    let mut paint = None;
    let mut font_type = None;
    loop {
        let key = p.token()?;
        if key == b"currentdict" {
            break;
        }
        if !seen.insert(key.to_vec()) {
            return Err(Error::UnsupportedHeader);
        }
        match key {
            b"/FontName" => {
                let t = p.token()?;
                if t.first() != Some(&b'/')
                    || t.len() < 2
                    || !t[1..].iter().all(|c| (33..=126).contains(c))
                {
                    return Err(Error::UnsupportedHeader);
                }
                name = Some(
                    std::str::from_utf8(&t[1..])
                        .map_err(|_| Error::UnsupportedHeader)?
                        .into(),
                );
                p.expect(b"def")?
            }
            b"/FontType" => {
                font_type = Some(p.integer(255)?);
                p.expect(b"def")?
            }
            b"/PaintType" => {
                paint = Some(p.integer(255)?);
                p.expect(b"def")?
            }
            b"/FontMatrix" => {
                p.expect(b"[")?;
                let zero = Rational::new(0, 1).map_err(|_| Error::Arithmetic)?;
                let mut values = [zero; 6];
                for v in &mut values {
                    *v = decimal(&mut p)?
                }
                p.expect(b"]")?;
                let end = p.token()?;
                if end == b"readonly" {
                    p.expect(b"def")?
                } else if end != b"def" {
                    return Err(Error::UnsupportedHeader);
                }
                matrix = Some(values)
            }
            _ => return Err(Error::UnsupportedHeader),
        }
    }
    p.words(&[b"end", b"currentfile", b"eexec"])?;
    if bytes[p.at..].iter().any(|b| !b.is_ascii_whitespace()) {
        return Err(Error::UnsupportedHeader);
    }
    if font_type != Some(1) {
        return Err(Error::MissingContext);
    }
    if paint != Some(0) {
        return Err(Error::Paint);
    }
    let m = matrix.ok_or(Error::MissingContext)?;
    let determinant = m[0]
        .checked_mul(m[3])
        .and_then(|a| {
            m[1].checked_mul(m[2]).and_then(|b| {
                Rational::new(
                    b.numerator()
                        .checked_neg()
                        .ok_or_else(|| crate::invalid("matrix determinant overflow"))?,
                    b.denominator(),
                )
                .and_then(|b| a.checked_add(b))
            })
        })
        .map_err(|_| Error::Arithmetic)?;
    if determinant.numerator() == 0 {
        return Err(Error::Arithmetic);
    }
    Ok((name.ok_or(Error::MissingContext)?, m))
}
pub struct Transformed {
    pub raw: Outline,
    pub context: Context,
    pub advance: RationalPoint,
    pub sidebearing: RationalPoint,
    pub commands: Vec<MatrixCommand>,
}
fn map(p: CubicPoint, m: [Rational; 6], vector: bool) -> Result<RationalPoint, Error> {
    let x = Rational::new(p.x.numerator(), 1i128 << p.x.shift()).map_err(|_| Error::Arithmetic)?;
    let y = Rational::new(p.y.numerator(), 1i128 << p.y.shift()).map_err(|_| Error::Arithmetic)?;
    let axis = |a: Rational, b: Rational, c: Rational| {
        x.checked_mul(a)
            .and_then(|x| y.checked_mul(b).and_then(|y| x.checked_add(y)))
            .and_then(|v| if vector { Ok(v) } else { v.checked_add(c) })
            .map_err(|_| Error::Arithmetic)
    };
    Ok(RationalPoint {
        x: axis(m[0], m[2], m[4])?,
        y: axis(m[1], m[3], m[5])?,
    })
}
pub fn transform(raw: Outline, context: &Context) -> Result<Transformed, Error> {
    if raw.identity != context.identity {
        return Err(Error::Identity);
    }
    if raw.commands.len() > 16384 || raw.commands.len() != raw.sources.len() {
        return Err(Error::Budget);
    }
    let m = context.matrix;
    let mut commands = Vec::with_capacity(raw.commands.len());
    for c in &raw.commands {
        commands.push(match *c {
            CubicCommand::MoveTo(p) => MatrixCommand::MoveTo(map(p, m, false)?),
            CubicCommand::LineTo(p) => MatrixCommand::LineTo(map(p, m, false)?),
            CubicCommand::CurveTo {
                control1,
                control2,
                end,
            } => MatrixCommand::CurveTo {
                control1: map(control1, m, false)?,
                control2: map(control2, m, false)?,
                end: map(end, m, false)?,
            },
            CubicCommand::Close => MatrixCommand::Close,
        })
    }
    Ok(Transformed {
        advance: map(raw.advance, m, true)?,
        sidebearing: map(raw.sidebearing, m, false)?,
        raw,
        context: context.clone(),
        commands,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn header(matrix: &str, paint: u8) -> String {
        format!("4 dict begin /FontName /Test def /FontType 1 def /PaintType {paint} def /FontMatrix [{matrix}] readonly def currentdict end currentfile eexec\n")
    }
    #[test]
    fn exact_nonidentity_translation_and_vector() {
        let (_, m) = parse(header("-2 0 0 0.5 10 -3", 0).as_bytes()).unwrap();
        let p = CubicPoint {
            x: crate::Coordinate::from_integer(4),
            y: crate::Coordinate::from_integer(6),
        };
        assert_eq!(
            map(p, m, false).unwrap(),
            RationalPoint {
                x: Rational::new(2, 1).unwrap(),
                y: Rational::new(0, 1).unwrap()
            }
        );
        assert_eq!(
            map(p, m, true).unwrap(),
            RationalPoint {
                x: Rational::new(-8, 1).unwrap(),
                y: Rational::new(3, 1).unwrap()
            }
        );
    }
    #[test]
    fn executable_missing_stroked_singular_and_overflow_refuse() {
        for s in [
            header("1 0 0 1 0 0", 2),
            header("0 0 0 0 0 0", 0),
            header("1 0 0 1 0 0", 0).replace("/PaintType 0 def", ""),
            header("1 0 0 1 0 0", 0).replace("1 0 0 1 0 0", "1 2 div 0 0 1 0 0"),
            header("1 0 0 1 0 0", 0) + "exec",
        ] {
            assert!(parse(s.as_bytes()).is_err())
        }
        let r = Rational::new(i128::MAX, 1).unwrap();
        let zero = Rational::new(0, 1).unwrap();
        let p = CubicPoint {
            x: crate::Coordinate::from_integer(2),
            y: crate::Coordinate::from_integer(0),
        };
        assert_eq!(
            map(p, [r, zero, zero, r, zero, zero], false),
            Err(Error::Arithmetic)
        );
    }
    #[test]
    fn transformed_output_retains_identity_and_refuses_cross_resource_binding() {
        let license = crate::LicenseMetadata {
            identifier: "LicenseRef-test".into(),
            copyright: "test".into(),
            source: "synthetic".into(),
            text_path: "LICENSE".into(),
            text_sha256: sha256(b"license"),
            embedding_permission: crate::EmbeddingPermission::Unknown,
        };
        let identity = Identity {
            resource_id: "test".into(),
            sha256: sha256(b"font"),
            byte_length: 4,
            license,
        };
        let (_, matrix) = parse(header("2 0 0 -1 10 20", 0).as_bytes()).unwrap();
        let context = Context {
            identity: identity.clone(),
            font_name: "Test".into(),
            matrix,
            header_sha256: sha256(header("2 0 0 -1 10 20", 0).as_bytes()),
        };
        let p = CubicPoint {
            x: crate::Coordinate::from_integer(3),
            y: crate::Coordinate::from_integer(4),
        };
        let raw = || Outline {
            identity: identity.clone(),
            glyph_name: "A".into(),
            charstring_sha256: sha256(b"glyph"),
            sidebearing: p,
            advance: p,
            commands: vec![CubicCommand::MoveTo(p)],
            sources: vec![crate::type1_outline::CommandSource {
                subroutine_chain: vec![2],
                byte_offset: 5,
            }],
            policy: crate::type1_outline::Policy::default(),
            stems: vec![],
        };
        let out = transform(raw(), &context).unwrap();
        assert_eq!(out.raw.identity, identity);
        assert_eq!(out.raw.sources[0].subroutine_chain, vec![2]);
        assert_eq!(out.advance.x, Rational::new(6, 1).unwrap());
        assert_eq!(out.sidebearing.x, Rational::new(16, 1).unwrap());
        let mut wrong = context;
        wrong.identity.resource_id = "other".into();
        assert!(matches!(transform(raw(), &wrong), Err(Error::Identity)));
    }
    #[test]
    fn redefinitions_branches_and_post_eexec_overrides_are_refused() {
        let valid = header("1 0 0 1 0 0", 0);
        for text in [
            valid.replace("/PaintType 0 def", "/PaintType 0 def /PaintType 2 def"),
            valid.replace(
                "/FontMatrix [",
                "false { /PaintType 2 def } if /FontMatrix [",
            ),
            format!("FontDirectory /Test known {{save true}}{{false}}ifelse {valid}"),
        ] {
            assert!(parse(text.as_bytes()).is_err())
        }
        let mut valid_trailer = vec![b'0'; 512];
        valid_trailer.extend(b"\ncleartomark\n");
        assert!(validate_trailer(&valid_trailer).is_ok());
        for tail in [
            b"{restore}if".as_slice(),
            b"/FontMatrix [2 0 0 2 0 0] def",
            b"exec",
        ] {
            let mut tampered = valid_trailer.clone();
            tampered.extend(tail);
            assert!(validate_trailer(&tampered).is_err())
        }
        assert!(validate_trailer(b"cleartomark").is_err());
    }
}

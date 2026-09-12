//! Exact PDF path operator handoff, not another PDF object/xref writer.
//! The current original backend lacks path/content input (issue #25). Decimal
//! output rejects nonterminating rationals and never routes through f64 rounding.
use crate::{
    batch::{ExactClip, PrimitiveId},
    mixed::{BoundedOutput, MixedBatch, MixedLimits},
    mixed_replay::{ReplayBatch, ReplayCommand, ReplayGeometry},
    outlines::{OutlineCoordinate as Q, OutlinePoint as P},
    shaped_replay::{ReplayLimits, ShapedReplay},
    shaped_run::PlacedShapedRun,
    *,
};
use serde_json::{json, Value};
use std::io::Write;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfStreamError {
    Geometry(ValidationError),
    Budget,
    Unsupported(&'static str),
    NonTerminatingDecimal,
    DecimalPrecision,
    PathState,
}
impl From<ValidationError> for PdfStreamError {
    fn from(e: ValidationError) -> Self {
        Self::Geometry(e)
    }
}
pub type PdfResult<T> = std::result::Result<T, PdfStreamError>;
#[derive(Debug, Clone, Copy)]
pub struct PdfPage {
    pub width: Tick,
    pub height: Tick,
}
#[derive(Debug, Clone, Copy)]
pub struct StreamLimits {
    pub max_operators: usize,
    pub max_bytes: usize,
    pub max_decimal_digits: usize,
}
impl Default for StreamLimits {
    fn default() -> Self {
        Self {
            max_operators: 2_000_000,
            max_bytes: MAX_MESSAGE_BYTES,
            max_decimal_digits: 64,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfOperator {
    Save,
    Restore,
    Rectangle([Q; 4]),
    ClipNonZero,
    EndPath,
    FillNonZero,
    FillRgb([Q; 3]),
    Move(P),
    Line(P),
    Cubic { control1: P, control2: P, end: P },
    Close,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorSpan {
    pub primitive: PrimitiveId,
    pub start: usize,
    pub end: usize,
}
pub struct PdfCommandStream {
    page: PdfPage,
    operators: Vec<PdfOperator>,
    spans: Vec<OperatorSpan>,
    source_kind: &'static str,
    source: Vec<u8>,
    source_sha256: String,
    limits: StreamLimits,
}
fn q(n: i128, d: u128) -> Result<Q> {
    Q::from_fraction(n, d)
}
fn sub(a: Q, b: Q) -> Result<Q> {
    a.checked_add(q(-b.numerator(), b.denominator())?)
}
fn rational(v: &Value) -> Result<Q> {
    let a = v
        .as_array()
        .ok_or_else(|| ValidationError("PDF rational array".into()))?;
    require(a.len() == 2, "PDF rational extent")?;
    let n = a[0]
        .as_str()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ValidationError("PDF rational numerator".into()))?;
    let d = a[1]
        .as_str()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ValidationError("PDF rational denominator".into()))?;
    q(n, d)
}
fn rectangle(v: &Value) -> Result<ExactClip> {
    let a = v
        .as_array()
        .ok_or_else(|| ValidationError("PDF rectangle array".into()))?;
    require(a.len() == 4, "PDF rectangle extent")?;
    Ok(ExactClip {
        left: rational(&a[0])?,
        top: rational(&a[1])?,
        right: rational(&a[2])?,
        bottom: rational(&a[3])?,
    })
}
fn exact_color(value: f64) -> PdfResult<Q> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(PdfStreamError::Unsupported("invalid color"));
    }
    if value == 0.0 {
        return Ok(q(0, 1)?);
    }
    let bits = value.to_bits();
    let exponent = ((bits >> 52) & 2047) as i32;
    let mantissa = (bits & ((1u64 << 52) - 1)) | (1u64 << 52);
    let shift = 1075 - exponent;
    if exponent == 0 || !(0..128).contains(&shift) {
        return Err(PdfStreamError::Unsupported(
            "color precision exceeds exact profile",
        ));
    }
    Ok(q(mantissa as i128, 1u128 << shift)?)
}
impl PdfCommandStream {
    pub fn operators(&self) -> &[PdfOperator] {
        &self.operators
    }
    pub fn spans(&self) -> &[OperatorSpan] {
        &self.spans
    }
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
    pub fn page(&self) -> PdfPage {
        self.page
    }
    pub fn from_mixed(batch: &MixedBatch, limits: StreamLimits) -> PdfResult<Self> {
        Self::from_mixed_replay(batch.fixture_bytes(), limits)
    }
    pub fn from_mixed_replay(bytes: &[u8], limits: StreamLimits) -> PdfResult<Self> {
        let replay = ReplayBatch::parse(
            bytes,
            MixedLimits {
                max_commands: limits.max_operators.min(2_000_000),
                max_serialized_bytes: limits.max_bytes,
                ..Default::default()
            },
        )
        .map_err(|e| {
            PdfStreamError::Geometry(ValidationError(format!("mixed PDF input: {e:?}")))
        })?;
        let size = &replay.metadata()["page_size"];
        let page = PdfPage {
            width: Tick(
                size[0]
                    .as_i64()
                    .ok_or_else(|| ValidationError("PDF page width".into()))?,
            ),
            height: Tick(
                size[1]
                    .as_i64()
                    .ok_or_else(|| ValidationError("PDF page height".into()))?,
            ),
        };
        let mut stream = Self::start(page, "mixed", bytes, limits)?;
        for (i, p) in replay.primitives().iter().enumerate() {
            let paint: Paint =
                serde_json::from_value(replay.metadata()["primitives"][i]["paint"].clone())
                    .map_err(|e| ValidationError(e.to_string()))?;
            stream.primitive(p.identity, &p.geometry, p.clip, paint)?;
        }
        Ok(stream)
    }
    pub fn from_shaped(
        run: &PlacedShapedRun,
        page: PdfPage,
        paint: Paint,
        limits: StreamLimits,
    ) -> PdfResult<Self> {
        let bytes = run.replay_bytes(limits.max_bytes)?;
        Self::from_shaped_replay(&bytes, page, paint, limits)
    }
    pub fn from_shaped_replay(
        bytes: &[u8],
        page: PdfPage,
        paint: Paint,
        limits: StreamLimits,
    ) -> PdfResult<Self> {
        let replay = ShapedReplay::parse(
            bytes,
            ReplayLimits {
                max_bytes: limits.max_bytes,
                max_commands: limits.max_operators.min(2_000_000),
                ..Default::default()
            },
        )?;
        let clip = rectangle(&replay.metadata()["clip"])?;
        let mut stream = Self::start(page, "shaped", bytes, limits)?;
        for i in 0..replay.glyph_count() {
            let id = &replay.metadata()["primitives"][i]["identity"];
            let identity = PrimitiveId {
                item_index: id["item_index"]
                    .as_u64()
                    .ok_or_else(|| ValidationError("PDF item ID".into()))?
                    as usize,
                glyph_index: Some(
                    id["glyph_index"]
                        .as_u64()
                        .ok_or_else(|| ValidationError("PDF glyph ID".into()))?
                        as usize,
                ),
            };
            stream.primitive(identity, &replay.geometry(i)?, clip, paint.clone())?;
        }
        Ok(stream)
    }
    fn start(
        page: PdfPage,
        kind: &'static str,
        bytes: &[u8],
        limits: StreamLimits,
    ) -> PdfResult<Self> {
        page.width.positive()?;
        page.height.positive()?;
        if !(1..=2_000_000).contains(&limits.max_operators)
            || !(1..=MAX_MESSAGE_BYTES).contains(&limits.max_bytes)
            || !(1..=128).contains(&limits.max_decimal_digits)
            || bytes.len() > limits.max_bytes
        {
            return Err(PdfStreamError::Budget);
        }
        let mut s = Self {
            page,
            operators: vec![],
            spans: vec![],
            source_kind: kind,
            source: bytes.into(),
            source_sha256: digest(bytes),
            limits,
        };
        // Explicit white paper independent of any preview theme. Document primitives
        // can subsequently paint a deliberately authored background.
        s.push(PdfOperator::Save)?;
        s.push(PdfOperator::FillRgb([q(1, 1)?; 3]))?;
        s.push(PdfOperator::Rectangle([
            q(0, 1)?,
            q(0, 1)?,
            q(page.width.0 as i128, TICKS_PER_BP as u128)?,
            q(page.height.0 as i128, TICKS_PER_BP as u128)?,
        ]))?;
        s.push(PdfOperator::FillNonZero)?;
        s.push(PdfOperator::Restore)?;
        Ok(s)
    }
    fn push(&mut self, operator: PdfOperator) -> PdfResult<()> {
        if self.operators.len() >= self.limits.max_operators {
            return Err(PdfStreamError::Budget);
        }
        self.operators.push(operator);
        Ok(())
    }
    fn point(&self, p: P) -> Result<P> {
        let unit = q(1, TICKS_PER_BP as u128)?;
        Ok(P {
            x: p.x.checked_multiply(unit)?,
            y: sub(q(self.page.height.0 as i128, 1)?, p.y)?.checked_multiply(unit)?,
        })
    }
    fn rect(&self, r: ExactClip) -> Result<[Q; 4]> {
        r.validate()?;
        let p = self.point(P {
            x: r.left,
            y: r.bottom,
        })?;
        let unit = q(1, TICKS_PER_BP as u128)?;
        Ok([
            p.x,
            p.y,
            sub(r.right, r.left)?.checked_multiply(unit)?,
            sub(r.bottom, r.top)?.checked_multiply(unit)?,
        ])
    }
    fn primitive(
        &mut self,
        id: PrimitiveId,
        geometry: &ReplayGeometry,
        clip: ExactClip,
        paint: Paint,
    ) -> PdfResult<()> {
        paint.validate()?;
        if paint.a != 1.0 {
            return Err(PdfStreamError::Unsupported(
                "PDF alpha graphics-state API missing",
            ));
        }
        let rgb = [
            exact_color(paint.r)?,
            exact_color(paint.g)?,
            exact_color(paint.b)?,
        ];
        let start = self.operators.len();
        self.push(PdfOperator::Save)?;
        self.push(PdfOperator::Rectangle(self.rect(clip)?))?;
        self.push(PdfOperator::ClipNonZero)?;
        self.push(PdfOperator::EndPath)?;
        self.push(PdfOperator::FillRgb(rgb))?;
        match geometry {
            ReplayGeometry::Rule(r) => self.push(PdfOperator::Rectangle(self.rect(*r)?))?,
            ReplayGeometry::Quadratic(commands) | ReplayGeometry::Cubic(commands) => {
                let mut current = None;
                let mut first = None;
                for c in commands {
                    match c {
                        ReplayCommand::Move(p) => {
                            let p = self.point(*p)?;
                            current = Some(p);
                            first = Some(p);
                            self.push(PdfOperator::Move(p))?;
                        }
                        ReplayCommand::Line(p) => {
                            if current.is_none() {
                                return Err(PdfStreamError::PathState);
                            }
                            let p = self.point(*p)?;
                            current = Some(p);
                            self.push(PdfOperator::Line(p))?;
                        }
                        ReplayCommand::Quadratic { control, end } => {
                            let begin = current.ok_or(PdfStreamError::PathState)?;
                            let control = self.point(*control)?;
                            let end = self.point(*end)?;
                            // Exact degree elevation: C1=P0+2/3(Q-P0), C2=P2+2/3(Q-P2).
                            let elevate = |p: P| -> Result<P> {
                                let k = q(2, 3)?;
                                Ok(P {
                                    x: p.x
                                        .checked_add(sub(control.x, p.x)?.checked_multiply(k)?)?,
                                    y: p.y
                                        .checked_add(sub(control.y, p.y)?.checked_multiply(k)?)?,
                                })
                            };
                            self.push(PdfOperator::Cubic {
                                control1: elevate(begin)?,
                                control2: elevate(end)?,
                                end,
                            })?;
                            current = Some(end);
                        }
                        ReplayCommand::Cubic {
                            control1,
                            control2,
                            end,
                        } => {
                            if current.is_none() {
                                return Err(PdfStreamError::PathState);
                            }
                            let end = self.point(*end)?;
                            self.push(PdfOperator::Cubic {
                                control1: self.point(*control1)?,
                                control2: self.point(*control2)?,
                                end,
                            })?;
                            current = Some(end);
                        }
                        ReplayCommand::Close => {
                            current = Some(first.ok_or(PdfStreamError::PathState)?);
                            self.push(PdfOperator::Close)?;
                        }
                    }
                }
            }
        }
        self.push(PdfOperator::FillNonZero)?;
        self.push(PdfOperator::Restore)?;
        self.spans.push(OperatorSpan {
            primitive: id,
            start,
            end: self.operators.len(),
        });
        Ok(())
    }
    /// PDF content operators only: not a standalone document. Font GID/source
    /// provenance is retained in evidence_bytes, not remapped to Unicode text.
    pub fn content_bytes(&self) -> PdfResult<Vec<u8>> {
        let mut output = BoundedOutput {
            bytes: vec![],
            limit: self.limits.max_bytes,
        };
        for operator in &self.operators {
            let (operands, tag): (Vec<Q>, &str) = match operator {
                PdfOperator::Save => (vec![], "q"),
                PdfOperator::Restore => (vec![], "Q"),
                PdfOperator::Rectangle(v) => (v.to_vec(), "re"),
                PdfOperator::ClipNonZero => (vec![], "W"),
                PdfOperator::EndPath => (vec![], "n"),
                PdfOperator::FillNonZero => (vec![], "f"),
                PdfOperator::FillRgb(v) => (v.to_vec(), "rg"),
                PdfOperator::Move(p) => (vec![p.x, p.y], "m"),
                PdfOperator::Line(p) => (vec![p.x, p.y], "l"),
                PdfOperator::Cubic {
                    control1,
                    control2,
                    end,
                } => (
                    vec![control1.x, control1.y, control2.x, control2.y, end.x, end.y],
                    "c",
                ),
                PdfOperator::Close => (vec![], "h"),
            };
            for value in operands {
                output
                    .write_all(decimal(value, self.limits.max_decimal_digits)?.as_bytes())
                    .map_err(|_| PdfStreamError::Budget)?;
                output.write_all(b" ").map_err(|_| PdfStreamError::Budget)?;
            }
            output
                .write_all(tag.as_bytes())
                .map_err(|_| PdfStreamError::Budget)?;
            output
                .write_all(b"\n")
                .map_err(|_| PdfStreamError::Budget)?;
        }
        Ok(output.bytes)
    }
    pub fn evidence_bytes(&self) -> PdfResult<Vec<u8>> {
        let source = crate::mixed_replay::parse_unique(&self.source)
            .map_err(|e| ValidationError(e.to_string()))?;
        struct Evidence<'a> {
            stream: &'a PdfCommandStream,
            source: &'a Value,
        }
        impl serde::Serialize for Evidence<'_> {
            fn serialize<S: serde::Serializer>(
                &self,
                serializer: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                use serde::ser::SerializeStruct;
                let mut out = serializer.serialize_struct("PdfOperatorEvidence", 11)?;
                out.serialize_field("format", "flashtex-pdf-operator-evidence-v1")?;
                out.serialize_field("source_sha256", &self.stream.source_sha256)?;
                out.serialize_field("source_kind", &self.stream.source_kind)?;
                out.serialize_field("original_fixture", self.source)?;
                out.serialize_field(
                    "page_ticks",
                    &[self.stream.page.width.0, self.stream.page.height.0],
                )?;
                out.serialize_field("operators", &ExactOperators(&self.stream.operators))?;
                out.serialize_field(
                    "primitive_spans",
                    &self
                        .stream
                        .spans
                        .iter()
                        .map(|s| json!({"identity":s.primitive,"start":s.start,"end":s.end}))
                        .collect::<Vec<_>>(),
                )?;
                out.serialize_field("paper", "white")?;
                out.serialize_field("preview_theme_applied", &false)?;
                out.serialize_field("resources_verified", &false)?;
                out.serialize_field("standalone_pdf", &false)?;
                out.end()
            }
        }
        let mut output = BoundedOutput {
            bytes: vec![],
            limit: self.limits.max_bytes,
        };
        serde_json::to_writer(
            &mut output,
            &Evidence {
                stream: self,
                source: &source,
            },
        )
        .map_err(|_| PdfStreamError::Budget)?;
        Ok(output.bytes)
    }
}
/// Exact finite decimal spelling, without exponent notation or rounding.
pub fn decimal(value: Q, max_digits: usize) -> PdfResult<String> {
    if max_digits > 128 {
        return Err(PdfStreamError::DecimalPrecision);
    }
    let d = value.denominator();
    let mut factor = d;
    while factor.is_multiple_of(2) {
        factor /= 2
    }
    while factor.is_multiple_of(5) {
        factor /= 5
    }
    if factor != 1 {
        return Err(PdfStreamError::NonTerminatingDecimal);
    }
    let n = value.numerator().unsigned_abs();
    let mut result = if value.numerator() < 0 {
        "-".to_owned()
    } else {
        String::new()
    };
    result.push_str(&(n / d).to_string());
    let mut remainder = n % d;
    if remainder > 0 {
        result.push('.');
    }
    let mut digits = 0;
    while remainder > 0 {
        if digits >= max_digits {
            return Err(PdfStreamError::DecimalPrecision);
        }
        let mut next = 0;
        let mut digit = 0;
        // 10*remainder without overflowing u128, since d <= i128::MAX.
        for _ in 0..10 {
            if next >= d - remainder {
                next -= d - remainder;
                digit += 1;
            } else {
                next += remainder;
            }
        }
        result.push(char::from(b'0' + digit));
        remainder = next;
        digits += 1;
    }
    Ok(result)
}

struct ExactOperators<'a>(&'a [PdfOperator]);
impl serde::Serialize for ExactOperators<'_> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut out = serializer.serialize_seq(Some(self.0.len()))?;
        let q = |v: Q| json!([v.numerator().to_string(), v.denominator().to_string()]);
        let p = |v: P| json!([q(v.x), q(v.y)]);
        for op in self.0 {
            let value = match op {
                PdfOperator::Save => json!(["q"]),
                PdfOperator::Restore => json!(["Q"]),
                PdfOperator::Rectangle(v) => {
                    json!(["re", v.iter().map(|v| q(*v)).collect::<Vec<_>>()])
                }
                PdfOperator::ClipNonZero => json!(["W"]),
                PdfOperator::EndPath => json!(["n"]),
                PdfOperator::FillNonZero => json!(["f"]),
                PdfOperator::FillRgb(v) => {
                    json!(["rg", v.iter().map(|v| q(*v)).collect::<Vec<_>>()])
                }
                PdfOperator::Move(v) => json!(["m", p(*v)]),
                PdfOperator::Line(v) => json!(["l", p(*v)]),
                PdfOperator::Cubic {
                    control1,
                    control2,
                    end,
                } => json!(["c", p(*control1), p(*control2), p(*end)]),
                PdfOperator::Close => json!(["h"]),
            };
            out.serialize_element(&value)?;
        }
        out.end()
    }
}

//! Experimental explicitly encoded TFM/VF positioning. Exact arithmetic is not
//! TeX's scaled-point rounding algorithm; callers must select this policy.
use crate::{outlines::OutlineCoordinate, Tick, ValidationError};
use flashtex_font_resources::{
    encoding::{BoundTfmFont, GlyphIdentity, MappedItem},
    tfm::{CharacterMetrics, FixWord, Tfm, TfmItem},
    vf::{Placement, VirtualFont},
    FontResource,
};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricPolicy {
    ExactRationalNoTexRounding,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    Resource(String),
    UnsupportedFont(String),
    UnsupportedPath(crate::glyph_cache::PathFailure),
    Arithmetic(String),
    UnsupportedNotdef { code: u8 },
    NonIntegralTicks,
    Budget,
}
pub type Result<T> = std::result::Result<T, AdapterError>;
impl From<ValidationError> for AdapterError {
    fn from(v: ValidationError) -> Self {
        Self::Arithmetic(v.0)
    }
}
fn resource(e: flashtex_font_resources::Error) -> AdapterError {
    match e {
        flashtex_font_resources::Error::UnsupportedFont(reason) => {
            AdapterError::UnsupportedFont(reason)
        }
        other => AdapterError::Resource(other.to_string()),
    }
}
/// Canonical page ticks as an exact reduced fraction; never implicitly rounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExactTicks(OutlineCoordinate);
impl ExactTicks {
    pub fn integer(value: Tick) -> Result<Self> {
        value.validate()?;
        Ok(Self(OutlineCoordinate::new(value.0 as i128, 1)?))
    }
    pub fn numerator(self) -> i128 {
        self.0.numerator()
    }
    pub fn denominator(self) -> u128 {
        self.0.denominator()
    }
    fn fraction(n: i128, d: u128) -> Result<Self> {
        Ok(Self(OutlineCoordinate::new(n, d)?))
    }
    fn add(self, other: Self) -> Result<Self> {
        let d = gcd(self.denominator(), other.denominator());
        let a = other.denominator() / d;
        let b = self.denominator() / d;
        let n = self
            .numerator()
            .checked_mul(a as i128)
            .and_then(|n| {
                other
                    .numerator()
                    .checked_mul(b as i128)
                    .and_then(|m| n.checked_add(m))
            })
            .ok_or_else(|| AdapterError::Arithmetic("metric addition overflow".into()))?;
        Self::fraction(
            n,
            self.denominator()
                .checked_mul(a)
                .ok_or_else(|| AdapterError::Arithmetic("metric denominator overflow".into()))?,
        )
    }
    fn multiply(self, n: i128, d: u128) -> Result<Self> {
        let a = gcd(self.numerator().unsigned_abs(), d);
        let b = gcd(n.unsigned_abs(), self.denominator());
        let n = (self.numerator() / a as i128)
            .checked_mul(n / b as i128)
            .ok_or_else(|| AdapterError::Arithmetic("metric product overflow".into()))?;
        Self::fraction(
            n,
            (self.denominator() / b)
                .checked_mul(d / a)
                .ok_or_else(|| AdapterError::Arithmetic("metric precision overflow".into()))?,
        )
    }
    pub fn require_integer(self) -> Result<Tick> {
        if self.denominator() != 1 {
            return Err(AdapterError::NonIntegralTicks);
        }
        Ok(Tick(
            i64::try_from(self.numerator()).map_err(|_| AdapterError::NonIntegralTicks)?,
        ))
    }
}
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}
#[derive(Debug, Clone, Copy)]
pub struct RunScale {
    size: ExactTicks,
    policy: MetricPolicy,
}
impl RunScale {
    pub fn canonical(size: Tick, policy: MetricPolicy) -> Result<Self> {
        size.positive()?;
        Ok(Self {
            size: ExactTicks::integer(size)?,
            policy,
        })
    }
    /// TFM design size is 12.20 TeX points. 1 TeX point = 7200/7227 bp.
    pub fn design_size(size: FixWord, policy: MetricPolicy) -> Result<Self> {
        if size.0 <= 0 {
            return Err(AdapterError::Arithmetic("nonpositive font size".into()));
        }
        Ok(Self {
            size: ExactTicks::fraction(i128::from(size.0) * 7200, 7227)?,
            policy,
        })
    }
    fn metric(self, metric: FixWord) -> Result<ExactTicks> {
        self.size.multiply(metric.0 as i128, 1 << 20)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputInterval {
    pub start: usize,
    pub end: usize,
}
#[derive(Debug)]
pub struct PhysicalGlyph<'a> {
    pub font: &'a FontResource,
    pub tfm_sha256: String,
    pub code: u8,
    pub original_gid: u16,
    pub x: ExactTicks,
    pub baseline_y: ExactTicks,
    pub size: ExactTicks,
    pub input: InputInterval,
    pub metrics: CharacterMetrics,
}
#[derive(Debug)]
pub enum Operation<'a> {
    Glyph(PhysicalGlyph<'a>),
    Rule {
        x: ExactTicks,
        top: ExactTicks,
        width: ExactTicks,
        height: ExactTicks,
        input: InputInterval,
    },
}
#[derive(Debug)]
pub struct EncodedRun<'a> {
    pub operations: Vec<Operation<'a>>,
    pub advance: ExactTicks,
    pub policy: MetricPolicy,
    pub vf_sha256: Option<String>,
}
const LIMIT: usize = 100000;
fn original(code: u8, id: GlyphIdentity) -> Result<u16> {
    match id {
        GlyphIdentity::Original(id) => Ok(id),
        GlyphIdentity::Notdef => Err(AdapterError::UnsupportedNotdef { code }),
    }
}
pub fn physical_run<'a>(
    binding: &BoundTfmFont<'a>,
    input: &[u8],
    scale: RunScale,
) -> Result<EncodedRun<'a>> {
    if input.len() > LIMIT {
        return Err(AdapterError::Budget);
    }
    let mut x = ExactTicks::integer(Tick(0))?;
    let mut operations = Vec::new();
    for item in binding.map_run(input).map_err(resource)? {
        match item {
            MappedItem::Kern(kern) => x = x.add(scale.metric(kern)?)?,
            MappedItem::Glyph {
                tfm_code,
                identity,
                metrics,
                input_start,
                input_end,
            } => {
                if operations.len() >= LIMIT {
                    return Err(AdapterError::Budget);
                }
                operations.push(Operation::Glyph(PhysicalGlyph {
                    font: binding.font(),
                    tfm_sha256: binding.tfm().source_sha256.clone(),
                    code: tfm_code,
                    original_gid: original(tfm_code, identity)?,
                    x,
                    baseline_y: ExactTicks::integer(Tick(0))?,
                    size: scale.size,
                    input: InputInterval {
                        start: input_start,
                        end: input_end,
                    },
                    metrics,
                }));
                x = x.add(scale.metric(metrics.width)?)?;
            }
        }
    }
    Ok(EncodedRun {
        operations,
        advance: x,
        policy: scale.policy,
        vf_sha256: None,
    })
}
pub fn virtual_run<'a>(
    vf: &VirtualFont,
    tfm: &Tfm,
    bindings: &BTreeMap<i32, BoundTfmFont<'a>>,
    input: &[u8],
    scale: RunScale,
) -> Result<EncodedRun<'a>> {
    if input.len() > LIMIT {
        return Err(AdapterError::Budget);
    }
    let mut x = ExactTicks::integer(Tick(0))?;
    let mut operations = Vec::new();
    let mut vf_sha256 = None;
    for item in tfm.apply_ligatures_kerns(input).map_err(resource)? {
        let glyph = match item {
            TfmItem::Kern(k) => {
                x = x.add(scale.metric(k)?)?;
                continue;
            }
            TfmItem::Glyph(g) => g,
        };
        let packet = vf
            .expand_packet(glyph.code, tfm, bindings)
            .map_err(resource)?;
        vf_sha256 = Some(packet.vf_sha256);
        let interval = InputInterval {
            start: glyph.input_start,
            end: glyph.input_end,
        };
        for placement in packet.placements {
            if operations.len() >= LIMIT {
                return Err(AdapterError::Budget);
            }
            let convert = |v: flashtex_font_resources::Coordinate| {
                scale.size.multiply(v.numerator(), 1u128 << v.shift())
            };
            match placement {
                Placement::Glyph {
                    local_font,
                    font_sha256,
                    tfm_sha256,
                    face_index,
                    glyph_id,
                    tfm_code,
                    x: px,
                    y,
                    scale: local_scale,
                } => {
                    let binding = bindings.get(&local_font).ok_or_else(|| {
                        AdapterError::Resource("missing bound virtual font".into())
                    })?;
                    let (identity, metrics) = binding.map_code(tfm_code).map_err(resource)?;
                    if binding.font().descriptor().sha256 != font_sha256
                        || binding.font().descriptor().face_index != face_index
                        || binding.tfm().source_sha256 != tfm_sha256
                        || original(tfm_code, identity)? != glyph_id
                    {
                        return Err(AdapterError::Resource(
                            "virtual placement resource mismatch".into(),
                        ));
                    }
                    operations.push(Operation::Glyph(PhysicalGlyph {
                        font: binding.font(),
                        tfm_sha256,
                        code: tfm_code,
                        original_gid: glyph_id,
                        x: x.add(convert(px)?)?,
                        baseline_y: convert(y)?,
                        size: scale.metric(local_scale)?,
                        input: interval.clone(),
                        metrics,
                    }));
                }
                Placement::Rule {
                    x: px,
                    y,
                    width,
                    height,
                } => {
                    let height = convert(height)?;
                    operations.push(Operation::Rule {
                        x: x.add(convert(px)?)?,
                        top: convert(y)?.add(height.multiply(-1, 1)?)?,
                        width: convert(width)?,
                        height,
                        input: interval.clone(),
                    });
                }
            }
        }
        x = x.add(scale.metric(packet.width)?)?;
    }
    Ok(EncodedRun {
        operations,
        advance: x,
        policy: scale.policy,
        vf_sha256,
    })
}

/// Explicit mapping from encoded input intervals to logical UTF-8 and TeX source.
/// Ligature intervals require their own supplied mapping; no interpolation occurs.
#[derive(Debug, Clone)]
pub struct Provenance {
    pub input: InputInterval,
    pub logical_start: u64,
    pub logical_end: u64,
    pub sources: Vec<crate::SourceRange>,
    pub synthetic_reason: Option<String>,
}
pub struct BatchContext<'a> {
    pub project_id: &'a str,
    pub revision: u64,
    pub page: u32,
    pub page_width: Tick,
    pub page_height: Tick,
    pub origin: crate::hit_test::Point,
    pub clip: Option<&'a crate::HitRect>,
    pub paint: crate::Paint,
    pub logical_text: &'a str,
    pub provenance: &'a [Provenance],
    pub documents: &'a [crate::DocumentResource],
    pub snapshots: &'a BTreeMap<String, crate::SourceSnapshot>,
}
impl EncodedRun<'_> {
    /// Convert exact rational positions to internal path geometry without changing
    /// the integer wire contract. Source mappings must be explicitly supplied.
    pub fn batch(
        &self,
        context: &BatchContext<'_>,
        limits: crate::batch::BatchLimits,
        cache: &mut crate::glyph_cache::GlyphPathCache,
    ) -> Result<crate::batch::DrawBatch> {
        use crate::{
            batch::{DrawBatch, DrawOperation},
            glyph_cache::PathOutcome,
            outlines::{place_path_exact, OutlinePoint, PositionedGlyph},
            require, HitRect,
        };
        require(
            (1..=100000).contains(&limits.max_operations)
                && (1..=2_000_000).contains(&limits.max_path_commands),
            "invalid encoded batch budget",
        )?;
        crate::id(context.project_id)?;
        require(context.page > 0, "invalid encoded batch page")?;
        context.page_width.positive()?;
        context.page_height.positive()?;
        context.paint.validate()?;
        context.origin.x.validate()?;
        context.origin.y.validate()?;
        require(
            context.provenance.len() <= LIMIT
                && context.logical_text.len() <= crate::MAX_MESSAGE_BYTES,
            "encoded provenance budget",
        )?;
        let mut docs = BTreeMap::new();
        for document in context.documents {
            let snapshot = context
                .snapshots
                .get(&document.path)
                .ok_or_else(|| AdapterError::Resource("missing source snapshot".into()))?;
            require(
                snapshot.revision == document.revision
                    && snapshot.text.len() as u64 == document.byte_length
                    && crate::digest(snapshot.text.as_bytes()) == document.sha256,
                "encoded source identity mismatch",
            )?;
            require(
                docs.insert(document.path.as_str(), document).is_none(),
                "duplicate encoded document",
            )?;
        }
        let mut provenance = BTreeMap::new();
        for p in context.provenance {
            require(
                p.input.start < p.input.end && p.logical_start < p.logical_end,
                "invalid encoded input/logical interval",
            )?;
            crate::boundary(context.logical_text, p.logical_start)?;
            crate::boundary(context.logical_text, p.logical_end)?;
            require(
                !p.sources.is_empty() || p.synthetic_reason.as_ref().is_some_and(|s| !s.is_empty()),
                "missing encoded source provenance",
            )?;
            require(p.sources.len() <= 128, "encoded source range budget")?;
            for range in &p.sources {
                crate::source(range, &docs)?;
            }
            crate::validate_source_bytes(&p.sources, context.snapshots)?;
            require(
                provenance.insert((p.input.start, p.input.end), p).is_none(),
                "duplicate encoded interval mapping",
            )?;
        }
        let full = HitRect {
            x: Tick(0),
            top: Tick(0),
            width: context.page_width,
            height: context.page_height,
        };
        let clip = if let Some(c) = context.clip {
            c.validate()?;
            crate::batch::intersection(&full, c)
        } else {
            Some(full)
        };
        let mut batch = DrawBatch {
            project_id: context.project_id.into(),
            revision: context.revision,
            page: context.page,
            page_width: context.page_width,
            page_height: context.page_height,
            visible_clip: clip,
            operations: vec![],
            path_commands: 0,
            color_space: "srgb",
            compositing: "source-over",
            hinting_applied: false,
        };
        if batch.visible_clip.is_none() {
            return Ok(batch);
        }
        for (index, op) in self.operations.iter().enumerate() {
            if batch.operations.len() >= limits.max_operations {
                return Err(AdapterError::Budget);
            }
            let interval = match op {
                Operation::Glyph(g) => &g.input,
                Operation::Rule { input, .. } => input,
            };
            let p = provenance
                .get(&(interval.start, interval.end))
                .ok_or_else(|| {
                    AdapterError::Resource("missing exact input interval provenance".into())
                })?;
            let origin_x = ExactTicks::integer(context.origin.x)?;
            let origin_y = ExactTicks::integer(context.origin.y)?;
            match op {
                Operation::Glyph(g) => {
                    let origin = OutlinePoint {
                        x: g.x.add(origin_x)?.0,
                        y: g.baseline_y.add(origin_y)?.0,
                    };
                    let size = g.size.0;
                    let cached = cache.lookup(g.font, g.original_gid)?;
                    let PathOutcome::Ready(path) = cached.outcome else {
                        return Err(match cached.outcome {
                            PathOutcome::Unavailable(reason) => {
                                AdapterError::UnsupportedPath((*reason).clone())
                            }
                            PathOutcome::Ready(_) => unreachable!(),
                        });
                    };
                    if batch
                        .path_commands
                        .checked_add(path.commands.len())
                        .is_none_or(|n| n > limits.max_path_commands)
                    {
                        return Err(AdapterError::Budget);
                    }
                    let commands = place_path_exact(
                        path.commands.iter().cloned(),
                        size,
                        g.font.descriptor().units_per_em,
                        origin,
                    )?;
                    batch.path_commands += commands.len();
                    batch.operations.push(DrawOperation::Glyph {
                        path: Box::new(PositionedGlyph {
                            project_id: context.project_id.into(),
                            revision: context.revision,
                            page: context.page,
                            item_index: index,
                            glyph_index: 0,
                            font_id: g.font.descriptor().font_id.clone(),
                            font_sha256: g.font.descriptor().sha256.clone(),
                            original_gid: g.original_gid as u32,
                            cluster_index: index as u32,
                            logical_start_byte: p.logical_start,
                            logical_end_byte: p.logical_end,
                            sources: p.sources.clone(),
                            synthetic_reason: p.synthetic_reason.clone(),
                            instances: path.instances.clone(),
                            commands,
                            hinting_applied: false,
                        }),
                        paint: context.paint.clone(),
                    });
                }
                Operation::Rule {
                    x,
                    top,
                    width,
                    height,
                    ..
                } => {
                    let left = x.add(origin_x)?;
                    let top = top.add(origin_y)?;
                    let geometry = crate::batch::ExactClip {
                        left: left.0,
                        top: top.0,
                        right: left.add(*width)?.0,
                        bottom: top.add(*height)?.0,
                    };
                    geometry.validate()?;
                    let clip =
                        crate::batch::ExactClip::from_rect(batch.visible_clip.as_ref().unwrap())?;
                    if geometry.intersect(clip)?.is_some() {
                        batch.operations.push(DrawOperation::ExactRule {
                            geometry,
                            paint: context.paint.clone(),
                            sources: p.sources.clone(),
                            synthetic_reason: p.synthetic_reason.clone(),
                        });
                    }
                }
            }
        }
        Ok(batch)
    }
}

impl EncodedRun<'_> {
    pub fn batch_with_exact_clip(
        &self,
        context: &BatchContext<'_>,
        clip: crate::batch::ExactClip,
        limits: crate::batch::BatchLimits,
        cache: &mut crate::glyph_cache::GlyphPathCache,
    ) -> Result<crate::batch::ExactDrawBatch> {
        clip.validate()?;
        let mut batch = self.batch(context, limits, cache)?;
        let visible_clip = match &batch.visible_clip {
            Some(integer) => crate::batch::ExactClip::from_rect(integer)?.intersect(clip)?,
            None => None,
        };
        if visible_clip.is_none() {
            batch.operations.clear();
            batch.path_commands = 0;
        }
        Ok(crate::batch::ExactDrawBatch {
            batch,
            visible_clip,
        })
    }
}

//! Opt-in placement of a fitted MATH construction. Caller supplies baseline;
//! vertical offsets progress upward from that origin, horizontal offsets right.
use super::*;
use crate::{
    batch::PrimitiveId,
    cubic::CubicProvider,
    glyph_cache::PathOutcome,
    mixed::{MixedBatch, MixedContext, MixedError, MixedGeometry, MixedLimits, MixedPrimitive},
    outlines::{place_path_exact, OutlinePoint as P},
};
use flashtex_font_resources::{
    cff::HintPolicy,
    math_adapter::BoundMathFit,
    math_fit::{FitError, FitLimits, FitStrategy, FittedShape},
    math_variants::Direction,
};
#[derive(Debug)]
pub enum AssemblyError {
    Consumer(MathConsumerError),
    Fit(FitError),
    Budget,
    UnsupportedDeviceAdjustment,
}
impl From<MathConsumerError> for AssemblyError {
    fn from(e: MathConsumerError) -> Self {
        Self::Consumer(e)
    }
}
impl From<ValidationError> for AssemblyError {
    fn from(e: ValidationError) -> Self {
        Self::Consumer(e.into())
    }
}
pub struct AssemblyRequest<'a> {
    pub metrics: MathQuery<'a>,
    pub direction: Direction,
    pub original_gid: u16,
    /// Exact design units, deliberately independent of page font size.
    pub target: Rational,
    pub strategy: FitStrategy,
    pub fit_limits: FitLimits,
    pub page: MixedContext<'a>,
    pub origin: P,
    pub paint: Paint,
    pub hint_policy: HintPolicy,
}
pub struct MathAssemblyFrame {
    metrics: MathMetricsSnapshot,
    fit: BoundMathFit,
    batch: MixedBatch,
}
impl MathAssemblyFrame {
    pub fn metrics(&self) -> &MathMetricsSnapshot {
        &self.metrics
    }
    pub fn fit(&self) -> &BoundMathFit {
        &self.fit
    }
    pub fn batch(&self) -> &MixedBatch {
        &self.batch
    }
    pub fn require_current(
        &self,
        r: &RegistryRenderer,
        path: &str,
        s: &SourceSnapshot,
    ) -> MathResult<()> {
        self.metrics.require_current(r, path, s)
    }
}
impl RegistryRenderer {
    pub fn math_assembly(
        &self,
        lease: &MathLease,
        request: AssemblyRequest<'_>,
        limits: MixedLimits,
    ) -> std::result::Result<MathAssemblyFrame, AssemblyError> {
        if !(1..=100000).contains(&limits.max_primitives)
            || !(1..=2_000_000).contains(&limits.max_commands)
            || !(1..=MAX_MESSAGE_BYTES).contains(&limits.max_serialized_bytes)
        {
            return Err(AssemblyError::Budget);
        }
        let metrics = self.math_metrics(lease, request.metrics)?;
        require(
            request.page.revision == metrics.source.revision,
            "MATH page/source revision",
        )?;
        request.paint.validate()?;
        request.page.clip.validate()?;
        let variants = lease.font.variants().map_err(MathConsumerError::from)?;
        let fit = variants
            .fit(
                request.direction,
                request.original_gid,
                request.target,
                request.strategy,
                request.fit_limits,
            )
            .map_err(AssemblyError::Fit)?;
        let parts: Vec<_> = match &fit.fit().shape {
            FittedShape::Variant(v) => vec![(v.glyph_id, Rational::new(0, 1).expect("zero"))],
            FittedShape::Assembly(a) => {
                if a.device_adjustment_present {
                    return Err(AssemblyError::UnsupportedDeviceAdjustment);
                }
                a.parts.iter().map(|p| (p.glyph_id, p.offset)).collect()
            }
        };
        if parts.len() > limits.max_primitives {
            return Err(AssemblyError::Budget);
        }
        let mut primitives = Vec::new();
        let mut commands = 0usize;
        for (index, (gid, offset)) in parts.into_iter().enumerate() {
            let displacement = exact(offset)?
                .checked_multiply(metrics.size)?
                .checked_multiply(Q::from_fraction(1, lease.font.units_per_em() as u128)?)?;
            let origin = match request.direction {
                Direction::Horizontal => P {
                    x: request.origin.x.checked_add(displacement)?,
                    y: request.origin.y,
                },
                Direction::Vertical => P {
                    x: request.origin.x,
                    y: request.origin.y.checked_add(Q::from_fraction(
                        -displacement.numerator(),
                        displacement.denominator(),
                    )?)?,
                },
            };
            let remaining = limits.max_commands.saturating_sub(commands);
            let geometry = match &lease.render.resource.backend {
                Backend::TrueType { resource, cache } => {
                    let mut cache = cache
                        .lock()
                        .map_err(|_| MathConsumerError::Binding(BindingError::CacheUnavailable))?;
                    let path = match cache
                        .lookup(resource, gid)
                        .map_err(MathConsumerError::from)?
                        .outcome
                    {
                        PathOutcome::Ready(p) => p,
                        PathOutcome::Unavailable(e) => {
                            return Err(
                                ValidationError(format!("MATH outline unavailable: {e:?}")).into()
                            )
                        }
                    };
                    if path.commands.len() > remaining {
                        return Err(AssemblyError::Budget);
                    }
                    MixedGeometry::Quadratic(place_path_exact(
                        path.commands.iter().cloned(),
                        metrics.size,
                        resource.descriptor().units_per_em,
                        origin,
                    )?)
                }
                Backend::Cff { resource } => MixedGeometry::Cubic(Box::new(
                    resource
                        .place_glyph(gid, request.hint_policy, metrics.size, origin, remaining)
                        .map_err(|e| match e {
                            crate::cubic::CubicError::Budget => AssemblyError::Budget,
                            other => {
                                ValidationError(format!("MATH cubic unavailable: {other:?}")).into()
                            }
                        })?,
                )),
            };
            commands += match &geometry {
                MixedGeometry::Quadratic(v) => v.len(),
                MixedGeometry::Cubic(v) => v.commands.len(),
                MixedGeometry::Rule(_) => 0,
            };
            if commands > limits.max_commands {
                return Err(AssemblyError::Budget);
            }
            primitives.push(MixedPrimitive {
                identity: PrimitiveId {
                    item_index: index,
                    glyph_index: Some(0),
                },
                geometry,
                clip: request.page.clip,
                paint: request.paint.clone(),
                source_chain: vec![],
                sources: vec![SourceRange {
                    path: metrics.source.path.clone(),
                    start_byte: metrics.source.range.start as u64,
                    end_byte: metrics.source.range.end as u64,
                }],
                synthetic_reason: None,
                logical_interval: Some((
                    metrics.source.range.start as u64,
                    metrics.source.range.end as u64,
                )),
                font_sha256: Some(lease.binding().declaration.font.sha256.clone()),
                original_gid: Some(gid as u32),
            });
        }
        let batch =
            MixedBatch::assembled(request.page, primitives, limits).map_err(|e| match e {
                MixedError::Budget => AssemblyError::Budget,
                other => ValidationError(format!("MATH batch: {other:?}")).into(),
            })?;
        Ok(MathAssemblyFrame {
            metrics,
            fit,
            batch,
        })
    }
}

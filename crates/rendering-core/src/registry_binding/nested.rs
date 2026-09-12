//! One explicit VF graph packet into a mixed exact registry frame. Registry and
//! TFM/encoding bindings remain separate; no Unicode/GID cast or TeX rounding.
use super::*;
use crate::{
    batch::{ExactClip, PrimitiveId},
    glyph_cache::PathOutcome,
    graph_cache::{GraphCache, Outcome},
    mixed::{MixedBatch, MixedContext, MixedGeometry, MixedLimits, MixedPrimitive},
    outlines::{place_path_exact, OutlineCoordinate as Q, OutlinePoint as P},
    tex_adapter::RunScale,
};
use flashtex_font_resources::{
    cff::BoundCffTfmFont,
    encoding::{BoundTfmFont, GlyphIdentity},
    vf_graph::{NestedPlacement, Resource, ResourceKey},
    Coordinate,
};
#[derive(Clone, Copy)]
pub enum MetricBinding<'a> {
    TrueType(&'a BoundTfmFont<'a>),
    Cff(&'a BoundCffTfmFont<'a>),
}
pub struct NestedBinding<'a> {
    lease: RenderLease,
    metric: MetricBinding<'a>,
    key: ResourceKey,
}
impl NestedBinding<'_> {
    pub fn key(&self) -> &ResourceKey {
        &self.key
    }
}
pub struct NestedContext<'a> {
    pub page: MixedContext<'a>,
    pub origin: P,
    pub scale: RunScale,
    pub source_path: &'a str,
    pub snapshot: &'a SourceSnapshot,
    pub source_range: std::ops::Range<usize>,
    pub paint: Paint,
    pub cff_policy: HintPolicy,
}
pub struct NestedRegistryFrame {
    batch: MixedBatch,
    leases: Vec<RenderLease>,
    source_path: String,
    source_sha256: String,
    source_revision: u64,
    advance: Q,
}
impl NestedRegistryFrame {
    pub fn batch(&self) -> &MixedBatch {
        &self.batch
    }
    pub fn advance(&self) -> Q {
        self.advance
    }
    /// Retained batch reads are immutable; edits require fresh source/registry state.
    pub fn require_current(
        &self,
        renderer: &RegistryRenderer,
        path: &str,
        snapshot: &SourceSnapshot,
    ) -> BoundResult<()> {
        for lease in &self.leases {
            renderer.current(lease)?;
        }
        require(
            path == self.source_path
                && snapshot.revision == self.source_revision
                && digest(snapshot.text.as_bytes()) == self.source_sha256,
            "stale nested registry source",
        )?;
        Ok(())
    }
}
impl RegistryRenderer {
    pub fn bind_metrics<'a>(
        &self,
        lease: &RenderLease,
        metric: MetricBinding<'a>,
    ) -> BoundResult<NestedBinding<'a>> {
        self.current(lease)?;
        let key = match metric {
            MetricBinding::TrueType(binding) => {
                require(
                    matches!(&lease.resource.backend, Backend::TrueType { .. })
                        && binding.font().descriptor() == &lease.binding().declaration.font,
                    "nested TrueType registry metrics binding",
                )?;
                Resource::Physical(binding).key()
            }
            MetricBinding::Cff(binding) => {
                let Backend::Cff { resource } = &lease.resource.backend else {
                    return Err(BindingError::Invalid(ValidationError(
                        "nested CFF backend mismatch".into(),
                    )));
                };
                require(
                    binding.identity() == resource.identity(),
                    "nested CFF table/font/face binding",
                )?;
                Resource::CffPhysical(binding).key()
            }
        };
        Ok(NestedBinding {
            lease: lease.clone(),
            metric,
            key,
        })
    }
    pub fn nested_packet(
        &self,
        cache: &mut GraphCache<'_, '_>,
        root: &ResourceKey,
        code: u8,
        bindings: &[NestedBinding<'_>],
        context: NestedContext<'_>,
        limits: MixedLimits,
    ) -> BoundResult<NestedRegistryFrame> {
        if bindings.len() > 128 {
            return Err(BindingError::Budget);
        }
        let mut by_key = BTreeMap::new();
        for b in bindings {
            self.current(&b.lease)?;
            require(
                by_key.insert(&b.key, b).is_none(),
                "duplicate nested registry key",
            )?;
        }
        path(context.source_path)?;
        require(
            context.source_range.start < context.source_range.end
                && context
                    .snapshot
                    .text
                    .get(context.source_range.clone())
                    .is_some(),
            "nested exact UTF8 source range",
        )?;
        require(
            context.snapshot.text.len() <= MAX_MESSAGE_BYTES,
            "nested source budget",
        )?;
        context.paint.validate()?;
        context.page.clip.validate()?;
        let packet = match cache.lookup(root, code)?.outcome {
            Outcome::Ready(packet) => packet,
            Outcome::Unavailable(e) => {
                return Err(BindingError::Invalid(ValidationError(format!(
                    "nested graph unavailable: {e:?}"
                ))))
            }
        };
        require(
            packet.resource == *root && packet.character == code,
            "nested graph packet identity",
        )?;
        let size = context.scale.exact_size();
        let convert = |v: Coordinate| -> Result<Q> {
            size.checked_multiply(Q::from_fraction(
                v.numerator(),
                1u128
                    .checked_shl(v.shift())
                    .ok_or_else(|| ValidationError("VF coordinate precision".into()))?,
            )?)
        };
        let mut primitives = Vec::new();
        let mut commands = 0usize;
        let mut used = BTreeMap::new();
        for (index, placement) in packet.placements.iter().enumerate() {
            if index >= limits.max_primitives {
                return Err(BindingError::Budget);
            }
            let (geometry, key, gid, chain) = match placement {
                NestedPlacement::Glyph {
                    resource,
                    glyph_id,
                    tfm_code,
                    x,
                    y,
                    scale,
                    source,
                } => {
                    let binding = by_key.get(resource).ok_or_else(|| {
                        ValidationError("missing explicit nested registry font binding".into())
                    })?;
                    let mapped = match binding.metric {
                        MetricBinding::TrueType(b) => b.map_code(*tfm_code),
                        MetricBinding::Cff(b) => b.map_code(*tfm_code),
                    }
                    .map_err(BindingError::Font)?
                    .0;
                    require(
                        mapped == GlyphIdentity::Original(*glyph_id) && *glyph_id > 0,
                        "nested original GID/encoding mismatch",
                    )?;
                    let origin = P {
                        x: context.origin.x.checked_add(convert(*x)?)?,
                        y: context.origin.y.checked_add(convert(*y)?)?,
                    };
                    let font_size = convert(*scale)?;
                    require(font_size.numerator() > 0, "nested positive font size")?;
                    let geometry = match &binding.lease.resource.backend {
                        Backend::TrueType { resource, cache } => {
                            let mut cache =
                                cache.lock().map_err(|_| BindingError::CacheUnavailable)?;
                            let path = match cache.lookup(resource, *glyph_id)?.outcome {
                                PathOutcome::Ready(path) => path,
                                PathOutcome::Unavailable(e) => {
                                    return Err(BindingError::Invalid(ValidationError(format!(
                                        "nested path unavailable: {e:?}"
                                    ))))
                                }
                            };
                            if path.commands.len() > limits.max_commands.saturating_sub(commands) {
                                return Err(BindingError::Budget);
                            }
                            MixedGeometry::Quadratic(place_path_exact(
                                path.commands.iter().cloned(),
                                font_size,
                                resource.descriptor().units_per_em,
                                origin,
                            )?)
                        }
                        Backend::Cff { resource } => {
                            use crate::cubic::CubicProvider;
                            let path = resource
                                .place_glyph(
                                    *glyph_id,
                                    context.cff_policy,
                                    font_size,
                                    origin,
                                    limits.max_commands.saturating_sub(commands),
                                )
                                .map_err(|e| {
                                    BindingError::Invalid(ValidationError(format!(
                                        "nested cubic unavailable: {e:?}"
                                    )))
                                })?;
                            MixedGeometry::Cubic(Box::new(path))
                        }
                    };
                    used.insert(binding.key.clone(), binding.lease.clone());
                    (
                        geometry,
                        Some(binding.lease.binding().declaration.font.sha256.clone()),
                        Some(*glyph_id as u32),
                        source,
                    )
                }
                NestedPlacement::Rule {
                    x,
                    y,
                    width,
                    height,
                    source,
                } => {
                    let left = context.origin.x.checked_add(convert(*x)?)?;
                    let bottom = context.origin.y.checked_add(convert(*y)?)?;
                    let h = convert(*height)?;
                    let bounds = ExactClip {
                        left,
                        top: bottom
                            .checked_add(Q::from_fraction(-h.numerator(), h.denominator())?)?,
                        right: left.checked_add(convert(*width)?)?,
                        bottom,
                    };
                    bounds.validate()?;
                    // Culling preserves original packet primitive index and attached source chain.
                    if bounds.intersect(context.page.clip)?.is_none() {
                        continue;
                    }
                    (MixedGeometry::Rule(bounds), None, None, source)
                }
            };
            require(
                chain
                    .first()
                    .is_some_and(|s| s.resource == *root && s.character == code),
                "nested source chain root mismatch",
            )?;
            commands = commands
                .checked_add(match &geometry {
                    MixedGeometry::Quadratic(v) => v.len(),
                    MixedGeometry::Cubic(v) => v.commands.len(),
                    MixedGeometry::Rule(_) => 0,
                })
                .ok_or(BindingError::Budget)?;
            if commands > limits.max_commands {
                return Err(BindingError::Budget);
            }
            primitives.push(MixedPrimitive {
                identity: PrimitiveId {
                    item_index: index,
                    glyph_index: gid.map(|_| 0),
                },
                geometry,
                clip: context.page.clip,
                paint: context.paint.clone(),
                source_chain: chain.clone(),
                sources: vec![SourceRange {
                    path: context.source_path.into(),
                    start_byte: context.source_range.start as u64,
                    end_byte: context.source_range.end as u64,
                }],
                synthetic_reason: None,
                logical_interval: Some((
                    context.source_range.start as u64,
                    context.source_range.end as u64,
                )),
                font_sha256: key,
                original_gid: gid,
            });
        }
        let advance = size.checked_multiply(Q::from_fraction(packet.width.0 as i128, 1 << 20)?)?;
        let batch = MixedBatch::assembled(context.page, primitives, limits).map_err(|e| {
            BindingError::Invalid(ValidationError(format!("nested mixed assembly: {e:?}")))
        })?;
        // Even a rule-only packet remains bound to the captured registry generation.
        let leases: Vec<RenderLease> = if used.is_empty() {
            bindings.iter().map(|b| b.lease.clone()).collect()
        } else {
            used.into_values().collect()
        };
        if leases.is_empty() {
            return Err(BindingError::Invalid(ValidationError(
                "nested frame requires a registry lease".into(),
            )));
        }
        Ok(NestedRegistryFrame {
            batch,
            leases,
            source_path: context.source_path.into(),
            source_sha256: digest(context.snapshot.text.as_bytes()),
            source_revision: context.snapshot.revision,
            advance,
        })
    }
}

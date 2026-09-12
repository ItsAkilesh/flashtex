//! Opt-in exact placement of the original engine's identity-bound Unicode runs.
//! Font-unit advances already contain kerning. No TFM spacing, fallback, wire
//! activation, raster hinting, or interpolation inside indivisible clusters.
use crate::{
    batch::{ExactClip, PrimitiveId},
    cubic::{CachedCffConsumer, CubicError, CubicProvider, PositionedCubic},
    glyph_cache::{GlyphPathCache, PathFailure, PathOutcome},
    outlines::{place_path_exact, OutlineCoordinate, OutlinePoint, PlacedPathCommand},
    *,
};
use flashtex_font_resources::{cff::HintPolicy, engine_adapter::BoundShapedRun, FontResource};
use std::{ops::Range, sync::Arc};

pub enum OutlineSource<'a> {
    TrueType {
        resource: &'a FontResource,
        cache: &'a mut GlyphPathCache,
    },
    Cff {
        resource: &'a CachedCffConsumer,
        policy: HintPolicy,
    },
}
#[derive(Debug)]
pub enum ShapePlacementError {
    Geometry(ValidationError),
    Cubic(CubicError),
    Outline(Arc<PathFailure>),
    Budget,
    Identity,
    StaleSource,
    Notdef,
}
impl From<ValidationError> for ShapePlacementError {
    fn from(e: ValidationError) -> Self {
        Self::Geometry(e)
    }
}
pub type PlacementResult<T> = std::result::Result<T, ShapePlacementError>;
#[derive(Debug, Clone, Copy)]
pub struct PlacementLimits {
    pub max_glyphs: usize,
    pub max_commands: usize,
    pub max_payload_bytes: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub item_index: usize,
    pub size: OutlineCoordinate,
    pub origin: OutlinePoint,
    pub clip: ExactClip,
}
#[derive(Debug, Clone)]
pub enum ShapedGeometry {
    Quadratic(Vec<PlacedPathCommand>),
    Cubic(Box<PositionedCubic>),
}
#[derive(Debug, Clone)]
pub struct PlacedShapedGlyph {
    pub primitive_id: PrimitiveId,
    pub cluster_index: usize,
    pub source_range: Range<usize>,
    pub original_gid: u16,
    pub origin: OutlinePoint,
    pub advance: OutlineCoordinate,
    pub geometry: ShapedGeometry,
}
/// Immutable result retains the complete shaping record, including empty clusters,
/// ligature count, unsupported-feature notes, exact source identity and cache key.
#[derive(Debug, Clone)]
pub struct PlacedShapedRun {
    run: BoundShapedRun,
    glyphs: Vec<PlacedShapedGlyph>,
    advance: OutlineCoordinate,
    placement: Placement,
    commands: usize,
    payload_bytes: usize,
}
impl PlacedShapedRun {
    pub fn run(&self) -> &BoundShapedRun {
        &self.run
    }
    pub fn glyphs(&self) -> &[PlacedShapedGlyph] {
        &self.glyphs
    }
    pub fn advance(&self) -> OutlineCoordinate {
        self.advance
    }
    pub fn placement(&self) -> Placement {
        self.placement
    }
    pub fn command_count(&self) -> usize {
        self.commands
    }
    /// Charged retained payload estimate, not allocator RSS or shared cache bytes.
    pub fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }
    pub fn prepare(
        run: &BoundShapedRun,
        snapshot: &SourceSnapshot,
        mut outlines: OutlineSource<'_>,
        placement: Placement,
        limits: PlacementLimits,
    ) -> PlacementResult<Self> {
        if snapshot.revision != run.source().revision
            || digest(snapshot.text.as_bytes()) != run.source().source_sha256
            || snapshot.text.get(run.source().range.clone()).is_none()
        {
            return Err(ShapePlacementError::StaleSource);
        }
        placement.clip.validate()?;
        require(
            placement.size.numerator() > 0,
            "shape font size must be positive",
        )?;
        if !(1..=100_000).contains(&limits.max_glyphs)
            || !(1..=2_000_000).contains(&limits.max_commands)
            || !(1..=MAX_MESSAGE_BYTES).contains(&limits.max_payload_bytes)
        {
            return Err(ShapePlacementError::Budget);
        }
        let identity = run.identity();
        let upem = run.shaped().units_per_em;
        match &outlines {
            OutlineSource::TrueType { resource, .. } => {
                let d = resource.descriptor();
                if d.sha256 != identity.font_sha256
                    || d.face_index != identity.face_index
                    || d.units_per_em != upem as u32
                {
                    return Err(ShapePlacementError::Identity);
                }
            }
            OutlineSource::Cff { resource, .. } => {
                let d = resource.identity();
                if d.font_sha256 != identity.font_sha256 || d.face_index != identity.face_index {
                    return Err(ShapePlacementError::Identity);
                }
            }
        }
        let unit = placement
            .size
            .checked_multiply(OutlineCoordinate::from_fraction(1, upem as u128)?)?;
        let units = |n: i64| unit.checked_multiply(OutlineCoordinate::from_fraction(n as i128, 1)?);
        let glyph_count = run.shaped().glyphs().count();
        let mut bytes = std::mem::size_of::<Self>()
            + run.source().path.len()
            + run.source().source_sha256.len()
            + run.cache_key().len()
            + identity.font_sha256.len()
            + 256;
        // Bound all retained cluster text/records and notes before cloning the run.
        for c in &run.shaped().clusters {
            bytes = bytes
                .checked_add(
                    256 + c.text.len() + c.glyphs.len() * std::mem::size_of::<PlacedShapedGlyph>(),
                )
                .ok_or(ShapePlacementError::Budget)?;
        }
        for note in &run.shaped().unsupported {
            bytes = bytes
                .checked_add(std::mem::size_of_val(note) + note.detail.len())
                .ok_or(ShapePlacementError::Budget)?;
        }
        if glyph_count > limits.max_glyphs || bytes > limits.max_payload_bytes {
            return Err(ShapePlacementError::Budget);
        }
        let mut glyphs = Vec::with_capacity(glyph_count);
        let mut pen = 0i64;
        let mut commands = 0usize;
        for (ci, cluster) in run.shaped().clusters.iter().enumerate() {
            let source_range = run
                .absolute_cluster_range(ci)
                .map_err(|_| ShapePlacementError::StaleSource)?;
            if snapshot.text.get(source_range.clone()) != Some(cluster.text.as_str()) {
                return Err(ShapePlacementError::StaleSource);
            }
            for glyph in &cluster.glyphs {
                let gid = glyph.gid.0;
                if gid == 0 {
                    return Err(ShapePlacementError::Notdef);
                }
                let x = pen
                    .checked_add(glyph.x_offset as i64)
                    .ok_or(ShapePlacementError::Budget)?;
                let origin = OutlinePoint {
                    x: placement.origin.x.checked_add(units(x)?)?,
                    y: placement
                        .origin
                        .y
                        .checked_add(units(-(glyph.y_offset as i64))?)?,
                };
                let (geometry, count, charge) = match &mut outlines {
                    OutlineSource::TrueType { resource, cache } => {
                        let path = match cache.lookup(resource, gid)?.outcome {
                            PathOutcome::Ready(p) => p,
                            PathOutcome::Unavailable(e) => {
                                return Err(ShapePlacementError::Outline(e))
                            }
                        };
                        let count = path.commands.len();
                        let charge = count
                            .checked_mul(std::mem::size_of::<PlacedPathCommand>())
                            .ok_or(ShapePlacementError::Budget)?;
                        if count > limits.max_commands - commands
                            || charge > limits.max_payload_bytes - bytes
                        {
                            return Err(ShapePlacementError::Budget);
                        }
                        (
                            ShapedGeometry::Quadratic(place_path_exact(
                                path.commands.iter().cloned(),
                                placement.size,
                                upem as u32,
                                origin,
                            )?),
                            count,
                            charge,
                        )
                    }
                    OutlineSource::Cff { resource, policy } => {
                        let remaining = (limits.max_commands - commands).min(
                            (limits.max_payload_bytes - bytes)
                                / std::mem::size_of::<crate::cubic::CubicPathCommand>(),
                        );
                        if remaining == 0 {
                            return Err(ShapePlacementError::Budget);
                        }
                        let p = resource
                            .place_glyph(gid, *policy, placement.size, origin, remaining)
                            .map_err(ShapePlacementError::Cubic)?;
                        let count = p.commands.len();
                        let mut charge =
                            count * std::mem::size_of::<crate::cubic::CubicPathCommand>() + 1024;
                        charge = charge
                            .checked_add(
                                p.hints.stems.len()
                                    * std::mem::size_of::<flashtex_font_resources::cff::StemHint>()
                                    + p.hints.flex_depths.len()
                                        * std::mem::size_of::<flashtex_font_resources::Coordinate>(
                                        ),
                            )
                            .ok_or(ShapePlacementError::Budget)?;
                        for mask in &p.hints.masks {
                            charge = charge
                                .checked_add(std::mem::size_of_val(mask) + mask.bytes.len())
                                .ok_or(ShapePlacementError::Budget)?;
                        }
                        (ShapedGeometry::Cubic(Box::new(p)), count, charge)
                    }
                };
                bytes = bytes
                    .checked_add(charge)
                    .ok_or(ShapePlacementError::Budget)?;
                commands = commands
                    .checked_add(count)
                    .ok_or(ShapePlacementError::Budget)?;
                if bytes > limits.max_payload_bytes || commands > limits.max_commands {
                    return Err(ShapePlacementError::Budget);
                }
                glyphs.push(PlacedShapedGlyph {
                    primitive_id: PrimitiveId {
                        item_index: placement.item_index,
                        glyph_index: Some(glyphs.len()),
                    },
                    cluster_index: ci,
                    source_range: source_range.clone(),
                    original_gid: gid,
                    origin,
                    advance: units(glyph.advance as i64)?,
                    geometry,
                });
                pen = pen
                    .checked_add(glyph.advance as i64)
                    .ok_or(ShapePlacementError::Budget)?;
            }
        }
        Ok(Self {
            run: run.clone(),
            glyphs,
            advance: units(pen)?,
            placement,
            commands,
            payload_bytes: bytes,
        })
    }
}

//! Opt-in CFF1 cubic consumer, separate from quadratic display-list primitives.
//! FontMatrix is applied by the immutable loader, then exact em-space coordinates
//! are scaled and baseline-flipped into page ticks. No wire or raster activation.
use crate::{
    batch::ExactClip,
    outlines::{OutlineCoordinate, OutlinePoint},
    *,
};
use flashtex_font_resources::cff::{
    Cff, HintMetadata, HintPolicy, MatrixCommand, Rational, RationalPoint,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CubicError {
    Resource(flashtex_font_resources::Error),
    Geometry(ValidationError),
    DigestMismatch,
    Budget,
    Notdef,
    CacheUnavailable,
}
impl From<ValidationError> for CubicError {
    fn from(e: ValidationError) -> Self {
        Self::Geometry(e)
    }
}
pub type CubicResult<T> = std::result::Result<T, CubicError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubicPathCommand {
    MoveTo(OutlinePoint),
    LineTo(OutlinePoint),
    CurveTo {
        control1: OutlinePoint,
        control2: OutlinePoint,
        end: OutlinePoint,
    },
    Close,
}
#[derive(Debug, Clone)]
pub struct PositionedCubic {
    pub cff_table_sha256: String,
    pub full_font_identity: Option<flashtex_font_resources::cff::CffIdentity>,
    pub original_gid: u16,
    pub font_matrix: [Rational; 6],
    pub advance: OutlinePoint,
    pub commands: Vec<CubicPathCommand>,
    pub hints: HintMetadata,
    pub hinting_applied: bool,
}
#[derive(Debug, Clone)]
pub struct ClippedCubic {
    pub outline: PositionedCubic,
    pub clip: ExactClip,
}
impl PositionedCubic {
    /// Consumer applies this exact clip; no quadratic conversion or approximate
    /// ink-bound culling is performed here.
    pub fn with_clip(self, clip: ExactClip) -> CubicResult<ClippedCubic> {
        clip.validate()?;
        Ok(ClippedCubic {
            outline: self,
            clip,
        })
    }
}
/// Encapsulates parsed CFF dictionaries so their immutable hash/matrix identity
/// cannot be mutated after the bytes are verified. This hashes the CFF table, not
/// its containing OpenType file; sfnt selection/license binding belongs upstream.
pub struct CffConsumer {
    cff: Cff,
}
impl CffConsumer {
    pub fn from_table(bytes: &[u8], expected_table_sha256: &str) -> CubicResult<Self> {
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(CubicError::Budget);
        }
        hash(expected_table_sha256)?;
        if digest(bytes) != expected_table_sha256 {
            return Err(CubicError::DigestMismatch);
        }
        Ok(Self {
            cff: Cff::parse(bytes).map_err(CubicError::Resource)?,
        })
    }
    pub fn table_sha256(&self) -> &str {
        &self.cff.sha256
    }
    pub fn glyph_count(&self) -> usize {
        self.cff.glyph_count()
    }
    pub fn place_glyph(
        &self,
        gid: u16,
        policy: HintPolicy,
        size: OutlineCoordinate,
        origin: OutlinePoint,
        max_commands: usize,
    ) -> CubicResult<PositionedCubic> {
        if gid == 0 {
            return Err(CubicError::Notdef);
        }
        require(size.numerator() > 0, "nonpositive cubic font size")?;
        if !(1..=2_000_000).contains(&max_commands) {
            return Err(CubicError::Budget);
        }
        let outline = self
            .cff
            .matrix_outline(gid, policy)
            .map_err(CubicError::Resource)?;
        place_matrix(&outline, size, origin, max_commands)
    }
}
fn place_matrix(
    outline: &flashtex_font_resources::cff::MatrixOutline,
    size: OutlineCoordinate,
    origin: OutlinePoint,
    max_commands: usize,
) -> CubicResult<PositionedCubic> {
    require(size.numerator() > 0, "nonpositive cached cubic size")?;
    if outline.glyph_id == 0 {
        return Err(CubicError::Notdef);
    }
    if !(1..=2_000_000).contains(&max_commands) {
        return Err(CubicError::Budget);
    }
    if outline.commands.len() > max_commands {
        return Err(CubicError::Budget);
    }
    let minus = OutlineCoordinate::from_fraction(-1, 1)?;
    let coordinate = |r: Rational| -> Result<OutlineCoordinate> {
        OutlineCoordinate::from_fraction(
            r.numerator(),
            u128::try_from(r.denominator())
                .map_err(|_| ValidationError("invalid cubic matrix denominator".into()))?,
        )
    };
    let vector = |p: RationalPoint| -> Result<OutlinePoint> {
        Ok(OutlinePoint {
            x: coordinate(p.x)?.checked_multiply(size)?,
            y: coordinate(p.y)?
                .checked_multiply(size)?
                .checked_multiply(minus)?,
        })
    };
    let point = |p: RationalPoint| -> Result<OutlinePoint> {
        let p = vector(p)?;
        Ok(OutlinePoint {
            x: p.x.checked_add(origin.x)?,
            y: p.y.checked_add(origin.y)?,
        })
    };
    let mut commands = Vec::with_capacity(outline.commands.len());
    for command in &outline.commands {
        commands.push(match *command {
            MatrixCommand::MoveTo(p) => CubicPathCommand::MoveTo(point(p)?),
            MatrixCommand::LineTo(p) => CubicPathCommand::LineTo(point(p)?),
            MatrixCommand::CurveTo {
                control1,
                control2,
                end,
            } => CubicPathCommand::CurveTo {
                control1: point(control1)?,
                control2: point(control2)?,
                end: point(end)?,
            },
            MatrixCommand::Close => CubicPathCommand::Close,
        });
    }
    Ok(PositionedCubic {
        cff_table_sha256: outline.cff_sha256.clone(),
        full_font_identity: None,
        original_gid: outline.glyph_id,
        font_matrix: outline.matrix,
        advance: vector(outline.advance)?,
        commands,
        hints: outline.hints.clone(),
        hinting_applied: false,
    })
}

/// Immutable resource interface shared by direct and full-font cache consumers.
pub trait CubicProvider {
    fn place_glyph(
        &self,
        gid: u16,
        policy: HintPolicy,
        size: OutlineCoordinate,
        origin: OutlinePoint,
        max_commands: usize,
    ) -> CubicResult<PositionedCubic>;
}
impl CubicProvider for CffConsumer {
    fn place_glyph(
        &self,
        gid: u16,
        policy: HintPolicy,
        size: OutlineCoordinate,
        origin: OutlinePoint,
        max_commands: usize,
    ) -> CubicResult<PositionedCubic> {
        CffConsumer::place_glyph(self, gid, policy, size, origin, max_commands)
    }
}
#[derive(Clone)]
pub struct CachedCffConsumer {
    cache: std::sync::Arc<std::sync::Mutex<flashtex_font_resources::cff::CffOutlineCache>>,
    identity: flashtex_font_resources::cff::CffIdentity,
}
pub struct CachedCubicPlacement {
    pub outline: PositionedCubic,
    pub cache_status: flashtex_font_resources::cff::CacheStatus,
}
impl CachedCffConsumer {
    pub fn new(cache: flashtex_font_resources::cff::CffOutlineCache) -> Self {
        let identity = cache.identity().clone();
        Self {
            cache: std::sync::Arc::new(std::sync::Mutex::new(cache)),
            identity,
        }
    }
    pub fn identity(&self) -> &flashtex_font_resources::cff::CffIdentity {
        &self.identity
    }
    pub fn place_cached(
        &self,
        gid: u16,
        policy: HintPolicy,
        size: OutlineCoordinate,
        origin: OutlinePoint,
        max_commands: usize,
    ) -> CubicResult<CachedCubicPlacement> {
        if gid == 0 {
            return Err(CubicError::Notdef);
        }
        let lookup = self
            .cache
            .lock()
            .map_err(|_| CubicError::CacheUnavailable)?
            .lookup(gid, policy);
        let matrix = lookup.result.map_err(CubicError::Resource)?;
        if matrix.cff_sha256 != self.identity.cff_sha256 {
            return Err(CubicError::DigestMismatch);
        }
        let mut outline = place_matrix(&matrix, size, origin, max_commands)?;
        outline.full_font_identity = Some(self.identity.clone());
        Ok(CachedCubicPlacement {
            outline,
            cache_status: lookup.status,
        })
    }
}
impl CubicProvider for CachedCffConsumer {
    fn place_glyph(
        &self,
        gid: u16,
        policy: HintPolicy,
        size: OutlineCoordinate,
        origin: OutlinePoint,
        max_commands: usize,
    ) -> CubicResult<PositionedCubic> {
        Ok(self
            .place_cached(gid, policy, size, origin, max_commands)?
            .outline)
    }
}

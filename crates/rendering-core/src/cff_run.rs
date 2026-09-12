//! Explicit 8-bit TFM-to-CFF runs. TFM widths/kerns determine pen movement;
//! transformed charstring advances remain separate evidence, never substitutes.
use crate::{
    cubic::{CachedCffConsumer, CubicError, PositionedCubic},
    outlines::{OutlineCoordinate, OutlinePoint},
    tex_adapter::{InputInterval, MetricPolicy, RunScale},
    *,
};
use flashtex_font_resources::{
    cff::{BoundCffTfmFont, CffIdentity, HintPolicy},
    encoding::{GlyphIdentity, MappedItem},
    tfm::{CharacterMetrics, FixWord},
};
#[derive(Debug)]
pub enum RunError {
    Resource(flashtex_font_resources::Error),
    Geometry(ValidationError),
    Cubic(CubicError),
    Identity,
    Budget,
    Notdef { code: u8 },
}
impl From<ValidationError> for RunError {
    fn from(e: ValidationError) -> Self {
        Self::Geometry(e)
    }
}
impl From<CubicError> for RunError {
    fn from(e: CubicError) -> Self {
        Self::Cubic(e)
    }
}
pub type RunResult<T> = std::result::Result<T, RunError>;
#[derive(Debug, Clone, Copy)]
pub struct RunLimits {
    pub max_glyphs: usize,
    pub max_commands: usize,
}
impl Default for RunLimits {
    fn default() -> Self {
        Self {
            max_glyphs: 100000,
            max_commands: 2_000_000,
        }
    }
}
#[derive(Debug)]
pub struct CffRunGlyph {
    pub tfm_code: u8,
    pub original_gid: u16,
    pub glyph_name: String,
    pub input: InputInterval,
    pub metrics: CharacterMetrics,
    pub tfm_advance: OutlineCoordinate,
    pub pen_x: OutlineCoordinate,
    pub outline: PositionedCubic,
}
#[derive(Debug)]
pub enum CffRunItem {
    Glyph(Box<CffRunGlyph>),
    Kern {
        tfm: FixWord,
        exact: OutlineCoordinate,
    },
}
pub struct CffRun {
    identity: CffIdentity,
    tfm_sha256: String,
    encoding_sha256: String,
    policy: MetricPolicy,
    items: Vec<CffRunItem>,
    advance: OutlineCoordinate,
    commands: usize,
}
impl CffRun {
    pub fn identity(&self) -> &CffIdentity {
        &self.identity
    }
    pub fn tfm_sha256(&self) -> &str {
        &self.tfm_sha256
    }
    pub fn encoding_sha256(&self) -> &str {
        &self.encoding_sha256
    }
    pub fn policy(&self) -> MetricPolicy {
        self.policy
    }
    pub fn items(&self) -> &[CffRunItem] {
        &self.items
    }
    pub fn advance(&self) -> OutlineCoordinate {
        self.advance
    }
    pub fn command_count(&self) -> usize {
        self.commands
    }
    pub fn prepare(
        binding: &BoundCffTfmFont<'_>,
        consumer: &CachedCffConsumer,
        input: &[u8],
        placement: RunPlacement,
        limits: RunLimits,
    ) -> RunResult<Self> {
        if binding.identity() != consumer.identity() {
            return Err(RunError::Identity);
        }
        if input.len() > 100000
            || !(1..=100000).contains(&limits.max_glyphs)
            || !(1..=2_000_000).contains(&limits.max_commands)
        {
            return Err(RunError::Budget);
        }
        let size = placement.scale.exact_size();
        require(size.numerator() > 0, "nonpositive CFF run size")?;
        let metric = |word: FixWord| -> Result<OutlineCoordinate> {
            OutlineCoordinate::from_fraction(word.0 as i128, 1 << 20)?.checked_multiply(size)
        };
        let mut x = OutlineCoordinate::from_fraction(0, 1)?;
        let mut items = Vec::new();
        let mut commands = 0usize;
        let mut glyphs = 0;
        for item in binding.map_run(input).map_err(RunError::Resource)? {
            match item {
                MappedItem::Kern(tfm) => {
                    let exact = metric(tfm)?;
                    x = x.checked_add(exact)?;
                    items.push(CffRunItem::Kern { tfm, exact });
                }
                MappedItem::Glyph {
                    tfm_code,
                    identity,
                    metrics,
                    input_start,
                    input_end,
                } => {
                    glyphs += 1;
                    if glyphs > limits.max_glyphs {
                        return Err(RunError::Budget);
                    }
                    let GlyphIdentity::Original(gid) = identity else {
                        return Err(RunError::Notdef { code: tfm_code });
                    };
                    let glyph_name = binding
                        .encoding()
                        .glyph_name(tfm_code)
                        .ok_or(RunError::Identity)?
                        .to_owned();
                    let remaining = limits
                        .max_commands
                        .checked_sub(commands)
                        .filter(|n| *n > 0)
                        .ok_or(RunError::Budget)?;
                    let placed = consumer.place_cached(
                        gid,
                        placement.hints,
                        size,
                        OutlinePoint {
                            x: placement.origin.x.checked_add(x)?,
                            y: placement.origin.y,
                        },
                        remaining,
                    )?;
                    if placed.outline.full_font_identity.as_ref() != Some(binding.identity())
                        || placed.outline.original_gid != gid
                        || placed.outline.hinting_applied
                    {
                        return Err(RunError::Identity);
                    }
                    commands = commands
                        .checked_add(placed.outline.commands.len())
                        .ok_or(RunError::Budget)?;
                    if commands > limits.max_commands {
                        return Err(RunError::Budget);
                    }
                    let tfm_advance = metric(metrics.width)?;
                    items.push(CffRunItem::Glyph(Box::new(CffRunGlyph {
                        tfm_code,
                        original_gid: gid,
                        glyph_name,
                        input: InputInterval {
                            start: input_start,
                            end: input_end,
                        },
                        metrics,
                        tfm_advance,
                        pen_x: x,
                        outline: placed.outline,
                    })));
                    x = x.checked_add(tfm_advance)?;
                }
            }
        }
        Ok(Self {
            identity: binding.identity().clone(),
            tfm_sha256: binding.tfm().source_sha256.clone(),
            encoding_sha256: binding.encoding().encoding_sha256().into(),
            policy: placement.scale.policy(),
            items,
            advance: x,
            commands,
        })
    }
}
#[derive(Clone, Copy)]
pub struct RunPlacement {
    pub scale: RunScale,
    pub origin: OutlinePoint,
    pub hints: HintPolicy,
}

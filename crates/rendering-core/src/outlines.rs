//! Exact, unhinted placement of loader-owned quadratic glyph paths. Glyph/font
//! identity and logical-source provenance are retained; no font substitution,
//! native/PDF activation, bytecode execution or raster parity is implied.
use crate::{font_adapter::descriptor, hit_test::Point, *};
use flashtex_font_resources::{
    Coordinate, ExpandedOutline, FontCollection, GlyphInstance, PathCommand,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlineCoordinate {
    numerator: i128,
    denominator: u128,
}
impl OutlineCoordinate {
    pub fn numerator(self) -> i128 {
        self.numerator
    }
    pub fn denominator(self) -> u128 {
        self.denominator
    }
    fn new(numerator: i128, denominator: u128) -> Result<Self> {
        require(
            denominator > 0 && denominator <= i128::MAX as u128,
            "outline denominator budget",
        )?;
        let divisor = gcd(numerator.unsigned_abs(), denominator);
        let numerator = numerator
            / i128::try_from(divisor)
                .map_err(|_| ValidationError("outline normalization overflow".into()))?;
        let denominator = denominator / divisor;
        if let Some(bound) = (MAX_EXACT_INTEGER as u128).checked_mul(denominator) {
            require(
                numerator.unsigned_abs() <= bound,
                "placed outline outside coordinate range",
            )?;
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
}
fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let rem = a % b;
        a = b;
        b = rem;
    }
    a
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutlinePoint {
    pub x: OutlineCoordinate,
    pub y: OutlineCoordinate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacedPathCommand {
    MoveTo(OutlinePoint),
    LineTo(OutlinePoint),
    QuadTo {
        control: OutlinePoint,
        end: OutlinePoint,
    },
    Close,
}

fn coordinate(
    value: Coordinate,
    size: Tick,
    units: u32,
    origin: Tick,
    flip: bool,
) -> Result<OutlineCoordinate> {
    size.positive()?;
    origin.validate()?;
    require(
        (16..=16384).contains(&units),
        "invalid outline units per em",
    )?;
    let base = (1u128
        .checked_shl(value.shift())
        .ok_or_else(|| ValidationError("outline precision overflow".into()))?)
    .checked_mul(u128::from(units))
    .ok_or_else(|| ValidationError("outline denominator overflow".into()))?;
    let divisor = gcd(size.0 as u128, base);
    let denominator = base / divisor;
    require(
        denominator <= i128::MAX as u128,
        "outline denominator overflow",
    )?;
    let factor = (size.0 as u128 / divisor) as i128;
    let mut numerator = value
        .numerator()
        .checked_mul(factor)
        .ok_or_else(|| ValidationError("outline scaling overflow".into()))?;
    if flip {
        numerator = numerator
            .checked_neg()
            .ok_or_else(|| ValidationError("outline y flip overflow".into()))?;
    }
    numerator = numerator
        .checked_add(
            i128::from(origin.0)
                .checked_mul(denominator as i128)
                .ok_or_else(|| ValidationError("outline origin overflow".into()))?,
        )
        .ok_or_else(|| ValidationError("outline placement overflow".into()))?;
    OutlineCoordinate::new(numerator, denominator)
}
/// Map positive-up design units to positive-down page coordinates exactly.
pub fn position_font_point(
    point: flashtex_font_resources::ExactPoint,
    size: Tick,
    units: u32,
    origin: Point,
) -> Result<OutlinePoint> {
    Ok(OutlinePoint {
        x: coordinate(point.x, size, units, origin.x, false)?,
        y: coordinate(point.y, size, units, origin.y, true)?,
    })
}
/// The loader supplies implied midpoints and component transforms. This adapter
/// only performs exact size/baseline placement and preserves contour closure.
pub fn place_outline(
    outline: &ExpandedOutline,
    size: Tick,
    units: u32,
    origin: Point,
) -> Result<Vec<PlacedPathCommand>> {
    size.positive()?;
    origin.x.validate()?;
    origin.y.validate()?;
    require(
        (16..=16384).contains(&units),
        "invalid outline units per em",
    )?;
    let path = outline
        .quadratic_path()
        .map_err(|error| ValidationError(format!("quadratic outline: {error}")))?;
    place_path(path, size, units, origin)
}
/// Place an immutable cached loader path without repeating glyph expansion.
pub fn place_path<I: IntoIterator<Item = PathCommand>>(
    path: I,
    size: Tick,
    units: u32,
    origin: Point,
) -> Result<Vec<PlacedPathCommand>> {
    size.positive()?;
    origin.x.validate()?;
    origin.y.validate()?;
    require(
        (16..=16384).contains(&units),
        "invalid outline units per em",
    )?;
    let mut commands = Vec::new();

    for command in path {
        require(commands.len() < 2_000_000, "placed path command budget")?;
        commands.push(match command {
            PathCommand::MoveTo(point) => {
                PlacedPathCommand::MoveTo(position_font_point(point, size, units, origin)?)
            }
            PathCommand::LineTo(point) => {
                PlacedPathCommand::LineTo(position_font_point(point, size, units, origin)?)
            }
            PathCommand::QuadTo { control, end } => PlacedPathCommand::QuadTo {
                control: position_font_point(control, size, units, origin)?,
                end: position_font_point(end, size, units, origin)?,
            },
            PathCommand::Close => PlacedPathCommand::Close,
        });
    }
    Ok(commands)
}
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub project_id: String,
    pub revision: u64,
    pub page: u32,
    pub item_index: usize,
    pub glyph_index: usize,
    pub font_id: String,
    pub font_sha256: String,
    pub original_gid: u32,
    pub cluster_index: u32,
    pub logical_start_byte: u64,
    pub logical_end_byte: u64,
    pub sources: Vec<SourceRange>,
    pub synthetic_reason: Option<String>,
    pub instances: Vec<GlyphInstance>,
    pub commands: Vec<PlacedPathCommand>,
    pub hinting_applied: bool,
}
/// Validate list/descriptor identity once, then resolve each glyph on demand.
/// Source UTF-8 content verification is still the separate source-snapshot gate.
pub struct PreparedOutlines<'a> {
    list: &'a DisplayList,
    fonts: &'a FontCollection,
}
impl<'a> PreparedOutlines<'a> {
    pub fn new(
        list: &'a DisplayList,
        capabilities: &Capabilities,
        fonts: &'a FontCollection,
    ) -> Result<Self> {
        list.validate(capabilities)?;
        for font in &list.fonts {
            let resource = fonts
                .get(&font.font_id)
                .map_err(|error| ValidationError(format!("outline font resource: {error}")))?;
            require(
                resource.descriptor() == &descriptor(font),
                "outline font descriptor mismatch",
            )?;
        }
        Ok(Self { list, fonts })
    }
    pub fn glyph(
        &self,
        page: u32,
        item_index: usize,
        glyph_index: usize,
    ) -> Result<PositionedGlyph> {
        let page_data = self
            .list
            .pages
            .iter()
            .find(|p| p.number == page)
            .ok_or_else(|| ValidationError("unknown outline page".into()))?;
        let run = match page_data.items.get(item_index) {
            Some(Item::GlyphRun(run)) => run,
            _ => return Err(ValidationError("outline item is not a glyph run".into())),
        };
        let glyph = run
            .glyphs
            .get(glyph_index)
            .ok_or_else(|| ValidationError("unknown outline glyph".into()))?;
        let resource = self
            .fonts
            .get(&run.font_id)
            .map_err(|error| ValidationError(format!("outline font resource: {error}")))?;
        let outline = resource
            .expanded_outline(glyph.gid as u16)
            .map_err(|error| ValidationError(format!("glyph outline unavailable: {error}")))?;
        let commands = place_outline(
            &outline,
            run.font_size,
            resource.descriptor().units_per_em,
            Point {
                x: glyph.origin_x,
                y: glyph.baseline_y,
            },
        )?;
        let cluster = &run.clusters[glyph.cluster as usize];
        Ok(PositionedGlyph {
            project_id: self.list.project_id.clone(),
            revision: self.list.revision,
            page,
            item_index,
            glyph_index,
            font_id: run.font_id.clone(),
            font_sha256: resource.descriptor().sha256.clone(),
            original_gid: glyph.gid,
            cluster_index: glyph.cluster,
            logical_start_byte: cluster.text_start_byte,
            logical_end_byte: cluster.text_end_byte,
            sources: cluster.sources.clone().unwrap_or_default(),
            synthetic_reason: cluster.synthetic_reason.clone(),
            instances: outline.instances,
            commands,
            hinting_applied: false,
        })
    }
}

impl PreparedOutlines<'_> {
    /// Reuse immutable unscaled paths across glyph placements; font size/origin
    /// remain authoritative per glyph and are applied exactly after cache lookup.
    pub fn glyph_cached(
        &self,
        page: u32,
        item_index: usize,
        glyph_index: usize,
        cache: &mut crate::glyph_cache::GlyphPathCache,
    ) -> Result<PositionedGlyph> {
        use crate::glyph_cache::PathOutcome;
        let page_index = page
            .checked_sub(1)
            .ok_or_else(|| ValidationError("unknown outline page".into()))?
            as usize;
        let page_data = self
            .list
            .pages
            .get(page_index)
            .ok_or_else(|| ValidationError("unknown outline page".into()))?;
        let run = match page_data.items.get(item_index) {
            Some(Item::GlyphRun(run)) => run,
            _ => return Err(ValidationError("outline item is not a glyph run".into())),
        };
        let glyph = run
            .glyphs
            .get(glyph_index)
            .ok_or_else(|| ValidationError("unknown outline glyph".into()))?;
        let resource = self
            .fonts
            .get(&run.font_id)
            .map_err(|error| ValidationError(format!("outline font resource: {error}")))?;
        let cached = match cache.lookup(resource, glyph.gid as u16)?.outcome {
            PathOutcome::Ready(data) => data,
            PathOutcome::Unavailable(error) => {
                return Err(ValidationError(format!(
                    "cached outline unavailable: {error:?}"
                )))
            }
        };
        let commands = place_path(
            cached.commands.iter().copied(),
            run.font_size,
            resource.descriptor().units_per_em,
            Point {
                x: glyph.origin_x,
                y: glyph.baseline_y,
            },
        )?;
        let cluster = &run.clusters[glyph.cluster as usize];
        Ok(PositionedGlyph {
            project_id: self.list.project_id.clone(),
            revision: self.list.revision,
            page,
            item_index,
            glyph_index,
            font_id: run.font_id.clone(),
            font_sha256: resource.descriptor().sha256.clone(),
            original_gid: glyph.gid,
            cluster_index: glyph.cluster,
            logical_start_byte: cluster.text_start_byte,
            logical_end_byte: cluster.text_end_byte,
            sources: cluster.sources.clone().unwrap_or_default(),
            synthetic_reason: cluster.synthetic_reason.clone(),
            instances: cached.instances.clone(),
            commands,
            hinting_applied: false,
        })
    }
}

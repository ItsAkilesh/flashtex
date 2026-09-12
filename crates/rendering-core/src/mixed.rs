//! Internal opt-in mixed path batches. This serialization is a bounded fixture /
//! consumer-conversion format, not the negotiated rendering display-list wire.
use crate::{
    batch::{DrawOperation, ExactClip, PrimitiveId},
    cubic::{CubicError, CubicPathCommand, CubicProvider, PositionedCubic},
    outlines::{OutlineCoordinate, OutlinePoint, PlacedPathCommand},
    tex_adapter::TracedBatch,
    *,
};
use flashtex_font_resources::{
    cff::HintPolicy,
    vf_graph::{ResourceKey, SourceStep},
};
use serde::Serialize;
use serde_json::{json, Value};
use std::io::{self, Write};
#[derive(Debug)]
pub enum MixedError {
    Geometry(ValidationError),
    Cubic(CubicError),
    Budget,
    Identity,
    Unsupported(String),
    Serialization(String),
}
impl From<ValidationError> for MixedError {
    fn from(e: ValidationError) -> Self {
        Self::Geometry(e)
    }
}
impl From<CubicError> for MixedError {
    fn from(e: CubicError) -> Self {
        Self::Cubic(e)
    }
}
pub type MixedResult<T> = std::result::Result<T, MixedError>;
pub struct CubicReplacement<'a> {
    pub resource: &'a dyn CubicProvider,
    pub original_gid: u16,
    pub policy: HintPolicy,
    pub size: OutlineCoordinate,
    pub origin: OutlinePoint,
}
pub struct MixedInput<'a> {
    pub source: &'a TracedBatch,
    pub primitive_index: usize,
    pub cubic: Option<CubicReplacement<'a>>,
}
pub struct MixedContext<'a> {
    pub project_id: &'a str,
    pub revision: u64,
    pub page: u32,
    pub page_width: Tick,
    pub page_height: Tick,
    pub clip: ExactClip,
}
#[derive(Debug, Clone, Copy)]
pub struct MixedLimits {
    pub max_primitives: usize,
    pub max_commands: usize,
    pub max_serialized_bytes: usize,
}
impl Default for MixedLimits {
    fn default() -> Self {
        Self {
            max_primitives: 100000,
            max_commands: 2_000_000,
            max_serialized_bytes: MAX_MESSAGE_BYTES,
        }
    }
}
#[derive(Debug, Clone)]
pub enum MixedGeometry {
    Quadratic(Vec<PlacedPathCommand>),
    Cubic(Box<PositionedCubic>),
    Rule(ExactClip),
}
#[derive(Debug, Clone)]
pub struct MixedPrimitive {
    pub identity: PrimitiveId,
    pub geometry: MixedGeometry,
    pub clip: ExactClip,
    pub paint: Paint,
    pub source_chain: Vec<SourceStep>,
    pub sources: Vec<SourceRange>,
    pub synthetic_reason: Option<String>,
    pub logical_interval: Option<(u64, u64)>,
    pub font_sha256: Option<String>,
    pub original_gid: Option<u32>,
}
pub struct MixedBatch {
    project_id: String,
    revision: u64,
    page: u32,
    page_width: Tick,
    page_height: Tick,
    clip: ExactClip,
    primitives: Vec<MixedPrimitive>,
    commands: usize,
    encoded: Vec<u8>,
}
impl MixedBatch {
    pub fn identity(&self) -> (&str, u64, u32) {
        (&self.project_id, self.revision, self.page)
    }
    pub fn primitives(&self) -> &[MixedPrimitive] {
        &self.primitives
    }
    pub fn command_count(&self) -> usize {
        self.commands
    }
    pub fn fixture_bytes(&self) -> &[u8] {
        &self.encoded
    }
    pub fn build(
        context: MixedContext<'_>,
        inputs: &[MixedInput<'_>],
        limits: MixedLimits,
    ) -> MixedResult<Self> {
        id(context.project_id)?;
        context.page_width.positive()?;
        context.page_height.positive()?;
        context.clip.validate()?;
        if context.page == 0 {
            return Err(MixedError::Identity);
        }
        if !(1..=100000).contains(&limits.max_primitives)
            || !(1..=2_000_000).contains(&limits.max_commands)
            || !(1..=MAX_MESSAGE_BYTES).contains(&limits.max_serialized_bytes)
            || inputs.len() > limits.max_primitives
        {
            return Err(MixedError::Budget);
        }
        let page_clip = ExactClip::from_rect(&HitRect {
            x: Tick(0),
            top: Tick(0),
            width: context.page_width,
            height: context.page_height,
        })?;
        let clip = context
            .clip
            .intersect(page_clip)?
            .ok_or_else(|| MixedError::Unsupported("empty mixed page clip".into()))?;
        let mut result = Self {
            project_id: context.project_id.into(),
            revision: context.revision,
            page: context.page,
            page_width: context.page_width,
            page_height: context.page_height,
            clip,
            primitives: vec![],
            commands: 0,
            encoded: vec![],
        };
        let mut identities = BTreeSet::new();
        for input in inputs {
            let source = input.source;
            if source.project_id != context.project_id
                || source.revision != context.revision
                || source.page != context.page
                || source.page_width != context.page_width
                || source.page_height != context.page_height
                || source.hinting_applied
                || source.color_space != "srgb"
                || source.compositing != "source-over"
            {
                return Err(MixedError::Identity);
            }
            let primitive = source
                .primitives
                .get(input.primitive_index)
                .ok_or(MixedError::Identity)?;
            if primitive.identity != primitive.operation.primitive_id()
                || !identities.insert(primitive.identity)
            {
                return Err(MixedError::Identity);
            }
            let Some(source_clip) = source.visible_clip else {
                continue;
            };
            let Some(clip) = clip.intersect(source_clip)? else {
                continue;
            };
            let (
                geometry,
                paint,
                sources,
                synthetic_reason,
                logical_interval,
                font_sha256,
                original_gid,
            ) = match &primitive.operation {
                DrawOperation::Glyph { path, paint } => {
                    if path.project_id != context.project_id
                        || path.revision != context.revision
                        || path.page != context.page
                        || path.hinting_applied
                        || path.original_gid == 0
                    {
                        return Err(MixedError::Identity);
                    }
                    hash(&path.font_sha256)?;
                    let (geometry, font_sha, gid) = if let Some(cubic) = &input.cubic {
                        let remaining = limits.max_commands.saturating_sub(result.commands);
                        if remaining == 0 {
                            return Err(MixedError::Budget);
                        }
                        let placed = cubic.resource.place_glyph(
                            cubic.original_gid,
                            cubic.policy,
                            cubic.size,
                            cubic.origin,
                            remaining,
                        )?;
                        if placed.original_gid != cubic.original_gid
                            || placed.hinting_applied
                            || placed.hints.policy != cubic.policy
                        {
                            return Err(MixedError::Identity);
                        }
                        hash(&placed.cff_table_sha256)?;
                        if let Some(identity) = &placed.full_font_identity {
                            hash(&identity.font_sha256)?;
                            if identity.cff_sha256 != placed.cff_table_sha256
                                || identity.face_index != 0
                                || identity.table_range.start >= identity.table_range.end
                            {
                                return Err(MixedError::Identity);
                            }
                        }
                        (
                            MixedGeometry::Cubic(Box::new(placed)),
                            None,
                            Some(cubic.original_gid as u32),
                        )
                    } else {
                        (
                            MixedGeometry::Quadratic(path.commands.clone()),
                            Some(path.font_sha256.clone()),
                            Some(path.original_gid),
                        )
                    };
                    (
                        geometry,
                        paint.clone(),
                        path.sources.clone(),
                        path.synthetic_reason.clone(),
                        Some((path.logical_start_byte, path.logical_end_byte)),
                        font_sha,
                        gid,
                    )
                }
                DrawOperation::Rule {
                    geometry,
                    paint,
                    sources,
                    synthetic_reason,
                    ..
                } => {
                    if input.cubic.is_some() {
                        return Err(MixedError::Unsupported(
                            "cannot replace a rule with a cubic glyph".into(),
                        ));
                    }
                    let geometry = ExactClip::from_rect(geometry)?;
                    if geometry.intersect(clip)?.is_none() {
                        continue;
                    }
                    (
                        MixedGeometry::Rule(geometry),
                        paint.clone(),
                        sources.clone(),
                        synthetic_reason.clone(),
                        None,
                        None,
                        None,
                    )
                }
                DrawOperation::ExactRule {
                    geometry,
                    paint,
                    sources,
                    synthetic_reason,
                    ..
                } => {
                    if input.cubic.is_some() {
                        return Err(MixedError::Unsupported(
                            "cannot replace a rule with a cubic glyph".into(),
                        ));
                    }
                    if geometry.intersect(clip)?.is_none() {
                        continue;
                    }
                    (
                        MixedGeometry::Rule(*geometry),
                        paint.clone(),
                        sources.clone(),
                        synthetic_reason.clone(),
                        None,
                        None,
                        None,
                    )
                }
            };
            paint.validate()?;
            let count = match &geometry {
                MixedGeometry::Quadratic(commands) => commands.len(),
                MixedGeometry::Cubic(path) => path.commands.len(),
                MixedGeometry::Rule(_) => 0,
            };
            result.commands = result
                .commands
                .checked_add(count)
                .ok_or(MixedError::Budget)?;
            if result.commands > limits.max_commands {
                return Err(MixedError::Budget);
            }
            result.primitives.push(MixedPrimitive {
                identity: primitive.identity,
                geometry,
                clip,
                paint,
                source_chain: primitive.source_chain.clone(),
                sources,
                synthetic_reason,
                logical_interval,
                font_sha256,
                original_gid,
            });
        }
        let mut output = BoundedOutput {
            bytes: vec![],
            limit: limits.max_serialized_bytes,
        };
        serde_json::to_writer(&mut output, &result).map_err(|e| {
            if e.is_io() {
                MixedError::Budget
            } else {
                MixedError::Serialization(e.to_string())
            }
        })?;
        result.encoded = output.bytes;
        Ok(result)
    }
}
pub(crate) struct BoundedOutput {
    pub(crate) bytes: Vec<u8>,
    pub(crate) limit: usize,
}
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("mixed serialized byte budget"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn scalar(value: OutlineCoordinate) -> Value {
    json!([
        value.numerator().to_string(),
        value.denominator().to_string()
    ])
}
fn point(p: OutlinePoint) -> Value {
    json!([scalar(p.x), scalar(p.y)])
}
fn rect(r: ExactClip) -> Value {
    json!([
        scalar(r.left),
        scalar(r.top),
        scalar(r.right),
        scalar(r.bottom)
    ])
}
fn resource(key: &ResourceKey) -> Value {
    match key {
        ResourceKey::Physical {
            font_sha256,
            tfm_sha256,
            face_index,
        } => {
            json!({"kind":"physical","font_sha256":font_sha256,"tfm_sha256":tfm_sha256,"face_index":face_index})
        }
        ResourceKey::Virtual {
            vf_sha256,
            tfm_sha256,
        } => json!({"kind":"virtual","vf_sha256":vf_sha256,"tfm_sha256":tfm_sha256}),
    }
}
fn quadratic(command: &PlacedPathCommand) -> Value {
    match command {
        PlacedPathCommand::MoveTo(p) => json!(["move", point(*p)]),
        PlacedPathCommand::LineTo(p) => json!(["line", point(*p)]),
        PlacedPathCommand::QuadTo { control, end } => json!(["quad", point(*control), point(*end)]),
        PlacedPathCommand::Close => json!(["close"]),
    }
}
fn cubic(command: &CubicPathCommand) -> Value {
    match command {
        CubicPathCommand::MoveTo(p) => json!(["move", point(*p)]),
        CubicPathCommand::LineTo(p) => json!(["line", point(*p)]),
        CubicPathCommand::CurveTo {
            control1,
            control2,
            end,
        } => json!(["cubic", point(*control1), point(*control2), point(*end)]),
        CubicPathCommand::Close => json!(["close"]),
    }
}
impl Serialize for MixedPrimitive {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let geometry = match &self.geometry {
            MixedGeometry::Quadratic(commands) => {
                json!({"kind":"quadratic","commands":commands.iter().map(quadratic).collect::<Vec<_>>()})
            }
            MixedGeometry::Rule(bounds) => json!({"kind":"rule","bounds":rect(*bounds)}),
            MixedGeometry::Cubic(path) => cubic_geometry_value(path),
        };
        json!({"identity":{"item_index":self.identity.item_index,"glyph_index":self.identity.glyph_index},"geometry":geometry,"clip":rect(self.clip),"paint":self.paint,"source_chain":self.source_chain.iter().map(|s|json!({"resource":resource(&s.resource),"character":s.character,"command_index":s.command_index})).collect::<Vec<_>>(),"sources":self.sources,"synthetic_reason":self.synthetic_reason,"logical_interval":self.logical_interval,"font_sha256":self.font_sha256,"original_gid":self.original_gid}).serialize(serializer)
    }
}
impl Serialize for MixedBatch {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("MixedBatch", 11)?;
        s.serialize_field("format", "flashtex-internal-mixed-v1")?;
        s.serialize_field("project_id", &self.project_id)?;
        s.serialize_field("revision", &self.revision)?;
        s.serialize_field("page", &self.page)?;
        s.serialize_field("page_size", &[self.page_width.0, self.page_height.0])?;
        s.serialize_field("clip", &rect(self.clip))?;
        s.serialize_field("primitives", &self.primitives)?;
        s.serialize_field("commands", &self.commands)?;
        s.serialize_field("color_space", "srgb")?;
        s.serialize_field("compositing", "source-over")?;
        s.serialize_field("hinting_applied", &false)?;
        s.end()
    }
}

pub(crate) fn cubic_geometry_value(path: &PositionedCubic) -> Value {
    let dyadic = |v: flashtex_font_resources::Coordinate| {
        json!([v.numerator().to_string(), (1u128 << v.shift()).to_string()])
    };
    json!({"kind":"cubic","cff_table_sha256":path.cff_table_sha256,"full_font_identity":path.full_font_identity.as_ref().map(|identity|json!({"font_sha256":identity.font_sha256,"cff_sha256":identity.cff_sha256,"face_index":identity.face_index,"table_range":[identity.table_range.start,identity.table_range.end]})),"font_matrix":path.font_matrix.iter().map(|v|json!([v.numerator().to_string(),v.denominator().to_string()])).collect::<Vec<_>>(),"advance":point(path.advance),"commands":path.commands.iter().map(cubic).collect::<Vec<_>>(),"hint_policy":match path.hints.policy{HintPolicy::Reject=>"reject",HintPolicy::Unhinted=>"unhinted"},"stems":path.hints.stems.iter().map(|h|json!({"vertical":h.vertical,"delta":dyadic(h.delta),"width":dyadic(h.width)})).collect::<Vec<_>>(),"masks":path.hints.masks.iter().map(|h|json!({"counter":h.counter,"stem_count":h.stem_count,"bytes":h.bytes})).collect::<Vec<_>>(),"flex_depths":path.hints.flex_depths.iter().map(|v|dyadic(*v)).collect::<Vec<_>>()})
}

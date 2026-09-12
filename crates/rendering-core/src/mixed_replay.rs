//! Bounded offline replay of the internal mixed fixture format. This validates
//! exact consumer geometry and retains metadata; it does not verify font bytes or
//! authorize painting, and is never a rendering wire negotiation path.
use crate::{
    batch::{ExactClip, PrimitiveId},
    mixed::{MixedError, MixedLimits, MixedResult},
    outlines::{OutlineCoordinate, OutlinePoint},
    *,
};
use serde_json::Value;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayCommand {
    Move(OutlinePoint),
    Line(OutlinePoint),
    Quadratic {
        control: OutlinePoint,
        end: OutlinePoint,
    },
    Cubic {
        control1: OutlinePoint,
        control2: OutlinePoint,
        end: OutlinePoint,
    },
    Close,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayGeometry {
    Quadratic(Vec<ReplayCommand>),
    Cubic(Vec<ReplayCommand>),
    Rule(ExactClip),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPrimitive {
    pub identity: PrimitiveId,
    pub geometry: ReplayGeometry,
    pub clip: ExactClip,
}
pub struct ReplayBatch {
    value: Value,
    primitives: Vec<ReplayPrimitive>,
    commands: usize,
}
impl ReplayBatch {
    pub fn primitives(&self) -> &[ReplayPrimitive] {
        &self.primitives
    }
    pub fn command_count(&self) -> usize {
        self.commands
    }
    pub fn metadata(&self) -> &Value {
        &self.value
    }
    pub fn canonical_bytes(&self) -> MixedResult<Vec<u8>> {
        serde_json::to_vec(&self.value).map_err(|e| MixedError::Serialization(e.to_string()))
    }
    pub fn parse(bytes: &[u8], limits: MixedLimits) -> MixedResult<Self> {
        if bytes.len() > limits.max_serialized_bytes.min(MAX_MESSAGE_BYTES)
            || limits.max_primitives == 0
            || limits.max_primitives > 100000
            || limits.max_commands == 0
            || limits.max_commands > 2_000_000
        {
            return Err(MixedError::Budget);
        }
        let StrictValue(value): StrictValue =
            serde_json::from_slice(bytes).map_err(|e| MixedError::Serialization(e.to_string()))?;
        object(
            &value,
            &[
                "format",
                "project_id",
                "revision",
                "page",
                "page_size",
                "clip",
                "primitives",
                "commands",
                "color_space",
                "compositing",
                "hinting_applied",
            ],
        )?;
        if string(&value["format"])? != "flashtex-internal-mixed-v1"
            || string(&value["color_space"])? != "srgb"
            || string(&value["compositing"])? != "source-over"
            || value["hinting_applied"] != false
        {
            return Err(MixedError::Unsupported(
                "mixed fixture format or rendering policy".into(),
            ));
        }
        id(string(&value["project_id"])?)?;
        integer(&value["revision"])?;
        if integer(&value["page"])? == 0 {
            return Err(MixedError::Identity);
        }
        let size = array(&value["page_size"], Some(2))?;
        for v in size {
            let n = v.as_i64().ok_or(MixedError::Identity)?;
            Tick(n).positive()?;
        }
        let clip = rectangle(&value["clip"])?;
        let page = ExactClip::from_rect(&HitRect {
            x: Tick(0),
            top: Tick(0),
            width: Tick(size[0].as_i64().unwrap()),
            height: Tick(size[1].as_i64().unwrap()),
        })?;
        if clip.intersect(page)? != Some(clip) {
            return Err(MixedError::Identity);
        }
        let input = array(&value["primitives"], None)?;
        if input.len() > limits.max_primitives {
            return Err(MixedError::Budget);
        }
        let mut primitives = Vec::new();
        let mut identities = BTreeSet::new();
        let mut commands = 0usize;
        for p in input {
            object(
                p,
                &[
                    "identity",
                    "geometry",
                    "clip",
                    "paint",
                    "source_chain",
                    "sources",
                    "synthetic_reason",
                    "logical_interval",
                    "font_sha256",
                    "original_gid",
                ],
            )?;
            object(&p["identity"], &["item_index", "glyph_index"])?;
            let identity = PrimitiveId {
                item_index: usize_integer(&p["identity"]["item_index"])?,
                glyph_index: optional_usize(&p["identity"]["glyph_index"])?,
            };
            if !identities.insert(identity) {
                return Err(MixedError::Identity);
            }
            let primitive_clip = rectangle(&p["clip"])?;
            if primitive_clip.intersect(clip)? != Some(primitive_clip) {
                return Err(MixedError::Identity);
            }
            let paint: Paint = serde_json::from_value(p["paint"].clone())
                .map_err(|e| MixedError::Serialization(e.to_string()))?;
            paint.validate()?;
            let sources: Vec<SourceRange> = serde_json::from_value(p["sources"].clone())
                .map_err(|e| MixedError::Serialization(e.to_string()))?;
            if sources.len() > 128 {
                return Err(MixedError::Budget);
            }
            for source in sources {
                path(&source.path)?;
                require(
                    source.start_byte < source.end_byte,
                    "invalid replay source interval",
                )?;
            }
            if !p["synthetic_reason"].is_null() {
                text(string(&p["synthetic_reason"])?, 1, 4096, "synthetic reason")?;
            }
            if !p["logical_interval"].is_null() {
                let range = array(&p["logical_interval"], Some(2))?;
                require(
                    integer(&range[0])? < integer(&range[1])?,
                    "invalid replay logical interval",
                )?;
            }
            let source_chain = array(&p["source_chain"], None)?;
            if source_chain.len() > 34 {
                return Err(MixedError::Budget);
            }
            for step in source_chain {
                validate_step(step)?;
            }
            if !p["font_sha256"].is_null() {
                hash(string(&p["font_sha256"])?)?;
            }
            let g = &p["geometry"];
            let kind = string(&g["kind"])?;
            let geometry = match kind {
                "quadratic" | "cubic" => {
                    if integer(&p["original_gid"])? == 0 || p["logical_interval"].is_null() {
                        return Err(MixedError::Identity);
                    }
                    if kind == "quadratic" {
                        object(g, &["kind", "commands"])?;
                        hash(string(&p["font_sha256"])?)?;
                    } else {
                        let mut fields = vec![
                            "kind",
                            "cff_table_sha256",
                            "font_matrix",
                            "advance",
                            "commands",
                            "hint_policy",
                            "stems",
                            "masks",
                            "flex_depths",
                        ];
                        if g.get("full_font_identity").is_some() {
                            fields.push("full_font_identity");
                        }
                        object(g, &fields)?;
                        if let Some(identity) = g.get("full_font_identity").filter(|v| !v.is_null())
                        {
                            object(
                                identity,
                                &["font_sha256", "cff_sha256", "face_index", "table_range"],
                            )?;
                            hash(string(&identity["font_sha256"])?)?;
                            require(
                                identity["cff_sha256"] == g["cff_table_sha256"]
                                    && integer(&identity["face_index"])? == 0,
                                "CFF full resource identity",
                            )?;
                            let range = array(&identity["table_range"], Some(2))?;
                            require(integer(&range[0])? < integer(&range[1])?, "CFF table range")?;
                        }
                        hash(string(&g["cff_table_sha256"])?)?;
                        for m in array(&g["font_matrix"], Some(6))? {
                            scalar(m)?;
                        }
                        point(&g["advance"])?;
                        if !["reject", "unhinted"].contains(&string(&g["hint_policy"])?) {
                            return Err(MixedError::Unsupported("hint policy".into()));
                        }
                        for stem in array(&g["stems"], None)? {
                            object(stem, &["vertical", "delta", "width"])?;
                            boolean(&stem["vertical"])?;
                            scalar(&stem["delta"])?;
                            scalar(&stem["width"])?;
                        }
                        for mask in array(&g["masks"], None)? {
                            object(mask, &["counter", "stem_count", "bytes"])?;
                            boolean(&mask["counter"])?;
                            let stems = integer(&mask["stem_count"])?;
                            let data = array(&mask["bytes"], None)?;
                            require(
                                stems <= 96 && data.len() as u64 == stems.div_ceil(8),
                                "invalid hint mask extent",
                            )?;
                            for byte in data {
                                require(integer(byte)? <= 255, "invalid hint byte")?;
                            }
                        }
                        for depth in array(&g["flex_depths"], None)? {
                            scalar(depth)?;
                        }
                    }
                    let input = array(&g["commands"], None)?;
                    commands = commands
                        .checked_add(input.len())
                        .ok_or(MixedError::Budget)?;
                    if commands > limits.max_commands {
                        return Err(MixedError::Budget);
                    }
                    let mut output = Vec::new();
                    for c in input {
                        let parts = array(c, None)?;
                        let Some(tag) = parts.first() else {
                            return Err(MixedError::Identity);
                        };
                        let tag = string(tag)?;
                        let (arity, command) = match tag {
                            "move" | "line" => {
                                if parts.len() != 2 {
                                    return Err(MixedError::Identity);
                                }
                                let p = point(&parts[1])?;
                                (
                                    2,
                                    if tag == "move" {
                                        ReplayCommand::Move(p)
                                    } else {
                                        ReplayCommand::Line(p)
                                    },
                                )
                            }
                            "quad" if kind == "quadratic" => {
                                if parts.len() != 3 {
                                    return Err(MixedError::Identity);
                                }
                                (
                                    3,
                                    ReplayCommand::Quadratic {
                                        control: point(&parts[1])?,
                                        end: point(&parts[2])?,
                                    },
                                )
                            }
                            "cubic" if kind == "cubic" => {
                                if parts.len() != 4 {
                                    return Err(MixedError::Identity);
                                }
                                (
                                    4,
                                    ReplayCommand::Cubic {
                                        control1: point(&parts[1])?,
                                        control2: point(&parts[2])?,
                                        end: point(&parts[3])?,
                                    },
                                )
                            }
                            "close" => (1, ReplayCommand::Close),
                            _ => {
                                return Err(MixedError::Unsupported(format!(
                                    "{kind} command {tag}"
                                )))
                            }
                        };
                        if parts.len() != arity {
                            return Err(MixedError::Identity);
                        }
                        output.push(command);
                    }
                    if kind == "quadratic" {
                        ReplayGeometry::Quadratic(output)
                    } else {
                        ReplayGeometry::Cubic(output)
                    }
                }
                "rule" => {
                    object(g, &["kind", "bounds"])?;
                    if !p["original_gid"].is_null() || !p["font_sha256"].is_null() {
                        return Err(MixedError::Identity);
                    }
                    ReplayGeometry::Rule(rectangle(&g["bounds"])?)
                }
                other => return Err(MixedError::Unsupported(format!("mixed primitive {other}"))),
            };
            primitives.push(ReplayPrimitive {
                identity,
                geometry,
                clip: primitive_clip,
            });
        }
        if integer(&value["commands"])? != commands as u64 {
            return Err(MixedError::Identity);
        }
        Ok(Self {
            value,
            primitives,
            commands,
        })
    }
}
fn object(value: &Value, fields: &[&str]) -> MixedResult<()> {
    let object = value.as_object().ok_or(MixedError::Identity)?;
    if object.len() != fields.len() || fields.iter().any(|field| !object.contains_key(*field)) {
        return Err(MixedError::Unsupported(
            "unknown or missing fixture field".into(),
        ));
    }
    Ok(())
}
fn array(value: &Value, length: Option<usize>) -> MixedResult<&[Value]> {
    let a = value.as_array().ok_or(MixedError::Identity)?;
    if length.is_some_and(|n| a.len() != n) {
        return Err(MixedError::Identity);
    }
    Ok(a)
}
fn string(value: &Value) -> MixedResult<&str> {
    value.as_str().ok_or(MixedError::Identity)
}
fn integer(value: &Value) -> MixedResult<u64> {
    value.as_u64().ok_or(MixedError::Identity)
}
fn usize_integer(value: &Value) -> MixedResult<usize> {
    usize::try_from(integer(value)?).map_err(|_| MixedError::Identity)
}
fn optional_usize(value: &Value) -> MixedResult<Option<usize>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(usize_integer(value)?))
    }
}
fn boolean(value: &Value) -> MixedResult<bool> {
    value.as_bool().ok_or(MixedError::Identity)
}
fn scalar(value: &Value) -> MixedResult<OutlineCoordinate> {
    let parts = array(value, Some(2))?;
    let n = string(&parts[0])?;
    let d = string(&parts[1])?;
    let coordinate = OutlineCoordinate::from_fraction(
        n.parse().map_err(|_| MixedError::Identity)?,
        d.parse().map_err(|_| MixedError::Identity)?,
    )?;
    if coordinate.numerator().to_string() != n || coordinate.denominator().to_string() != d {
        return Err(MixedError::Unsupported("noncanonical rational".into()));
    }
    Ok(coordinate)
}
fn point(value: &Value) -> MixedResult<OutlinePoint> {
    let a = array(value, Some(2))?;
    Ok(OutlinePoint {
        x: scalar(&a[0])?,
        y: scalar(&a[1])?,
    })
}
fn rectangle(value: &Value) -> MixedResult<ExactClip> {
    let a = array(value, Some(4))?;
    let clip = ExactClip {
        left: scalar(&a[0])?,
        top: scalar(&a[1])?,
        right: scalar(&a[2])?,
        bottom: scalar(&a[3])?,
    };
    clip.validate()?;
    Ok(clip)
}
fn validate_step(step: &Value) -> MixedResult<()> {
    object(step, &["resource", "character", "command_index"])?;
    require(
        integer(&step["character"])? <= 255,
        "encoded character range",
    )?;
    optional_usize(&step["command_index"])?;
    let r = &step["resource"];
    match string(&r["kind"])? {
        "physical" => {
            object(r, &["kind", "font_sha256", "tfm_sha256", "face_index"])?;
            hash(string(&r["font_sha256"])?)?;
            hash(string(&r["tfm_sha256"])?)?;
            require(integer(&r["face_index"])? == 0, "resource face")?;
        }
        "virtual" => {
            object(r, &["kind", "vf_sha256", "tfm_sha256"])?;
            hash(string(&r["vf_sha256"])?)?;
            hash(string(&r["tfm_sha256"])?)?;
        }
        other => return Err(MixedError::Unsupported(format!("resource {other}"))),
    }
    Ok(())
}

// serde_json::Value normally accepts duplicate keys. Fixture replay rejects them
// recursively so a consumer cannot select a different geometry or provenance field.
struct StrictValue(Value);
impl<'de> serde::Deserialize<'de> for StrictValue {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StrictValue;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("JSON with unique object keys")
            }
            fn visit_bool<E: serde::de::Error>(
                self,
                v: bool,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::from(v)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::from(v)))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> std::result::Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|v| StrictValue(Value::Number(v)))
                    .ok_or_else(|| E::custom("nonfinite number"))
            }
            fn visit_str<E: serde::de::Error>(
                self,
                v: &str,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::String(v.into())))
            }
            fn visit_string<E: serde::de::Error>(
                self,
                v: String,
            ) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::String(v)))
            }
            fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictValue(value)) = seq.next_element()? {
                    values.push(value);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(serde::de::Error::custom("duplicate object key"));
                    }
                    let StrictValue(value) = map.next_value()?;
                    values.insert(key, value);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

pub(crate) fn parse_unique(bytes: &[u8]) -> std::result::Result<Value, serde_json::Error> {
    serde_json::from_slice::<StrictValue>(bytes).map(|v| v.0)
}

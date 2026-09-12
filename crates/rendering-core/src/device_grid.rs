//! Explicit device-context paths, separate from the size-independent unhinted
//! cache. Component offset grid policy does not execute TrueType instructions.
use crate::{
    outlines::{place_path_exact, OutlineCoordinate, OutlinePoint, PlacedPathCommand},
    *,
};
use flashtex_font_resources::{
    CompositeDeviceGrid, FontResource as LoadedFont, GlyphInstance, GridTieRule, PathCommand,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TieRule {
    AwayFromZero,
    TowardPositive,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformOrder {
    OffsetTransformThenGridChildAssemblyBeforeParentV1,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceContext {
    ppem_x: u32,
    ppem_y: u32,
    tie_rule: TieRule,
    transform_order: TransformOrder,
    outline_policy_sha256: String,
}
impl DeviceContext {
    pub fn new(
        ppem_x: u32,
        ppem_y: u32,
        tie_rule: TieRule,
        outline_policy_sha256: &str,
    ) -> Result<Self> {
        let value = Self {
            ppem_x,
            ppem_y,
            tie_rule,
            transform_order: TransformOrder::OffsetTransformThenGridChildAssemblyBeforeParentV1,
            outline_policy_sha256: outline_policy_sha256.into(),
        };
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        require(
            (1..=65536).contains(&self.ppem_x) && (1..=65536).contains(&self.ppem_y),
            "device ppem outside bounded profile",
        )?;
        hash(&self.outline_policy_sha256)
    }
    fn grid(&self) -> CompositeDeviceGrid {
        CompositeDeviceGrid {
            ppem_x: self.ppem_x,
            ppem_y: self.ppem_y,
            tie_rule: match self.tie_rule {
                TieRule::AwayFromZero => GridTieRule::AwayFromZero,
                TieRule::TowardPositive => GridTieRule::TowardPositive,
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceKey {
    pub font_sha256: String,
    pub face_index: u32,
    pub original_gid: u16,
    pub context: DeviceContext,
}
#[derive(Debug)]
pub struct DevicePath {
    pub key: DeviceKey,
    pub units_per_em: u32,
    pub instances: Vec<GlyphInstance>,
    pub commands: Vec<PathCommand>,
}
#[derive(Debug, Clone)]
pub enum DeviceFailure {
    Font(flashtex_font_resources::Error),
    PayloadBudget { required: usize, limit: usize },
}
#[derive(Debug, Clone)]
pub enum DeviceOutcome {
    Ready(Arc<DevicePath>),
    Unavailable(Arc<DeviceFailure>),
}
#[derive(Debug, Clone)]
pub struct DeviceLookup {
    pub outcome: DeviceOutcome,
    pub cache_hit: bool,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct DeviceStats {
    pub hits: u64,
    pub expansions: u64,
    pub context_invalidations: u64,
    pub evictions: u64,
    pub entries: usize,
    pub retained_payload_bytes: usize,
}
struct Entry {
    outcome: DeviceOutcome,
    bytes: usize,
    used: u64,
}
pub struct DevicePathCache {
    context: Option<DeviceContext>,
    entries: BTreeMap<DeviceKey, Entry>,
    max_entries: usize,
    max_bytes: usize,
    clock: u64,
    stats: DeviceStats,
}
impl DevicePathCache {
    pub fn new(max_entries: usize, max_bytes: usize) -> Result<Self> {
        require(
            (1..=4096).contains(&max_entries) && (512..=256 * 1024 * 1024).contains(&max_bytes),
            "invalid device cache budget",
        )?;
        Ok(Self {
            context: None,
            entries: BTreeMap::new(),
            max_entries,
            max_bytes,
            clock: 0,
            stats: DeviceStats::default(),
        })
    }
    pub fn stats(&self) -> DeviceStats {
        DeviceStats {
            entries: self.entries.len(),
            ..self.stats
        }
    }
    pub fn lookup(
        &mut self,
        font: &LoadedFont,
        gid: u16,
        context: Option<&DeviceContext>,
    ) -> Result<DeviceLookup> {
        let context =
            context.ok_or_else(|| ValidationError("explicit device context required".into()))?;
        context.validate()?;
        if self.context.as_ref() != Some(context) {
            if self.context.is_some() {
                self.stats.context_invalidations += 1;
            }
            self.entries.clear();
            self.stats.retained_payload_bytes = 0;
            self.context = Some(context.clone());
        }
        self.clock = self
            .clock
            .checked_add(1)
            .ok_or_else(|| ValidationError("device LRU clock exhausted".into()))?;
        let key = DeviceKey {
            font_sha256: font.descriptor().sha256.clone(),
            face_index: font.descriptor().face_index,
            original_gid: gid,
            context: context.clone(),
        };
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.used = self.clock;
            self.stats.hits += 1;
            return Ok(DeviceLookup {
                outcome: entry.outcome.clone(),
                cache_hit: true,
            });
        }
        self.stats.expansions += 1;
        let (outcome, bytes) = match expand_device(font, gid, context) {
            Ok(path) => {
                let bytes = 512usize
                    .saturating_add(
                        path.commands
                            .len()
                            .saturating_mul(std::mem::size_of::<PathCommand>()),
                    )
                    .saturating_add(
                        path.instances
                            .len()
                            .saturating_mul(std::mem::size_of::<GlyphInstance>()),
                    );
                if bytes > self.max_bytes {
                    (
                        DeviceOutcome::Unavailable(Arc::new(DeviceFailure::PayloadBudget {
                            required: bytes,
                            limit: self.max_bytes,
                        })),
                        512,
                    )
                } else {
                    (DeviceOutcome::Ready(Arc::new(path)), bytes)
                }
            }
            Err(error) => {
                let bytes = 512usize.saturating_add(error.to_string().len());
                (
                    DeviceOutcome::Unavailable(Arc::new(DeviceFailure::Font(error))),
                    bytes,
                )
            }
        };
        if bytes <= self.max_bytes {
            while self.entries.len() >= self.max_entries
                || self.stats.retained_payload_bytes > self.max_bytes - bytes
            {
                let oldest = self
                    .entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.used)
                    .map(|(key, _)| key.clone())
                    .ok_or_else(|| ValidationError("device cache eviction invariant".into()))?;
                let old = self.entries.remove(&oldest).unwrap();
                self.stats.retained_payload_bytes -= old.bytes;
                self.stats.evictions += 1;
            }
            self.stats.retained_payload_bytes += bytes;
            self.entries.insert(
                key,
                Entry {
                    outcome: outcome.clone(),
                    bytes,
                    used: self.clock,
                },
            );
        }
        Ok(DeviceLookup {
            outcome,
            cache_hit: false,
        })
    }
}
pub fn expand_device(
    font: &LoadedFont,
    gid: u16,
    context: &DeviceContext,
) -> flashtex_font_resources::Result<DevicePath> {
    context
        .validate()
        .map_err(|e| flashtex_font_resources::Error::InvalidFont(e.0))?;
    let expanded = font.expanded_outline_with_grid(gid, context.grid())?;
    let commands = expanded.outline.quadratic_path()?.collect();
    Ok(DevicePath {
        key: DeviceKey {
            font_sha256: font.descriptor().sha256.clone(),
            face_index: font.descriptor().face_index,
            original_gid: gid,
            context: context.clone(),
        },
        units_per_em: expanded.units_per_em,
        instances: expanded.outline.instances,
        commands,
    })
}
#[derive(Debug)]
pub struct PositionedDevicePath {
    pub key: DeviceKey,
    pub units_per_em: u32,
    pub commands: Vec<PlacedPathCommand>,
    pub hinting_applied: bool,
    size: OutlineCoordinate,
    origin: OutlinePoint,
}
impl DevicePath {
    pub fn place(
        &self,
        size: OutlineCoordinate,
        origin: OutlinePoint,
    ) -> Result<PositionedDevicePath> {
        Ok(PositionedDevicePath {
            key: self.key.clone(),
            units_per_em: self.units_per_em,
            commands: place_path_exact(
                self.commands.iter().copied(),
                size,
                self.units_per_em,
                origin,
            )?,
            hinting_applied: false,
            size,
            origin,
        })
    }
}
impl PositionedDevicePath {
    pub fn comparison_fixture(&self, max_bytes: usize) -> Result<Vec<u8>> {
        use serde_json::json;
        require(
            (1..=MAX_MESSAGE_BYTES).contains(&max_bytes),
            "device fixture byte budget",
        )?;
        let scalar =
            |v: OutlineCoordinate| json!([v.numerator().to_string(), v.denominator().to_string()]);
        let point = |v: OutlinePoint| json!([scalar(v.x), scalar(v.y)]);
        let commands = self
            .commands
            .iter()
            .map(|c| match c {
                PlacedPathCommand::MoveTo(p) => json!(["move", point(*p)]),
                PlacedPathCommand::LineTo(p) => json!(["line", point(*p)]),
                PlacedPathCommand::QuadTo { control, end } => {
                    json!(["quad", point(*control), point(*end)])
                }
                PlacedPathCommand::Close => json!(["close"]),
            })
            .collect::<Vec<_>>();
        let value = json!({"format":"flashtex-internal-device-v1","font_sha256":self.key.font_sha256,"face_index":self.key.face_index,"original_gid":self.key.original_gid,"device_context":self.key.context,"units_per_em":self.units_per_em,"size":scalar(self.size),"origin":point(self.origin),"commands":commands,"hinting_applied":false});
        let mut output = crate::mixed::BoundedOutput {
            bytes: vec![],
            limit: max_bytes,
        };
        serde_json::to_writer(&mut output, &value)
            .map_err(|_| ValidationError("device fixture exceeds byte budget".into()))?;
        Ok(output.bytes)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceFixture {
    format: String,
    font_sha256: String,
    face_index: u32,
    original_gid: u16,
    device_context: DeviceContext,
    units_per_em: u32,
    size: serde_json::Value,
    origin: serde_json::Value,
    commands: Vec<serde_json::Value>,
    hinting_applied: bool,
}
pub(crate) fn validate_fixture(bytes: &[u8]) -> Result<serde_json::Value> {
    require(
        bytes.len() <= MAX_MESSAGE_BYTES,
        "device fixture byte budget",
    )?;
    let value =
        crate::mixed_replay::parse_unique(bytes).map_err(|e| ValidationError(e.to_string()))?;
    let data: DeviceFixture =
        serde_json::from_value(value.clone()).map_err(|e| ValidationError(e.to_string()))?;
    require(
        data.format == "flashtex-internal-device-v1"
            && data.face_index == 0
            && data.original_gid > 0
            && !data.hinting_applied,
        "invalid device fixture identity/policy",
    )?;
    hash(&data.font_sha256)?;
    data.device_context.validate()?;
    require(
        (16..=16384).contains(&data.units_per_em) && data.commands.len() <= 2_000_000,
        "device geometry budget",
    )?;
    let scalar = |v: &serde_json::Value| -> Result<OutlineCoordinate> {
        let a = v
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| ValidationError("device rational pair".into()))?;
        let n = a[0]
            .as_str()
            .ok_or_else(|| ValidationError("device numerator string".into()))?;
        let d = a[1]
            .as_str()
            .ok_or_else(|| ValidationError("device denominator string".into()))?;
        let c = OutlineCoordinate::from_fraction(
            n.parse()
                .map_err(|_| ValidationError("device numerator".into()))?,
            d.parse()
                .map_err(|_| ValidationError("device denominator".into()))?,
        )?;
        require(
            c.numerator().to_string() == n && c.denominator().to_string() == d,
            "noncanonical device rational",
        )?;
        Ok(c)
    };
    let point = |v: &serde_json::Value| -> Result<()> {
        let a = v
            .as_array()
            .filter(|a| a.len() == 2)
            .ok_or_else(|| ValidationError("device point".into()))?;
        scalar(&a[0])?;
        scalar(&a[1])?;
        Ok(())
    };
    require(
        scalar(&data.size)?.numerator() > 0,
        "nonpositive device size",
    )?;
    point(&data.origin)?;
    for command in data.commands {
        let a = command
            .as_array()
            .ok_or_else(|| ValidationError("device command array".into()))?;
        let tag = a
            .first()
            .and_then(|v| v.as_str())
            .ok_or_else(|| ValidationError("device command kind".into()))?;
        let length = match tag {
            "move" | "line" => 2,
            "quad" => 3,
            "close" => 1,
            _ => return Err(ValidationError("unsupported device command".into())),
        };
        require(a.len() == length, "device command arity")?;
        for p in &a[1..] {
            point(p)?;
        }
    }
    Ok(value)
}

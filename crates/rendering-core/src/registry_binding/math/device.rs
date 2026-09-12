//! Explicit pixel correction conversion. Never inferred from page size or zoom.
use super::*;
use flashtex_font_resources::{math_device::*, math_kern::Corner};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelScale {
    pub horizontal: Q,
    pub vertical: Q,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceQuery {
    Constant {
        record: ConstantDeviceRecord,
        context: DeviceContext,
        axis: Axis,
    },
    Glyph {
        original_gid: u16,
        kind: GlyphDeviceKind,
        context: DeviceContext,
        axis: Axis,
    },
    Kern {
        original_gid: u16,
        corner: Corner,
        height: Rational,
        context: KernDeviceContext,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceValue {
    pub query: DeviceQuery,
    pub design_units: Option<i16>,
    pub delta_pixels: i8,
    pub base_ticks: Option<Q>,
    pub correction_ticks: Q,
    pub combined_ticks: Option<Q>,
    pub table_offset: Option<usize>,
    pub table_sha256: Option<String>,
    pub selected_interval: Option<usize>,
}
#[derive(Debug)]
pub enum DeviceConsumerError {
    Metric(MathConsumerError),
    Device(DeviceError),
}
impl From<MathConsumerError> for DeviceConsumerError {
    fn from(e: MathConsumerError) -> Self {
        Self::Metric(e)
    }
}
impl From<ValidationError> for DeviceConsumerError {
    fn from(e: ValidationError) -> Self {
        Self::Metric(e.into())
    }
}
impl From<DeviceError> for DeviceConsumerError {
    fn from(e: DeviceError) -> Self {
        Self::Device(e)
    }
}
pub struct MathDeviceSnapshot {
    metrics: MathMetricsSnapshot,
    scale: PixelScale,
    values: Vec<DeviceValue>,
}
impl MathDeviceSnapshot {
    pub fn metrics(&self) -> &MathMetricsSnapshot {
        &self.metrics
    }
    pub fn values(&self) -> &[DeviceValue] {
        &self.values
    }
    pub fn pixel_scale(&self) -> PixelScale {
        self.scale
    }
    pub fn require_current(
        &self,
        r: &RegistryRenderer,
        path: &str,
        s: &SourceSnapshot,
    ) -> MathResult<()> {
        self.metrics.require_current(r, path, s)
    }
    pub fn replay_bytes(&self, max_bytes: usize) -> MathResult<Vec<u8>> {
        let metrics: serde_json::Value =
            serde_json::from_slice(&self.metrics.replay_bytes(max_bytes)?)
                .map_err(|e| ValidationError(e.to_string()))?;
        let q = |v: Q| serde_json::json!([v.numerator().to_string(), v.denominator().to_string()]);
        let values:Vec<_>=self.values.iter().map(|v|{
   let query=match v.query {
    DeviceQuery::Constant{record,context,axis}=>serde_json::json!({"kind":"constant","record":record.index(),"ppem":context.ppem(),"axis":format!("{axis:?}")}),
    DeviceQuery::Glyph{original_gid,kind,context,axis}=>serde_json::json!({"kind":"glyph","gid":original_gid,"record":format!("{kind:?}"),"ppem":context.ppem(),"axis":format!("{axis:?}")}),
    DeviceQuery::Kern{original_gid,corner,height,context}=>serde_json::json!({"kind":"kern","gid":original_gid,"corner":format!("{corner:?}"),"height_design_units":[height.numerator().to_string(),height.denominator().to_string()],"ppem_x":context.horizontal.ppem(),"ppem_y":context.vertical.ppem()}),
   };
   serde_json::json!({"query":query,"raw_design_units":v.design_units,"delta_pixels":v.delta_pixels,"base_ticks":v.base_ticks.map(q),"correction_ticks":q(v.correction_ticks),"combined_ticks":v.combined_ticks.map(q),"table_offset":v.table_offset,"table_sha256":v.table_sha256,"selected_interval":v.selected_interval})
  }).collect();
        let value = serde_json::json!({"format":"flashtex-internal-math-device-v1","consumer_source_sha256":digest(include_bytes!("device.rs")),"metrics":metrics,"pixel_scale_x":q(self.scale.horizontal),"pixel_scale_y":q(self.scale.vertical),"values":values,"outline_hinting_applied":false,"script_layout_performed":false});
        let mut out = crate::mixed::BoundedOutput {
            bytes: vec![],
            limit: max_bytes,
        };
        serde_json::to_writer(&mut out, &value).map_err(|_| MathConsumerError::Budget)?;
        Ok(out.bytes)
    }
    pub fn verify_replay(
        &self,
        r: &RegistryRenderer,
        path: &str,
        s: &SourceSnapshot,
        bytes: &[u8],
    ) -> MathResult<()> {
        self.require_current(r, path, s)?;
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(MathConsumerError::Budget);
        }
        let actual =
            crate::mixed_replay::parse_unique(bytes).map_err(|e| ValidationError(e.to_string()))?;
        let expected: serde_json::Value =
            serde_json::from_slice(&self.replay_bytes(MAX_MESSAGE_BYTES)?)
                .map_err(|e| ValidationError(e.to_string()))?;
        require(actual == expected, "MATH device replay mismatch")?;
        Ok(())
    }
}
impl RegistryRenderer {
    pub fn math_devices(
        &self,
        lease: &MathLease,
        query: MathQuery<'_>,
        scale: PixelScale,
        requests: &[DeviceQuery],
    ) -> std::result::Result<MathDeviceSnapshot, DeviceConsumerError> {
        self.math_devices_with_values(lease, query, scale, requests, None)
    }
    pub(super) fn math_devices_with_values(
        &self,
        lease: &MathLease,
        query: MathQuery<'_>,
        scale: PixelScale,
        requests: &[DeviceQuery],
        cached: Option<&[flashtex_font_resources::math_cache::Value]>,
    ) -> std::result::Result<MathDeviceSnapshot, DeviceConsumerError> {
        if requests.len() > 256 {
            return Err(MathConsumerError::Budget.into());
        }
        require(
            scale.horizontal.numerator() > 0 && scale.vertical.numerator() > 0,
            "positive explicit pixel scales",
        )?;
        let metrics = self.math_metrics(lease, query)?;
        let mut values = Vec::with_capacity(requests.len());
        for (index, &query) in requests.iter().enumerate() {
            let cached = cached.map(|values| &values[index]);
            let (raw, delta, offset, sha, interval, axis) = match query {
                DeviceQuery::Constant {
                    record,
                    context,
                    axis,
                } => {
                    let c = match cached {
                        Some(flashtex_font_resources::math_cache::Value::Constant(c)) => c.clone(),
                        None => lease
                            .font
                            .constant_device(record, context)?
                            .correction()
                            .clone(),
                        _ => {
                            return Err(
                                ValidationError("cached constant kind mismatch".into()).into()
                            )
                        }
                    };
                    let raw = MathConstant::ALL[usize::from(record.index()) + 2]
                        .raw(&lease.font)
                        .0;
                    (
                        Some(
                            i16::try_from(raw).map_err(|_| {
                                ValidationError("constant device raw bounds".into())
                            })?,
                        ),
                        c.delta_pixels,
                        c.device_table_offset,
                        c.device_table_sha256.clone(),
                        None,
                        axis,
                    )
                }
                DeviceQuery::Glyph {
                    original_gid,
                    kind,
                    context,
                    axis,
                } => {
                    let (design, c) = match cached {
                        Some(flashtex_font_resources::math_cache::Value::Glyph {
                            design_units,
                            correction,
                        }) => (*design_units, correction.clone()),
                        None => {
                            let v = lease.font.glyph_device(original_gid, kind, context)?;
                            (v.design_units(), v.correction().cloned())
                        }
                        _ => {
                            return Err(ValidationError("cached glyph kind mismatch".into()).into())
                        }
                    };
                    let c = c.as_ref();
                    (
                        design,
                        c.map_or(0, |v| v.delta_pixels),
                        c.and_then(|v| v.device_table_offset),
                        c.and_then(|v| v.device_table_sha256.clone()),
                        None,
                        axis,
                    )
                }
                DeviceQuery::Kern {
                    original_gid,
                    corner,
                    height,
                    context,
                } => {
                    let k = match cached {
                        Some(flashtex_font_resources::math_cache::Value::Kern(k)) => k.clone(),
                        None => lease
                            .font
                            .kern_device(original_gid, corner, height, context)?
                            .correction()
                            .clone(),
                        _ => return Err(ValidationError("cached kern kind mismatch".into()).into()),
                    };
                    let c = &k.correction;
                    (
                        Some(c.design_units),
                        c.delta_pixels,
                        c.device_table_offset,
                        c.device_table_sha256.clone(),
                        Some(k.selected_interval),
                        Axis::Horizontal,
                    )
                }
            };
            let correction_ticks = match axis {
                Axis::Horizontal => scale.horizontal,
                Axis::Vertical => scale.vertical,
            }
            .checked_multiply(Q::from_fraction(delta as i128, 1)?)?;
            let base_ticks = raw
                .map(|v| -> MathResult<Q> {
                    Ok(exact(
                        lease.font.scale_design_units(
                            v as i32,
                            rational(metrics.size)
                                .map_err(|e| MathConsumerError::Binding(BindingError::Font(e)))?,
                        )?,
                    )?)
                })
                .transpose()?;
            let combined_ticks = base_ticks
                .map(|v| v.checked_add(correction_ticks))
                .transpose()?;
            values.push(DeviceValue {
                query,
                design_units: raw,
                delta_pixels: delta,
                base_ticks,
                correction_ticks,
                combined_ticks,
                table_offset: offset,
                table_sha256: sha,
                selected_interval: interval,
            });
        }
        Ok(MathDeviceSnapshot {
            metrics,
            scale,
            values,
        })
    }
}

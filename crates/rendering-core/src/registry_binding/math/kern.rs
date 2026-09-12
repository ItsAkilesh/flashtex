//! Exact unhinted MathKernInfo lookup replay; this does not position scripts.
use super::*;
use flashtex_font_resources::math_kern::Corner;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernQuery {
    pub original_gid: u16,
    pub corner: Corner,
    pub height: Rational,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernValue {
    pub query: KernQuery,
    pub value: ScaledLength,
    pub value_device_adjustment_present: bool,
    pub height_device_adjustment_present: bool,
}
pub struct MathKernSnapshot {
    metrics: MathMetricsSnapshot,
    values: Vec<KernValue>,
}
impl MathKernSnapshot {
    pub fn metrics(&self) -> &MathMetricsSnapshot {
        &self.metrics
    }
    pub fn values(&self) -> &[KernValue] {
        &self.values
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
        let metrics = self.metrics.replay_bytes(max_bytes)?;
        let metrics: serde_json::Value =
            serde_json::from_slice(&metrics).map_err(|e| ValidationError(e.to_string()))?;
        let q = |v: Q| serde_json::json!([v.numerator().to_string(), v.denominator().to_string()]);
        let values:Vec<_>=self.values.iter().map(|v|serde_json::json!({"original_gid":v.query.original_gid,"corner":format!("{:?}",v.query.corner),"height_design_units":[v.query.height.numerator().to_string(),v.query.height.denominator().to_string()],"raw":v.value.design_units,"ticks":q(v.value.ticks),"value_device_present":v.value_device_adjustment_present,"height_device_present":v.height_device_adjustment_present})).collect();
        let value = serde_json::json!({"format":"flashtex-internal-math-kern-v1","consumer_source_sha256":digest(include_bytes!("kern.rs")),"metrics":metrics,"values":values,"tie_policy":"upper_bound","device_adjustments_applied":false,"script_layout_performed":false});
        let mut output = crate::mixed::BoundedOutput {
            bytes: vec![],
            limit: max_bytes,
        };
        serde_json::to_writer(&mut output, &value).map_err(|_| MathConsumerError::Budget)?;
        Ok(output.bytes)
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
        let actual = crate::mixed_replay::parse_unique(bytes)
            .map_err(|e| ValidationError(format!("MATH kern replay: {e:?}")))?;
        let expected: serde_json::Value =
            serde_json::from_slice(&self.replay_bytes(MAX_MESSAGE_BYTES)?)
                .map_err(|e| ValidationError(e.to_string()))?;
        require(actual == expected, "MATH kern replay mismatch")?;
        Ok(())
    }
}
impl RegistryRenderer {
    pub fn math_kerns(
        &self,
        lease: &MathLease,
        query: MathQuery<'_>,
        requests: &[KernQuery],
    ) -> MathResult<MathKernSnapshot> {
        self.math_kerns_with_values(lease, query, requests, None)
    }
    pub(super) fn math_kerns_with_values(
        &self,
        lease: &MathLease,
        query: MathQuery<'_>,
        requests: &[KernQuery],
        cached: Option<&[flashtex_font_resources::math_cache::Value]>,
    ) -> MathResult<MathKernSnapshot> {
        if requests.len() > 256 {
            return Err(MathConsumerError::Budget);
        }
        let metrics = self.math_metrics(lease, query)?;
        let kern = if cached.is_none() {
            Some(lease.font.kerns()?)
        } else {
            None
        };
        let mut values = Vec::with_capacity(requests.len());
        for (index, &query) in requests.iter().enumerate() {
            let (raw, height_device_adjustment_present) = if let Some(values) = cached {
                match &values[index] {
                    flashtex_font_resources::math_cache::Value::UnhintedKern {
                        value,
                        height_device_adjustment_present,
                    } => (*value, *height_device_adjustment_present),
                    _ => return Err(ValidationError("cached unhinted kern kind".into()).into()),
                }
            } else {
                let kern = kern.as_ref().expect("direct kern parser");
                let raw = kern
                    .data()
                    .lookup(query.original_gid, query.corner, query.height)
                    .map_err(|e| MathConsumerError::Binding(BindingError::Font(e)))?;
                let flag = kern
                    .data()
                    .records()
                    .get(&(query.original_gid, query.corner))
                    .is_some_and(|t| {
                        t.correction_heights()
                            .iter()
                            .any(|v| v.device_adjustment_present)
                    });
                (raw, flag)
            };
            let ticks = exact(
                lease.font.scale_design_units(
                    raw.design_units as i32,
                    rational(metrics.size)
                        .map_err(|e| MathConsumerError::Binding(BindingError::Font(e)))?,
                )?,
            )?;
            values.push(KernValue {
                query,
                value: ScaledLength {
                    design_units: raw.design_units,
                    ticks,
                },
                value_device_adjustment_present: raw.device_adjustment_present,
                height_device_adjustment_present,
            });
        }
        Ok(MathKernSnapshot { metrics, values })
    }
}

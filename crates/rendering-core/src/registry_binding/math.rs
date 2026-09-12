//! Exact registry-bound MATH metric consumer. No parser/layout duplication,
//! device evaluation, variants selection, implicit accent center or pixel rounding.
use super::*;
pub mod assembly;
pub mod kern;
use crate::outlines::OutlineCoordinate as Q;
use flashtex_font_resources::{
    cff::Rational,
    math_adapter::{BoundMathFont, Capability, MathError, MathIdentity, MathPolicy},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDimension {
    CanonicalTicks,
    DimensionlessRatio,
}
macro_rules! constants {
 (length:[$($length:ident),*]; percent:[$($percent:ident),*])=>{
 #[allow(non_camel_case_types)]
 #[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Serialize,Deserialize)]
 #[serde(rename_all="snake_case")]
 pub enum MathConstant{$($length,)*$($percent,)*}
 impl MathConstant{
 pub const ALL:&'static[Self]=&[$(Self::$length,)*$(Self::$percent,)*];
 fn raw(self,font:&BoundMathFont)->(i32,MetricDimension){let c=font.constants();match self{$(Self::$length=>(c.$length as i32,MetricDimension::CanonicalTicks),)*$(Self::$percent=>(c.$percent as i32,MetricDimension::DimensionlessRatio),)*}}
 }
 }
}
constants! { length:[delimited_sub_formula_min_height,display_operator_min_height,math_leading,axis_height,accent_base_height,flattened_accent_base_height,subscript_shift_down,subscript_top_max,subscript_baseline_drop_min,superscript_shift_up,superscript_shift_up_cramped,superscript_bottom_min,superscript_baseline_drop_max,sub_superscript_gap_min,superscript_bottom_max_with_subscript,space_after_script,upper_limit_gap_min,upper_limit_baseline_rise_min,lower_limit_gap_min,lower_limit_baseline_drop_min,stack_top_shift_up,stack_top_display_style_shift_up,stack_bottom_shift_down,stack_bottom_display_style_shift_down,stack_gap_min,stack_display_style_gap_min,stretch_stack_top_shift_up,stretch_stack_bottom_shift_down,stretch_stack_gap_above_min,stretch_stack_gap_below_min,fraction_numerator_shift_up,fraction_numerator_display_style_shift_up,fraction_denominator_shift_down,fraction_denominator_display_style_shift_down,fraction_numerator_gap_min,fraction_num_display_style_gap_min,fraction_rule_thickness,fraction_denominator_gap_min,fraction_denom_display_style_gap_min,skewed_fraction_horizontal_gap,skewed_fraction_vertical_gap,overbar_vertical_gap,overbar_rule_thickness,overbar_extra_ascender,underbar_vertical_gap,underbar_rule_thickness,underbar_extra_ascender,radical_vertical_gap,radical_display_style_vertical_gap,radical_rule_thickness,radical_extra_ascender,radical_kern_before_degree,radical_kern_after_degree]; percent:[script_percent_scale_down,script_script_percent_scale_down,radical_degree_bottom_raise_percent] }
#[derive(Debug)]
pub enum MathConsumerError {
    Binding(BindingError),
    Metric(MathError),
    Geometry(ValidationError),
    Budget,
}
impl From<BindingError> for MathConsumerError {
    fn from(v: BindingError) -> Self {
        Self::Binding(v)
    }
}
impl From<MathError> for MathConsumerError {
    fn from(v: MathError) -> Self {
        Self::Metric(v)
    }
}
impl From<ValidationError> for MathConsumerError {
    fn from(v: ValidationError) -> Self {
        Self::Geometry(v)
    }
}
pub type MathResult<T> = std::result::Result<T, MathConsumerError>;
#[derive(Clone)]
pub struct MathLease {
    render: RenderLease,
    font: Arc<BoundMathFont>,
}
impl MathLease {
    pub fn identity(&self) -> &MathIdentity {
        self.font.identity()
    }
    pub fn binding(&self) -> &ResourceBinding {
        self.render.binding()
    }
    pub fn require(&self, capability: Capability) -> MathResult<()> {
        self.font.require(capability)?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaledConstant {
    pub constant: MathConstant,
    pub raw: i32,
    pub dimension: MetricDimension,
    pub value: Q,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaledLength {
    pub design_units: i16,
    pub ticks: Q,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaledGlyphMath {
    pub original_gid: u16,
    pub italic_correction: ScaledLength,
    pub top_accent_attachment: Option<ScaledLength>,
}
pub struct MathQuery<'a> {
    pub source_path: &'a str,
    pub snapshot: &'a SourceSnapshot,
    pub source_range: std::ops::Range<usize>,
    pub font_size: Q,
    pub original_gids: &'a [u16],
}
#[derive(Debug, Clone, Serialize)]
struct SourceEvidence {
    path: String,
    revision: u64,
    sha256: String,
    range: std::ops::Range<usize>,
}
pub struct MathMetricsSnapshot {
    lease: MathLease,
    source: SourceEvidence,
    size: Q,
    constants: Vec<ScaledConstant>,
    glyphs: Vec<ScaledGlyphMath>,
}
fn exact(v: Rational) -> Result<Q> {
    Q::from_fraction(v.numerator(), v.denominator() as u128)
}
fn rational(v: Q) -> std::result::Result<Rational, flashtex_font_resources::Error> {
    Rational::new(v.numerator(), v.denominator() as i128)
}
impl MathMetricsSnapshot {
    pub fn identity(&self) -> &MathIdentity {
        self.lease.identity()
    }
    pub fn constants(&self) -> &[ScaledConstant] {
        &self.constants
    }
    pub fn constant(&self, key: MathConstant) -> Option<&ScaledConstant> {
        self.constants.iter().find(|v| v.constant == key)
    }
    pub fn glyphs(&self) -> &[ScaledGlyphMath] {
        &self.glyphs
    }
    pub fn font_size(&self) -> Q {
        self.size
    }
    pub fn require_current(
        &self,
        renderer: &RegistryRenderer,
        path: &str,
        snapshot: &SourceSnapshot,
    ) -> MathResult<()> {
        renderer.current(&self.lease.render)?;
        require(
            path == self.source.path
                && snapshot.revision == self.source.revision
                && digest(snapshot.text.as_bytes()) == self.source.sha256
                && snapshot.text.get(self.source.range.clone()).is_some(),
            "stale MATH source",
        )?;
        Ok(())
    }
    pub fn replay_bytes(&self, max_bytes: usize) -> MathResult<Vec<u8>> {
        if !(1..=MAX_MESSAGE_BYTES).contains(&max_bytes) {
            return Err(MathConsumerError::Budget);
        }
        let scalar =
            |v: Q| serde_json::json!([v.numerator().to_string(), v.denominator().to_string()]);
        let length = |v: ScaledLength| serde_json::json!({"design_units":v.design_units,"ticks":scalar(v.ticks)});
        let identity = self.identity();
        let value = serde_json::json!({"format":"flashtex-internal-math-metrics-v1","consumer_source_sha256":digest(include_bytes!("math.rs")),"binding":self.lease.binding(),"math_identity":{"registry_generation":identity.registry_generation,"declaration":identity.declaration,"selection":identity.binding,"font_sha256":identity.font.font_sha256,"engine_font_id":identity.font.engine_font_id.content_hex(),"face_index":identity.font.face_index,"math_table_sha256":identity.math_table_sha256,"math_table_byte_length":identity.math_table_byte_length,"engine_source_sha256":identity.engine_source_sha256,"policy":"unhinted_design_units"},"source":self.source,"font_size_ticks":scalar(self.size),"units_per_em":self.lease.font.units_per_em(),"constants":self.constants.iter().map(|v|serde_json::json!({"name":v.constant,"raw":v.raw,"dimension":v.dimension,"value":scalar(v.value)})).collect::<Vec<_>>(),"glyphs":self.glyphs.iter().map(|v|serde_json::json!({"original_gid":v.original_gid,"italic_correction":length(v.italic_correction),"top_accent_attachment":v.top_accent_attachment.map(length)})).collect::<Vec<_>>(),"device_adjustments_applied":false,"layout_performed":false});
        let mut output = crate::mixed::BoundedOutput {
            bytes: vec![],
            limit: max_bytes,
        };
        serde_json::to_writer(&mut output, &value).map_err(|_| MathConsumerError::Budget)?;
        Ok(output.bytes)
    }
    /// Validate supplied evidence against these metrics derived from current bound
    /// font bytes and source. Never accepts arbitrary offline constants as trusted.
    pub fn verify_replay(
        &self,
        renderer: &RegistryRenderer,
        path: &str,
        snapshot: &SourceSnapshot,
        bytes: &[u8],
    ) -> MathResult<()> {
        self.require_current(renderer, path, snapshot)?;
        if bytes.len() > MAX_MESSAGE_BYTES {
            return Err(MathConsumerError::Budget);
        }
        let supplied =
            crate::mixed_replay::parse_unique(bytes).map_err(|e| ValidationError(e.to_string()))?;
        let expected = crate::mixed_replay::parse_unique(&self.replay_bytes(MAX_MESSAGE_BYTES)?)
            .map_err(|e| ValidationError(e.to_string()))?;
        require(
            supplied == expected,
            "MATH replay differs from verified exact metrics",
        )?;
        Ok(())
    }
}
impl RegistryRenderer {
    pub fn math(&self, lease: &RenderLease, policy: MathPolicy) -> MathResult<MathLease> {
        self.current(lease)?;
        let font = BoundMathFont::from_registry(
            &self.registry,
            &lease.binding().selection,
            self.generation(),
            policy,
        )?;
        require(
            font.identity().declaration == lease.binding().declaration
                && font.identity().font == *lease.resource.engine.identity(),
            "MATH render resource identity mismatch",
        )?;
        Ok(MathLease {
            render: lease.clone(),
            font: Arc::new(font),
        })
    }
    pub fn math_metrics(
        &self,
        lease: &MathLease,
        query: MathQuery<'_>,
    ) -> MathResult<MathMetricsSnapshot> {
        self.current(&lease.render)?;
        path(query.source_path)?;
        require(
            query.source_range.start < query.source_range.end
                && query
                    .snapshot
                    .text
                    .get(query.source_range.clone())
                    .is_some(),
            "MATH source UTF8 range",
        )?;
        if query.snapshot.text.len() > MAX_MESSAGE_BYTES || query.original_gids.len() > 256 {
            return Err(MathConsumerError::Budget);
        }
        let size = rational(query.font_size).map_err(MathError::from)?;
        require(query.font_size.numerator() > 0, "MATH positive font size")?;
        let mut constants = Vec::with_capacity(MathConstant::ALL.len());
        for &constant in MathConstant::ALL {
            let (raw, dimension) = constant.raw(&lease.font);
            let value = match dimension {
                MetricDimension::CanonicalTicks => {
                    exact(lease.font.scale_design_units(raw, size)?)?
                }
                MetricDimension::DimensionlessRatio => exact(
                    flashtex_font_resources::math_adapter::percent_ratio(raw as i16)?,
                )?,
            };
            constants.push(ScaledConstant {
                constant,
                raw,
                dimension,
                value,
            });
        }
        let scale = |v: i16| -> MathResult<ScaledLength> {
            Ok(ScaledLength {
                design_units: v,
                ticks: exact(lease.font.scale_design_units(v as i32, size)?)?,
            })
        };
        let glyphs = lease
            .font
            .glyphs(query.original_gids)?
            .into_iter()
            .map(|g| {
                Ok(ScaledGlyphMath {
                    original_gid: g.glyph_id,
                    italic_correction: scale(g.italic_correction)?,
                    top_accent_attachment: g.top_accent_attachment.map(scale).transpose()?,
                })
            })
            .collect::<MathResult<Vec<_>>>()?;
        Ok(MathMetricsSnapshot {
            lease: lease.clone(),
            source: SourceEvidence {
                path: query.source_path.into(),
                revision: query.snapshot.revision,
                sha256: digest(query.snapshot.text.as_bytes()),
                range: query.source_range,
            },
            size: query.font_size,
            constants,
            glyphs,
        })
    }
}

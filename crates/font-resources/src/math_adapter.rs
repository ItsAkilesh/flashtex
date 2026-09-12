//! Identity-bound consumer of the original font-engine MATH parser.
//! Device adjustments are deliberately not evaluated or validated by that parser.
use crate::{
    cff::Rational,
    engine_adapter::{EngineFontAdapter, ShapeIdentity},
    registry::{ProjectFontRegistry, RegistryResource, StyleBinding},
    ManifestEntry,
};
use flashtex_font_engine::{math::MathConstants, Face, GlyphId};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathPolicy {
    UnhintedDesignUnits,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Constants,
    ItalicCorrection,
    TopAccentAttachment,
    DeviceAdjustments,
    Variants,
    MathKern,
    ExtendedShapeCoverage,
}
#[derive(Debug)]
pub enum MathError {
    Registry(crate::registry::RegistryError),
    Resource(crate::Error),
    MissingMathTable,
    Unsupported(Capability),
    GlyphOutOfBounds(u16),
    LookupBudget,
}
impl From<crate::Error> for MathError {
    fn from(e: crate::Error) -> Self {
        Self::Resource(e)
    }
}
impl From<crate::registry::RegistryError> for MathError {
    fn from(e: crate::registry::RegistryError) -> Self {
        Self::Registry(e)
    }
}
impl std::fmt::Display for MathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MathError {}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathIdentity {
    pub registry_generation: String,
    pub binding: StyleBinding,
    /// Exact project-relative font source and declared license provenance.
    pub declaration: ManifestEntry,
    pub font: ShapeIdentity,
    pub math_table_sha256: String,
    pub math_table_byte_length: usize,
    pub engine_source_sha256: String,
    pub policy: MathPolicy,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphMath {
    pub glyph_id: u16,
    pub italic_correction: i16,
    pub top_accent_attachment: Option<i16>,
}
pub struct BoundMathFont {
    engine: EngineFontAdapter,
    identity: MathIdentity,
}
impl BoundMathFont {
    pub fn from_registry(
        registry: &ProjectFontRegistry,
        binding: &StyleBinding,
        generation: &str,
        policy: MathPolicy,
    ) -> Result<Self, MathError> {
        registry.require_generation(generation)?;
        let engine = match registry.resource(binding)? {
            RegistryResource::TrueType(r) => EngineFontAdapter::from_resource(&r)?,
            RegistryResource::Cff(r) => r.shape_adapter()?,
        };
        let declaration = registry
            .discovery()
            .iter()
            .find(|e| &e.binding == binding)
            .ok_or(MathError::MissingMathTable)?
            .resource
            .clone();
        Self::bind(
            engine,
            generation.into(),
            binding.clone(),
            declaration,
            policy,
        )
    }
    fn bind(
        engine: EngineFontAdapter,
        registry_generation: String,
        binding: StyleBinding,
        declaration: ManifestEntry,
        policy: MathPolicy,
    ) -> Result<Self, MathError> {
        let table = engine
            .face()
            .table(b"MATH")
            .ok_or(MathError::MissingMathTable)?;
        if engine.face().math().is_none() {
            return Err(MathError::MissingMathTable);
        }
        let identity = MathIdentity {
            registry_generation,
            binding,
            declaration,
            font: engine.identity().clone(),
            math_table_sha256: crate::sha256(table),
            math_table_byte_length: table.len(),
            engine_source_sha256: crate::engine_adapter::ENGINE_SOURCE_SHA256.into(),
            policy,
        };
        Ok(Self { engine, identity })
    }
    /// Separate additive parse retains exact immutable parent identity.
    pub fn variants(&self) -> Result<BoundMathVariants, MathError> {
        Ok(BoundMathVariants {
            identity: self.identity.clone(),
            data: crate::math_variants::MathVariants::parse(
                self.engine.face().table(b"MATH").expect("bound MATH table"),
                self.glyph_count(),
            )?,
        })
    }
    pub fn identity(&self) -> &MathIdentity {
        &self.identity
    }
    pub fn units_per_em(&self) -> u16 {
        self.engine.units_per_em()
    }
    pub fn glyph_count(&self) -> u16 {
        self.engine.face().num_glyphs()
    }
    pub fn constants(&self) -> &MathConstants {
        &self
            .engine
            .face()
            .math()
            .expect("bound MATH table")
            .constants
    }
    pub fn require(&self, capability: Capability) -> Result<(), MathError> {
        match capability {
            Capability::Constants
            | Capability::ItalicCorrection
            | Capability::TopAccentAttachment => Ok(()),
            other => Err(MathError::Unsupported(other)),
        }
    }
    /// Maximum 256 original GIDs per request. Missing italic entries mean zero
    /// per the peer parser; absent accent entries remain None, not invented centers.
    pub fn glyphs(&self, ids: &[u16]) -> Result<Vec<GlyphMath>, MathError> {
        if ids.len() > 256 {
            return Err(MathError::LookupBudget);
        }
        ids.iter()
            .map(|&glyph_id| {
                if glyph_id >= self.glyph_count() {
                    return Err(MathError::GlyphOutOfBounds(glyph_id));
                }
                let math = self.engine.face().math().expect("bound MATH table");
                Ok(GlyphMath {
                    glyph_id,
                    italic_correction: math.italics_correction(GlyphId(glyph_id)),
                    top_accent_attachment: math.top_accent_attachment(GlyphId(glyph_id)),
                })
            })
            .collect()
    }
    /// Caller chooses output length units with size. No pixel-grid rounding.
    pub fn scale_design_units(&self, value: i32, size: Rational) -> Result<Rational, MathError> {
        scale_design_units(value, size, self.units_per_em())
    }
}
pub fn scale_design_units(
    value: i32,
    size: Rational,
    units_per_em: u16,
) -> Result<Rational, MathError> {
    if size.numerator() <= 0 || units_per_em == 0 {
        return Err(
            crate::Error::InvalidFont("positive math size and UPEM required".into()).into(),
        );
    }
    Ok(size.checked_mul(Rational::new(value as i128, units_per_em as i128)?)?)
}
/// Percent constants are dimensionless; never scale them by font size/UPEM.
pub fn percent_ratio(value: i16) -> Result<Rational, MathError> {
    Ok(Rational::new(value as i128, 100)?)
}
/// Variants capability is supported only after this separate bounded parse succeeds.
pub struct BoundMathVariants {
    identity: MathIdentity,
    data: crate::math_variants::MathVariants,
}
impl BoundMathVariants {
    pub fn identity(&self) -> &MathIdentity {
        &self.identity
    }
    pub fn data(&self) -> &crate::math_variants::MathVariants {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_scaling_and_percent_units() {
        assert_eq!(
            scale_design_units(-333, Rational::new(21, 2).unwrap(), 1000).unwrap(),
            Rational::new(-6993, 2000).unwrap()
        );
        assert_eq!(percent_ratio(80).unwrap(), Rational::new(4, 5).unwrap());
        assert!(scale_design_units(1, Rational::new(0, 1).unwrap(), 1000).is_err());
        assert!(scale_design_units(1, Rational::new(1, 1).unwrap(), 0).is_err());
        assert!(scale_design_units(i32::MAX, Rational::new(i128::MAX, 1).unwrap(), 1).is_err());
    }
}

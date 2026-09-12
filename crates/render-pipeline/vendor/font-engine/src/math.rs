//! OpenType `MATH` table: `MathConstants` (all 56 values), per-glyph italics
//! correction and top-accent attachment from `MathGlyphInfo`.
//!
//! Not parsed: extended-shape coverage, `MathKernInfo`, and `MathVariants`
//! (vertical/horizontal glyph construction for stretchy delimiters). Device
//! tables are ignored (values are the unhinted design values).

use crate::GlyphId;
use crate::otl::Coverage;
use crate::reader::{i16_at, u16_at, u32_at};

/// `MathConstants`, in font units (percent fields in percent).
/// Field order follows the OpenType specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MathConstants {
    pub script_percent_scale_down: i16,
    pub script_script_percent_scale_down: i16,
    pub delimited_sub_formula_min_height: u16,
    pub display_operator_min_height: u16,
    pub math_leading: i16,
    pub axis_height: i16,
    pub accent_base_height: i16,
    pub flattened_accent_base_height: i16,
    pub subscript_shift_down: i16,
    pub subscript_top_max: i16,
    pub subscript_baseline_drop_min: i16,
    pub superscript_shift_up: i16,
    pub superscript_shift_up_cramped: i16,
    pub superscript_bottom_min: i16,
    pub superscript_baseline_drop_max: i16,
    pub sub_superscript_gap_min: i16,
    pub superscript_bottom_max_with_subscript: i16,
    pub space_after_script: i16,
    pub upper_limit_gap_min: i16,
    pub upper_limit_baseline_rise_min: i16,
    pub lower_limit_gap_min: i16,
    pub lower_limit_baseline_drop_min: i16,
    pub stack_top_shift_up: i16,
    pub stack_top_display_style_shift_up: i16,
    pub stack_bottom_shift_down: i16,
    pub stack_bottom_display_style_shift_down: i16,
    pub stack_gap_min: i16,
    pub stack_display_style_gap_min: i16,
    pub stretch_stack_top_shift_up: i16,
    pub stretch_stack_bottom_shift_down: i16,
    pub stretch_stack_gap_above_min: i16,
    pub stretch_stack_gap_below_min: i16,
    pub fraction_numerator_shift_up: i16,
    pub fraction_numerator_display_style_shift_up: i16,
    pub fraction_denominator_shift_down: i16,
    pub fraction_denominator_display_style_shift_down: i16,
    pub fraction_numerator_gap_min: i16,
    pub fraction_num_display_style_gap_min: i16,
    pub fraction_rule_thickness: i16,
    pub fraction_denominator_gap_min: i16,
    pub fraction_denom_display_style_gap_min: i16,
    pub skewed_fraction_horizontal_gap: i16,
    pub skewed_fraction_vertical_gap: i16,
    pub overbar_vertical_gap: i16,
    pub overbar_rule_thickness: i16,
    pub overbar_extra_ascender: i16,
    pub underbar_vertical_gap: i16,
    pub underbar_rule_thickness: i16,
    pub underbar_extra_ascender: i16,
    pub radical_vertical_gap: i16,
    pub radical_display_style_vertical_gap: i16,
    pub radical_rule_thickness: i16,
    pub radical_extra_ascender: i16,
    pub radical_kern_before_degree: i16,
    pub radical_kern_after_degree: i16,
    pub radical_degree_bottom_raise_percent: i16,
}

#[derive(Debug, Clone)]
struct GlyphValues {
    coverage: Coverage,
    values: Vec<i16>,
}

impl GlyphValues {
    fn parse(b: &[u8], at: usize) -> Result<GlyphValues, crate::Error> {
        let coverage = Coverage::parse(b, at + usize::from(u16_at(b, at)?))?;
        let n = usize::from(u16_at(b, at + 2)?);
        let mut values = Vec::with_capacity(n);
        for i in 0..n {
            values.push(i16_at(b, at + 4 + 4 * i)?); // MathValueRecord: value, deviceOffset
        }
        Ok(GlyphValues { coverage, values })
    }

    fn get(&self, gid: GlyphId) -> Option<i16> {
        let i = self.coverage.index(gid.0)?;
        self.values.get(usize::from(i)).copied()
    }
}

#[derive(Debug, Clone)]
pub struct MathTable {
    pub constants: MathConstants,
    italics_correction: Option<GlyphValues>,
    top_accent: Option<GlyphValues>,
}

impl MathTable {
    pub fn parse(b: &[u8]) -> Result<MathTable, crate::Error> {
        let version = u32_at(b, 0)?;
        if version >> 16 != 1 {
            return Err(crate::Error::Unsupported(format!(
                "MATH table version {}.{}",
                version >> 16,
                version & 0xFFFF
            )));
        }
        let constants_at = usize::from(u16_at(b, 4)?);
        let glyph_info_at = usize::from(u16_at(b, 6)?);
        let c = constants_at;
        let mut vals = [0i16; 51];
        for (i, v) in vals.iter_mut().enumerate() {
            *v = i16_at(b, c + 8 + 4 * i)?;
        }
        let constants = MathConstants {
            script_percent_scale_down: i16_at(b, c)?,
            script_script_percent_scale_down: i16_at(b, c + 2)?,
            delimited_sub_formula_min_height: u16_at(b, c + 4)?,
            display_operator_min_height: u16_at(b, c + 6)?,
            math_leading: vals[0],
            axis_height: vals[1],
            accent_base_height: vals[2],
            flattened_accent_base_height: vals[3],
            subscript_shift_down: vals[4],
            subscript_top_max: vals[5],
            subscript_baseline_drop_min: vals[6],
            superscript_shift_up: vals[7],
            superscript_shift_up_cramped: vals[8],
            superscript_bottom_min: vals[9],
            superscript_baseline_drop_max: vals[10],
            sub_superscript_gap_min: vals[11],
            superscript_bottom_max_with_subscript: vals[12],
            space_after_script: vals[13],
            upper_limit_gap_min: vals[14],
            upper_limit_baseline_rise_min: vals[15],
            lower_limit_gap_min: vals[16],
            lower_limit_baseline_drop_min: vals[17],
            stack_top_shift_up: vals[18],
            stack_top_display_style_shift_up: vals[19],
            stack_bottom_shift_down: vals[20],
            stack_bottom_display_style_shift_down: vals[21],
            stack_gap_min: vals[22],
            stack_display_style_gap_min: vals[23],
            stretch_stack_top_shift_up: vals[24],
            stretch_stack_bottom_shift_down: vals[25],
            stretch_stack_gap_above_min: vals[26],
            stretch_stack_gap_below_min: vals[27],
            fraction_numerator_shift_up: vals[28],
            fraction_numerator_display_style_shift_up: vals[29],
            fraction_denominator_shift_down: vals[30],
            fraction_denominator_display_style_shift_down: vals[31],
            fraction_numerator_gap_min: vals[32],
            fraction_num_display_style_gap_min: vals[33],
            fraction_rule_thickness: vals[34],
            fraction_denominator_gap_min: vals[35],
            fraction_denom_display_style_gap_min: vals[36],
            skewed_fraction_horizontal_gap: vals[37],
            skewed_fraction_vertical_gap: vals[38],
            overbar_vertical_gap: vals[39],
            overbar_rule_thickness: vals[40],
            overbar_extra_ascender: vals[41],
            underbar_vertical_gap: vals[42],
            underbar_rule_thickness: vals[43],
            underbar_extra_ascender: vals[44],
            radical_vertical_gap: vals[45],
            radical_display_style_vertical_gap: vals[46],
            radical_rule_thickness: vals[47],
            radical_extra_ascender: vals[48],
            radical_kern_before_degree: vals[49],
            radical_kern_after_degree: vals[50],
            radical_degree_bottom_raise_percent: i16_at(b, c + 8 + 4 * 51)?,
        };
        let mut italics_correction = None;
        let mut top_accent = None;
        if glyph_info_at != 0 {
            let ic = usize::from(u16_at(b, glyph_info_at)?);
            if ic != 0 {
                italics_correction = Some(GlyphValues::parse(b, glyph_info_at + ic)?);
            }
            let ta = usize::from(u16_at(b, glyph_info_at + 2)?);
            if ta != 0 {
                top_accent = Some(GlyphValues::parse(b, glyph_info_at + ta)?);
            }
        }
        Ok(MathTable {
            constants,
            italics_correction,
            top_accent,
        })
    }

    /// Italics correction of `gid` in font units (0 when the font lists none).
    pub fn italics_correction(&self, gid: GlyphId) -> i16 {
        self.italics_correction
            .as_ref()
            .and_then(|v| v.get(gid))
            .unwrap_or(0)
    }

    /// Horizontal top-accent attachment point, if the font lists one; the
    /// specification's default is half the advance width.
    pub fn top_accent_attachment(&self, gid: GlyphId) -> Option<i16> {
        self.top_accent.as_ref().and_then(|v| v.get(gid))
    }
}

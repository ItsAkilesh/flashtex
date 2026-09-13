//! Word shaping for Unicode-engine fonts: TeX ligature mapping, character
//! to glyph mapping (with canonical composition under HarfBuzz), GSUB/GPOS
//! for the fontspec feature plan under the selected script, and the legacy
//! `kern` table where the emulated renderer uses it.
//!
//! What a shaper must provide (the contract other crates can implement
//! instead of this one): per-glyph original glyph id, advance in font units
//! after positioning, and the source byte range; plus the list of lookups it
//! could not apply. Line positions only need advances — glyph placements
//! (mark offsets, `yPlacement`) do not move later glyphs.

use std::collections::BTreeSet;
use std::ops::Range;

use flashtex_font_engine::shape::{ShapeOptions as EngineShapeOptions, shape as engine_shape};
use flashtex_font_engine::{Face, GlyphId, TrueTypeFace};

use crate::engine::{Engine, EngineProfile, Renderer};
use crate::fontspec::FeaturePlan;
use crate::otl::{Applied, BufGlyph, Gdef, Layout, TableKind, Tag};
use crate::texlig;

/// HarfBuzz's default horizontal features (`hb-ot-shape.cc`:
/// common + horizontal features) as applied by XeTeX.
pub const HARFBUZZ_DEFAULT_FEATURES: &[&str] = &[
    "abvm", "blwm", "ccmp", "locl", "mark", "mkmk", "rlig", "calt", "clig", "curs", "dist", "kern",
    "liga", "rclt",
];

/// luaotfload's default feature set (`luaotfload-configuration.lua`,
/// `base_features`: "Adopt the generic default from HarfBuzz" plus the
/// luaotfload-specific `itlc`, which is not a GSUB/GPOS feature).
pub const LUAOTFLOAD_DEFAULT_FEATURES: &[&str] = &[
    "abvm", "blwm", "ccmp", "locl", "mark", "mkmk", "rlig", "calt", "clig", "curs", "dist", "kern",
    "liga", "rclt",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapedGlyph {
    pub gid: GlyphId,
    /// Byte range of the input text.
    pub source: Range<usize>,
    /// Advance in font units, positioning included.
    pub advance: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapedText {
    pub glyphs: Vec<ShapedGlyph>,
    pub units_per_em: u16,
    /// Characters with no glyph. XeTeX and LuaTeX drop them from the output
    /// (with a "Missing character" log line); so does this shaper.
    pub missing: Vec<(char, usize)>,
    /// Script tag whose lookups were used (None: no GSUB/GPOS script matched).
    pub script_used: Option<Tag>,
    pub applied: Applied,
    /// Human-readable notes on approximations (AAT fonts, skipped lookups).
    pub notes: Vec<String>,
}

impl ShapedText {
    pub fn advance_units(&self) -> i64 {
        self.glyphs.iter().map(|g| i64::from(g.advance)).sum()
    }

    /// Width in points at `size_pt` with `letter_space_pt` between glyphs
    /// (not after the last one — measured: XeTeX/LuaTeX `LetterSpace=10`
    /// leaves a single `A` at its natural width).
    pub fn width_pt(&self, size_pt: f64, letter_space_pt: f64) -> f64 {
        let base = self.advance_units() as f64 * size_pt / f64::from(self.units_per_em);
        base + letter_space_pt * self.glyphs.len().saturating_sub(1) as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShapeParams<'a> {
    pub profile: EngineProfile,
    pub plan: &'a FeaturePlan,
}

/// GPOS advance adjustment (font units) for the glyph pair `left`,`right`
/// under the active features — used for pairs across an interword space.
/// luaotfload node mode shapes the whole node list, so GPOS pairs with the
/// space glyph (Arial `space`+`Y`, `A`+`space`) move the glue; HarfBuzz in
/// XeTeX shapes each word separately and never applies them.
pub fn pair_adjustment(
    face: &TrueTypeFace,
    left: GlyphId,
    right: GlyphId,
    params: ShapeParams<'_>,
) -> i32 {
    let Some(gpos) = face
        .table(b"GPOS")
        .and_then(|t| Layout::new(t, TableKind::Gpos))
    else {
        return 0;
    };
    let defaults = match params.profile.renderer {
        Renderer::HarfBuzz => HARFBUZZ_DEFAULT_FEATURES,
        Renderer::LuaNode | Renderer::LuaBase => LUAOTFLOAD_DEFAULT_FEATURES,
    };
    let active = params.plan.active(defaults);
    let (_, lookups) = gpos.select(params.plan.script, params.plan.language, &active);
    let gdef = Gdef::new(face.table(b"GDEF"));
    let mut buf = vec![
        BufGlyph {
            gid: left.0,
            source: 0..0,
            advance: 0,
            is_mark: false,
        },
        BufGlyph {
            gid: right.0,
            source: 0..0,
            advance: 0,
            is_mark: false,
        },
    ];
    let mut applied = Applied::default();
    gpos.apply_gpos(&lookups, &mut buf, &gdef, &mut applied);
    buf[0].advance + buf[1].advance
}

/// Shapes one run of text (normally a word) in `face`.
pub fn shape(face: &TrueTypeFace, text: &str, params: ShapeParams<'_>) -> ShapedText {
    let EngineProfile { engine, renderer } = params.profile;
    let plan = params.plan;
    let mut notes = Vec::new();

    // 1. TeX ligatures (`Ligatures=TeX`).
    let mapped = if plan.tex_ligatures && engine != Engine::PdfTeX {
        texlig::apply(text, engine, &|c| face.glyph_id(c).is_some())
    } else {
        texlig::Mapped {
            text: text.to_string(),
            source: text
                .char_indices()
                .map(|(i, c)| i..i + c.len_utf8())
                .collect(),
        }
    };
    // byte offset in mapped text -> char index
    let char_starts: Vec<usize> = mapped.text.char_indices().map(|(i, _)| i).collect();
    let orig_range = |r: &Range<usize>| -> Range<usize> {
        let a = char_starts.partition_point(|&s| s < r.start);
        let b = char_starts.partition_point(|&s| s < r.end);
        if a >= b || b > mapped.source.len() {
            return 0..0;
        }
        mapped.source[a].start..mapped.source[b - 1].end
    };

    let gdef = Gdef::new(face.table(b"GDEF"));
    let upem = face.units_per_em();
    let adv = |g: u16| i32::from(face.advance(GlyphId(g)).unwrap_or(0));

    // 2. cmap (+ canonical composition for HarfBuzz, which normalizes).
    let compose = renderer == Renderer::HarfBuzz;
    let opts = EngineShapeOptions {
        compose_marks: compose,
        ..EngineShapeOptions::PLAIN
    };
    let mut buf: Vec<BufGlyph> = Vec::new();
    let mut missing = Vec::new();
    match engine_shape(face, &mapped.text, &opts) {
        Ok(s) => {
            for m in &s.missing {
                missing.push((m.ch, m.byte_offset));
            }
            for cl in &s.clusters {
                let src = orig_range(&cl.source_range);
                for (k, g) in cl.glyphs.iter().enumerate() {
                    if g.gid == GlyphId::NOTDEF {
                        continue;
                    }
                    let is_mark = k > 0 || gdef.is_mark(g.gid.0);
                    buf.push(BufGlyph {
                        gid: g.gid.0,
                        source: src.clone(),
                        advance: adv(g.gid.0),
                        is_mark,
                    });
                }
            }
        }
        Err(e) => notes.push(format!("font-engine mapping failed: {e}")),
    }

    // 3. OpenType layout for the active feature set.
    let defaults = match renderer {
        Renderer::HarfBuzz => HARFBUZZ_DEFAULT_FEATURES,
        Renderer::LuaNode | Renderer::LuaBase => LUAOTFLOAD_DEFAULT_FEATURES,
    };
    let active: BTreeSet<Tag> = plan.active(defaults);
    let mut applied = Applied::default();
    let mut script_used = None;
    let gsub = face
        .table(b"GSUB")
        .and_then(|t| Layout::new(t, TableKind::Gsub));
    let gpos = face
        .table(b"GPOS")
        .and_then(|t| Layout::new(t, TableKind::Gpos));
    if gsub.is_none() && gpos.is_none() && face.has_table(b"morx") {
        notes.push(match renderer {
            Renderer::HarfBuzz => "AAT font (morx): XeTeX applies its morx ligatures/substitutions, which are not implemented here".to_string(),
            _ => "AAT font (morx) without GSUB/GPOS: luaotfload applies no substitutions".to_string(),
        });
    }
    if let Some(g) = gsub {
        let (used, lookups) = g.select(plan.script, plan.language, &active);
        script_used = used;
        g.apply_gsub(&lookups, &mut buf, &gdef, &adv, &mut applied);
    }
    let mut gpos_kern = false;
    if let Some(g) = gpos {
        let (used, lookups) = g.select(plan.script, plan.language, &active);
        script_used = script_used.or(used);
        gpos_kern = lookups.iter().any(|l| l.feature == Tag::from_str("kern"));
        g.apply_gpos(&lookups, &mut buf, &gdef, &mut applied);
    }

    // Marks take no advance (HarfBuzz zeroes mark advances; luaotfload's
    // mark feature positions them with zero width).
    for g in &mut buf {
        if g.is_mark {
            g.advance = 0;
        }
    }

    // 4. Legacy `kern` table: HarfBuzz uses it when GPOS has no `kern`
    //    feature; luaotfload node mode ignores it.
    if renderer == Renderer::HarfBuzz && !gpos_kern && active.contains(&Tag::from_str("kern")) {
        let bases: Vec<usize> = (0..buf.len()).filter(|&i| !buf[i].is_mark).collect();
        for w in bases.windows(2) {
            if let Some(k) =
                face.legacy_kern_table_kerning(GlyphId(buf[w[0]].gid), GlyphId(buf[w[1]].gid))
            {
                buf[w[0]].advance += i32::from(k);
            }
        }
    }

    for (f, i, t) in &applied.skipped {
        notes.push(format!(
            "{} lookup {i} (type {t}) not implemented",
            f.as_str()
        ));
    }
    ShapedText {
        glyphs: buf
            .into_iter()
            .map(|g| ShapedGlyph {
                gid: GlyphId(g.gid),
                source: g.source,
                advance: g.advance,
            })
            .collect(),
        units_per_em: upem,
        missing,
        script_used,
        applied,
        notes,
    }
}

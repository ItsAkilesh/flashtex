//! rendering-v2 display list → exact export.
//!
//! Consumes the `display_list` envelope written by `flashtex-render --v2`
//! (`crates/render-pipeline`, shaped after `protocol/rendering-v2.schema.json`
//! and `crates/rendering-core`'s model) and builds an [`ExactDocument`]:
//!
//! - `coordinate_unit` must be `bp_2pow20`: signed integer ticks, 2^20 per
//!   PDF point, y downward from the page's top-left corner. Every tick value
//!   is converted to an exact terminating decimal (`ticks / 2^20`, at most
//!   20 fractional digits) after the y flip is done in integer arithmetic,
//!   so no `f64` is involved in a coordinate.
//! - Each `glyph_run` glyph carries its original glyph id and an absolute
//!   origin; the origin is authoritative and advances are never added again
//!   (rendering-v2 proposal). A glyph joins the previous glyph's string only
//!   when its origin equals the previous origin plus the font's own `hmtx`
//!   advance at that size (checked exactly in integer ticks); otherwise it
//!   gets its own `Tm`. Either way the written position is the envelope's.
//! - Fonts are content-addressed (`font_id` = SHA-256 of the program) and
//!   the envelope carries no path, so the bytes are resolved by hashing
//!   candidate files (`--font-dir`, `FLASHTEX_FONT_DIRS`, `FLASHTEX_LM_DIR`,
//!   the TeX Live Latin Modern directories) whose size matches, then
//!   embedded through [`ExactFont::cid_from_opentype`] (CFF: GID-preserving
//!   CID-keyed subset; TrueType: glyph order kept). `format` is accepted as
//!   `opentype-cff` (the pipeline's documented deviation from the schema
//!   token) or `static-truetype`; `core14-afm` has no bytes and is refused
//!   when a run uses it.
//! - `rule` items become `Op::rule`. Paint must be opaque; black is the
//!   default fill, any other opaque colour is written as an exact `rg`.
//! - Cluster ActualText is reduced to a per-glyph ToUnicode entry (the
//!   cluster's text); a glyph seen with two different texts keeps the first
//!   and the report says so. Marked-content `/ActualText` is outside the
//!   bounded operator set.
//!
//! Everything unsupported is an error naming the item; nothing is dropped.

use crate::exact::{
    Content, Decimal, ExactDocument, ExactFont, ExactPage, GlyphRun, Op, PlacedGlyph, SubsetOutcome,
};
use crate::json::{self, Value};
use crate::sha256;
use crate::truetype::TrueTypeFont;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const TICKS_PER_BP: i128 = 1 << 20;
pub const COORDINATE_UNIT: &str = "bp_2pow20";
/// Directories searched for font bytes after `--font-dir`, `FLASHTEX_FONT_DIRS`
/// and `FLASHTEX_LM_DIR` (the same list `flashtex-render` uses).
pub const DEFAULT_FONT_DIRS: [&str; 12] = [
    "/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2026/texmf-dist/fonts/opentype/public/lm-math",
    "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2026basic/texmf-dist/fonts/opentype/public/lm-math",
    "/usr/local/texlive/2025/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2025/texmf-dist/fonts/opentype/public/lm-math",
    "/usr/local/texlive/2025basic/texmf-dist/fonts/opentype/public/lm",
    "/usr/local/texlive/2025basic/texmf-dist/fonts/opentype/public/lm-math",
    "/usr/share/texmf/fonts/opentype/public/lm",
    "/usr/share/texmf/fonts/opentype/public/lm-math",
    "/usr/share/texlive/texmf-dist/fonts/opentype/public/lm",
    "/usr/share/texlive/texmf-dist/fonts/opentype/public/lm-math",
];
/// Largest envelope accepted.
pub const MAX_ENVELOPE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, Default)]
pub struct V2Options {
    /// Directories probed first, in order.
    pub font_dirs: Vec<PathBuf>,
}

/// One embedded font, for the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontNote {
    pub resource: String,
    pub font_id: String,
    pub postscript_name: String,
    pub path: PathBuf,
    pub hash_form: HashForm,
    pub glyphs: usize,
    pub outcome: SubsetOutcome,
    pub program_bytes: usize,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct V2Report {
    pub fonts: Vec<FontNote>,
    pub pages: usize,
    pub glyphs: usize,
    pub runs: usize,
    pub rules: usize,
    /// Glyphs that continued the previous string (origin = previous + hmtx advance).
    pub joined_glyphs: usize,
    pub diagnostics: Vec<String>,
    pub notes: Vec<String>,
}

fn f(v: Option<&Value>, what: &str) -> Result<f64, String> {
    v.and_then(Value::as_f64)
        .ok_or_else(|| format!("{what}: expected a number"))
}

fn ticks(v: Option<&Value>, what: &str) -> Result<i128, String> {
    let n = f(v, what)?;
    if !n.is_finite() || n.fract() != 0.0 || n.abs() > 9.0e15 {
        return Err(format!("{what}: {n} is not an integer tick value"));
    }
    Ok(n as i128)
}

fn s<'a>(v: Option<&'a Value>, what: &str) -> Result<&'a str, String> {
    v.and_then(Value::as_str)
        .ok_or_else(|| format!("{what}: expected a string"))
}

fn arr<'a>(v: Option<&'a Value>, what: &str) -> Result<&'a [Value], String> {
    v.and_then(Value::as_array)
        .ok_or_else(|| format!("{what}: expected an array"))
}

/// Exact decimal of `ticks / 2^20`.
pub fn bp(t: i128) -> Result<Decimal, String> {
    Decimal::from_ratio(t, TICKS_PER_BP as u128, 20)
        .ok_or_else(|| format!("tick value {t} has no terminating decimal (impossible for 2^20)"))
}

/// Exact decimal of a finite `f64` in `[0, 1]` (colour component).
fn exact_unit(v: f64, what: &str) -> Result<Decimal, String> {
    if !v.is_finite() || !(0.0..=1.0).contains(&v) {
        return Err(format!("{what}: {v} is not in [0, 1]"));
    }
    if v == 0.0 {
        return Ok(Decimal::from_i64(0));
    }
    if v == 1.0 {
        return Ok(Decimal::from_i64(1));
    }
    let bits = v.to_bits();
    let exponent = ((bits >> 52) & 2047) as i32;
    let mantissa = (bits & ((1u64 << 52) - 1)) | (1u64 << 52);
    let shift = 1075 - exponent;
    if exponent == 0 || !(0..64).contains(&shift) {
        return Err(format!(
            "{what}: {v} is not exactly representable within the decimal bound"
        ));
    }
    Decimal::from_ratio(mantissa as i128, 1u128 << shift, 20)
        .ok_or_else(|| format!("{what}: {v} needs more than 20 decimal digits"))
}

struct Paint {
    rgb: Option<[Decimal; 3]>,
}

fn paint(v: Option<&Value>, what: &str) -> Result<Paint, String> {
    let Some(p) = v else {
        return Ok(Paint { rgb: None });
    };
    let a = f(p.get("a"), &format!("{what}.paint.a"))?;
    if a != 1.0 {
        return Err(format!(
            "{what}: paint alpha {a} is not 1; alpha needs an ExtGState, which is outside the bounded operator set"
        ));
    }
    let r = f(p.get("r"), &format!("{what}.paint.r"))?;
    let g = f(p.get("g"), &format!("{what}.paint.g"))?;
    let b = f(p.get("b"), &format!("{what}.paint.b"))?;
    if r == 0.0 && g == 0.0 && b == 0.0 {
        return Ok(Paint { rgb: None });
    }
    Ok(Paint {
        rgb: Some([
            exact_unit(r, what)?,
            exact_unit(g, what)?,
            exact_unit(b, what)?,
        ]),
    })
}

struct FontEntry {
    font_id: String,
    sha256: String,
    byte_length: u64,
    format: String,
    face_index: u64,
    units_per_em: u64,
    glyph_count: u64,
    postscript_name: String,
    resource: String,
}

/// How a font file matched the envelope's `sha256`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashForm {
    /// SHA-256 over the file bytes (rendering-core's validator, the contract).
    Bytes,
    /// SHA-256 over the file bytes followed by the big-endian `u32` face index
    /// (font-engine's `content_sha256`, what `flashtex-render` emits today).
    BytesAndFaceIndex,
}

/// Finds a file with the given SHA-256 among the candidate directories,
/// hashing only files whose size matches. Both hash forms are tried.
pub fn resolve_font(
    dirs: &[PathBuf],
    sha: &str,
    byte_length: u64,
    face_index: u32,
) -> Option<(PathBuf, HashForm)> {
    let mut seen = BTreeSet::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            if !seen.insert(p.clone()) {
                continue;
            }
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_ascii_lowercase());
            if !matches!(ext.as_deref(), Some("otf" | "ttf")) {
                continue;
            }
            let Ok(meta) = std::fs::metadata(&p) else {
                continue;
            };
            if meta.len() != byte_length {
                continue;
            }
            if let Ok(mut bytes) = std::fs::read(&p) {
                if sha256::hex(&bytes) == sha {
                    return Some((p, HashForm::Bytes));
                }
                bytes.extend_from_slice(&face_index.to_be_bytes());
                if sha256::hex(&bytes) == sha {
                    return Some((p, HashForm::BytesAndFaceIndex));
                }
            }
        }
    }
    None
}

/// The font directories to probe: `options.font_dirs`, then the environment
/// and the defaults.
pub fn font_dirs(options: &V2Options) -> Vec<PathBuf> {
    let mut dirs = options.font_dirs.clone();
    if let Ok(v) = std::env::var("FLASHTEX_FONT_DIRS") {
        dirs.extend(v.split(':').filter(|x| !x.is_empty()).map(PathBuf::from));
    }
    if let Ok(v) = std::env::var("FLASHTEX_LM_DIR")
        && !v.is_empty()
    {
        dirs.push(PathBuf::from(v));
    }
    dirs.extend(DEFAULT_FONT_DIRS.iter().map(PathBuf::from));
    dirs
}

/// Builds the exact document from a rendering-v2 `display_list` envelope.
pub fn from_v2(envelope: &str, options: &V2Options) -> Result<(ExactDocument, V2Report), String> {
    if envelope.len() > MAX_ENVELOPE_BYTES {
        return Err("envelope larger than 256 MiB".into());
    }
    let root = json::parse(envelope)?;
    let version = f(root.get("protocol_version"), "protocol_version")?;
    if version != 2.0 {
        return Err(format!("protocol_version {version} is not 2"));
    }
    let kind = s(root.get("type"), "type")?;
    if kind != "display_list" {
        return Err(format!("type {kind:?} is not display_list"));
    }
    let p = root.get("payload").ok_or("missing payload")?;
    let render_format = s(p.get("render_format"), "payload.render_format")?;
    if render_format != "display-list-v2" {
        return Err(format!(
            "render_format {render_format:?} is not display-list-v2"
        ));
    }
    let unit = s(p.get("coordinate_unit"), "payload.coordinate_unit")?;
    if unit != COORDINATE_UNIT {
        return Err(format!("coordinate_unit {unit:?} is not {COORDINATE_UNIT}"));
    }
    let color_space = s(p.get("color_space"), "payload.color_space")?;
    if color_space != "srgb" {
        return Err(format!("color_space {color_space:?} is not srgb"));
    }
    let mut report = V2Report::default();
    for d in arr(p.get("diagnostics"), "payload.diagnostics").unwrap_or(&[]) {
        report.diagnostics.push(format!(
            "{}: {}: {}",
            s(d.get("severity"), "severity").unwrap_or("?"),
            s(d.get("code"), "code").unwrap_or("?"),
            s(d.get("message"), "message").unwrap_or("?")
        ));
    }

    // Fonts, in envelope order; resource names follow that order.
    let mut fonts: BTreeMap<String, FontEntry> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for (i, fv) in arr(p.get("fonts"), "payload.fonts")?.iter().enumerate() {
        let what = format!("payload.fonts[{i}]");
        let font_id = s(fv.get("font_id"), &format!("{what}.font_id"))?.to_string();
        let entry = FontEntry {
            sha256: s(fv.get("sha256"), &format!("{what}.sha256"))?.to_string(),
            byte_length: f(fv.get("byte_length"), &format!("{what}.byte_length"))? as u64,
            format: s(fv.get("format"), &format!("{what}.format"))?.to_string(),
            face_index: f(fv.get("face_index"), &format!("{what}.face_index"))? as u64,
            units_per_em: f(fv.get("units_per_em"), &format!("{what}.units_per_em"))? as u64,
            glyph_count: f(fv.get("glyph_count"), &format!("{what}.glyph_count"))? as u64,
            postscript_name: s(
                fv.get("postscript_name"),
                &format!("{what}.postscript_name"),
            )?
            .to_string(),
            resource: format!("F{}", i + 1),
            font_id: font_id.clone(),
        };
        if fonts.insert(font_id.clone(), entry).is_some() {
            return Err(format!("{what}: duplicate font_id {font_id}"));
        }
        order.push(font_id);
    }

    // Pass 1: walk pages, collect glyph ids and cluster texts per font, and
    // build operators with resource names.
    struct Used {
        gids: BTreeSet<u16>,
        to_unicode: BTreeMap<u16, String>,
        conflicts: usize,
    }
    let mut used: BTreeMap<String, Used> = BTreeMap::new();
    // A pending glyph run before joining decisions (needs font metrics).
    struct Glyph {
        gid: u16,
        origin_x: i128,
        y_pdf: i128,
        advance_x: i128,
        advance_y: i128,
    }
    enum Pending {
        Run {
            font_id: String,
            resource: String,
            size_ticks: i128,
            rgb: Option<[Decimal; 3]>,
            glyphs: Vec<Glyph>,
        },
        Ops(Vec<Op>),
    }
    let mut pages_ops: Vec<(Decimal, Decimal, Vec<Pending>, BTreeSet<String>)> = Vec::new();
    for (pi, pv) in arr(p.get("pages"), "payload.pages")?.iter().enumerate() {
        let what = format!("payload.pages[{pi}]");
        let width = ticks(pv.get("width"), &format!("{what}.width"))?;
        let height = ticks(pv.get("height"), &format!("{what}.height"))?;
        if width <= 0 || height <= 0 {
            return Err(format!(
                "{what}: page size {width}x{height} ticks is not positive"
            ));
        }
        let mut items: Vec<Pending> = Vec::new();
        let mut page_fonts = BTreeSet::new();
        for (ii, iv) in arr(pv.get("items"), &format!("{what}.items"))?
            .iter()
            .enumerate()
        {
            let iw = format!("{what}.items[{ii}]");
            let kind = s(iv.get("kind"), &format!("{iw}.kind"))?;
            let pt = paint(iv.get("paint"), &iw)?;
            match kind {
                "glyph_run" => {
                    let font_id = s(iv.get("font_id"), &format!("{iw}.font_id"))?;
                    let entry = fonts.get(font_id).ok_or_else(|| {
                        format!("{iw}: font_id {font_id} is not in payload.fonts")
                    })?;
                    if entry.format == "core14-afm" {
                        return Err(format!(
                            "{iw}: font {} ({font_id}) is format core14-afm, which carries no program bytes; the exact route needs the font program (render-pipeline: ship bytes or emit a real font resource)",
                            entry.postscript_name
                        ));
                    }
                    let size = ticks(iv.get("font_size"), &format!("{iw}.font_size"))?;
                    if size <= 0 {
                        return Err(format!("{iw}: font_size {size} ticks is not positive"));
                    }
                    let text = s(iv.get("text"), &format!("{iw}.text"))?;
                    let clusters = arr(iv.get("clusters"), &format!("{iw}.clusters"))?;
                    let u = used.entry(font_id.to_string()).or_insert_with(|| Used {
                        gids: BTreeSet::new(),
                        to_unicode: BTreeMap::new(),
                        conflicts: 0,
                    });
                    let mut glyphs = Vec::new();
                    for (gi, gv) in arr(iv.get("glyphs"), &format!("{iw}.glyphs"))?
                        .iter()
                        .enumerate()
                    {
                        let gw = format!("{iw}.glyphs[{gi}]");
                        let gid = f(gv.get("gid"), &format!("{gw}.gid"))?;
                        if gid.fract() != 0.0 || !(0.0..=65535.0).contains(&gid) {
                            return Err(format!("{gw}: gid {gid} is not a glyph id"));
                        }
                        let gid = gid as u16;
                        if gid == 0 {
                            return Err(format!(
                                "{gw}: gid 0 (missing glyph) must not be exported"
                            ));
                        }
                        if u64::from(gid) >= entry.glyph_count {
                            return Err(format!(
                                "{gw}: gid {gid} is outside the font's {} glyphs",
                                entry.glyph_count
                            ));
                        }
                        let origin_x = ticks(gv.get("origin_x"), &format!("{gw}.origin_x"))?;
                        let baseline_y = ticks(gv.get("baseline_y"), &format!("{gw}.baseline_y"))?;
                        let advance_x = ticks(gv.get("advance_x"), &format!("{gw}.advance_x"))?;
                        let advance_y = ticks(gv.get("advance_y"), &format!("{gw}.advance_y"))?;
                        let cluster = f(gv.get("cluster"), &format!("{gw}.cluster"))? as usize;
                        let cv = clusters
                            .get(cluster)
                            .ok_or_else(|| format!("{gw}: cluster {cluster} is out of range"))?;
                        let a = f(cv.get("text_start_byte"), "cluster.text_start_byte")? as usize;
                        let b = f(cv.get("text_end_byte"), "cluster.text_end_byte")? as usize;
                        if let Some(t) = text.get(a..b) {
                            match u.to_unicode.get(&gid) {
                                Some(prev) if prev != t => u.conflicts += 1,
                                Some(_) => {}
                                None => {
                                    u.to_unicode.insert(gid, t.to_string());
                                }
                            }
                        }
                        u.gids.insert(gid);
                        glyphs.push(Glyph {
                            gid,
                            origin_x,
                            y_pdf: height - baseline_y,
                            advance_x,
                            advance_y,
                        });
                        report.glyphs += 1;
                    }
                    if glyphs.is_empty() {
                        return Err(format!("{iw}: glyph run without glyphs"));
                    }
                    report.runs += 1;
                    page_fonts.insert(entry.resource.clone());
                    items.push(Pending::Run {
                        font_id: font_id.to_string(),
                        resource: entry.resource.clone(),
                        size_ticks: size,
                        rgb: pt.rgb,
                        glyphs,
                    });
                }
                "rule" => {
                    let x = ticks(iv.get("x"), &format!("{iw}.x"))?;
                    let top = ticks(iv.get("top"), &format!("{iw}.top"))?;
                    let w = ticks(iv.get("width"), &format!("{iw}.width"))?;
                    let h = ticks(iv.get("height"), &format!("{iw}.height"))?;
                    if w <= 0 || h <= 0 {
                        return Err(format!("{iw}: rule {w}x{h} ticks is not positive"));
                    }
                    let mut ops = Vec::new();
                    if let Some(rgb) = pt.rgb {
                        ops.push(Op::Save);
                        ops.push(Op::FillRgb(rgb));
                    }
                    ops.extend(Op::rule(bp(x)?, bp(height - top - h)?, bp(w)?, bp(h)?));
                    if ops.len() == 4 {
                        ops.push(Op::Restore);
                    }
                    report.rules += 1;
                    items.push(Pending::Ops(ops));
                }
                other => {
                    return Err(format!(
                        "{iw}: item kind {other:?} is not supported by the exact route (glyph_run and rule only)"
                    ));
                }
            }
        }
        pages_ops.push((bp(width)?, bp(height)?, items, page_fonts));
        report.pages += 1;
    }

    // Pass 2: resolve and embed the fonts that are used.
    let dirs = font_dirs(options);
    let mut exact_fonts: BTreeMap<String, ExactFont> = BTreeMap::new();
    let mut loaded: BTreeMap<String, TrueTypeFont> = BTreeMap::new();
    for font_id in &order {
        let Some(u) = used.get(font_id) else {
            continue;
        };
        let entry = &fonts[font_id];
        if entry.face_index != 0 {
            return Err(format!(
                "font {}: face_index {} is not 0 (collections are not supported)",
                entry.postscript_name, entry.face_index
            ));
        }
        if !matches!(entry.format.as_str(), "opentype-cff" | "static-truetype") {
            return Err(format!(
                "font {}: format {:?} is not opentype-cff or static-truetype",
                entry.postscript_name, entry.format
            ));
        }
        if entry.sha256 != entry.font_id {
            report.notes.push(format!(
                "font {}: font_id differs from sha256; resolved by sha256",
                entry.postscript_name
            ));
        }
        let (path, hash_form) = resolve_font(
            &dirs,
            &entry.sha256,
            entry.byte_length,
            entry.face_index as u32,
        )
        .ok_or_else(|| {
            format!(
                "font {} (sha256 {}, {} bytes) was not found in {} director{}: {}",
                entry.postscript_name,
                entry.sha256,
                entry.byte_length,
                dirs.len(),
                if dirs.len() == 1 { "y" } else { "ies" },
                dirs.iter()
                    .map(|d| d.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
        if hash_form == HashForm::BytesAndFaceIndex {
            report.notes.push(format!(
                "font {}: sha256 matched SHA-256(bytes || face_index), font-engine's content hash, not SHA-256(bytes) as rendering-core's validator requires (render-pipeline deviation)",
                entry.postscript_name
            ));
        }
        let font = TrueTypeFont::load(&path)?;
        if u64::from(font.num_glyphs()) != entry.glyph_count
            || u64::from(font.units_per_em) != entry.units_per_em
        {
            return Err(format!(
                "font {}: envelope says {} glyphs at {} units/em but {} has {} at {}",
                entry.postscript_name,
                entry.glyph_count,
                entry.units_per_em,
                path.display(),
                font.num_glyphs(),
                font.units_per_em
            ));
        }
        let expected_format = match font.outlines {
            crate::truetype::Outlines::Cff => "opentype-cff",
            crate::truetype::Outlines::TrueType => "static-truetype",
        };
        if entry.format != expected_format {
            return Err(format!(
                "font {}: envelope format {:?} but the file has {expected_format} outlines",
                entry.postscript_name, entry.format
            ));
        }
        let (exact, outcome, note) =
            ExactFont::cid_from_opentype(&font, &u.gids, u.to_unicode.clone())
                .map_err(|e| e.to_string())?;
        if u.conflicts > 0 {
            report.notes.push(format!(
                "font {}: {} glyph occurrence(s) with a different cluster text than the first; ToUnicode keeps the first mapping (cluster ActualText needs marked content, outside the bounded set)",
                entry.postscript_name, u.conflicts
            ));
        }
        report.fonts.push(FontNote {
            resource: entry.resource.clone(),
            font_id: font_id.clone(),
            postscript_name: entry.postscript_name.clone(),
            path: path.clone(),
            hash_form,
            glyphs: u.gids.len(),
            outcome,
            program_bytes: exact.program_identity().map_or(0, |(n, _)| n),
            note,
        });
        exact_fonts.insert(entry.resource.clone(), exact);
        loaded.insert(font_id.clone(), font);
    }

    // Pass 3: finish the glyph runs now that advances are known.
    let mut pages = Vec::with_capacity(pages_ops.len());
    for (width, height, items, page_fonts) in pages_ops {
        let mut ops = Vec::new();
        for item in items {
            let (font_id, resource, size_ticks, rgb, glyphs) = match item {
                Pending::Ops(o) => {
                    ops.extend(o);
                    continue;
                }
                Pending::Run {
                    font_id,
                    resource,
                    size_ticks,
                    rgb,
                    glyphs,
                } => (font_id, resource, size_ticks, rgb, glyphs),
            };
            let font = &loaded[&font_id];
            let upem = font.units_per_em as i128;
            let mut placed = Vec::with_capacity(glyphs.len());
            let mut prev: Option<&Glyph> = None;
            for g in &glyphs {
                let joins = prev.is_some_and(|p| {
                    let hmtx = font.advance(p.gid) as i128;
                    p.y_pdf == g.y_pdf
                        && p.advance_y == 0
                        && (p.origin_x + p.advance_x) == g.origin_x
                        && p.advance_x * upem == hmtx * size_ticks
                });
                if joins {
                    report.joined_glyphs += 1;
                }
                placed.push(PlacedGlyph {
                    gid: g.gid,
                    origin: if joins {
                        None
                    } else {
                        Some((bp(g.origin_x)?, bp(g.y_pdf)?))
                    },
                });
                prev = Some(g);
            }
            let run = GlyphRun {
                font: resource,
                size: bp(size_ticks)?,
                glyphs: placed,
            };
            let run_ops = run.to_ops().map_err(|e| e.to_string())?;
            if let Some(c) = rgb {
                ops.push(Op::Save);
                ops.push(Op::FillRgb(c));
                ops.extend(run_ops);
                ops.push(Op::Restore);
            } else {
                ops.extend(run_ops);
            }
        }
        pages.push(ExactPage {
            width,
            height,
            content: Content::Ops(ops),
            fonts: Some(page_fonts.into_iter().collect()),
        });
    }
    if pages.is_empty() {
        return Err("display list has no pages".into());
    }
    Ok((
        ExactDocument {
            pages,
            fonts: exact_fonts,
        },
        report,
    ))
}

/// Convenience for callers with a path.
pub fn from_v2_file(path: &Path, options: &V2Options) -> Result<(ExactDocument, V2Report), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    from_v2(&text, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks_to_bp_is_exact() {
        assert_eq!(bp(75_497_472).unwrap().as_str(), "72");
        assert_eq!(bp(1).unwrap().as_str(), "0.00000095367431640625");
        assert_eq!(bp(-524_288).unwrap().as_str(), "-0.5");
        assert_eq!(bp(88_033_374).unwrap().as_str(), "83.9551677703857421875");
    }

    #[test]
    fn colour_components_are_exact_or_refused() {
        assert_eq!(exact_unit(0.5, "c").unwrap().as_str(), "0.5");
        assert_eq!(exact_unit(0.125, "c").unwrap().as_str(), "0.125");
        assert!(
            exact_unit(0.1, "c").is_err(),
            "0.1 is not a short binary fraction"
        );
        assert!(exact_unit(1.5, "c").is_err());
    }

    #[test]
    fn rejects_wrong_envelopes() {
        let e = from_v2(
            r#"{"protocol_version":1,"type":"compile","payload":{}}"#,
            &V2Options::default(),
        )
        .unwrap_err();
        assert!(e.contains("protocol_version 1"), "{e}");
        let e = from_v2(
            r#"{"protocol_version":2,"type":"display_list","payload":{"render_format":"display-list-v2","coordinate_unit":"pt","color_space":"srgb","fonts":[],"pages":[]}}"#,
            &V2Options::default(),
        )
        .unwrap_err();
        assert!(e.contains("coordinate_unit"), "{e}");
        let e = from_v2(
            r#"{"protocol_version":2,"type":"display_list","payload":{"render_format":"display-list-v2","coordinate_unit":"bp_2pow20","color_space":"srgb","fonts":[{"font_id":"x","sha256":"x","byte_length":1,"format":"core14-afm","face_index":0,"units_per_em":1000,"glyph_count":300,"postscript_name":"Times-Roman"}],"pages":[{"number":1,"width":1048576,"height":1048576,"items":[{"kind":"glyph_run","font_id":"x","font_size":1048576,"text":"a","paint":{"r":0,"g":0,"b":0,"a":1},"glyphs":[{"gid":5,"origin_x":0,"baseline_y":0,"advance_x":0,"advance_y":0,"cluster":0}],"clusters":[{"text_start_byte":0,"text_end_byte":1}]}]}]}}"#,
            &V2Options::default(),
        )
        .unwrap_err();
        assert!(e.contains("core14-afm"), "{e}");
    }
}

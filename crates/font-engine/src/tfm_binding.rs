//! Exact TFM 8-bit encoding bound to declared ORIGINAL glyph ids (FT-018 rev 4).
//!
//! pdfTeX sets type from three separate artefacts: a TFM (per-slot metrics
//! and the lig/kern program, all `fix_word` = 2⁻²⁰ em), an encoding vector
//! (`.enc`: slot → PostScript glyph name) and the font program. This module
//! binds the three without ever casting a slot to a Unicode scalar or a
//! glyph id:
//!
//! ```text
//! EncodingCode(u8) --.enc--> glyph name --font's own names--> GlyphId (original)
//!                                          (CFF charset / post 2.0)
//!                                       --uniXXXX--> cmap --> GlyphId
//!                  --T1/OT1 table--> char --cmap--> GlyphId   (cross-check only)
//! ```
//!
//! Every slot ends in one explicit [`SlotState`]; using an unbound slot is an
//! [`Error::UnboundSlot`], never `.notdef`. The TFM parser and lig/kern
//! interpreter are `crates/font-resources`' (`tfm::Tfm`), and the CFF charset
//! reader is its `cff::Cff`; this crate adds the CFF standard strings, `post`
//! 2.0 names, the OpenType side of the comparison, and TeX's own scaled
//! arithmetic (`store_scaled`, `print_scaled`) so widths at a size are the
//! integers TeX would compute.

use std::collections::BTreeMap;
use std::ops::Range;

use flashtex_font_resources::cff::Cff;
use flashtex_font_resources::encoding::{EncodingEntry, EncodingManifest, NamedGlyph};
use flashtex_font_resources::tfm::{Tfm, TfmItem};
pub use flashtex_font_resources::tfm::{CharacterMetrics, FixWord, PairAction};

use crate::encoding::{Encoding, EncodingCode};
use crate::glyph_names::{CFF_STANDARD_STRINGS, MAC_GLYPH_NAMES};
use crate::reader::{u16_at, u32_at};
use crate::shape::{Cluster, Glyph, Shaped};
use crate::{Error, Face, GlyphId, KerningSource, TrueTypeFace, sha256};

fn resource_error(e: flashtex_font_resources::Error) -> Error {
    use flashtex_font_resources::Error as R;
    match e {
        R::UnsupportedFont(m) => Error::Unsupported(format!("font-resources: {m}")),
        other => Error::Malformed(format!("font-resources: {other}")),
    }
}

// ---------------------------------------------------------------------------
// Encoding vector (.enc)
// ---------------------------------------------------------------------------

/// A dvips/pdfTeX encoding vector: exactly 256 glyph names in slot order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingVector {
    /// The PostScript name of the vector (`/enclmec` → `enclmec`).
    pub name: String,
    pub sha256: [u8; 32],
    slots: Vec<String>,
}

impl EncodingVector {
    /// Parses `/Name [ /n0 /n1 ... /n255 ] def` with `%` comments. Any other
    /// count of names is malformed; `.notdef` entries are kept as such.
    pub fn parse(text: &str) -> Result<EncodingVector, Error> {
        if text.len() > 1 << 20 {
            return Err(Error::Malformed("encoding vector over 1 MiB".into()));
        }
        let mut tokens: Vec<&str> = Vec::new();
        for line in text.lines() {
            let line = line.split('%').next().unwrap_or("");
            for tok in line.split_whitespace() {
                // Split "[" and "]" glued to names, e.g. "/enclmec[" or "/ffl]".
                let mut rest = tok;
                while let Some(i) = rest.find(['[', ']']) {
                    if i > 0 {
                        tokens.push(&rest[..i]);
                    }
                    tokens.push(&rest[i..=i]);
                    rest = &rest[i + 1..];
                }
                if !rest.is_empty() {
                    tokens.push(rest);
                }
            }
        }
        let name_tok = tokens
            .first()
            .filter(|t| t.starts_with('/'))
            .ok_or_else(|| Error::Malformed("encoding vector: missing /Name".into()))?;
        if tokens.get(1) != Some(&"[") {
            return Err(Error::Malformed("encoding vector: expected '['".into()));
        }
        let close = tokens
            .iter()
            .position(|t| *t == "]")
            .ok_or_else(|| Error::Malformed("encoding vector: missing ']'".into()))?;
        if tokens.get(close + 1) != Some(&"def") {
            return Err(Error::Malformed("encoding vector: expected 'def'".into()));
        }
        let names = &tokens[2..close];
        if names.len() != 256 {
            return Err(Error::Malformed(format!(
                "encoding vector has {} names, expected 256",
                names.len()
            )));
        }
        let mut slots = Vec::with_capacity(256);
        for n in names {
            let n = n
                .strip_prefix('/')
                .ok_or_else(|| Error::Malformed(format!("encoding vector: bad name {n}")))?;
            if n.is_empty() || n.len() > 127 || !n.is_ascii() {
                return Err(Error::Malformed(format!("encoding vector: bad name {n}")));
            }
            slots.push(n.to_string());
        }
        Ok(EncodingVector {
            name: name_tok[1..].to_string(),
            sha256: sha256::digest(text.as_bytes()),
            slots,
        })
    }

    /// The glyph name at `code`, or `None` when the vector says `.notdef`.
    pub fn glyph_name(&self, code: EncodingCode) -> Option<&str> {
        let n = self.slots[usize::from(code.0)].as_str();
        (n != ".notdef").then_some(n)
    }
}

// ---------------------------------------------------------------------------
// The font's own glyph names
// ---------------------------------------------------------------------------

/// Where a face's glyph names were read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphNameSource {
    /// CFF charset (SIDs → standard strings / the font's string INDEX).
    CffCharset,
    /// `post` table format 2.0.
    PostFormat2,
}

/// Original glyph id ↔ the font's own glyph name.
#[derive(Debug, Clone)]
pub struct GlyphNames {
    pub source: GlyphNameSource,
    by_gid: Vec<String>,
    by_name: BTreeMap<String, u16>,
}

impl GlyphNames {
    /// Reads the names the font itself declares. CFF faces use the charset;
    /// `glyf` faces need a `post` 2.0 table. Fonts with neither (`post` 3.0
    /// only) are refused: names cannot be guessed.
    pub fn read(face: &TrueTypeFace) -> Result<GlyphNames, Error> {
        let n = usize::from(face.num_glyphs());
        if let Some(cff) = face.cff_table() {
            let cff = Cff::parse(cff).map_err(resource_error)?;
            if cff.glyph_count() != n {
                return Err(Error::Malformed(format!(
                    "CFF has {} charstrings, maxp says {n}",
                    cff.glyph_count()
                )));
            }
            let mut by_gid = Vec::with_capacity(n);
            for sid in &cff.charset {
                let name = if usize::from(*sid) < CFF_STANDARD_STRINGS.len() {
                    CFF_STANDARD_STRINGS[usize::from(*sid)].to_string()
                } else {
                    let bytes = cff
                        .custom_string(*sid)
                        .ok_or_else(|| Error::Malformed(format!("CFF SID {sid} unresolved")))?;
                    String::from_utf8(bytes.to_vec())
                        .map_err(|_| Error::Malformed(format!("CFF SID {sid} not UTF-8")))?
                };
                by_gid.push(name);
            }
            return Self::index(GlyphNameSource::CffCharset, by_gid);
        }
        let post = face
            .table(b"post")
            .ok_or_else(|| Error::MissingTable("post".into()))?;
        let version = u32_at(post, 0)?;
        if version != 0x0002_0000 {
            return Err(Error::Unsupported(format!(
                "post table version {:#x} carries no glyph names",
                version
            )));
        }
        let count = usize::from(u16_at(post, 32)?);
        if count != n {
            return Err(Error::Malformed(format!(
                "post 2.0 has {count} entries, maxp says {n}"
            )));
        }
        let mut indices = Vec::with_capacity(count);
        for i in 0..count {
            indices.push(u16_at(post, 34 + i * 2)?);
        }
        let mut strings = Vec::new();
        let mut at = 34 + count * 2;
        while at < post.len() {
            let len = usize::from(post[at]);
            let s = post
                .get(at + 1..at + 1 + len)
                .ok_or_else(|| Error::Malformed("post 2.0 string past end".into()))?;
            strings.push(String::from_utf8_lossy(s).into_owned());
            at += 1 + len;
        }
        let mut by_gid = Vec::with_capacity(count);
        for idx in indices {
            let name = if usize::from(idx) < MAC_GLYPH_NAMES.len() {
                MAC_GLYPH_NAMES[usize::from(idx)].to_string()
            } else {
                strings
                    .get(usize::from(idx) - MAC_GLYPH_NAMES.len())
                    .cloned()
                    .ok_or_else(|| Error::Malformed(format!("post 2.0 index {idx} unresolved")))?
            };
            by_gid.push(name);
        }
        Self::index(GlyphNameSource::PostFormat2, by_gid)
    }

    fn index(source: GlyphNameSource, by_gid: Vec<String>) -> Result<GlyphNames, Error> {
        let mut by_name = BTreeMap::new();
        for (gid, name) in by_gid.iter().enumerate() {
            // First occurrence wins; duplicates are legal in post 2.0 and
            // are simply not addressable by name (they stay reachable by gid).
            by_name.entry(name.clone()).or_insert(gid as u16);
        }
        Ok(GlyphNames {
            source,
            by_gid,
            by_name,
        })
    }

    pub fn gid(&self, name: &str) -> Option<GlyphId> {
        self.by_name.get(name).map(|g| GlyphId(*g))
    }

    pub fn name(&self, gid: GlyphId) -> Option<&str> {
        self.by_gid.get(usize::from(gid.0)).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_gid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_gid.is_empty()
    }
}

/// `uniXXXX` / `uXXXX[XX]` (Adobe glyph naming) → scalar; anything else `None`.
pub fn uni_name_to_char(name: &str) -> Option<char> {
    let hex = if let Some(h) = name.strip_prefix("uni") {
        (h.len() == 4).then_some(h)?
    } else if let Some(h) = name.strip_prefix('u') {
        ((4..=6).contains(&h.len())).then_some(h)?
    } else {
        return None;
    };
    if !hex.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_lowercase()) {
        return None;
    }
    char::from_u32(u32::from_str_radix(hex, 16).ok()?)
}

// ---------------------------------------------------------------------------
// Slot binding
// ---------------------------------------------------------------------------

/// How a bound slot's glyph id was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// The font declares a glyph with exactly the encoding's name.
    FontName,
    /// The encoding name is `uniXXXX`/`uXXXX` and the cmap maps that scalar.
    UniName,
    /// The font has no glyph by that name; the slot's T1/OT1 table scalar is
    /// in the cmap (Latin Modern names its ligatures `f_i`, the encoding
    /// says `fi`). Recorded, never silent.
    EncodingTable,
}

/// One slot with everything a TeX-exact consumer needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundSlot {
    pub code: EncodingCode,
    /// The encoding vector's name for the slot.
    pub glyph_name: String,
    /// The font's own name for `gid` (may differ from `glyph_name`).
    pub font_glyph_name: String,
    /// ORIGINAL glyph id in the face.
    pub gid: GlyphId,
    /// Exact TFM metrics (`fix_word`, 2⁻²⁰ em) for the slot.
    pub metrics: CharacterMetrics,
    pub resolution: Resolution,
    /// `true` when the font-name route and the T1/OT1-table→cmap route both
    /// resolved and agreed.
    pub cross_checked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotState {
    Bound(BoundSlot),
    /// The encoding vector says `.notdef` here.
    Unencoded { has_tfm_char: bool },
    /// Named in the encoding but the TFM has no character at the slot.
    NoTfmChar { glyph_name: String },
    /// Named and metricated, but no route reaches a glyph in this face.
    NoGlyph { glyph_name: String },
    /// The name resolves to glyph 0.
    Notdef { glyph_name: String },
    /// The font's own name and the table→cmap route point at different glyphs.
    Disagreement {
        glyph_name: String,
        by_name: GlyphId,
        by_table: GlyphId,
    },
}

impl SlotState {
    fn reason(&self) -> &'static str {
        match self {
            SlotState::Bound(_) => "bound",
            SlotState::Unencoded { has_tfm_char: true } => {
                "encoding vector has .notdef at a slot the TFM metricates"
            }
            SlotState::Unencoded { .. } => "encoding vector has .notdef and the TFM has no char",
            SlotState::NoTfmChar { .. } => "TFM has no character at this slot",
            SlotState::NoGlyph { .. } => "no glyph in the face by name, uniXXXX or the table route",
            SlotState::Notdef { .. } => "name resolves to glyph 0 (.notdef)",
            SlotState::Disagreement { .. } => "font name and table→cmap route disagree",
        }
    }
}

/// TFM width of one slot next to the OpenType advance of the glyph it binds.
#[derive(Debug, Clone, PartialEq)]
pub struct Discrepancy {
    pub code: EncodingCode,
    pub glyph_name: String,
    pub gid: GlyphId,
    pub tfm_width: FixWord,
    /// `tfm_width / 2^20 × units_per_em` (exact rational, as f64).
    pub tfm_width_units: f64,
    pub otf_advance_units: u16,
    /// `tfm_width_units − otf_advance_units`.
    pub delta_units: f64,
    /// Both at the TFM design size, in points.
    pub tfm_width_pt: f64,
    pub otf_advance_pt: f64,
    pub delta_pt: f64,
}

/// Distribution of [`Discrepancy::delta_units`] over every bound slot.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscrepancyReport {
    pub entries: Vec<Discrepancy>,
    /// |Δ| < 1e-9 units.
    pub exact: usize,
    /// 1e-9 ≤ |Δ| < 0.5 units.
    pub under_half_unit: usize,
    /// 0.5 ≤ |Δ| < 1 unit.
    pub under_one_unit: usize,
    /// |Δ| ≥ 1 unit; the offending entries.
    pub one_unit_or_more: Vec<Discrepancy>,
    pub max_abs_units: f64,
    pub max_abs_slot: Option<EncodingCode>,
}

impl DiscrepancyReport {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// One line per bucket, for reports and test output.
    pub fn summary(&self) -> String {
        format!(
            "{} slots: exact {}, <0.5 unit {}, <1 unit {}, >=1 unit {}; max |Δ| {:.4} units at slot {}",
            self.entries.len(),
            self.exact,
            self.under_half_unit,
            self.under_one_unit,
            self.one_unit_or_more.len(),
            self.max_abs_units,
            self.max_abs_slot.map_or("-".to_string(), |c| c.0.to_string())
        )
    }
}

/// TFM kern versus the face's own (GPOS/kern/AFM) kern for the same pair.
#[derive(Debug, Clone, PartialEq)]
pub struct KernComparison {
    pub left: EncodingCode,
    pub right: EncodingCode,
    /// `None` when the TFM program has no kern for the pair (a ligature or
    /// nothing).
    pub tfm: Option<FixWord>,
    pub tfm_units: f64,
    pub opentype_units: i16,
    pub opentype_source: KerningSource,
    /// `tfm_units − opentype_units`.
    pub delta_units: f64,
}

/// Face + TFM + encoding vector, bound slot by slot.
pub struct TfmBinding<'a> {
    face: &'a TrueTypeFace,
    tfm: Tfm,
    tfm_sha256: [u8; 32],
    vector: EncodingVector,
    table: Encoding,
    names: GlyphNames,
    slots: Vec<SlotState>,
}

impl<'a> TfmBinding<'a> {
    /// Binds every slot. `table` is the Unicode view of the same encoding
    /// (T1 for `ec-*`, OT1 for `rm-*`), used for the cross-check and to map
    /// input text to slots when shaping; it never overrides the vector.
    pub fn bind(
        face: &'a TrueTypeFace,
        tfm_bytes: &[u8],
        enc_text: &str,
        table: Encoding,
    ) -> Result<TfmBinding<'a>, Error> {
        let tfm = Tfm::parse(tfm_bytes).map_err(resource_error)?;
        let vector = EncodingVector::parse(enc_text)?;
        let names = GlyphNames::read(face)?;
        let mut slots = Vec::with_capacity(256);
        for c in 0..=255u8 {
            let code = EncodingCode(c);
            let metrics = tfm.char_metrics(c);
            let Some(name) = vector.glyph_name(code) else {
                slots.push(SlotState::Unencoded {
                    has_tfm_char: metrics.is_some(),
                });
                continue;
            };
            let Some(metrics) = metrics else {
                slots.push(SlotState::NoTfmChar {
                    glyph_name: name.to_string(),
                });
                continue;
            };
            let by_name = names.gid(name);
            let by_uni = uni_name_to_char(name).and_then(|ch| face.glyph_id(ch));
            let by_table = table.to_unicode(code).and_then(|ch| face.glyph_id(ch));
            let (gid, resolution) = match (by_name, by_uni, by_table) {
                (Some(g), _, _) => (g, Resolution::FontName),
                (None, Some(g), _) => (g, Resolution::UniName),
                (None, None, Some(g)) => (g, Resolution::EncodingTable),
                (None, None, None) => {
                    slots.push(SlotState::NoGlyph {
                        glyph_name: name.to_string(),
                    });
                    continue;
                }
            };
            if gid == GlyphId::NOTDEF {
                slots.push(SlotState::Notdef {
                    glyph_name: name.to_string(),
                });
                continue;
            }
            let cross_checked = match (by_name.or(by_uni), by_table) {
                (Some(a), Some(b)) if a != b => {
                    slots.push(SlotState::Disagreement {
                        glyph_name: name.to_string(),
                        by_name: a,
                        by_table: b,
                    });
                    continue;
                }
                (Some(_), Some(_)) => true,
                _ => false,
            };
            slots.push(SlotState::Bound(BoundSlot {
                code,
                glyph_name: name.to_string(),
                font_glyph_name: names.name(gid).unwrap_or("").to_string(),
                gid,
                metrics,
                resolution,
                cross_checked,
            }));
        }
        Ok(TfmBinding {
            face,
            tfm_sha256: sha256::digest(tfm_bytes),
            tfm,
            vector,
            table,
            names,
            slots,
        })
    }

    pub fn face(&self) -> &'a TrueTypeFace {
        self.face
    }

    pub fn tfm(&self) -> &Tfm {
        &self.tfm
    }

    pub fn tfm_sha256(&self) -> [u8; 32] {
        self.tfm_sha256
    }

    pub fn vector(&self) -> &EncodingVector {
        &self.vector
    }

    pub fn table(&self) -> Encoding {
        self.table
    }

    pub fn glyph_names(&self) -> &GlyphNames {
        &self.names
    }

    /// Design size as a `fix_word` in points (2⁻²⁰ pt).
    pub fn design_size(&self) -> FixWord {
        self.tfm.design_size
    }

    pub fn design_size_pt(&self) -> f64 {
        fix_to_f64(self.tfm.design_size)
    }

    pub fn state(&self, code: EncodingCode) -> &SlotState {
        &self.slots[usize::from(code.0)]
    }

    /// The bound slot, or an explicit [`Error::UnboundSlot`].
    pub fn slot(&self, code: EncodingCode) -> Result<&BoundSlot, Error> {
        match self.state(code) {
            SlotState::Bound(b) => Ok(b),
            other => Err(Error::UnboundSlot {
                code: code.0,
                glyph_name: self.vector.glyph_name(code).map(str::to_string),
                reason: other.reason(),
            }),
        }
    }

    pub fn bound_slots(&self) -> impl Iterator<Item = &BoundSlot> {
        self.slots.iter().filter_map(|s| match s {
            SlotState::Bound(b) => Some(b),
            _ => None,
        })
    }

    /// Slots that are not bound, with their state, for reports.
    pub fn unbound_slots(&self) -> Vec<(EncodingCode, &SlotState)> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, s)| !matches!(s, SlotState::Bound(_)))
            .map(|(i, s)| (EncodingCode(i as u8), s))
            .collect()
    }

    /// The lig/kern instruction for `left` followed by `right`, straight
    /// from the TFM program.
    pub fn pair(&self, left: EncodingCode, right: EncodingCode) -> Result<Option<PairAction>, Error> {
        self.slot(left)?;
        self.slot(right)?;
        self.tfm.pair_action(left.0, right.0).map_err(resource_error)
    }

    /// Every (right, action) the TFM program defines for `left`, in slot
    /// order, over the slots the TFM metricates.
    pub fn program(&self, left: EncodingCode) -> Result<Vec<(EncodingCode, PairAction)>, Error> {
        self.slot(left)?;
        let mut out = Vec::new();
        for r in 0..=255u8 {
            if self.tfm.char_metrics(r).is_none() {
                continue;
            }
            if let Some(action) = self.tfm.pair_action(left.0, r).map_err(resource_error)? {
                out.push((EncodingCode(r), action));
            }
        }
        Ok(out)
    }

    /// TFM width vs the OpenType advance of the bound glyph.
    pub fn discrepancy(&self, code: EncodingCode) -> Result<Discrepancy, Error> {
        let slot = self.slot(code)?;
        let upem = f64::from(self.face.units_per_em());
        let design = self.design_size_pt();
        let otf = self.face.advance(slot.gid)?;
        let tfm_units = fix_to_f64(slot.metrics.width) * upem;
        let tfm_pt = fix_to_f64(slot.metrics.width) * design;
        let otf_pt = f64::from(otf) / upem * design;
        Ok(Discrepancy {
            code,
            glyph_name: slot.glyph_name.clone(),
            gid: slot.gid,
            tfm_width: slot.metrics.width,
            tfm_width_units: tfm_units,
            otf_advance_units: otf,
            delta_units: tfm_units - f64::from(otf),
            tfm_width_pt: tfm_pt,
            otf_advance_pt: otf_pt,
            delta_pt: tfm_pt - otf_pt,
        })
    }

    pub fn discrepancy_report(&self) -> DiscrepancyReport {
        let mut report = DiscrepancyReport {
            entries: Vec::new(),
            exact: 0,
            under_half_unit: 0,
            under_one_unit: 0,
            one_unit_or_more: Vec::new(),
            max_abs_units: 0.0,
            max_abs_slot: None,
        };
        for slot in self.bound_slots() {
            // A bound slot's gid is in range by construction.
            let Ok(d) = self.discrepancy(slot.code) else {
                continue;
            };
            let a = d.delta_units.abs();
            if a < 1e-9 {
                report.exact += 1;
            } else if a < 0.5 {
                report.under_half_unit += 1;
            } else if a < 1.0 {
                report.under_one_unit += 1;
            } else {
                report.one_unit_or_more.push(d.clone());
            }
            if a > report.max_abs_units {
                report.max_abs_units = a;
                report.max_abs_slot = Some(slot.code);
            }
            report.entries.push(d);
        }
        report
    }

    /// TFM kern for the pair next to the face's own kerning for the same
    /// glyph ids.
    pub fn kern_comparison(
        &self,
        left: EncodingCode,
        right: EncodingCode,
    ) -> Result<KernComparison, Error> {
        let (l, r) = (self.slot(left)?, self.slot(right)?);
        let tfm = match self.pair(left, right)? {
            Some(PairAction::Kern(k)) => Some(k),
            _ => None,
        };
        let upem = f64::from(self.face.units_per_em());
        let tfm_units = tfm.map_or(0.0, |k| fix_to_f64(k) * upem);
        let (ot, source) = self.face.kerning(l.gid, r.gid);
        Ok(KernComparison {
            left,
            right,
            tfm,
            tfm_units,
            opentype_units: ot,
            opentype_source: source,
            delta_units: tfm_units - f64::from(ot),
        })
    }

    /// The binding in `crates/font-resources`' `EncodingManifest` shape:
    /// `font_sha256` is over the program bytes (its convention), bound slots
    /// become `encoding` entries and `declared_glyphs` (plus `.notdef` → 0).
    /// Unbound slots are left out so `EncodingMap::bind` rejects them
    /// explicitly.
    pub fn to_encoding_manifest(&self) -> EncodingManifest {
        let mut declared: BTreeMap<String, u16> = BTreeMap::new();
        declared.insert(".notdef".into(), 0);
        let mut encoding = Vec::new();
        for s in self.bound_slots() {
            encoding.push(EncodingEntry {
                code: s.code.0,
                glyph_name: s.glyph_name.clone(),
            });
            declared.entry(s.glyph_name.clone()).or_insert(s.gid.0);
        }
        EncodingManifest {
            tfm_sha256: sha256::hex(&self.tfm_sha256),
            font_sha256: sha256::hex(&sha256::digest(self.face.program())),
            face_index: match self.face.id().source {
                crate::FontSource::File { face_index, .. }
                | crate::FontSource::Memory { face_index } => face_index,
                crate::FontSource::Core14 { .. } => 0,
            },
            encoding,
            declared_glyphs: declared
                .into_iter()
                .map(|(glyph_name, glyph_id)| NamedGlyph {
                    glyph_name,
                    glyph_id,
                })
                .collect(),
        }
    }

    /// Slot for one scalar through the binding's T1/OT1 table.
    pub fn slot_for_char(&self, ch: char) -> Option<EncodingCode> {
        self.table.from_unicode(ch)
    }

    /// Sets `text` the way pdfTeX would with this TFM at `size_sp`
    /// (scaled points; use [`sp_from_pt`]): slots through the T1/OT1 table,
    /// the TFM lig/kern program instead of GSUB/GPOS, character widths and
    /// kerns through TeX's `store_scaled`, and one interword glue per
    /// whitespace run from `\fontdimen2..4`. `\spacefactor` is not modelled
    /// (that is the paragraph builder's job); boundary-character programs
    /// are refused by the interpreter.
    ///
    /// Fails explicitly for a scalar with no slot ([`Error::NoEncodingSlot`])
    /// and for a slot (input or ligature result) that is not bound
    /// ([`Error::UnboundSlot`]).
    pub fn shape(&self, text: &str, size_sp: i32) -> Result<TfmShaped, Error> {
        let scale = TexScale::at_sp(size_sp)?;
        let mut items = Vec::new();
        let mut ligatures = 0;
        let mut kerns = 0;
        let space = |k: usize| -> Result<i32, Error> {
            scale.scale(self.tfm.parameter(k).unwrap_or(FixWord(0)))
        };
        let (glue_w, glue_st, glue_sh) = (space(2)?, space(3)?, space(4)?);
        let mut byte = 0;
        for word in text.split_inclusive(|c: char| c.is_ascii_whitespace()) {
            // `word` ends with at most one whitespace char, or none at the end.
            let (body, ws) = match word.char_indices().last() {
                Some((i, c)) if c.is_ascii_whitespace() => (&word[..i], &word[i..]),
                _ => (word, ""),
            };
            if !body.is_empty() {
                let mut codes = Vec::with_capacity(body.len());
                let mut offsets = Vec::with_capacity(body.len() + 1);
                for (i, ch) in body.char_indices() {
                    let code = self.slot_for_char(ch).ok_or(Error::NoEncodingSlot {
                        ch,
                        byte_offset: byte + i,
                        encoding: self.table,
                    })?;
                    self.slot(code)?;
                    codes.push(code.0);
                    offsets.push(byte + i);
                }
                offsets.push(byte + body.len());
                let run = self
                    .tfm
                    .apply_ligatures_kerns(&codes)
                    .map_err(resource_error)?;
                for item in run {
                    match item {
                        TfmItem::Kern(k) => {
                            kerns += 1;
                            items.push(TfmShapedItem::Kern {
                                amount: k,
                                amount_sp: scale.scale(k)?,
                            });
                        }
                        TfmItem::Glyph(g) => {
                            let slot = self.slot(EncodingCode(g.code))?;
                            if g.input_end - g.input_start > 1 {
                                ligatures += 1;
                            }
                            let range = offsets[g.input_start]..offsets[g.input_end];
                            items.push(TfmShapedItem::Glyph {
                                code: slot.code,
                                gid: slot.gid,
                                glyph_name: slot.glyph_name.clone(),
                                metrics: slot.metrics,
                                width_sp: scale.scale(slot.metrics.width)?,
                                height_sp: scale.scale(slot.metrics.height)?,
                                depth_sp: scale.scale(slot.metrics.depth)?,
                                italic_sp: scale.scale(slot.metrics.italic)?,
                                text: text[range.clone()].to_string(),
                                source_range: range,
                            });
                        }
                    }
                }
            }
            if !ws.is_empty() {
                let start = byte + body.len();
                // Merge a whitespace run into one glue (TeX skips the rest).
                if let Some(TfmShapedItem::Glue { source_range, text: t, .. }) = items.last_mut()
                    && source_range.end == start
                {
                    source_range.end = start + ws.len();
                    t.push_str(ws);
                } else {
                    items.push(TfmShapedItem::Glue {
                        natural_sp: glue_w,
                        stretch_sp: glue_st,
                        shrink_sp: glue_sh,
                        source_range: start..start + ws.len(),
                        text: ws.to_string(),
                    });
                }
            }
            byte += word.len();
        }
        Ok(TfmShaped {
            items,
            size_sp,
            units_per_em: self.face.units_per_em(),
            font: self.face.id().clone(),
            space_gid: self.face.glyph_id(' '),
            ligatures_applied: ligatures,
            kerns_applied: kerns,
        })
    }
}

// ---------------------------------------------------------------------------
// TeX scaled arithmetic (tex.web §571–575, §103)
// ---------------------------------------------------------------------------

/// `fix_word` as an f64 em fraction (exact for the dyadic value).
pub fn fix_to_f64(f: FixWord) -> f64 {
    f64::from(f.0) / f64::from(1u32 << 20)
}

/// Scaled points for a size in printer's points, TeX's `unity` = 2¹⁶ sp,
/// rounded to the nearest sp (what `\font\x=... at 10pt` scans to).
pub fn sp_from_pt(pt: f64) -> i32 {
    (pt * 65536.0).round() as i32
}

/// TeX's `print_scaled`: the decimal TeX itself prints for `sp` (5 digits,
/// rounded as in §103), so `\showbox` output can be compared textually.
pub fn print_scaled(mut s: i32) -> String {
    let unity = 65536;
    let mut out = String::new();
    if s < 0 {
        out.push('-');
        s = -s;
    }
    out.push_str(&(s / unity).to_string());
    out.push('.');
    s = 10 * (s % unity) + 5;
    let mut delta = 10;
    loop {
        if delta > unity {
            s += 32768 - 50000; // round the last digit
        }
        out.push(char::from(b'0' + (s / unity) as u8));
        s = 10 * (s % unity);
        delta *= 10;
        if s <= delta {
            break;
        }
    }
    out
}

/// The `z`/`alpha`/`beta` triple TeX derives from a font's at-size (§572)
/// and uses to convert every `fix_word` of the TFM to scaled points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TexScale {
    pub z: i32,
    pub alpha: i32,
    pub beta: i32,
}

impl TexScale {
    /// `size_sp` must be positive and below 2²⁷ (TeX's limit for font sizes).
    pub fn at_sp(size_sp: i32) -> Result<TexScale, Error> {
        if size_sp <= 0 || size_sp >= 1 << 27 {
            return Err(Error::Unsupported(format!(
                "font size {size_sp} sp outside TeX's (0, 2^27) range"
            )));
        }
        let mut z = size_sp;
        let mut alpha = 16;
        while z >= 1 << 23 {
            z /= 2;
            alpha *= 2;
        }
        let beta = 256 / alpha;
        Ok(TexScale {
            z,
            alpha: alpha * z,
            beta,
        })
    }

    /// TeX's `store_scaled` (§571): the scaled-point value of `fix` at this
    /// size, computed with TeX's integer arithmetic, not floating point.
    pub fn scale(&self, fix: FixWord) -> Result<i32, Error> {
        let [a, b, c, d] = fix.0.to_be_bytes();
        let z = i64::from(self.z);
        let sw = (((i64::from(d) * z) / 256 + i64::from(c) * z) / 256 + i64::from(b) * z)
            / i64::from(self.beta);
        let sw = match a {
            0 => sw,
            255 => sw - i64::from(self.alpha),
            _ => {
                return Err(Error::Malformed(format!(
                    "fix_word {:#010x} outside TeX's scalable range",
                    fix.0
                )));
            }
        };
        i32::try_from(sw).map_err(|_| Error::Malformed("scaled value overflow".into()))
    }
}

// ---------------------------------------------------------------------------
// Shaping result
// ---------------------------------------------------------------------------

/// One item of a TeX-exact horizontal list. `_sp` values are TeX scaled
/// points at the shaping size, computed as TeX computes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TfmShapedItem {
    Glyph {
        code: EncodingCode,
        /// ORIGINAL glyph id in the bound face.
        gid: GlyphId,
        glyph_name: String,
        /// Exact TFM `fix_word` metrics.
        metrics: CharacterMetrics,
        width_sp: i32,
        height_sp: i32,
        depth_sp: i32,
        italic_sp: i32,
        source_range: Range<usize>,
        text: String,
    },
    /// A kern from the lig/kern program (TeX's `\kern` in `\showbox`).
    Kern { amount: FixWord, amount_sp: i32 },
    /// Interword glue from `\fontdimen2..4`.
    Glue {
        natural_sp: i32,
        stretch_sp: i32,
        shrink_sp: i32,
        source_range: Range<usize>,
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TfmShaped {
    pub items: Vec<TfmShapedItem>,
    pub size_sp: i32,
    pub units_per_em: u16,
    pub font: crate::FontId,
    /// The face's cmap glyph for U+0020, used by [`TfmShaped::to_shaped`]
    /// to carry glue as a visible-ink-free glyph.
    pub space_gid: Option<GlyphId>,
    pub ligatures_applied: usize,
    pub kerns_applied: usize,
}

impl TfmShaped {
    /// Natural width in scaled points: glyph widths + kerns + glue natural
    /// widths, exactly as TeX's `hpack` would sum them.
    pub fn natural_width_sp(&self) -> i64 {
        self.items
            .iter()
            .map(|i| match i {
                TfmShapedItem::Glyph { width_sp, .. } => i64::from(*width_sp),
                TfmShapedItem::Kern { amount_sp, .. } => i64::from(*amount_sp),
                TfmShapedItem::Glue { natural_sp, .. } => i64::from(*natural_sp),
            })
            .sum()
    }

    pub fn natural_width_pt(&self) -> f64 {
        self.natural_width_sp() as f64 / 65536.0
    }

    /// `\showbox`-style listing: one line per item, TeX's `print_scaled`
    /// decimals, glyphs by slot and name.
    pub fn showbox(&self) -> String {
        let mut out = String::new();
        for i in &self.items {
            match i {
                TfmShapedItem::Glyph {
                    code,
                    glyph_name,
                    width_sp,
                    ..
                } => out.push_str(&format!(
                    "char {} ({}) width {}\n",
                    code.0,
                    glyph_name,
                    print_scaled(*width_sp)
                )),
                TfmShapedItem::Kern { amount_sp, .. } => {
                    out.push_str(&format!("\\kern{}\n", print_scaled(*amount_sp)));
                }
                TfmShapedItem::Glue {
                    natural_sp,
                    stretch_sp,
                    shrink_sp,
                    ..
                } => out.push_str(&format!(
                    "\\glue {} plus {} minus {}\n",
                    print_scaled(*natural_sp),
                    print_scaled(*stretch_sp),
                    print_scaled(*shrink_sp)
                )),
            }
        }
        out
    }

    /// Projects the TeX-exact list onto the engine's [`Shaped`] contract
    /// (integer font units, kerns folded into the preceding advance, glue as
    /// the cmap space glyph at the glue's natural width). Each item rounds
    /// once from sp to units (≤ 0.5 unit); consumers that need TeX's exact
    /// scaled points read `items` directly.
    pub fn to_shaped(&self) -> Result<Shaped, Error> {
        let to_units = |sp: i32| -> i32 {
            (f64::from(sp) * f64::from(self.units_per_em) / f64::from(self.size_sp)).round() as i32
        };
        let mut clusters: Vec<Cluster> = Vec::new();
        for item in &self.items {
            match item {
                TfmShapedItem::Glyph {
                    gid,
                    width_sp,
                    source_range,
                    text,
                    ..
                } => clusters.push(Cluster {
                    glyphs: vec![Glyph {
                        gid: *gid,
                        advance: to_units(*width_sp),
                        x_offset: 0,
                        y_offset: 0,
                    }],
                    source_range: source_range.clone(),
                    text: text.clone(),
                    font: 0,
                }),
                TfmShapedItem::Kern { amount_sp, .. } => {
                    let Some(g) = clusters.last_mut().and_then(|c| c.glyphs.last_mut()) else {
                        return Err(Error::Malformed("TFM kern before any glyph".into()));
                    };
                    g.advance += to_units(*amount_sp);
                }
                TfmShapedItem::Glue {
                    natural_sp,
                    source_range,
                    text,
                    ..
                } => {
                    let gid = self.space_gid.ok_or_else(|| {
                        Error::Unsupported(
                            "face has no U+0020 glyph to carry interword glue in Shaped".into(),
                        )
                    })?;
                    clusters.push(Cluster {
                        glyphs: vec![Glyph {
                            gid,
                            advance: to_units(*natural_sp),
                            x_offset: 0,
                            y_offset: 0,
                        }],
                        source_range: source_range.clone(),
                        text: text.clone(),
                        font: 0,
                    });
                }
            }
        }
        Ok(Shaped {
            clusters,
            fonts: vec![self.font.clone()],
            missing: Vec::new(),
            unsupported: Vec::new(),
            units_per_em: self.units_per_em,
            kerning_source: if self.kerns_applied > 0 {
                KerningSource::Tfm
            } else {
                KerningSource::None
            },
            ligatures_applied: self.ligatures_applied,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_scaled_matches_tex_for_known_values() {
        // At 10 pt: z = 655360, alpha = 16 * 655360, beta = 16.
        let s = TexScale::at_sp(655360).unwrap();
        assert_eq!((s.z, s.alpha, s.beta), (655360, 16 * 655360, 16));
        // 0.75 em → 7.5 pt = 491520 sp exactly.
        assert_eq!(s.scale(FixWord(786432)).unwrap(), 491520);
        // 1 em (quad) → 10 pt.
        assert_eq!(s.scale(FixWord(1 << 20)).unwrap(), 655360);
        // A negative kern goes through the a = 255 branch.
        let k = s.scale(FixWord(-116509)).unwrap();
        assert!(k < 0 && (k as f64 / 65536.0 + 1.11112).abs() < 1e-4, "{k}");
        // Values with a ∉ {0, 255} are outside TeX's range.
        assert!(s.scale(FixWord(1 << 24)).is_err());
        assert!(TexScale::at_sp(0).is_err());
        assert!(TexScale::at_sp(1 << 27).is_err());
    }

    #[test]
    fn print_scaled_matches_tex_output() {
        assert_eq!(print_scaled(655360), "10.0");
        assert_eq!(print_scaled(491520), "7.5");
        assert_eq!(print_scaled(-72818), "-1.11111");
        assert_eq!(print_scaled(1), "0.00002");
        assert_eq!(print_scaled(65535), "0.99998");
    }

    #[test]
    fn encoding_vector_parses_and_rejects() {
        let mut text = String::from("% comment\n/enctest[\n");
        for i in 0..256 {
            text.push_str(&format!("/g{i}{}\n", if i == 255 { "]" } else { "" }));
        }
        text.push_str("def\n");
        let v = EncodingVector::parse(&text).unwrap();
        assert_eq!(v.name, "enctest");
        assert_eq!(v.glyph_name(EncodingCode(255)), Some("g255"));
        let short = text.replacen("/g100\n", "", 1);
        assert!(EncodingVector::parse(&short).is_err());
        assert!(EncodingVector::parse("garbage").is_err());
    }

    #[test]
    fn uni_names() {
        assert_eq!(uni_name_to_char("uni00E9"), Some('é'));
        assert_eq!(uni_name_to_char("u1F600"), Some('😀'));
        assert_eq!(uni_name_to_char("uni00e9"), None);
        assert_eq!(uni_name_to_char("union"), None);
        assert_eq!(uni_name_to_char("A"), None);
    }
}

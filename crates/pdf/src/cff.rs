//! Compact Font Format reader and glyph-identity-preserving subsetter.
//!
//! The subset keeps every retained glyph's **original glyph id** as its CID:
//! the output is a CID-keyed CFF (ROS `Adobe-Identity-0`) whose charset maps
//! subset glyph `i` to CID = original GID, with one FD in `FDArray` carrying
//! the original Private DICT and local subroutines verbatim. Charstrings,
//! global subroutines and local subroutines are copied byte for byte, so a
//! retained glyph's outline bytes are identical to the source font's and the
//! subroutine biases are unchanged. A `Type0`/`Identity-H` font over this
//! program therefore selects glyphs by the source font's own GIDs (the
//! writer's `/CIDToGIDMap`-free `CIDFontType0C` route), which is the
//! contract issue #25 asks for: no re-encoding by character.
//!
//! Bounded on purpose:
//! - Only non-CID-keyed, Type 2 charstring sources are subset. CID-keyed
//!   sources and Type 1 charstrings are reported, not guessed at (they can
//!   still be embedded whole through the existing route).
//! - A glyph whose charstring composes an accent through `endchar` (the
//!   `seac` form) is refused: in a CID-keyed program its component lookup
//!   goes through CIDs instead of names and would pick the wrong glyph.
//! - Glyph count and byte size are checked against explicit limits.
//!
//! The writer expects [`CffFont::parse`] on the raw `CFF ` table of an
//! OpenType font (or a bare `.cff` file) and [`CffFont::subset`] for the
//! retained GIDs.

use std::collections::BTreeSet;

/// Highest glyph count accepted for a subset (CFF glyph ids are 16-bit).
pub const MAX_GLYPHS: usize = 65535;
/// Largest source table accepted (bytes).
pub const MAX_BYTES: usize = 64 * 1024 * 1024;
/// Number of standard strings in the CFF specification (Appendix A).
pub const STANDARD_STRINGS: usize = 391;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CffError {
    Truncated(&'static str),
    Malformed(String),
    /// The font is CID-keyed already; identity subsetting is not attempted.
    CidKeyedSource,
    /// `CharstringType` is not 2.
    CharstringType(i32),
    /// A retained glyph composes an accent via `endchar`; see module docs.
    Seac {
        gid: u16,
    },
    /// A requested glyph id is not in the font.
    GlyphOutOfRange {
        gid: u16,
        glyph_count: u16,
    },
    Limit(&'static str),
}

impl std::fmt::Display for CffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CffError::Truncated(w) => write!(f, "CFF truncated at {w}"),
            CffError::Malformed(m) => write!(f, "malformed CFF: {m}"),
            CffError::CidKeyedSource => write!(
                f,
                "source CFF is CID-keyed; identity subsetting is not supported, embed it whole"
            ),
            CffError::CharstringType(t) => write!(f, "CFF CharstringType {t} is not supported"),
            CffError::Seac { gid } => write!(
                f,
                "glyph {gid} composes an accent through endchar (seac); GID-preserving subsetting would break its components, embed the font whole"
            ),
            CffError::GlyphOutOfRange { gid, glyph_count } => {
                write!(f, "glyph {gid} is outside the font's {glyph_count} glyphs")
            }
            CffError::Limit(w) => write!(f, "CFF limit exceeded: {w}"),
        }
    }
}

type R<T> = Result<T, CffError>;

fn u8_at(b: &[u8], at: usize, what: &'static str) -> R<u8> {
    b.get(at).copied().ok_or(CffError::Truncated(what))
}

fn u16_at(b: &[u8], at: usize, what: &'static str) -> R<u16> {
    b.get(at..at + 2)
        .map(|s| u16::from_be_bytes([s[0], s[1]]))
        .ok_or(CffError::Truncated(what))
}

fn offset_at(b: &[u8], at: usize, size: u8, what: &'static str) -> R<usize> {
    let s = b
        .get(at..at + size as usize)
        .ok_or(CffError::Truncated(what))?;
    Ok(s.iter().fold(0usize, |acc, &x| (acc << 8) | x as usize))
}

/// A parsed INDEX: the byte range of each item in the source, plus the byte
/// range of the whole structure so it can be copied verbatim.
#[derive(Debug, Clone)]
struct Index {
    start: usize,
    end: usize,
    items: Vec<(usize, usize)>,
}

impl Index {
    fn parse(b: &[u8], start: usize, what: &'static str) -> R<Index> {
        let count = u16_at(b, start, what)? as usize;
        if count == 0 {
            return Ok(Index {
                start,
                end: start + 2,
                items: Vec::new(),
            });
        }
        let off_size = u8_at(b, start + 2, what)?;
        if !(1..=4).contains(&off_size) {
            return Err(CffError::Malformed(format!(
                "{what} INDEX offSize {off_size}"
            )));
        }
        let offsets_at = start + 3;
        let data_at = offsets_at + (count + 1) * off_size as usize - 1;
        let mut items = Vec::with_capacity(count);
        let mut prev = offset_at(b, offsets_at, off_size, what)?;
        if prev != 1 {
            return Err(CffError::Malformed(format!(
                "{what} INDEX first offset {prev}"
            )));
        }
        for i in 1..=count {
            let next = offset_at(b, offsets_at + i * off_size as usize, off_size, what)?;
            if next < prev {
                return Err(CffError::Malformed(format!(
                    "{what} INDEX offsets decrease"
                )));
            }
            let (s, e) = (data_at + prev, data_at + next);
            if e > b.len() {
                return Err(CffError::Truncated(what));
            }
            items.push((s, e));
            prev = next;
        }
        Ok(Index {
            start,
            end: data_at + prev,
            items,
        })
    }

    fn raw<'a>(&self, b: &'a [u8]) -> &'a [u8] {
        &b[self.start..self.end]
    }
}

/// One DICT entry: operator (escaped operators are `1200 + second byte`)
/// and the verbatim operand bytes that preceded it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DictEntry {
    op: u16,
    operands: Vec<u8>,
}

/// Decodes the integer operands of a DICT entry (reals are rejected).
fn dict_ints(operands: &[u8]) -> R<Vec<i32>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < operands.len() {
        let b0 = operands[i];
        let (v, n) = match b0 {
            32..=246 => (b0 as i32 - 139, 1),
            247..=250 => {
                let b1 = *operands
                    .get(i + 1)
                    .ok_or(CffError::Truncated("dict operand"))?;
                ((b0 as i32 - 247) * 256 + b1 as i32 + 108, 2)
            }
            251..=254 => {
                let b1 = *operands
                    .get(i + 1)
                    .ok_or(CffError::Truncated("dict operand"))?;
                (-(b0 as i32 - 251) * 256 - b1 as i32 - 108, 2)
            }
            28 => (
                i16::from_be_bytes([
                    *operands
                        .get(i + 1)
                        .ok_or(CffError::Truncated("dict operand"))?,
                    *operands
                        .get(i + 2)
                        .ok_or(CffError::Truncated("dict operand"))?,
                ]) as i32,
                3,
            ),
            29 => (
                i32::from_be_bytes([
                    *operands
                        .get(i + 1)
                        .ok_or(CffError::Truncated("dict operand"))?,
                    *operands
                        .get(i + 2)
                        .ok_or(CffError::Truncated("dict operand"))?,
                    *operands
                        .get(i + 3)
                        .ok_or(CffError::Truncated("dict operand"))?,
                    *operands
                        .get(i + 4)
                        .ok_or(CffError::Truncated("dict operand"))?,
                ]),
                5,
            ),
            30 => {
                return Err(CffError::Malformed(
                    "real operand where an integer was expected".into(),
                ));
            }
            _ => return Err(CffError::Malformed(format!("dict operand byte {b0}"))),
        };
        out.push(v);
        i += n;
    }
    Ok(out)
}

fn parse_dict(b: &[u8]) -> R<Vec<DictEntry>> {
    let mut entries = Vec::new();
    let mut operands = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let b0 = b[i];
        match b0 {
            0..=21 => {
                let op = if b0 == 12 {
                    i += 1;
                    1200 + *b.get(i).ok_or(CffError::Truncated("dict escape"))? as u16
                } else {
                    b0 as u16
                };
                entries.push(DictEntry {
                    op,
                    operands: std::mem::take(&mut operands),
                });
                i += 1;
            }
            28 => {
                operands.extend_from_slice(b.get(i..i + 3).ok_or(CffError::Truncated("dict"))?);
                i += 3;
            }
            29 => {
                operands.extend_from_slice(b.get(i..i + 5).ok_or(CffError::Truncated("dict"))?);
                i += 5;
            }
            30 => {
                // Real: nibbles until 0xF.
                let start = i;
                i += 1;
                loop {
                    let x = *b.get(i).ok_or(CffError::Truncated("dict real"))?;
                    i += 1;
                    if x & 0x0F == 0x0F || x >> 4 == 0x0F {
                        break;
                    }
                }
                operands.extend_from_slice(&b[start..i]);
            }
            32..=246 => {
                operands.push(b0);
                i += 1;
            }
            247..=254 => {
                operands.extend_from_slice(b.get(i..i + 2).ok_or(CffError::Truncated("dict"))?);
                i += 2;
            }
            _ => return Err(CffError::Malformed(format!("dict byte {b0} at {i}"))),
        }
    }
    if !operands.is_empty() {
        return Err(CffError::Malformed(
            "dict ends with dangling operands".into(),
        ));
    }
    Ok(entries)
}

/// Five-byte integer operand (`29 b1 b2 b3 b4`), used for every offset the
/// subsetter writes so sizes do not depend on the value.
fn int5(v: i32) -> [u8; 5] {
    let b = v.to_be_bytes();
    [29, b[0], b[1], b[2], b[3]]
}

fn encode_op(op: u16, out: &mut Vec<u8>) {
    if op >= 1200 {
        out.push(12);
        out.push((op - 1200) as u8);
    } else {
        out.push(op as u8);
    }
}

const OP_CHARSET: u16 = 15;
const OP_ENCODING: u16 = 16;
const OP_CHARSTRINGS: u16 = 17;
const OP_PRIVATE: u16 = 18;
const OP_SUBRS: u16 = 19;
const OP_CHARSTRING_TYPE: u16 = 1206;
const OP_ROS: u16 = 1230;
const OP_CID_COUNT: u16 = 1234;
const OP_FDARRAY: u16 = 1236;
const OP_FDSELECT: u16 = 1237;

/// Where a Private DICT and its local subroutines live in the source.
#[derive(Debug, Clone)]
struct PrivateInfo {
    /// (offset, size) of the Private DICT, if any.
    private: Option<(usize, usize)>,
    local_subrs: Option<Index>,
}

fn parse_private(data: &[u8], entry: Option<&DictEntry>) -> R<PrivateInfo> {
    let Some(p) = entry else {
        return Ok(PrivateInfo {
            private: None,
            local_subrs: None,
        });
    };
    let v = dict_ints(&p.operands)?;
    if v.len() != 2 || v[0] < 0 || v[1] < 0 {
        return Err(CffError::Malformed("Private operands".into()));
    }
    let (size, off) = (v[0] as usize, v[1] as usize);
    if off + size > data.len() {
        return Err(CffError::Truncated("Private DICT"));
    }
    let pdict = parse_dict(&data[off..off + size])?;
    let mut local_subrs = None;
    if let Some(s) = pdict.iter().find(|e| e.op == OP_SUBRS) {
        let rel = *dict_ints(&s.operands)?
            .first()
            .ok_or_else(|| CffError::Malformed("Subrs operand".into()))?;
        let at = off as i64 + rel as i64;
        if at < 0 || at as usize >= data.len() {
            return Err(CffError::Malformed("Subrs offset outside table".into()));
        }
        local_subrs = Some(Index::parse(data, at as usize, "Local Subr")?);
    }
    Ok(PrivateInfo {
        private: Some((off, size)),
        local_subrs,
    })
}

/// A parsed CFF program (one font).
#[derive(Debug, Clone)]
pub struct CffFont {
    data: Vec<u8>,
    name_index: Index,
    top_dict: Vec<DictEntry>,
    string_index: Index,
    global_subrs: Index,
    charstrings: Index,
    /// Private DICT and local subroutines: one entry for a plain font, one
    /// per Font DICT for a CID-keyed font.
    fds: Vec<PrivateInfo>,
    is_cid: bool,
    charstring_type: i32,
    /// Charset: glyph id to SID (or CID for CID-keyed fonts).
    charset: Vec<u16>,
}

impl CffFont {
    /// Parses a raw CFF table.
    pub fn parse(data: &[u8]) -> R<CffFont> {
        if data.len() > MAX_BYTES {
            return Err(CffError::Limit("table larger than 64 MiB"));
        }
        if u8_at(data, 0, "header")? != 1 {
            return Err(CffError::Malformed("major version is not 1".into()));
        }
        let header_len = u8_at(data, 2, "header")? as usize;
        if header_len < 4 {
            return Err(CffError::Malformed("header shorter than 4 bytes".into()));
        }
        let name_index = Index::parse(data, header_len, "Name")?;
        let top_index = Index::parse(data, name_index.end, "Top DICT")?;
        if top_index.items.len() != 1 || name_index.items.len() != 1 {
            return Err(CffError::Malformed(
                "only single-font CFF tables are supported".into(),
            ));
        }
        let string_index = Index::parse(data, top_index.end, "String")?;
        let global_subrs = Index::parse(data, string_index.end, "Global Subr")?;
        let (ts, te) = top_index.items[0];
        let top_dict = parse_dict(&data[ts..te])?;
        let get = |op: u16| top_dict.iter().find(|e| e.op == op);
        let is_cid = get(OP_ROS).is_some();
        let charstring_type = match get(OP_CHARSTRING_TYPE) {
            Some(e) => *dict_ints(&e.operands)?.first().unwrap_or(&2),
            None => 2,
        };
        let cs_off = get(OP_CHARSTRINGS)
            .ok_or_else(|| CffError::Malformed("Top DICT has no CharStrings".into()))?;
        let cs_off = *dict_ints(&cs_off.operands)?
            .first()
            .ok_or_else(|| CffError::Malformed("CharStrings operand".into()))?;
        if cs_off < 0 {
            return Err(CffError::Malformed("negative CharStrings offset".into()));
        }
        let charstrings = Index::parse(data, cs_off as usize, "CharStrings")?;
        let glyph_count = charstrings.items.len();
        if glyph_count == 0 || glyph_count > MAX_GLYPHS {
            return Err(CffError::Limit("glyph count"));
        }
        // Private DICTs: one from the Top DICT for a plain font, one per
        // Font DICT in FDArray for a CID-keyed font.
        let mut fds = Vec::new();
        if is_cid {
            let fda = get(OP_FDARRAY)
                .ok_or_else(|| CffError::Malformed("CID font without FDArray".into()))?;
            let off = *dict_ints(&fda.operands)?
                .first()
                .ok_or_else(|| CffError::Malformed("FDArray operand".into()))?;
            if off < 0 {
                return Err(CffError::Malformed("negative FDArray offset".into()));
            }
            let fd_index = Index::parse(data, off as usize, "FDArray")?;
            if fd_index.items.is_empty() || fd_index.items.len() > 256 {
                return Err(CffError::Malformed("FDArray count".into()));
            }
            for &(s, e) in &fd_index.items {
                let fd = parse_dict(&data[s..e])?;
                fds.push(parse_private(data, fd.iter().find(|e| e.op == OP_PRIVATE))?);
            }
        } else {
            fds.push(parse_private(data, get(OP_PRIVATE))?);
        }
        let charset = parse_charset(data, get(OP_CHARSET), glyph_count)?;
        Ok(CffFont {
            data: data.to_vec(),
            name_index,
            top_dict,
            string_index,
            global_subrs,
            charstrings,
            fds,
            is_cid,
            charstring_type,
            charset,
        })
    }

    pub fn glyph_count(&self) -> u16 {
        self.charstrings.items.len() as u16
    }

    pub fn is_cid_keyed(&self) -> bool {
        self.is_cid
    }

    /// Number of Private DICTs (Font DICTs for a CID-keyed font, else 1).
    pub fn fd_count(&self) -> usize {
        self.fds.len()
    }

    /// The raw source bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }

    /// The font name from the Name INDEX.
    pub fn name(&self) -> String {
        let (s, e) = self.name_index.items[0];
        String::from_utf8_lossy(&self.data[s..e]).into_owned()
    }

    /// The raw charstring bytes of a glyph.
    pub fn charstring(&self, gid: u16) -> Option<&[u8]> {
        self.charstrings
            .items
            .get(gid as usize)
            .map(|&(s, e)| &self.data[s..e])
    }

    /// The SID (or CID) of a glyph in the charset.
    pub fn charset_entry(&self, gid: u16) -> Option<u16> {
        self.charset.get(gid as usize).copied()
    }

    /// Glyph name for a non-CID font (standard or custom string).
    pub fn glyph_name(&self, gid: u16) -> Option<String> {
        if self.is_cid {
            return None;
        }
        let sid = self.charset_entry(gid)? as usize;
        if sid < STANDARD_STRINGS {
            return STANDARD_STRING_NAMES.get(sid).map(|s| s.to_string());
        }
        let (s, e) = *self.string_index.items.get(sid - STANDARD_STRINGS)?;
        Some(String::from_utf8_lossy(&self.data[s..e]).into_owned())
    }

    /// Glyph id of a glyph name (non-CID fonts), scanning the charset.
    pub fn gid_by_name(&self, name: &str) -> Option<u16> {
        (0..self.glyph_count()).find(|&g| self.glyph_name(g).as_deref() == Some(name))
    }

    fn bias(count: usize) -> i32 {
        if count < 1240 {
            107
        } else if count < 33900 {
            1131
        } else {
            32768
        }
    }

    /// True when the glyph's charstring (following subroutines) ends with a
    /// `seac`-style `endchar`. Interprets Type 2 operators for stack depth only.
    pub fn uses_seac(&self, gid: u16) -> R<bool> {
        let cs = self.charstring(gid).ok_or(CffError::GlyphOutOfRange {
            gid,
            glyph_count: self.glyph_count(),
        })?;
        let mut st = SeacScan {
            font: self,
            stack: 0,
            stems: 0,
            width_parsed: false,
            depth: 0,
            found: false,
            last_value: None,
        };
        st.run(cs)?;
        Ok(st.found)
    }

    /// Builds the GID-preserving CID-keyed subset described in the module
    /// docs. `.notdef` (GID 0) is always included.
    pub fn subset(&self, gids: &BTreeSet<u16>) -> R<CffSubset> {
        if self.is_cid {
            return Err(CffError::CidKeyedSource);
        }
        if self.charstring_type != 2 {
            return Err(CffError::CharstringType(self.charstring_type));
        }
        let glyph_count = self.glyph_count();
        let mut keep: Vec<u16> = vec![0];
        for &g in gids {
            if g >= glyph_count {
                return Err(CffError::GlyphOutOfRange {
                    gid: g,
                    glyph_count,
                });
            }
            if g != 0 {
                keep.push(g);
            }
        }
        for &g in &keep {
            if self.uses_seac(g)? {
                return Err(CffError::Seac { gid: g });
            }
        }
        let d = &self.data;

        // Strings: original strings plus the ROS names.
        let mut strings: Vec<&[u8]> = self
            .string_index
            .items
            .iter()
            .map(|&(s, e)| &d[s..e])
            .collect();
        let sid_adobe = (STANDARD_STRINGS + strings.len()) as i32;
        strings.push(b"Adobe");
        let sid_identity = (STANDARD_STRINGS + strings.len()) as i32;
        strings.push(b"Identity");
        let string_index = build_index(&strings);

        // Private DICT: verbatim, except that Subrs must point just past it.
        let (private_bytes, local_subrs_bytes): (Vec<u8>, Vec<u8>) = match self.fds[0].private {
            Some((off, size)) => {
                let entries = parse_dict(&d[off..off + size])?;
                let mut out = Vec::new();
                let mut subrs = Vec::new();
                let has_subrs = entries.iter().any(|e| e.op == OP_SUBRS);
                // Emit all entries except Subrs; Subrs goes last with a fixed
                // 5-byte operand so its value can equal the final dict length.
                for e in &entries {
                    if e.op == OP_SUBRS {
                        continue;
                    }
                    out.extend_from_slice(&e.operands);
                    encode_op(e.op, &mut out);
                }
                if has_subrs {
                    let idx = self.fds[0]
                        .local_subrs
                        .as_ref()
                        .ok_or_else(|| CffError::Malformed("Subrs without INDEX".into()))?;
                    let final_len = out.len() + 5 + 1;
                    out.extend_from_slice(&int5(final_len as i32));
                    encode_op(OP_SUBRS, &mut out);
                    subrs = idx.raw(d).to_vec();
                }
                (out, subrs)
            }
            None => (Vec::new(), Vec::new()),
        };

        // Charset format 0: CID of each glyph after .notdef = original GID.
        let mut charset = vec![0u8];
        for &g in &keep[1..] {
            charset.extend_from_slice(&g.to_be_bytes());
        }
        // FDSelect format 3: one range, all glyphs in FD 0.
        let mut fdselect = vec![3u8, 0, 1, 0, 0, 0];
        fdselect.extend_from_slice(&(keep.len() as u16).to_be_bytes());

        let charstrings: Vec<&[u8]> = keep
            .iter()
            .map(|&g| self.charstring(g).expect("range checked"))
            .collect();
        let charstrings_index = build_index(&charstrings);

        // Layout (in order): header, Name INDEX, Top DICT INDEX, String INDEX,
        // Global Subr INDEX, charset, FDSelect, CharStrings INDEX, FDArray
        // INDEX, Private DICT, Local Subr INDEX. The Top DICT is built with
        // fixed-size offset operands so its length is known before the
        // offsets are.
        let header: Vec<u8> = vec![1, 0, 4, 4];
        let name_index = self.name_index.raw(d).to_vec();
        let global_subrs = self.global_subrs.raw(d).to_vec();
        let cid_count = keep.last().map_or(1, |&g| g as i32 + 1);

        let build_top = |charset_off: i32, fdselect_off: i32, cs_off: i32, fdarray_off: i32| {
            let mut t = Vec::new();
            t.extend_from_slice(&int5(sid_adobe));
            t.extend_from_slice(&int5(sid_identity));
            t.extend_from_slice(&int5(0));
            encode_op(OP_ROS, &mut t);
            for e in &self.top_dict {
                if matches!(
                    e.op,
                    OP_CHARSET
                        | OP_ENCODING
                        | OP_CHARSTRINGS
                        | OP_PRIVATE
                        | OP_ROS
                        | OP_CID_COUNT
                        | OP_FDARRAY
                        | OP_FDSELECT
                ) {
                    continue;
                }
                t.extend_from_slice(&e.operands);
                encode_op(e.op, &mut t);
            }
            t.extend_from_slice(&int5(cid_count));
            encode_op(OP_CID_COUNT, &mut t);
            t.extend_from_slice(&int5(charset_off));
            encode_op(OP_CHARSET, &mut t);
            t.extend_from_slice(&int5(fdselect_off));
            encode_op(OP_FDSELECT, &mut t);
            t.extend_from_slice(&int5(cs_off));
            encode_op(OP_CHARSTRINGS, &mut t);
            t.extend_from_slice(&int5(fdarray_off));
            encode_op(OP_FDARRAY, &mut t);
            t
        };
        let top_len = build_index(&[&build_top(0, 0, 0, 0)]).len();
        let fixed_prefix = header.len() + name_index.len() + top_len;
        let charset_off = fixed_prefix + string_index.len() + global_subrs.len();
        let fdselect_off = charset_off + charset.len();
        let cs_off = fdselect_off + fdselect.len();
        let fdarray_off = cs_off + charstrings_index.len();

        // FDArray with one Font DICT: Private size + offset.
        let build_fd = |private_off: i32| {
            let mut fd = Vec::new();
            fd.extend_from_slice(&int5(private_bytes.len() as i32));
            fd.extend_from_slice(&int5(private_off));
            encode_op(OP_PRIVATE, &mut fd);
            build_index(&[&fd])
        };
        let fdarray_len = build_fd(0).len();
        let private_off = fdarray_off + fdarray_len;

        let mut out = Vec::new();
        out.extend_from_slice(&header);
        out.extend_from_slice(&name_index);
        out.extend_from_slice(&build_index(&[&build_top(
            charset_off as i32,
            fdselect_off as i32,
            cs_off as i32,
            fdarray_off as i32,
        )]));
        out.extend_from_slice(&string_index);
        out.extend_from_slice(&global_subrs);
        debug_assert_eq!(out.len(), charset_off);
        out.extend_from_slice(&charset);
        out.extend_from_slice(&fdselect);
        out.extend_from_slice(&charstrings_index);
        debug_assert_eq!(out.len(), fdarray_off);
        out.extend_from_slice(&build_fd(private_off as i32));
        debug_assert_eq!(out.len(), private_off);
        out.extend_from_slice(&private_bytes);
        out.extend_from_slice(&local_subrs_bytes);
        Ok(CffSubset {
            bytes: out,
            glyphs: keep,
        })
    }
}

/// The result of [`CffFont::subset`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CffSubset {
    /// The CID-keyed CFF program.
    pub bytes: Vec<u8>,
    /// Original GID of each subset glyph, in subset order (`glyphs[0] == 0`).
    /// Because the charset maps subset glyph `i` to CID `glyphs[i]`, a
    /// `Type0`/`Identity-H` font selects glyph `glyphs[i]` with the two-byte
    /// code `glyphs[i]`.
    pub glyphs: Vec<u16>,
}

fn build_index(items: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(items.len() as u16).to_be_bytes());
    if items.is_empty() {
        return out;
    }
    let total: usize = items.iter().map(|i| i.len()).sum();
    let end = total + 1;
    let off_size: u8 = if end < 0x100 {
        1
    } else if end < 0x1_0000 {
        2
    } else if end < 0x100_0000 {
        3
    } else {
        4
    };
    out.push(off_size);
    let mut off = 1usize;
    let push_off = |out: &mut Vec<u8>, v: usize| {
        for i in (0..off_size).rev() {
            out.push((v >> (8 * i as usize)) as u8);
        }
    };
    push_off(&mut out, off);
    for i in items {
        off += i.len();
        push_off(&mut out, off);
    }
    for i in items {
        out.extend_from_slice(i);
    }
    out
}

fn parse_charset(data: &[u8], entry: Option<&DictEntry>, glyph_count: usize) -> R<Vec<u16>> {
    let off = match entry {
        Some(e) => *dict_ints(&e.operands)?
            .first()
            .ok_or_else(|| CffError::Malformed("charset operand".into()))?,
        None => 0,
    };
    let mut charset = Vec::with_capacity(glyph_count);
    charset.push(0);
    match off {
        0 => {
            // ISOAdobe: SIDs 1..228 in order.
            for i in 1..glyph_count {
                charset.push(i as u16);
            }
        }
        1 | 2 => {
            return Err(CffError::Malformed(
                "Expert charsets are not supported".into(),
            ));
        }
        off if off > 0 => {
            let off = off as usize;
            let format = u8_at(data, off, "charset")?;
            let mut pos = off + 1;
            match format {
                0 => {
                    while charset.len() < glyph_count {
                        charset.push(u16_at(data, pos, "charset")?);
                        pos += 2;
                    }
                }
                1 | 2 => {
                    while charset.len() < glyph_count {
                        let first = u16_at(data, pos, "charset")?;
                        let n_left = if format == 1 {
                            let v = u8_at(data, pos + 2, "charset")? as usize;
                            pos += 3;
                            v
                        } else {
                            let v = u16_at(data, pos + 2, "charset")? as usize;
                            pos += 4;
                            v
                        };
                        for k in 0..=n_left {
                            if charset.len() >= glyph_count {
                                break;
                            }
                            charset.push(first.wrapping_add(k as u16));
                        }
                    }
                }
                f => return Err(CffError::Malformed(format!("charset format {f}"))),
            }
        }
        _ => return Err(CffError::Malformed("negative charset offset".into())),
    }
    Ok(charset)
}

/// Type 2 charstring walker that only tracks the operand stack depth and
/// the stem count, enough to find `endchar` with accent-composition operands.
struct SeacScan<'a> {
    font: &'a CffFont,
    stack: usize,
    stems: usize,
    width_parsed: bool,
    depth: usize,
    found: bool,
    /// The most recently pushed operand (a subroutine index when `callsubr`
    /// follows).
    last_value: Option<i32>,
}

impl SeacScan<'_> {
    fn push(&mut self, v: i32) {
        self.stack += 1;
        self.last_value = Some(v);
    }

    fn run(&mut self, cs: &[u8]) -> R<()> {
        if self.depth > 10 {
            return Err(CffError::Malformed(
                "subroutine nesting deeper than 10".into(),
            ));
        }
        let mut i = 0;
        while i < cs.len() {
            let b0 = cs[i];
            match b0 {
                32..=246 => {
                    self.push(b0 as i32 - 139);
                    i += 1;
                }
                247..=250 => {
                    let b1 = *cs.get(i + 1).ok_or(CffError::Truncated("charstring"))?;
                    self.push((b0 as i32 - 247) * 256 + b1 as i32 + 108);
                    i += 2;
                }
                251..=254 => {
                    let b1 = *cs.get(i + 1).ok_or(CffError::Truncated("charstring"))?;
                    self.push(-(b0 as i32 - 251) * 256 - b1 as i32 - 108);
                    i += 2;
                }
                28 => {
                    let w = cs
                        .get(i + 1..i + 3)
                        .ok_or(CffError::Truncated("charstring"))?;
                    self.push(i16::from_be_bytes([w[0], w[1]]) as i32);
                    i += 3;
                }
                255 => {
                    let w = cs
                        .get(i + 1..i + 5)
                        .ok_or(CffError::Truncated("charstring"))?;
                    // 16.16 fixed; only the integer part matters for an index.
                    self.push(i32::from_be_bytes([w[0], w[1], w[2], w[3]]) >> 16);
                    i += 5;
                }
                1 | 3 | 18 | 23 => {
                    // hstem vstem hstemhm vstemhm: pairs, odd leading arg is width
                    if self.stack % 2 == 1 && !self.width_parsed {
                        self.width_parsed = true;
                    }
                    self.stems += self.stack / 2;
                    self.stack = 0;
                    i += 1;
                }
                19 | 20 => {
                    // hintmask cntrmask: implicit vstem
                    if self.stack % 2 == 1 && !self.width_parsed {
                        self.width_parsed = true;
                    }
                    self.stems += self.stack / 2;
                    self.stack = 0;
                    i += 1 + self.stems.div_ceil(8);
                }
                21 => {
                    // rmoveto: 2 args (+ width)
                    if self.stack > 2 && !self.width_parsed {
                        self.width_parsed = true;
                    }
                    self.stack = 0;
                    i += 1;
                }
                4 | 22 => {
                    // vmoveto hmoveto: 1 arg (+ width)
                    if self.stack > 1 && !self.width_parsed {
                        self.width_parsed = true;
                    }
                    self.stack = 0;
                    i += 1;
                }
                14 => {
                    // endchar: 0 or 4 args, plus an optional width.
                    let args = if !self.width_parsed && (self.stack == 1 || self.stack == 5) {
                        self.width_parsed = true;
                        self.stack - 1
                    } else {
                        self.stack
                    };
                    if args == 4 {
                        self.found = true;
                    }
                    self.stack = 0;
                    return Ok(());
                }
                10 | 29 => {
                    // callsubr / callgsubr: pop index, follow.
                    if self.stack == 0 {
                        return Err(CffError::Malformed("callsubr with empty stack".into()));
                    }
                    self.stack -= 1;
                    let (idx_bytes, count) = if b0 == 10 {
                        let ls = self.font.fds[0]
                            .local_subrs
                            .as_ref()
                            .ok_or_else(|| CffError::Malformed("callsubr without Subrs".into()))?;
                        (ls, ls.items.len())
                    } else {
                        (&self.font.global_subrs, self.font.global_subrs.items.len())
                    };
                    let n = self
                        .last_value
                        .ok_or_else(|| CffError::Malformed("subroutine index operand".into()))?
                        + CffFont::bias(count);
                    self.last_value = None;
                    let (s, e) =
                        *idx_bytes
                            .items
                            .get(usize::try_from(n).map_err(|_| {
                                CffError::Malformed("negative subroutine index".into())
                            })?)
                            .ok_or_else(|| CffError::Malformed("subroutine index".into()))?;
                    let sub = self.font.data[s..e].to_vec();
                    self.depth += 1;
                    self.run(&sub)?;
                    self.depth -= 1;
                    if self.found {
                        return Ok(());
                    }
                    i += 1;
                }
                11 => return Ok(()), // return
                12 => {
                    // escaped: arithmetic/flex operators; all clear or reduce the stack.
                    let b1 = *cs.get(i + 1).ok_or(CffError::Truncated("charstring"))?;
                    match b1 {
                        34..=37 => self.stack = 0, // flex variants
                        3 | 4 | 5 | 9 | 10 | 11 | 12 | 14 | 15 | 18 | 21 | 22 | 23 | 24 | 26
                        | 27 | 28 | 29 | 30 => self.stack = self.stack.saturating_sub(1),
                        _ => self.stack = 0,
                    }
                    i += 2;
                }
                _ => {
                    // Path operators clear the stack.
                    self.stack = 0;
                    i += 1;
                }
            }
        }
        Ok(())
    }
}

/// The 391 standard strings (CFF specification Appendix A).
pub const STANDARD_STRING_NAMES: [&str; STANDARD_STRINGS] = [
    ".notdef",
    "space",
    "exclam",
    "quotedbl",
    "numbersign",
    "dollar",
    "percent",
    "ampersand",
    "quoteright",
    "parenleft",
    "parenright",
    "asterisk",
    "plus",
    "comma",
    "hyphen",
    "period",
    "slash",
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "colon",
    "semicolon",
    "less",
    "equal",
    "greater",
    "question",
    "at",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "bracketleft",
    "backslash",
    "bracketright",
    "asciicircum",
    "underscore",
    "quoteleft",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "braceleft",
    "bar",
    "braceright",
    "asciitilde",
    "exclamdown",
    "cent",
    "sterling",
    "fraction",
    "yen",
    "florin",
    "section",
    "currency",
    "quotesingle",
    "quotedblleft",
    "guillemotleft",
    "guilsinglleft",
    "guilsinglright",
    "fi",
    "fl",
    "endash",
    "dagger",
    "daggerdbl",
    "periodcentered",
    "paragraph",
    "bullet",
    "quotesinglbase",
    "quotedblbase",
    "quotedblright",
    "guillemotright",
    "ellipsis",
    "perthousand",
    "questiondown",
    "grave",
    "acute",
    "circumflex",
    "tilde",
    "macron",
    "breve",
    "dotaccent",
    "dieresis",
    "ring",
    "cedilla",
    "hungarumlaut",
    "ogonek",
    "caron",
    "emdash",
    "AE",
    "ordfeminine",
    "Lslash",
    "Oslash",
    "OE",
    "ordmasculine",
    "ae",
    "dotlessi",
    "lslash",
    "oslash",
    "oe",
    "germandbls",
    "onesuperior",
    "logicalnot",
    "mu",
    "trademark",
    "Eth",
    "onehalf",
    "plusminus",
    "Thorn",
    "onequarter",
    "divide",
    "brokenbar",
    "degree",
    "thorn",
    "threequarters",
    "twosuperior",
    "registered",
    "minus",
    "eth",
    "multiply",
    "threesuperior",
    "copyright",
    "Aacute",
    "Acircumflex",
    "Adieresis",
    "Agrave",
    "Aring",
    "Atilde",
    "Ccedilla",
    "Eacute",
    "Ecircumflex",
    "Edieresis",
    "Egrave",
    "Iacute",
    "Icircumflex",
    "Idieresis",
    "Igrave",
    "Ntilde",
    "Oacute",
    "Ocircumflex",
    "Odieresis",
    "Ograve",
    "Otilde",
    "Scaron",
    "Uacute",
    "Ucircumflex",
    "Udieresis",
    "Ugrave",
    "Yacute",
    "Ydieresis",
    "Zcaron",
    "aacute",
    "acircumflex",
    "adieresis",
    "agrave",
    "aring",
    "atilde",
    "ccedilla",
    "eacute",
    "ecircumflex",
    "edieresis",
    "egrave",
    "iacute",
    "icircumflex",
    "idieresis",
    "igrave",
    "ntilde",
    "oacute",
    "ocircumflex",
    "odieresis",
    "ograve",
    "otilde",
    "scaron",
    "uacute",
    "ucircumflex",
    "udieresis",
    "ugrave",
    "yacute",
    "ydieresis",
    "zcaron",
    "exclamsmall",
    "Hungarumlautsmall",
    "dollaroldstyle",
    "dollarsuperior",
    "ampersandsmall",
    "Acutesmall",
    "parenleftsuperior",
    "parenrightsuperior",
    "twodotenleader",
    "onedotenleader",
    "zerooldstyle",
    "oneoldstyle",
    "twooldstyle",
    "threeoldstyle",
    "fouroldstyle",
    "fiveoldstyle",
    "sixoldstyle",
    "sevenoldstyle",
    "eightoldstyle",
    "nineoldstyle",
    "commasuperior",
    "threequartersemdash",
    "periodsuperior",
    "questionsmall",
    "asuperior",
    "bsuperior",
    "centsuperior",
    "dsuperior",
    "esuperior",
    "isuperior",
    "lsuperior",
    "msuperior",
    "nsuperior",
    "osuperior",
    "rsuperior",
    "ssuperior",
    "tsuperior",
    "ff",
    "ffi",
    "ffl",
    "parenleftinferior",
    "parenrightinferior",
    "Circumflexsmall",
    "hyphensuperior",
    "Gravesmall",
    "Asmall",
    "Bsmall",
    "Csmall",
    "Dsmall",
    "Esmall",
    "Fsmall",
    "Gsmall",
    "Hsmall",
    "Ismall",
    "Jsmall",
    "Ksmall",
    "Lsmall",
    "Msmall",
    "Nsmall",
    "Osmall",
    "Psmall",
    "Qsmall",
    "Rsmall",
    "Ssmall",
    "Tsmall",
    "Usmall",
    "Vsmall",
    "Wsmall",
    "Xsmall",
    "Ysmall",
    "Zsmall",
    "colonmonetary",
    "onefitted",
    "rupiah",
    "Tildesmall",
    "exclamdownsmall",
    "centoldstyle",
    "Lslashsmall",
    "Scaronsmall",
    "Zcaronsmall",
    "Dieresissmall",
    "Brevesmall",
    "Caronsmall",
    "Dotaccentsmall",
    "Macronsmall",
    "figuredash",
    "hypheninferior",
    "Ogoneksmall",
    "Ringsmall",
    "Cedillasmall",
    "questiondownsmall",
    "oneeighth",
    "threeeighths",
    "fiveeighths",
    "seveneighths",
    "onethird",
    "twothirds",
    "zerosuperior",
    "foursuperior",
    "fivesuperior",
    "sixsuperior",
    "sevensuperior",
    "eightsuperior",
    "ninesuperior",
    "zeroinferior",
    "oneinferior",
    "twoinferior",
    "threeinferior",
    "fourinferior",
    "fiveinferior",
    "sixinferior",
    "seveninferior",
    "eightinferior",
    "nineinferior",
    "centinferior",
    "dollarinferior",
    "periodinferior",
    "commainferior",
    "Agravesmall",
    "Aacutesmall",
    "Acircumflexsmall",
    "Atildesmall",
    "Adieresissmall",
    "Aringsmall",
    "AEsmall",
    "Ccedillasmall",
    "Egravesmall",
    "Eacutesmall",
    "Ecircumflexsmall",
    "Edieresissmall",
    "Igravesmall",
    "Iacutesmall",
    "Icircumflexsmall",
    "Idieresissmall",
    "Ethsmall",
    "Ntildesmall",
    "Ogravesmall",
    "Oacutesmall",
    "Ocircumflexsmall",
    "Otildesmall",
    "Odieresissmall",
    "OEsmall",
    "Oslashsmall",
    "Ugravesmall",
    "Uacutesmall",
    "Ucircumflexsmall",
    "Udieresissmall",
    "Yacutesmall",
    "Thornsmall",
    "Ydieresissmall",
    "001.000",
    "001.001",
    "001.002",
    "001.003",
    "Black",
    "Bold",
    "Book",
    "Light",
    "Medium",
    "Regular",
    "Roman",
    "Semibold",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal non-CID CFF with `n` glyphs whose charstrings are the
    /// given bytes, a Private DICT with one local subroutine, and one global
    /// subroutine. Used to test the subsetter without a font file.
    pub(crate) fn synthetic(charstrings: &[&[u8]]) -> Vec<u8> {
        let name = build_index(&[b"SynthFont"]);
        // String INDEX: one custom string "glyphA" (SID 391).
        let strings = build_index(&[b"glyphA"]);
        // Global subr: "rlineto return" = 2 args? keep simple: `11` (return).
        let gsubrs = build_index(&[&[11u8]]);
        let lsubrs = build_index(&[&[11u8]]);
        let cs = build_index(charstrings);
        // charset format 0: glyph i -> SID (391 for glyph 1, i for others)
        let mut charset = vec![0u8];
        for i in 1..charstrings.len() {
            let sid: u16 = if i == 1 { 391 } else { i as u16 };
            charset.extend_from_slice(&sid.to_be_bytes());
        }
        // Private DICT: BlueValues (op 6) with two operands, then Subrs (19).
        let mut private = vec![139u8, 139, 6];
        let private_len = private.len() + 5 + 1;
        private.extend_from_slice(&int5(private_len as i32));
        private.push(19);
        assert_eq!(private.len(), private_len);
        let build_top = |charset_off: i32, cs_off: i32, priv_off: i32| {
            let mut t = Vec::new();
            // FontBBox (5): four operands
            t.extend_from_slice(&[139, 139, 139 + 10, 139 + 10, 5]);
            t.extend_from_slice(&int5(charset_off));
            t.push(OP_CHARSET as u8);
            t.extend_from_slice(&int5(cs_off));
            t.push(OP_CHARSTRINGS as u8);
            t.extend_from_slice(&int5(private.len() as i32));
            t.extend_from_slice(&int5(priv_off));
            t.push(OP_PRIVATE as u8);
            build_index(&[&t])
        };
        let header = [1u8, 0, 4, 4];
        let top_len = build_top(0, 0, 0).len();
        let charset_off = header.len() + name.len() + top_len + strings.len() + gsubrs.len();
        let cs_off = charset_off + charset.len();
        let priv_off = cs_off + cs.len();
        let mut out = Vec::new();
        out.extend_from_slice(&header);
        out.extend_from_slice(&name);
        out.extend_from_slice(&build_top(
            charset_off as i32,
            cs_off as i32,
            priv_off as i32,
        ));
        out.extend_from_slice(&strings);
        out.extend_from_slice(&gsubrs);
        out.extend_from_slice(&charset);
        out.extend_from_slice(&cs);
        out.extend_from_slice(&private);
        out.extend_from_slice(&lsubrs);
        out
    }

    // Type 2 charstrings: `<w> hmoveto ... endchar` style; endchar = 14.
    const CS_NOTDEF: &[u8] = &[14];
    // 100 200 rmoveto 50 hlineto endchar
    const CS_BOX: &[u8] = &[139 + 100, 247, 0, 21, 139 + 50, 6, 14];
    // seac: adx ady bchar achar endchar (4 operands before endchar)
    const CS_SEAC: &[u8] = &[139 + 10, 139 + 20, 139 + 65, 139 + 66, 14];
    // callsubr with index -107 (bias 107 -> subr 0) then endchar
    const CS_SUBR: &[u8] = &[139 - 107, 10, 14];

    #[test]
    fn parses_synthetic_font() {
        let data = synthetic(&[CS_NOTDEF, CS_BOX, CS_SEAC, CS_SUBR]);
        let font = CffFont::parse(&data).unwrap();
        assert_eq!(font.glyph_count(), 4);
        assert_eq!(font.name(), "SynthFont");
        assert!(!font.is_cid_keyed());
        assert_eq!(font.glyph_name(0).as_deref(), Some(".notdef"));
        assert_eq!(font.glyph_name(1).as_deref(), Some("glyphA"));
        assert_eq!(font.glyph_name(2).as_deref(), Some("exclam"));
        assert_eq!(font.gid_by_name("glyphA"), Some(1));
        assert_eq!(font.charstring(1), Some(CS_BOX));
        assert!(!font.uses_seac(1).unwrap());
        assert!(font.uses_seac(2).unwrap());
        assert!(!font.uses_seac(3).unwrap(), "subroutine is followed");
    }

    #[test]
    fn subset_preserves_gids_as_cids_and_charstring_bytes() {
        let data = synthetic(&[CS_NOTDEF, CS_BOX, CS_SEAC, CS_SUBR]);
        let font = CffFont::parse(&data).unwrap();
        let sub = font.subset(&BTreeSet::from([3u16])).unwrap();
        assert_eq!(sub.glyphs, vec![0, 3]);
        let parsed = CffFont::parse(&sub.bytes).unwrap();
        assert!(parsed.is_cid_keyed());
        assert_eq!(parsed.glyph_count(), 2);
        assert_eq!(
            parsed.charset_entry(1),
            Some(3),
            "CID of subset glyph 1 is original GID 3"
        );
        assert_eq!(parsed.charstring(0), Some(CS_NOTDEF));
        assert_eq!(parsed.charstring(1), Some(CS_SUBR));
        assert_eq!(parsed.name(), "SynthFont");
        // Local subrs survived with the same content and the FD's Private DICT resolves.
        assert_eq!(parsed.fds.len(), 1);
        assert!(parsed.fds[0].private.is_some());
        assert_eq!(
            parsed.fds[0].local_subrs.as_ref().map(|i| i.items.len()),
            Some(1)
        );
        assert_eq!(parsed.fd_count(), 1);
        // Same input twice is byte-identical.
        assert_eq!(sub, font.subset(&BTreeSet::from([3u16])).unwrap());
        // The seac glyph is refused, not silently broken.
        assert_eq!(
            font.subset(&BTreeSet::from([2u16])),
            Err(CffError::Seac { gid: 2 })
        );
        assert_eq!(
            font.subset(&BTreeSet::from([9u16])),
            Err(CffError::GlyphOutOfRange {
                gid: 9,
                glyph_count: 4
            })
        );
    }

    #[test]
    fn cid_keyed_subset_output_is_not_subset_again() {
        let data = synthetic(&[CS_NOTDEF, CS_BOX]);
        let font = CffFont::parse(&data).unwrap();
        let sub = font.subset(&BTreeSet::from([1u16])).unwrap();
        let again = CffFont::parse(&sub.bytes).unwrap();
        assert_eq!(
            again.subset(&BTreeSet::new()),
            Err(CffError::CidKeyedSource)
        );
    }

    #[test]
    fn dict_round_trip() {
        let d = [139u8, 247, 0, 28, 1, 2, 29, 0, 0, 1, 0, 5, 12, 7];
        let e = parse_dict(&d).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].op, 5);
        assert_eq!(dict_ints(&e[0].operands).unwrap(), vec![0, 108, 258, 256]);
        assert_eq!(e[1].op, 1207);
    }
}

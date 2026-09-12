//! A bounded PDF object reader for comparison tooling.
//!
//! Reads the objects of a finished PDF (this crate's own output or a
//! pdfTeX reference) into a small object model: numbers are kept as their
//! verbatim tokens, streams are decoded (`FlateDecode` via `crate::inflate`
//! or no filter), object streams and cross-reference streams are expanded.
//! Objects are found by scanning for `N G obj` headers rather than trusting
//! the cross-reference table, which keeps the reader independent of the
//! writer under test. This is not a general-purpose PDF parser: encryption,
//! incremental updates that shadow objects, and filters other than Flate
//! are reported, not handled.

use crate::inflate;
use std::collections::{BTreeMap, BTreeSet};

/// Largest decoded stream accepted.
pub const MAX_STREAM_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Obj {
    Null,
    Bool(bool),
    /// Verbatim numeric token.
    Number(String),
    String(Vec<u8>),
    Name(String),
    Array(Vec<Obj>),
    Dict(BTreeMap<String, Obj>),
    Ref(u32, u16),
    Stream {
        dict: BTreeMap<String, Obj>,
        /// Raw (still encoded) bytes.
        raw: Vec<u8>,
    },
}

impl Obj {
    pub fn as_dict(&self) -> Option<&BTreeMap<String, Obj>> {
        match self {
            Obj::Dict(d) | Obj::Stream { dict: d, .. } => Some(d),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            Obj::Name(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<&str> {
        match self {
            Obj::Number(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_number()?.parse().ok()
    }

    pub fn as_array(&self) -> Option<&[Obj]> {
        match self {
            Obj::Array(a) => Some(a),
            _ => None,
        }
    }
}

/// Structural facts about how the file was written.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Features {
    pub version: String,
    pub xref_stream: bool,
    pub object_streams: usize,
    pub filters: BTreeSet<String>,
    pub has_id: bool,
    pub object_count: usize,
    /// Object numbers in file order (top-level objects only).
    pub object_order: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct PdfFile {
    pub objects: BTreeMap<u32, Obj>,
    pub trailer: BTreeMap<String, Obj>,
    pub features: Features,
}

struct Lexer<'a> {
    b: &'a [u8],
    pos: usize,
}

fn is_ws(c: u8) -> bool {
    matches!(c, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\0')
}

fn is_delim(c: u8) -> bool {
    matches!(
        c,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

impl<'a> Lexer<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.b.len() {
            let c = self.b[self.pos];
            if is_ws(c) {
                self.pos += 1;
            } else if c == b'%' {
                while self.pos < self.b.len()
                    && self.b[self.pos] != b'\n'
                    && self.b[self.pos] != b'\r'
                {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn word(&mut self) -> &'a [u8] {
        let start = self.pos;
        while self.pos < self.b.len() && !is_ws(self.b[self.pos]) && !is_delim(self.b[self.pos]) {
            self.pos += 1;
        }
        &self.b[start..self.pos]
    }

    fn peek_keyword(&mut self, kw: &[u8]) -> bool {
        self.skip_ws();
        self.b[self.pos..].starts_with(kw)
            && self
                .b
                .get(self.pos + kw.len())
                .is_none_or(|&c| is_ws(c) || is_delim(c))
    }

    /// Parses one object. Indirect references `n g R` are recognised by
    /// lookahead after an integer.
    fn object(&mut self, depth: usize) -> Result<Obj, String> {
        if depth > 64 {
            return Err("object nesting deeper than 64".into());
        }
        self.skip_ws();
        let c = *self.b.get(self.pos).ok_or("unexpected end of file")?;
        match c {
            b'/' => {
                self.pos += 1;
                let w = self.word();
                Ok(Obj::Name(decode_name(w)))
            }
            b'(' => {
                let (s, next) = literal(self.b, self.pos)?;
                self.pos = next;
                Ok(Obj::String(s))
            }
            b'<' => {
                if self.b.get(self.pos + 1) == Some(&b'<') {
                    self.pos += 2;
                    let mut d = BTreeMap::new();
                    loop {
                        self.skip_ws();
                        if self.b[self.pos..].starts_with(b">>") {
                            self.pos += 2;
                            break;
                        }
                        if self.b.get(self.pos) != Some(&b'/') {
                            return Err(format!("dictionary key expected at byte {}", self.pos));
                        }
                        self.pos += 1;
                        let key = decode_name(self.word());
                        let value = self.object(depth + 1)?;
                        d.insert(key, value);
                    }
                    Ok(Obj::Dict(d))
                } else {
                    let end = self.b[self.pos..]
                        .iter()
                        .position(|&x| x == b'>')
                        .ok_or("unterminated hex string")?;
                    let hex: Vec<u8> = self.b[self.pos + 1..self.pos + end]
                        .iter()
                        .copied()
                        .filter(|x| !is_ws(*x))
                        .collect();
                    let mut out = Vec::new();
                    for pair in hex.chunks(2) {
                        let hi = hexv(pair[0])?;
                        let lo = pair.get(1).map_or(Ok(0), |&x| hexv(x))?;
                        out.push(hi * 16 + lo);
                    }
                    self.pos += end + 1;
                    Ok(Obj::String(out))
                }
            }
            b'[' => {
                self.pos += 1;
                let mut a = Vec::new();
                loop {
                    self.skip_ws();
                    if self.b.get(self.pos) == Some(&b']') {
                        self.pos += 1;
                        break;
                    }
                    a.push(self.object(depth + 1)?);
                }
                Ok(Obj::Array(a))
            }
            b']' | b'>' | b')' | b'{' | b'}' => Err(format!(
                "unexpected delimiter {:?} at byte {}",
                c as char, self.pos
            )),
            _ => {
                let w = self.word();
                if w.is_empty() {
                    return Err(format!("empty token at byte {}", self.pos));
                }
                let s = std::str::from_utf8(w).map_err(|_| "non-ASCII token")?;
                match s {
                    "true" => return Ok(Obj::Bool(true)),
                    "false" => return Ok(Obj::Bool(false)),
                    "null" => return Ok(Obj::Null),
                    _ => {}
                }
                if s.starts_with(|ch: char| {
                    ch.is_ascii_digit() || ch == '-' || ch == '+' || ch == '.'
                }) {
                    // Lookahead for `gen R`.
                    if s.bytes().all(|x| x.is_ascii_digit()) {
                        let save = self.pos;
                        self.skip_ws();
                        let w2 = self.word();
                        if !w2.is_empty() && w2.iter().all(|x| x.is_ascii_digit()) {
                            self.skip_ws();
                            if self.b.get(self.pos) == Some(&b'R')
                                && self
                                    .b
                                    .get(self.pos + 1)
                                    .is_none_or(|&x| is_ws(x) || is_delim(x))
                            {
                                self.pos += 1;
                                let num: u32 = s.parse().map_err(|_| "object number")?;
                                let generation: u16 = std::str::from_utf8(w2)
                                    .unwrap_or("0")
                                    .parse()
                                    .map_err(|_| "generation")?;
                                return Ok(Obj::Ref(num, generation));
                            }
                        }
                        self.pos = save;
                    }
                    return Ok(Obj::Number(s.to_string()));
                }
                Err(format!("unexpected keyword {s:?} at byte {}", self.pos))
            }
        }
    }
}

fn hexv(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("bad hex digit {:?}", c as char)),
    }
}

fn decode_name(raw: &[u8]) -> String {
    let mut s = String::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'#'
            && raw.len() >= i + 3
            && let (Ok(h), Ok(l)) = (hexv(raw[i + 1]), hexv(raw[i + 2]))
        {
            s.push((h * 16 + l) as char);
            i += 3;
            continue;
        }
        s.push(raw[i] as char);
        i += 1;
    }
    s
}

fn literal(b: &[u8], start: usize) -> Result<(Vec<u8>, usize), String> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut i = start + 1;
    while i < b.len() {
        match b[i] {
            b'\\' => {
                i += 1;
                let e = *b.get(i).ok_or("unterminated escape")?;
                match e {
                    b'n' => out.push(b'\n'),
                    b'r' => out.push(b'\r'),
                    b't' => out.push(b'\t'),
                    b'b' => out.push(8),
                    b'f' => out.push(12),
                    b'0'..=b'7' => {
                        let mut v = 0u32;
                        let mut n = 0;
                        while n < 3 && i < b.len() && (b'0'..=b'7').contains(&b[i]) {
                            v = v * 8 + (b[i] - b'0') as u32;
                            i += 1;
                            n += 1;
                        }
                        out.push((v & 0xFF) as u8);
                        continue;
                    }
                    b'\n' => {}
                    b'\r' => {
                        if b.get(i + 1) == Some(&b'\n') {
                            i += 1;
                        }
                    }
                    other => out.push(other),
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                out.push(b'(');
                i += 1;
            }
            b')' => {
                if depth == 0 {
                    return Ok((out, i + 1));
                }
                depth -= 1;
                out.push(b')');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    Err("unterminated literal string".into())
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if from >= hay.len() {
        return None;
    }
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}

impl PdfFile {
    pub fn parse(bytes: &[u8]) -> Result<PdfFile, String> {
        let version = if bytes.starts_with(b"%PDF-") {
            String::from_utf8_lossy(&bytes[5..bytes.len().min(8)]).to_string()
        } else {
            return Err("missing %PDF header".into());
        };
        let mut objects = BTreeMap::new();
        let mut order = Vec::new();
        let mut features = Features {
            version,
            ..Default::default()
        };
        // Scan for "obj" headers.
        let mut pos = 0;
        while let Some(at) = find(bytes, b"obj", pos) {
            pos = at + 3;
            // Must be followed by a delimiter/whitespace and preceded by "num gen ".
            if bytes
                .get(at + 3)
                .is_some_and(|&c| !is_ws(c) && !is_delim(c))
            {
                continue;
            }
            let Some((num, _generation, _header_start)) = header_before(bytes, at) else {
                continue;
            };
            let mut lx = Lexer {
                b: bytes,
                pos: at + 3,
            };
            let obj = match lx.object(0) {
                Ok(o) => o,
                Err(e) => return Err(format!("object {num}: {e}")),
            };
            let obj = if lx.peek_keyword(b"stream") {
                let dict = match obj {
                    Obj::Dict(d) => d,
                    _ => return Err(format!("object {num}: stream without dictionary")),
                };
                lx.pos += "stream".len();
                // EOL after `stream`: CRLF or LF.
                if bytes.get(lx.pos) == Some(&b'\r') {
                    lx.pos += 1;
                }
                if bytes.get(lx.pos) == Some(&b'\n') {
                    lx.pos += 1;
                }
                let data_start = lx.pos;
                let length = match dict.get("Length") {
                    Some(Obj::Number(n)) => n.parse::<usize>().ok(),
                    _ => None,
                };
                let data_end = match length {
                    Some(l)
                        if data_start + l <= bytes.len()
                            && find(bytes, b"endstream", data_start + l)
                                .is_some_and(|e| e <= data_start + l + 2) =>
                    {
                        data_start + l
                    }
                    _ => {
                        // Indirect or wrong /Length: search for endstream.
                        let e = find(bytes, b"endstream", data_start)
                            .ok_or_else(|| format!("object {num}: no endstream"))?;
                        let mut end = e;
                        if end > data_start && bytes[end - 1] == b'\n' {
                            end -= 1;
                        }
                        if end > data_start && bytes[end - 1] == b'\r' {
                            end -= 1;
                        }
                        end
                    }
                };
                pos = data_end;
                if let Some(Obj::Name(f)) = dict.get("Filter") {
                    features.filters.insert(f.clone());
                }
                if let Some(Obj::Array(fs)) = dict.get("Filter") {
                    for f in fs {
                        if let Obj::Name(n) = f {
                            features.filters.insert(n.clone());
                        }
                    }
                }
                Obj::Stream {
                    dict,
                    raw: bytes[data_start..data_end].to_vec(),
                }
            } else {
                obj
            };
            order.push(num);
            objects.insert(num, obj);
        }
        // Trailer: classic keyword, else the cross-reference stream's dictionary.
        let mut trailer = BTreeMap::new();
        let mut tpos = 0;
        while let Some(t) = find(bytes, b"trailer", tpos) {
            tpos = t + 7;
            let mut lx = Lexer {
                b: bytes,
                pos: t + 7,
            };
            if let Ok(Obj::Dict(d)) = lx.object(0) {
                trailer = d;
            }
        }
        for obj in objects.values() {
            if let Obj::Stream { dict, .. } = obj
                && dict.get("Type").and_then(Obj::as_name) == Some("XRef")
            {
                features.xref_stream = true;
                if trailer.is_empty() {
                    trailer = dict.clone();
                }
            }
        }
        features.has_id = trailer.contains_key("ID");
        // Expand object streams.
        let mut file = PdfFile {
            objects,
            trailer,
            features,
        };
        let objstms: Vec<u32> = file
            .objects
            .iter()
            .filter(|(_, o)| {
                o.as_dict()
                    .and_then(|d| d.get("Type"))
                    .and_then(Obj::as_name)
                    == Some("ObjStm")
            })
            .map(|(n, _)| *n)
            .collect();
        for n in objstms {
            file.features.object_streams += 1;
            let obj = file.objects[&n].clone();
            let data = file.decode_stream(&obj)?;
            let dict = obj.as_dict().expect("stream");
            let count = dict.get("N").and_then(Obj::as_usize).ok_or("ObjStm /N")?;
            let first = dict
                .get("First")
                .and_then(Obj::as_usize)
                .ok_or("ObjStm /First")?;
            let header = std::str::from_utf8(data.get(..first).ok_or("ObjStm header")?)
                .map_err(|_| "ObjStm header")?;
            let nums: Vec<usize> = header
                .split_ascii_whitespace()
                .map(|x| x.parse::<usize>())
                .collect::<Result<_, _>>()
                .map_err(|_| "ObjStm header numbers")?;
            if nums.len() < 2 * count {
                return Err("ObjStm header too short".into());
            }
            for i in 0..count {
                let onum = nums[2 * i] as u32;
                let off = first + nums[2 * i + 1];
                let mut lx = Lexer { b: &data, pos: off };
                let o = lx
                    .object(0)
                    .map_err(|e| format!("ObjStm object {onum}: {e}"))?;
                file.objects.entry(onum).or_insert(o);
            }
        }
        file.features.object_count = file.objects.len();
        file.features.object_order = order;
        Ok(file)
    }

    /// Follows references.
    pub fn resolve<'a>(&'a self, o: &'a Obj) -> &'a Obj {
        let mut cur = o;
        for _ in 0..32 {
            match cur {
                Obj::Ref(n, _) => cur = self.objects.get(n).unwrap_or(&Obj::Null),
                other => return other,
            }
        }
        &Obj::Null
    }

    /// A dictionary entry, dereferenced.
    pub fn get<'a>(&'a self, dict: &'a BTreeMap<String, Obj>, key: &str) -> Option<&'a Obj> {
        dict.get(key).map(|o| self.resolve(o))
    }

    /// Decoded bytes of a stream object (no filter or `FlateDecode`).
    pub fn decode_stream(&self, obj: &Obj) -> Result<Vec<u8>, String> {
        let Obj::Stream { dict, raw } = self.resolve(obj) else {
            return Err("not a stream".into());
        };
        let filters: Vec<String> = match self.get(dict, "Filter") {
            None => vec![],
            Some(Obj::Name(n)) => vec![n.clone()],
            Some(Obj::Array(a)) => a
                .iter()
                .filter_map(|f| f.as_name().map(String::from))
                .collect(),
            Some(_) => return Err("unsupported /Filter value".into()),
        };
        if filters.iter().any(|f| f == "FlateDecode")
            && let Some(Obj::Dict(p)) = self.get(dict, "DecodeParms")
            && p.get("Predictor")
                .and_then(Obj::as_usize)
                .is_some_and(|v| v > 1)
        {
            return Err("FlateDecode with a predictor is not supported".into());
        }
        let mut data = raw.clone();
        for f in filters {
            data = match f.as_str() {
                "FlateDecode" => {
                    inflate::inflate_zlib(&data, MAX_STREAM_BYTES).map_err(|e| e.to_string())?
                }
                other => return Err(format!("filter {other} is not supported")),
            };
        }
        Ok(data)
    }

    pub fn catalog(&self) -> Result<&BTreeMap<String, Obj>, String> {
        if let Some(root) = self.trailer.get("Root")
            && let Some(d) = self.resolve(root).as_dict()
        {
            return Ok(d);
        }
        self.objects
            .values()
            .find_map(|o| {
                o.as_dict()
                    .filter(|d| d.get("Type").and_then(Obj::as_name) == Some("Catalog"))
            })
            .ok_or_else(|| "no catalog".into())
    }

    pub fn info(&self) -> Option<&BTreeMap<String, Obj>> {
        self.trailer
            .get("Info")
            .and_then(|i| self.resolve(i).as_dict())
    }

    /// Page dictionaries in document order.
    pub fn pages(&self) -> Result<Vec<&BTreeMap<String, Obj>>, String> {
        let cat = self.catalog()?;
        let root = self
            .get(cat, "Pages")
            .and_then(Obj::as_dict)
            .ok_or("no /Pages")?;
        let mut out = Vec::new();
        self.walk_pages(root, &mut out, 0)?;
        Ok(out)
    }

    fn walk_pages<'a>(
        &'a self,
        node: &'a BTreeMap<String, Obj>,
        out: &mut Vec<&'a BTreeMap<String, Obj>>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 32 {
            return Err("page tree deeper than 32".into());
        }
        match node.get("Type").and_then(Obj::as_name) {
            Some("Page") => out.push(node),
            _ => {
                let kids = self
                    .get(node, "Kids")
                    .and_then(Obj::as_array)
                    .ok_or("page node without /Kids")?;
                for k in kids {
                    let kd = self.resolve(k).as_dict().ok_or("kid is not a dictionary")?;
                    self.walk_pages(kd, out, depth + 1)?;
                }
            }
        }
        Ok(())
    }

    /// The decoded, concatenated content of a page.
    pub fn page_content(&self, page: &BTreeMap<String, Obj>) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        match self.get(page, "Contents") {
            None => {}
            Some(Obj::Array(parts)) => {
                for (i, p) in parts.iter().enumerate() {
                    if i > 0 {
                        out.push(b'\n');
                    }
                    out.extend(self.decode_stream(p)?);
                }
            }
            Some(s) => out.extend(self.decode_stream(s)?),
        }
        Ok(out)
    }

    /// A page's inherited attribute (`MediaBox`, `Resources`).
    pub fn page_attr<'a>(&'a self, page: &'a BTreeMap<String, Obj>, key: &str) -> Option<&'a Obj> {
        let mut node = page;
        for _ in 0..32 {
            if let Some(v) = self.get(node, key) {
                return Some(v);
            }
            node = self.get(node, "Parent")?.as_dict()?;
        }
        None
    }

    /// Font resources of a page: resource name to font dictionary.
    pub fn page_fonts<'a>(
        &'a self,
        page: &'a BTreeMap<String, Obj>,
    ) -> BTreeMap<String, &'a BTreeMap<String, Obj>> {
        let mut out = BTreeMap::new();
        if let Some(res) = self.page_attr(page, "Resources").and_then(Obj::as_dict)
            && let Some(fonts) = self.get(res, "Font").and_then(Obj::as_dict)
        {
            for (name, f) in fonts {
                if let Some(d) = self.resolve(f).as_dict() {
                    out.insert(name.clone(), d);
                }
            }
        }
        out
    }
}

/// Finds `num gen ` immediately before an `obj` keyword at `at`.
fn header_before(b: &[u8], at: usize) -> Option<(u32, u16, usize)> {
    let mut i = at;
    if i == 0 || !is_ws(b[i - 1]) {
        return None;
    }
    while i > 0 && is_ws(b[i - 1]) {
        i -= 1;
    }
    let gen_end = i;
    while i > 0 && b[i - 1].is_ascii_digit() {
        i -= 1;
    }
    let gen_start = i;
    if gen_start == gen_end || i == 0 || !is_ws(b[i - 1]) {
        return None;
    }
    while i > 0 && is_ws(b[i - 1]) {
        i -= 1;
    }
    let num_end = i;
    while i > 0 && b[i - 1].is_ascii_digit() {
        i -= 1;
    }
    let num_start = i;
    if num_start == num_end || (i > 0 && !is_ws(b[i - 1]) && !is_delim(b[i - 1])) {
        return None;
    }
    let num = std::str::from_utf8(&b[num_start..num_end])
        .ok()?
        .parse()
        .ok()?;
    let generation = std::str::from_utf8(&b[gen_start..gen_end])
        .ok()?
        .parse()
        .ok()?;
    Some((num, generation, num_start))
}

/// Renders an object back to compact PDF syntax (for reports and tests).
pub fn render(o: &Obj) -> String {
    match o {
        Obj::Null => "null".into(),
        Obj::Bool(b) => b.to_string(),
        Obj::Number(n) => n.clone(),
        Obj::String(s) => {
            let mut out = String::from("(");
            for &b in s {
                match b {
                    b'(' | b')' | b'\\' => {
                        out.push('\\');
                        out.push(b as char);
                    }
                    32..=126 => out.push(b as char),
                    _ => out.push_str(&format!("\\{b:03o}")),
                }
            }
            out.push(')');
            out
        }
        Obj::Name(n) => format!("/{n}"),
        Obj::Array(a) => format!("[{}]", a.iter().map(render).collect::<Vec<_>>().join(" ")),
        Obj::Dict(d) => format!(
            "<<{}>>",
            d.iter()
                .map(|(k, v)| format!("/{k} {}", render(v)))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        Obj::Ref(n, g) => format!("{n} {g} R"),
        Obj::Stream { dict, raw } => format!(
            "{} stream[{} bytes]",
            render(&Obj::Dict(dict.clone())),
            raw.len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_objects_streams_and_refs() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R /K [1 2.5 -3 /N (a\\)b) <414243>] >>\nendobj\n2 0 obj\n<< /Length 5 >>\nstream\nhello\nendstream\nendobj\ntrailer\n<< /Root 1 0 R /Size 3 >>\n%%EOF\n";
        let f = PdfFile::parse(pdf).unwrap();
        assert_eq!(f.objects.len(), 2);
        let cat = f.catalog().unwrap();
        assert_eq!(cat.get("Pages"), Some(&Obj::Ref(2, 0)));
        assert_eq!(render(cat.get("K").unwrap()), "[1 2.5 -3 /N (a\\)b) (ABC)]");
        assert_eq!(f.decode_stream(&Obj::Ref(2, 0)).unwrap(), b"hello");
        assert!(!f.features.xref_stream);
        assert_eq!(f.features.object_order, vec![1, 2]);
    }

    #[test]
    fn number_followed_by_non_ref_is_a_number() {
        let mut lx = Lexer {
            b: b"[1 2 3]",
            pos: 0,
        };
        assert_eq!(
            lx.object(0).unwrap(),
            Obj::Array(vec![
                Obj::Number("1".into()),
                Obj::Number("2".into()),
                Obj::Number("3".into())
            ])
        );
        let mut lx = Lexer {
            b: b"[1 2 R 3]",
            pos: 0,
        };
        assert_eq!(
            lx.object(0).unwrap(),
            Obj::Array(vec![Obj::Ref(1, 2), Obj::Number("3".into())])
        );
    }
}

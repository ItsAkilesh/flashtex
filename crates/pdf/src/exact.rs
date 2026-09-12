//! Exact export: glyph runs by original glyph id, typed page operators, and
//! content streams whose decimal operands are written **verbatim**.
//!
//! This is the additive API GitHub issue #25 asks for. The runtime-v1 route
//! (`crate::writer`) selects fonts by character and formats numbers to three
//! decimals; this route does neither:
//!
//! - Every number is a [`Decimal`]: a validated PDF numeric token carried as
//!   text. `12.34500` stays `12.34500`; nothing is parsed into `f64` and
//!   re-formatted. Exponents are rejected because PDF has no exponent syntax.
//! - Text is shown by **codes**, not characters. A [`ExactFont::CidCff`] or
//!   [`ExactFont::CidTrueType`] font takes two-byte codes that *are* the
//!   source font's glyph ids (`Identity-H`; the CFF program is rewritten so
//!   CID = original GID, see `crate::cff`; the TrueType program keeps its
//!   glyph order with `/CIDToGIDMap /Identity`). An [`ExactFont::Simple`]
//!   font takes one-byte codes with an explicit `/Differences` encoding, the
//!   form pdfTeX writes for Type 1 fonts.
//! - Rules and outlines are typed path operators ([`Op`]), including
//!   `q`/`Q`, clipping, colour, `m`/`l`/`c`/`h`, `re`, fill and stroke.
//! - A page's content is either a typed operator list, serialised one
//!   operator per line, or [`Content::Verbatim`] bytes that are validated by
//!   parsing into the same bounded operator set and then inserted unchanged.
//!
//! Pages are white: the writer paints no background and takes no theme
//! input. Coordinates are PDF user space (origin bottom-left, y up); the
//! caller does any flip before handing operands over, so no arithmetic
//! happens here. Alpha, shading, images and inline images are outside the
//! bounded operator set and are reported as errors, never dropped.
//!
//! Output is deterministic: the same [`ExactDocument`] serialises to the
//! same bytes (no timestamps, no `/ID`), which the tests check.

use crate::cff::{CffError, CffFont};
use crate::sha256;
use crate::truetype::{Outlines, TrueTypeFont};
use crate::writer::Document;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// Longest numeric token accepted (digits plus sign and point).
pub const MAX_DECIMAL_LEN: usize = 64;
/// Largest content stream accepted, in bytes.
pub const MAX_CONTENT_BYTES: usize = 64 * 1024 * 1024;
/// Most operators accepted per page.
pub const MAX_OPERATORS: usize = 4_000_000;
/// Largest font program accepted, in bytes.
pub const MAX_FONT_BYTES: usize = 64 * 1024 * 1024;
pub const PRODUCER: &str = "FlashTeX flashtex-pdf 0.1.0 exact";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExactError {
    /// A numeric token is not PDF real/integer syntax.
    Decimal(String),
    /// The document has no pages, a page has a non-positive size, etc.
    Invalid(String),
    /// A content stream did not validate: message names the operator index.
    Content {
        page: usize,
        op: usize,
        message: String,
    },
    /// A font program could not be prepared.
    Font {
        resource: String,
        message: String,
    },
    Limit(&'static str),
}

impl std::fmt::Display for ExactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExactError::Decimal(m) => write!(f, "invalid decimal operand: {m}"),
            ExactError::Invalid(m) => write!(f, "invalid exact document: {m}"),
            ExactError::Content { page, op, message } => {
                write!(f, "page {page} operator {op}: {message}")
            }
            ExactError::Font { resource, message } => write!(f, "font /{resource}: {message}"),
            ExactError::Limit(w) => write!(f, "limit exceeded: {w}"),
        }
    }
}

impl std::error::Error for ExactError {}

impl From<CffError> for ExactError {
    fn from(e: CffError) -> Self {
        ExactError::Font {
            resource: String::new(),
            message: e.to_string(),
        }
    }
}

/// A PDF numeric token, validated once and written verbatim.
///
/// Accepted syntax (PDF 32000-1 §7.3.3): optional sign, digits, optional
/// point, digits; at least one digit overall; no exponent, no whitespace.
/// `-0`, `007`, `1.`, `.5` and `3.1400` are all valid and preserved as is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal(String);

impl Decimal {
    pub fn new(token: &str) -> Result<Decimal, ExactError> {
        if token.is_empty() || token.len() > MAX_DECIMAL_LEN {
            return Err(ExactError::Decimal(format!(
                "{token:?} is empty or longer than {MAX_DECIMAL_LEN} characters"
            )));
        }
        let body = token.strip_prefix(['+', '-']).unwrap_or(token);
        let (int, frac) = match body.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (body, None),
        };
        let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
        if !all_digits(int) || !frac.is_none_or(all_digits) {
            return Err(ExactError::Decimal(format!(
                "{token:?} is not a PDF number (digits with an optional sign and point)"
            )));
        }
        if int.is_empty() && frac.is_none_or(str::is_empty) {
            return Err(ExactError::Decimal(format!("{token:?} has no digits")));
        }
        Ok(Decimal(token.to_string()))
    }

    pub fn from_i64(v: i64) -> Decimal {
        Decimal(v.to_string())
    }

    /// The exact quotient `n / d` as a terminating decimal, or `None` if it
    /// does not terminate within `max_frac_digits`.
    pub fn from_ratio(n: i128, d: u128, max_frac_digits: usize) -> Option<Decimal> {
        if d == 0 {
            return None;
        }
        let neg = n < 0;
        let mut int = n.unsigned_abs() / d;
        let mut rem = n.unsigned_abs() % d;
        let mut frac = String::new();
        while rem != 0 {
            if frac.len() >= max_frac_digits {
                return None;
            }
            rem = rem.checked_mul(10)?;
            let digit = rem / d;
            rem %= d;
            frac.push(char::from(b'0' + digit as u8));
        }
        if neg && (int != 0 || !frac.is_empty()) {
            // Keep "-0.5" but never "-0".
            let mut s = String::from("-");
            s.push_str(&int.to_string());
            if !frac.is_empty() {
                s.push('.');
                s.push_str(&frac);
            }
            return Some(Decimal(s));
        }
        let mut s = String::new();
        if int == 0 {
            int = 0;
        }
        s.push_str(&int.to_string());
        if !frac.is_empty() {
            s.push('.');
            s.push_str(&frac);
        }
        Some(Decimal(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Numeric value for checks that need one (sign tests, comparisons).
    /// Never used for output.
    pub fn approx(&self) -> f64 {
        self.0.parse().unwrap_or(f64::NAN)
    }
}

impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One element of a `TJ` array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TjElement {
    Text(Vec<u8>),
    /// Position adjustment in thousandths of text space, written verbatim.
    Adjust(Decimal),
}

/// The bounded operator set. Operands are exact decimals or raw code bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Save,
    Restore,
    /// `a b c d e f cm`
    Concat([Decimal; 6]),
    LineWidth(Decimal),
    LineCap(u8),
    LineJoin(u8),
    /// `[array] phase d`
    Dash(Vec<Decimal>, Decimal),
    FillGray(Decimal),
    StrokeGray(Decimal),
    FillRgb([Decimal; 3]),
    StrokeRgb([Decimal; 3]),
    Move(Decimal, Decimal),
    Line(Decimal, Decimal),
    /// `x1 y1 x2 y2 x3 y3 c`
    Cubic([Decimal; 6]),
    Close,
    /// `x y w h re`
    Rect([Decimal; 4]),
    Stroke,
    Fill,
    FillEvenOdd,
    EndPath,
    ClipNonZero,
    ClipEvenOdd,
    BeginText,
    EndText,
    /// `/Name size Tf`; the name is the resource key without the slash.
    Font(String, Decimal),
    TextMove(Decimal, Decimal),
    TextMatrix([Decimal; 6]),
    ShowText(Vec<u8>),
    ShowTextArray(Vec<TjElement>),
}

impl Op {
    /// A filled rectangle: the typed rule primitive (`x y w h re f`).
    pub fn rule(x: Decimal, y: Decimal, w: Decimal, h: Decimal) -> [Op; 2] {
        [Op::Rect([x, y, w, h]), Op::Fill]
    }

    fn mnemonic(&self) -> &'static str {
        match self {
            Op::Save => "q",
            Op::Restore => "Q",
            Op::Concat(_) => "cm",
            Op::LineWidth(_) => "w",
            Op::LineCap(_) => "J",
            Op::LineJoin(_) => "j",
            Op::Dash(..) => "d",
            Op::FillGray(_) => "g",
            Op::StrokeGray(_) => "G",
            Op::FillRgb(_) => "rg",
            Op::StrokeRgb(_) => "RG",
            Op::Move(..) => "m",
            Op::Line(..) => "l",
            Op::Cubic(_) => "c",
            Op::Close => "h",
            Op::Rect(_) => "re",
            Op::Stroke => "S",
            Op::Fill => "f",
            Op::FillEvenOdd => "f*",
            Op::EndPath => "n",
            Op::ClipNonZero => "W",
            Op::ClipEvenOdd => "W*",
            Op::BeginText => "BT",
            Op::EndText => "ET",
            Op::Font(..) => "Tf",
            Op::TextMove(..) => "Td",
            Op::TextMatrix(_) => "Tm",
            Op::ShowText(_) => "Tj",
            Op::ShowTextArray(_) => "TJ",
        }
    }

    /// Serialises one operator: operands separated by single spaces, then
    /// the mnemonic. Strings are literal `( … )` with the three delimiters
    /// and control bytes escaped; nothing else is touched.
    pub fn write(&self, out: &mut Vec<u8>) {
        fn nums(out: &mut Vec<u8>, ds: &[Decimal]) {
            for d in ds {
                out.extend_from_slice(d.as_str().as_bytes());
                out.push(b' ');
            }
        }
        match self {
            Op::Save
            | Op::Restore
            | Op::Close
            | Op::Stroke
            | Op::Fill
            | Op::FillEvenOdd
            | Op::EndPath
            | Op::ClipNonZero
            | Op::ClipEvenOdd
            | Op::BeginText
            | Op::EndText => {}
            Op::Concat(v) | Op::Cubic(v) | Op::TextMatrix(v) => nums(out, v),
            Op::LineWidth(d) | Op::FillGray(d) | Op::StrokeGray(d) => {
                nums(out, std::slice::from_ref(d))
            }
            Op::LineCap(n) | Op::LineJoin(n) => {
                out.extend_from_slice(n.to_string().as_bytes());
                out.push(b' ');
            }
            Op::Dash(array, phase) => {
                out.push(b'[');
                for (i, d) in array.iter().enumerate() {
                    if i > 0 {
                        out.push(b' ');
                    }
                    out.extend_from_slice(d.as_str().as_bytes());
                }
                out.extend_from_slice(b"] ");
                nums(out, std::slice::from_ref(phase));
            }
            Op::FillRgb(v) | Op::StrokeRgb(v) => nums(out, v),
            Op::Move(x, y) | Op::Line(x, y) | Op::TextMove(x, y) => {
                nums(out, &[x.clone(), y.clone()]);
            }
            Op::Rect(v) => nums(out, v),
            Op::Font(name, size) => {
                out.push(b'/');
                out.extend_from_slice(name.as_bytes());
                out.push(b' ');
                nums(out, std::slice::from_ref(size));
            }
            Op::ShowText(bytes) => {
                write_literal(out, bytes);
                out.push(b' ');
            }
            Op::ShowTextArray(elements) => {
                out.push(b'[');
                for e in elements {
                    match e {
                        TjElement::Text(b) => write_literal(out, b),
                        TjElement::Adjust(d) => out.extend_from_slice(d.as_str().as_bytes()),
                    }
                }
                out.extend_from_slice(b"] ");
            }
        }
        out.extend_from_slice(self.mnemonic().as_bytes());
        out.push(b'\n');
    }
}

fn write_literal(out: &mut Vec<u8>, bytes: &[u8]) {
    out.push(b'(');
    for &b in bytes {
        match b {
            b'(' | b')' | b'\\' => {
                out.push(b'\\');
                out.push(b);
            }
            0..=31 | 127..=255 => {
                let _ = write!(out_string(out), "\\{b:03o}");
            }
            _ => out.push(b),
        }
    }
    out.push(b')');
}

// Small adapter so `write!` can target a Vec<u8> with a String formatter.
struct VecWriter<'a>(&'a mut Vec<u8>);
impl std::fmt::Write for VecWriter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
fn out_string(out: &mut Vec<u8>) -> VecWriter<'_> {
    VecWriter(out)
}

/// Serialises operators one per line.
pub fn serialize(ops: &[Op]) -> Vec<u8> {
    let mut out = Vec::new();
    for op in ops {
        op.write(&mut out);
    }
    out
}

// ---------------------------------------------------------------------------
// Content parsing: the same bounded operator set, read back from bytes.

/// A content-stream operand as read from bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Operand {
    Number(String),
    Name(String),
    String(Vec<u8>),
    Array(Vec<Operand>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Operand(Operand),
    ArrayOpen,
    ArrayClose,
    Operator(String),
}

fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\n' | b'\r' | b'\t' | b'\x0c' | b'\0')
}

fn is_delimiter(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}

fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Decodes `#xx` escapes in a name.
fn decode_name(raw: &[u8]) -> Result<String, String> {
    let mut name = String::with_capacity(raw.len());
    let mut k = 0;
    while k < raw.len() {
        if raw[k] == b'#' && raw.len() >= k + 3 {
            let hi = hex_digit(raw[k + 1]).ok_or("bad #xx escape in name")?;
            let lo = hex_digit(raw[k + 2]).ok_or("bad #xx escape in name")?;
            name.push((hi * 16 + lo) as char);
            k += 3;
        } else {
            name.push(raw[k] as char);
            k += 1;
        }
    }
    Ok(name)
}

fn tokenize(content: &[u8]) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < content.len() {
        let b = content[i];
        if is_whitespace(b) {
            i += 1;
            continue;
        }
        match b {
            b'%' => {
                while i < content.len() && content[i] != b'\n' && content[i] != b'\r' {
                    i += 1;
                }
            }
            b'[' => {
                tokens.push(Token::ArrayOpen);
                i += 1;
            }
            b']' => {
                tokens.push(Token::ArrayClose);
                i += 1;
            }
            b'/' => {
                let start = i + 1;
                i = start;
                while i < content.len() && !is_whitespace(content[i]) && !is_delimiter(content[i]) {
                    i += 1;
                }
                tokens.push(Token::Operand(Operand::Name(decode_name(
                    &content[start..i],
                )?)));
            }
            b'(' => {
                let (s, next) = read_literal(content, i)?;
                tokens.push(Token::Operand(Operand::String(s)));
                i = next;
            }
            b'<' => {
                if content.get(i + 1) == Some(&b'<') {
                    return Err(format!("dictionary at byte {i} is not a content operand"));
                }
                let end = content[i..]
                    .iter()
                    .position(|&c| c == b'>')
                    .ok_or_else(|| format!("unterminated hex string at byte {i}"))?;
                let hex: Vec<u8> = content[i + 1..i + end]
                    .iter()
                    .copied()
                    .filter(|c| !is_whitespace(*c))
                    .collect();
                let mut bytes = Vec::with_capacity(hex.len() / 2 + 1);
                for pair in hex.chunks(2) {
                    let hi = hex_digit(pair[0]);
                    let lo = pair.get(1).map_or(Some(0), |&c| hex_digit(c));
                    match (hi, lo) {
                        (Some(h), Some(l)) => bytes.push(h * 16 + l),
                        _ => return Err(format!("bad hex string at byte {i}")),
                    }
                }
                tokens.push(Token::Operand(Operand::String(bytes)));
                i += end + 1;
            }
            b')' | b'>' | b'{' | b'}' => {
                return Err(format!("unexpected delimiter {:?} at byte {i}", b as char));
            }
            _ => {
                let start = i;
                while i < content.len() && !is_whitespace(content[i]) && !is_delimiter(content[i]) {
                    i += 1;
                }
                let word = std::str::from_utf8(&content[start..i])
                    .map_err(|_| format!("non-ASCII token at byte {start}"))?;
                if word
                    .starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '+' || c == '.')
                {
                    tokens.push(Token::Operand(Operand::Number(word.to_string())));
                } else {
                    tokens.push(Token::Operator(word.to_string()));
                }
            }
        }
    }
    Ok(tokens)
}

fn read_literal(content: &[u8], start: usize) -> Result<(Vec<u8>, usize), String> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut i = start + 1;
    while i < content.len() {
        let b = content[i];
        match b {
            b'\\' => {
                i += 1;
                let e = *content.get(i).ok_or("unterminated string escape")?;
                match e {
                    b'n' => out.push(b'\n'),
                    b'r' => out.push(b'\r'),
                    b't' => out.push(b'\t'),
                    b'b' => out.push(8),
                    b'f' => out.push(12),
                    b'(' | b')' | b'\\' => out.push(e),
                    b'0'..=b'7' => {
                        let mut v: u32 = 0;
                        let mut n = 0;
                        while n < 3 && i < content.len() && (b'0'..=b'7').contains(&content[i]) {
                            v = v * 8 + (content[i] - b'0') as u32;
                            i += 1;
                            n += 1;
                        }
                        out.push((v & 0xFF) as u8);
                        continue;
                    }
                    b'\n' => {}
                    b'\r' => {
                        if content.get(i + 1) == Some(&b'\n') {
                            i += 1;
                        }
                    }
                    other => out.push(other),
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                out.push(b);
                i += 1;
            }
            b')' => {
                if depth == 0 {
                    return Ok((out, i + 1));
                }
                depth -= 1;
                out.push(b);
                i += 1;
            }
            _ => {
                out.push(b);
                i += 1;
            }
        }
    }
    Err(format!("unterminated literal string at byte {start}"))
}

/// Parses content bytes into the bounded operator set. Any operator outside
/// the set, wrong arity, or malformed operand is an error naming the
/// operator index (page 0 in the error; callers fill in the page).
pub fn parse(content: &[u8]) -> Result<Vec<Op>, ExactError> {
    if content.len() > MAX_CONTENT_BYTES {
        return Err(ExactError::Limit("content stream bytes"));
    }
    let err = |op: usize, m: String| ExactError::Content {
        page: 0,
        op,
        message: m,
    };
    let tokens = tokenize(content).map_err(|m| err(0, m))?;
    let mut ops = Vec::new();
    let mut operands: Vec<Operand> = Vec::new();
    let mut array: Option<Vec<Operand>> = None;
    for t in tokens {
        match t {
            Token::ArrayOpen => {
                if array.is_some() {
                    return Err(err(ops.len(), "nested array operand".into()));
                }
                array = Some(Vec::new());
            }
            Token::ArrayClose => {
                let a = array
                    .take()
                    .ok_or_else(|| err(ops.len(), "] without [".into()))?;
                operands.push(Operand::Array(a));
            }
            Token::Operator(name) => {
                if array.is_some() {
                    return Err(err(ops.len(), format!("operator {name} inside an array")));
                }
                let op = build_op(&name, &operands).map_err(|m| err(ops.len(), m))?;
                ops.push(op);
                operands.clear();
                if ops.len() > MAX_OPERATORS {
                    return Err(ExactError::Limit("operators per page"));
                }
            }
            Token::Operand(o) => match &mut array {
                Some(a) => a.push(o),
                None => operands.push(o),
            },
        }
    }
    if array.is_some() {
        return Err(err(ops.len(), "unterminated array".into()));
    }
    if !operands.is_empty() {
        return Err(err(
            ops.len(),
            "trailing operands without an operator".into(),
        ));
    }
    Ok(ops)
}

fn decimal(t: &Operand) -> Result<Decimal, String> {
    match t {
        Operand::Number(s) => Decimal::new(s).map_err(|e| e.to_string()),
        other => Err(format!("expected a number, found {other:?}")),
    }
}

fn decimals<const N: usize>(operands: &[Operand], name: &str) -> Result<[Decimal; N], String> {
    if operands.len() != N {
        return Err(format!(
            "{name} takes {N} operand(s), found {}",
            operands.len()
        ));
    }
    let mut out: Vec<Decimal> = Vec::with_capacity(N);
    for t in operands {
        out.push(decimal(t)?);
    }
    Ok(out.try_into().expect("length checked"))
}

fn small_int(operands: &[Operand], name: &str) -> Result<u8, String> {
    match operands {
        [Operand::Number(s)] => s
            .parse::<u8>()
            .ok()
            .filter(|v| *v <= 2)
            .ok_or_else(|| format!("{name} takes 0, 1 or 2, found {s}")),
        _ => Err(format!("{name} takes one integer operand")),
    }
}

fn build_op(name: &str, operands: &[Operand]) -> Result<Op, String> {
    let none = |op: Op| {
        if operands.is_empty() {
            Ok(op)
        } else {
            Err(format!(
                "{name} takes no operands, found {}",
                operands.len()
            ))
        }
    };
    Ok(match name {
        "q" => none(Op::Save)?,
        "Q" => none(Op::Restore)?,
        "cm" => Op::Concat(decimals::<6>(operands, name)?),
        "w" => Op::LineWidth(decimals::<1>(operands, name)?[0].clone()),
        "J" => Op::LineCap(small_int(operands, name)?),
        "j" => Op::LineJoin(small_int(operands, name)?),
        "d" => match operands {
            [Operand::Array(items), phase] => Op::Dash(
                items.iter().map(decimal).collect::<Result<Vec<_>, _>>()?,
                decimal(phase)?,
            ),
            _ => return Err("d takes an array and a phase".into()),
        },
        "g" => Op::FillGray(decimals::<1>(operands, name)?[0].clone()),
        "G" => Op::StrokeGray(decimals::<1>(operands, name)?[0].clone()),
        "rg" => Op::FillRgb(decimals::<3>(operands, name)?),
        "RG" => Op::StrokeRgb(decimals::<3>(operands, name)?),
        "m" => {
            let [x, y] = decimals::<2>(operands, name)?;
            Op::Move(x, y)
        }
        "l" => {
            let [x, y] = decimals::<2>(operands, name)?;
            Op::Line(x, y)
        }
        "c" => Op::Cubic(decimals::<6>(operands, name)?),
        "h" => none(Op::Close)?,
        "re" => Op::Rect(decimals::<4>(operands, name)?),
        "S" => none(Op::Stroke)?,
        "f" => none(Op::Fill)?,
        "f*" => none(Op::FillEvenOdd)?,
        "n" => none(Op::EndPath)?,
        "W" => none(Op::ClipNonZero)?,
        "W*" => none(Op::ClipEvenOdd)?,
        "BT" => none(Op::BeginText)?,
        "ET" => none(Op::EndText)?,
        "Tf" => match operands {
            [Operand::Name(n), size] => Op::Font(n.clone(), decimal(size)?),
            _ => return Err("Tf takes a name and a size".into()),
        },
        "Td" => {
            let [x, y] = decimals::<2>(operands, name)?;
            Op::TextMove(x, y)
        }
        "Tm" => Op::TextMatrix(decimals::<6>(operands, name)?),
        "Tj" => match operands {
            [Operand::String(s)] => Op::ShowText(s.clone()),
            _ => return Err("Tj takes one string".into()),
        },
        "TJ" => match operands {
            [Operand::Array(items)] => {
                let mut elements = Vec::with_capacity(items.len());
                for t in items {
                    elements.push(match t {
                        Operand::String(s) => TjElement::Text(s.clone()),
                        Operand::Number(n) => {
                            TjElement::Adjust(Decimal::new(n).map_err(|e| e.to_string())?)
                        }
                        other => return Err(format!("TJ array element {other:?}")),
                    });
                }
                Op::ShowTextArray(elements)
            }
            _ => return Err("TJ takes exactly one array".into()),
        },
        other => {
            return Err(format!(
                "operator {other:?} is outside the exact export's bounded set"
            ));
        }
    })
}

// ---------------------------------------------------------------------------
// Fonts

/// A font program to embed, byte for byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontProgram {
    /// Type 1 in PFA/PFB segment form: `/FontFile` with `/Length1`
    /// (clear-text portion), `/Length2` (encrypted portion) and `/Length3`
    /// (trailer, 0 when the 512 zeros are omitted). The bytes are written
    /// unchanged.
    Type1 {
        bytes: Vec<u8>,
        length1: usize,
        length2: usize,
        length3: usize,
    },
    /// A bare CFF program: `/FontFile3` `/Subtype /CIDFontType0C` under a
    /// CID font, or `/Type1C` under a simple font.
    Cff(Vec<u8>),
    /// A TrueType program (`glyf`): `/FontFile2`.
    TrueType(Vec<u8>),
}

impl FontProgram {
    pub fn bytes(&self) -> &[u8] {
        match self {
            FontProgram::Type1 { bytes, .. }
            | FontProgram::Cff(bytes)
            | FontProgram::TrueType(bytes) => bytes,
        }
    }

    /// Parses a PFB file (segment headers `0x80 0x01/0x02/0x03`) into the
    /// three lengths PDF wants. A PFA file (no segment headers) is accepted
    /// only when its binary section is already binary, i.e. never for the
    /// usual hex-encoded PFA; such input is reported.
    pub fn type1_from_pfb(pfb: &[u8]) -> Result<FontProgram, String> {
        let mut bytes = Vec::with_capacity(pfb.len());
        let mut lengths = Vec::new();
        let mut i = 0;
        loop {
            if pfb.get(i) != Some(&0x80) {
                return Err(format!("not a PFB file: no segment header at byte {i}"));
            }
            let kind = *pfb.get(i + 1).ok_or("truncated PFB segment header")?;
            if kind == 3 {
                break;
            }
            let len = u32::from_le_bytes(
                pfb.get(i + 2..i + 6)
                    .ok_or("truncated PFB segment length")?
                    .try_into()
                    .expect("4 bytes"),
            ) as usize;
            let seg = pfb.get(i + 6..i + 6 + len).ok_or("truncated PFB segment")?;
            bytes.extend_from_slice(seg);
            lengths.push((kind, len));
            i += 6 + len;
        }
        // Segments: 1 (ascii), 2 (binary), 1 (trailer ascii). Merge adjacent
        // segments of the same kind.
        let mut merged: Vec<(u8, usize)> = Vec::new();
        for (k, l) in lengths {
            match merged.last_mut() {
                Some((lk, ll)) if *lk == k => *ll += l,
                _ => merged.push((k, l)),
            }
        }
        match merged.as_slice() {
            [(1, l1), (2, l2)] => Ok(FontProgram::Type1 {
                bytes,
                length1: *l1,
                length2: *l2,
                length3: 0,
            }),
            [(1, l1), (2, l2), (1, l3)] => Ok(FontProgram::Type1 {
                bytes,
                length1: *l1,
                length2: *l2,
                length3: *l3,
            }),
            other => Err(format!("unexpected PFB segment layout {other:?}")),
        }
    }
}

/// `/FontDescriptor` values, all carried as given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDescriptor {
    pub flags: i64,
    pub bbox: [Decimal; 4],
    pub italic_angle: Decimal,
    pub ascent: Decimal,
    pub descent: Decimal,
    pub cap_height: Decimal,
    pub stem_v: Decimal,
    pub x_height: Option<Decimal>,
    /// `/CharSet` string for Type 1 subsets, written verbatim when present.
    pub char_set: Option<String>,
}

/// A simple font's encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Encoding {
    /// A predefined encoding name such as `WinAnsiEncoding`.
    Named(String),
    /// `/Differences` over an optional base encoding: `(code, glyph name)`.
    Differences {
        base: Option<String>,
        differences: Vec<(u8, String)>,
    },
}

/// A one-byte-code font (pdfTeX's route for Type 1 programs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleFont {
    /// `Type1` or `TrueType`.
    pub subtype: String,
    pub base_font: String,
    pub program: Option<FontProgram>,
    pub first_char: u8,
    /// Widths for codes `first_char..=first_char + widths.len() - 1`. Empty
    /// means no `/Widths` array (a standard-14 font using its own metrics),
    /// in which case every code is accepted.
    pub widths: Vec<Decimal>,
    pub encoding: Option<Encoding>,
    pub descriptor: Option<FontDescriptor>,
    /// A complete ToUnicode CMap stream body, written verbatim.
    pub to_unicode: Option<Vec<u8>>,
}

/// A two-byte-code font (`Type0`, `Identity-H`) whose codes are the
/// source font's glyph ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CidFont {
    pub base_font: String,
    pub program: FontProgram,
    /// `/W` entries per CID (1000/em units, verbatim). CIDs absent here use
    /// `default_width`.
    pub widths: BTreeMap<u16, Decimal>,
    pub default_width: Decimal,
    pub descriptor: FontDescriptor,
    /// Unicode text for CIDs, for `ToUnicode`; a CID may map to several
    /// scalars (ligatures). Empty means no ToUnicode stream.
    pub to_unicode: BTreeMap<u16, String>,
    /// Glyph ids the caller may show; a code outside this set is an error.
    /// Empty means "any CID with a width".
    pub glyphs: BTreeSet<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExactFont {
    CidCff(CidFont),
    CidTrueType(CidFont),
    Simple(SimpleFont),
}

impl ExactFont {
    fn object_count(&self) -> usize {
        match self {
            ExactFont::CidCff(c) | ExactFont::CidTrueType(c) => {
                4 + usize::from(!c.to_unicode.is_empty())
            }
            ExactFont::Simple(s) => {
                1 + usize::from(s.descriptor.is_some())
                    + usize::from(s.program.is_some())
                    + usize::from(s.to_unicode.is_some())
                    + usize::from(matches!(s.encoding, Some(Encoding::Differences { .. })))
            }
        }
    }

    /// The program bytes and their SHA-256, for evidence.
    pub fn program_identity(&self) -> Option<(usize, String)> {
        let p = match self {
            ExactFont::CidCff(c) | ExactFont::CidTrueType(c) => Some(&c.program),
            ExactFont::Simple(s) => s.program.as_ref(),
        }?;
        Some((p.bytes().len(), sha256::hex(p.bytes())))
    }
}

/// Six uppercase letters derived deterministically from the glyph set and
/// the program identity, for subset `/BaseFont` names.
pub fn subset_tag(gids: &BTreeSet<u16>, program_sha256: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for g in gids {
        h ^= *g as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    for b in program_sha256.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (0..6)
        .map(|i| (b'A' + ((h >> (8 * i)) % 26) as u8) as char)
        .collect()
}

/// What [`ExactFont::cid_from_opentype`] did to the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsetOutcome {
    /// CFF rewritten as a CID-keyed subset; CID = original GID.
    CffSubset,
    /// CFF embedded whole because subsetting was refused (reason in the
    /// returned note); glyph ids are still the font's own.
    CffWhole,
    /// TrueType subset with glyph order preserved (unused glyphs replaced by
    /// empty outlines), `/CIDToGIDMap /Identity`.
    TrueTypeIdentity,
}

impl ExactFont {
    /// Builds a CID font from an OpenType file for the given glyph ids.
    ///
    /// CFF outlines: [`CffFont::subset`] keeps original GIDs as CIDs. If the
    /// subsetter refuses (CID-keyed source, `seac`), the whole `CFF ` table
    /// is embedded and the second return value says so; glyph ids stay the
    /// font's own either way. TrueType outlines are embedded through
    /// [`TrueTypeFont::subset_keep_gids`], which keeps every glyph id in
    /// place. Widths come from `hmtx` scaled to 1000/em exactly (a warning
    /// is not needed: units-per-em values are powers of two or 1000 in
    /// practice, and a non-terminating ratio is rounded to four places and
    /// reported in the note).
    pub fn cid_from_opentype(
        font: &TrueTypeFont,
        gids: &BTreeSet<u16>,
        to_unicode: BTreeMap<u16, String>,
    ) -> Result<(ExactFont, SubsetOutcome, Option<String>), ExactError> {
        let resource = font.postscript_name.clone();
        let err = |m: String| ExactError::Font {
            resource: resource.clone(),
            message: m,
        };
        for &g in gids {
            if g >= font.num_glyphs() {
                return Err(err(format!(
                    "glyph {g} is outside the font's {} glyphs",
                    font.num_glyphs()
                )));
            }
        }
        let mut note = None;
        let mut widths = BTreeMap::new();
        for &g in gids {
            let adv = font.advance(g) as i128;
            let w = match Decimal::from_ratio(adv * 1000, font.units_per_em as u128, 12) {
                Some(d) => d,
                None => {
                    let rounded = (adv as f64 * 1000.0 / font.units_per_em as f64 * 10000.0)
                        .round()
                        / 10000.0;
                    note.get_or_insert_with(|| {
                        format!(
                            "advance widths are not exact in 1000/em for unitsPerEm {}; rounded to 4 places",
                            font.units_per_em
                        )
                    });
                    Decimal::new(&format!("{rounded}")).map_err(|e| err(e.to_string()))?
                }
            };
            widths.insert(g, w);
        }
        let scale = |v: i16| -> Decimal {
            Decimal::from_ratio(v as i128 * 1000, font.units_per_em as u128, 12).unwrap_or_else(
                || Decimal::from_i64((v as f64 * 1000.0 / font.units_per_em as f64).round() as i64),
            )
        };
        let descriptor = FontDescriptor {
            flags: 4,
            bbox: [
                scale(font.bbox[0]),
                scale(font.bbox[1]),
                scale(font.bbox[2]),
                scale(font.bbox[3]),
            ],
            italic_angle: Decimal::new(&format!("{}", font.italic_angle))
                .unwrap_or(Decimal::from_i64(0)),
            ascent: scale(font.ascender),
            descent: scale(font.descender),
            cap_height: scale(font.cap_height.unwrap_or(font.ascender)),
            stem_v: Decimal::from_i64(80),
            x_height: None,
            char_set: None,
        };
        let (program, outcome, base_font) = match font.outlines {
            Outlines::Cff => {
                let table = font
                    .cff_table()
                    .ok_or_else(|| err("OTTO font without CFF table".into()))?;
                let cff = CffFont::parse(table).map_err(|e| err(e.to_string()))?;
                match cff.subset(gids) {
                    Ok(sub) => {
                        let tag = subset_tag(gids, &sha256::hex(table));
                        (
                            FontProgram::Cff(sub.bytes),
                            SubsetOutcome::CffSubset,
                            format!("{tag}+{}", font.postscript_name),
                        )
                    }
                    Err(e @ (CffError::CidKeyedSource | CffError::Seac { .. })) => {
                        note = Some(format!("embedded whole: {e}"));
                        (
                            FontProgram::Cff(table.to_vec()),
                            SubsetOutcome::CffWhole,
                            font.postscript_name.clone(),
                        )
                    }
                    Err(e) => return Err(err(e.to_string())),
                }
            }
            Outlines::TrueType => {
                let sub = font.subset_keep_gids(gids).map_err(err)?;
                let tag = subset_tag(gids, &sha256::hex(&sub));
                (
                    FontProgram::TrueType(sub),
                    SubsetOutcome::TrueTypeIdentity,
                    format!("{tag}+{}", font.postscript_name),
                )
            }
        };
        if program.bytes().len() > MAX_FONT_BYTES {
            return Err(ExactError::Limit("font program bytes"));
        }
        let cid = CidFont {
            base_font,
            program,
            widths,
            default_width: Decimal::from_i64(1000),
            descriptor,
            to_unicode,
            glyphs: gids.clone(),
        };
        let font = match outcome {
            SubsetOutcome::TrueTypeIdentity => ExactFont::CidTrueType(cid),
            _ => ExactFont::CidCff(cid),
        };
        Ok((font, outcome, note))
    }
}

// ---------------------------------------------------------------------------
// Documents

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Ops(Vec<Op>),
    /// Bytes inserted unchanged after validation by [`parse`].
    Verbatim(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactPage {
    pub width: Decimal,
    pub height: Decimal,
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExactDocument {
    pub pages: Vec<ExactPage>,
    /// Font resources by name (the `Tf` operand without the slash).
    pub fonts: BTreeMap<String, ExactFont>,
}

/// One glyph in a [`GlyphRun`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedGlyph {
    /// Two-byte code = original glyph id for CID fonts.
    pub gid: u16,
    /// Absolute origin in PDF user space; `None` continues at the previous
    /// glyph's natural advance (same string).
    pub origin: Option<(Decimal, Decimal)>,
}

/// A typed glyph run: font resource, size, and glyphs by original id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphRun {
    pub font: String,
    pub size: Decimal,
    pub glyphs: Vec<PlacedGlyph>,
}

impl GlyphRun {
    /// Expands to `BT … ET`: each glyph with an explicit origin starts a new
    /// `1 0 0 1 x y Tm` + string; glyphs without one join the current
    /// string. The first glyph must carry an origin.
    pub fn to_ops(&self) -> Result<Vec<Op>, ExactError> {
        let mut ops = vec![
            Op::BeginText,
            Op::Font(self.font.clone(), self.size.clone()),
        ];
        let mut current: Vec<u8> = Vec::new();
        let one = Decimal::from_i64(1);
        let zero = Decimal::from_i64(0);
        for (i, g) in self.glyphs.iter().enumerate() {
            match &g.origin {
                Some((x, y)) => {
                    if !current.is_empty() {
                        ops.push(Op::ShowText(std::mem::take(&mut current)));
                    }
                    ops.push(Op::TextMatrix([
                        one.clone(),
                        zero.clone(),
                        zero.clone(),
                        one.clone(),
                        x.clone(),
                        y.clone(),
                    ]));
                }
                None if i == 0 => {
                    return Err(ExactError::Invalid(
                        "the first glyph of a run needs an origin".into(),
                    ));
                }
                None => {}
            }
            current.extend_from_slice(&g.gid.to_be_bytes());
        }
        if !current.is_empty() {
            ops.push(Op::ShowText(current));
        }
        ops.push(Op::EndText);
        Ok(ops)
    }
}

/// Validates the operator sequence against the document's fonts and the
/// PDF graphics/text state rules this writer enforces.
fn validate(
    page_index: usize,
    ops: &[Op],
    fonts: &BTreeMap<String, ExactFont>,
) -> Result<(), ExactError> {
    let err = |op: usize, m: String| ExactError::Content {
        page: page_index + 1,
        op,
        message: m,
    };
    let mut depth = 0i32;
    let mut in_text = false;
    let mut font: Option<&ExactFont> = None;
    let mut has_path = false;
    let mut has_point = false;
    let check_string =
        |i: usize, bytes: &[u8], font: Option<&ExactFont>| -> Result<(), ExactError> {
            let f = font.ok_or_else(|| err(i, "text shown before Tf".into()))?;
            match f {
                ExactFont::Simple(s) => {
                    if s.widths.is_empty() {
                        return Ok(());
                    }
                    let last = s.first_char as usize + s.widths.len();
                    for &b in bytes {
                        if (b as usize) < s.first_char as usize || (b as usize) >= last {
                            return Err(err(
                                i,
                                format!(
                                    "code {b} has no width in font (FirstChar {}, {} widths)",
                                    s.first_char,
                                    s.widths.len()
                                ),
                            ));
                        }
                    }
                }
                ExactFont::CidCff(c) | ExactFont::CidTrueType(c) => {
                    if !bytes.len().is_multiple_of(2) {
                        return Err(err(i, "odd byte count for a two-byte CID font".into()));
                    }
                    for pair in bytes.chunks(2) {
                        let cid = u16::from_be_bytes([pair[0], pair[1]]);
                        let known = if c.glyphs.is_empty() {
                            c.widths.contains_key(&cid)
                        } else {
                            c.glyphs.contains(&cid)
                        };
                        if !known {
                            return Err(err(
                                i,
                                format!("glyph id {cid} is not in the font's subset"),
                            ));
                        }
                    }
                }
            }
            Ok(())
        };
    for (i, op) in ops.iter().enumerate() {
        match op {
            Op::Save => depth += 1,
            Op::Restore => {
                depth -= 1;
                if depth < 0 {
                    return Err(err(i, "Q without matching q".into()));
                }
            }
            Op::BeginText => {
                if in_text {
                    return Err(err(i, "BT inside a text object".into()));
                }
                in_text = true;
            }
            Op::EndText => {
                if !in_text {
                    return Err(err(i, "ET without BT".into()));
                }
                in_text = false;
            }
            Op::Font(name, size) => {
                let f = fonts
                    .get(name)
                    .ok_or_else(|| err(i, format!("font resource /{name} is not declared")))?;
                if size.approx() == 0.0 || !size.approx().is_finite() {
                    return Err(err(i, format!("font size {size} is zero or not finite")));
                }
                font = Some(f);
            }
            Op::TextMove(..) | Op::TextMatrix(_) => {
                if !in_text {
                    return Err(err(i, "text positioning outside BT/ET".into()));
                }
            }
            Op::ShowText(bytes) => {
                if !in_text {
                    return Err(err(i, "Tj outside BT/ET".into()));
                }
                check_string(i, bytes, font)?;
            }
            Op::ShowTextArray(elements) => {
                if !in_text {
                    return Err(err(i, "TJ outside BT/ET".into()));
                }
                for e in elements {
                    if let TjElement::Text(b) = e {
                        check_string(i, b, font)?;
                    }
                }
            }
            Op::Move(..) => {
                has_path = true;
                has_point = true;
            }
            Op::Line(..) | Op::Cubic(_) => {
                if !has_point {
                    return Err(err(i, "path segment without a current point".into()));
                }
            }
            Op::Close => {
                if !has_path {
                    return Err(err(i, "h without a path".into()));
                }
            }
            Op::Rect(_) => {
                has_path = true;
                has_point = false;
            }
            Op::Stroke | Op::Fill | Op::FillEvenOdd | Op::EndPath => {
                if !has_path {
                    return Err(err(i, format!("{} without a path", op.mnemonic())));
                }
                has_path = false;
                has_point = false;
            }
            Op::ClipNonZero | Op::ClipEvenOdd => {
                if !has_path {
                    return Err(err(i, "clip without a path".into()));
                }
            }
            Op::LineCap(_)
            | Op::LineJoin(_)
            | Op::LineWidth(_)
            | Op::Dash(..)
            | Op::Concat(_)
            | Op::FillGray(_)
            | Op::StrokeGray(_)
            | Op::FillRgb(_)
            | Op::StrokeRgb(_) => {}
        }
        if in_text
            && matches!(
                op,
                Op::Move(..) | Op::Line(..) | Op::Cubic(_) | Op::Rect(_) | Op::Stroke | Op::Fill
            )
        {
            return Err(err(i, "path construction inside BT/ET".into()));
        }
    }
    if depth != 0 {
        return Err(err(ops.len(), format!("{depth} unmatched q")));
    }
    if in_text {
        return Err(err(ops.len(), "unterminated text object".into()));
    }
    if has_path {
        return Err(err(ops.len(), "page ends with an unpainted path".into()));
    }
    Ok(())
}

/// Renders an exact document. Returns the bytes plus notes (never silent
/// substitutions: there are none on this route).
pub fn render_exact(doc: &ExactDocument) -> Result<crate::PdfOutput, ExactError> {
    if doc.pages.is_empty() {
        return Err(ExactError::Invalid("a PDF needs at least one page".into()));
    }
    for (i, p) in doc.pages.iter().enumerate() {
        for (name, v) in [("width", &p.width), ("height", &p.height)] {
            if v.approx().partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
                return Err(ExactError::Invalid(format!(
                    "page {}: {name} {v} is not positive",
                    i + 1
                )));
            }
        }
    }
    for (name, f) in &doc.fonts {
        let valid = !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.');
        if !valid {
            return Err(ExactError::Font {
                resource: name.clone(),
                message: "resource names must be non-empty ASCII alphanumerics, '_', '-' or '.'"
                    .into(),
            });
        }
        let program_len = f.program_identity().map_or(0, |(n, _)| n);
        if program_len > MAX_FONT_BYTES {
            return Err(ExactError::Limit("font program bytes"));
        }
        if let ExactFont::Simple(s) = f
            && s.first_char as usize + s.widths.len() > 256
        {
            return Err(ExactError::Font {
                resource: name.clone(),
                message: format!(
                    "FirstChar {} with {} widths does not fit in one byte",
                    s.first_char,
                    s.widths.len()
                ),
            });
        }
    }

    // Validate and serialise each page's content first so errors surface
    // before any object is written.
    let mut contents: Vec<Vec<u8>> = Vec::with_capacity(doc.pages.len());
    for (i, page) in doc.pages.iter().enumerate() {
        let bytes = match &page.content {
            Content::Ops(ops) => {
                if ops.len() > MAX_OPERATORS {
                    return Err(ExactError::Limit("operators per page"));
                }
                validate(i, ops, &doc.fonts)?;
                serialize(ops)
            }
            Content::Verbatim(bytes) => {
                let ops = parse(bytes).map_err(|e| match e {
                    ExactError::Content { op, message, .. } => ExactError::Content {
                        page: i + 1,
                        op,
                        message,
                    },
                    other => other,
                })?;
                validate(i, &ops, &doc.fonts)?;
                bytes.clone()
            }
        };
        contents.push(bytes);
    }

    // Object numbering: 1 catalog, 2 pages, 3 info, then page/content pairs,
    // then fonts in resource-name order.
    let page_count = doc.pages.len();
    let first_page = 4;
    let mut next = first_page + 2 * page_count;
    let mut font_objects: BTreeMap<&str, usize> = BTreeMap::new();
    for (name, f) in &doc.fonts {
        font_objects.insert(name, next);
        next += f.object_count();
    }
    let mut resources = String::from("/Font <<");
    for (name, obj) in &font_objects {
        let _ = write!(resources, " /{name} {obj} 0 R");
    }
    resources.push_str(" >>");

    let mut d = Document::new();
    d.object(1, b"<< /Type /Catalog /Pages 2 0 R >>");
    let mut kids = String::new();
    for i in 0..page_count {
        let _ = write!(kids, "{} 0 R ", first_page + 2 * i);
    }
    d.object(
        2,
        format!("<< /Type /Pages /Kids [ {kids}] /Count {page_count} >>").as_bytes(),
    );
    d.object(
        3,
        format!("<< /Producer ({PRODUCER}) /Creator (FlashTeX) >>").as_bytes(),
    );
    for (i, page) in doc.pages.iter().enumerate() {
        let page_obj = first_page + 2 * i;
        d.object(
            page_obj,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [ 0 0 {} {} ] /Resources << {resources} >> /Contents {} 0 R >>",
                page.width,
                page.height,
                page_obj + 1
            )
            .as_bytes(),
        );
        d.stream(page_obj + 1, &contents[i]);
    }
    for (name, f) in &doc.fonts {
        write_font(&mut d, font_objects[name.as_str()], f);
    }
    Ok(crate::PdfOutput {
        bytes: d.finish_with_info(3),
        warnings: Vec::new(),
    })
}

fn descriptor_body(d: &FontDescriptor, name: &str, file_entry: &str) -> String {
    let mut s = format!(
        "<< /Type /FontDescriptor /FontName /{name} /Flags {} /FontBBox [ {} {} {} {} ] /ItalicAngle {} /Ascent {} /Descent {} /CapHeight {} /StemV {}",
        d.flags,
        d.bbox[0],
        d.bbox[1],
        d.bbox[2],
        d.bbox[3],
        d.italic_angle,
        d.ascent,
        d.descent,
        d.cap_height,
        d.stem_v
    );
    if let Some(x) = &d.x_height {
        let _ = write!(s, " /XHeight {x}");
    }
    if let Some(c) = &d.char_set {
        let _ = write!(s, " /CharSet ({})", escape_pdf_string(c));
    }
    s.push_str(file_entry);
    s.push_str(" >>");
    s
}

fn escape_pdf_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' | ')' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

fn program_stream(d: &mut Document, obj: usize, p: &FontProgram, cid: bool) -> &'static str {
    match p {
        FontProgram::Type1 {
            bytes,
            length1,
            length2,
            length3,
        } => {
            d.stream_with(
                obj,
                &format!("/Length1 {length1} /Length2 {length2} /Length3 {length3}"),
                bytes,
            );
            "FontFile"
        }
        FontProgram::Cff(bytes) => {
            d.stream_with(
                obj,
                if cid {
                    "/Subtype /CIDFontType0C"
                } else {
                    "/Subtype /Type1C"
                },
                bytes,
            );
            "FontFile3"
        }
        FontProgram::TrueType(bytes) => {
            d.stream_with(obj, &format!("/Length1 {}", bytes.len()), bytes);
            "FontFile2"
        }
    }
}

fn write_font(d: &mut Document, obj: usize, f: &ExactFont) {
    match f {
        ExactFont::CidCff(c) | ExactFont::CidTrueType(c) => {
            let truetype = matches!(f, ExactFont::CidTrueType(_));
            let cid_obj = obj + 1;
            let desc_obj = obj + 2;
            let file_obj = obj + 3;
            let tounicode = if c.to_unicode.is_empty() {
                String::new()
            } else {
                format!(" /ToUnicode {} 0 R", obj + 4)
            };
            d.object(
                obj,
                format!(
                    "<< /Type /Font /Subtype /Type0 /BaseFont /{} /Encoding /Identity-H /DescendantFonts [ {cid_obj} 0 R ]{tounicode} >>",
                    c.base_font
                )
                .as_bytes(),
            );
            let mut w = String::from("[");
            for (cid, width) in &c.widths {
                let _ = write!(w, " {cid} [ {width} ]");
            }
            w.push_str(" ]");
            let (subtype, gid_map) = if truetype {
                ("CIDFontType2", " /CIDToGIDMap /Identity")
            } else {
                ("CIDFontType0", "")
            };
            d.object(
                cid_obj,
                format!(
                    "<< /Type /Font /Subtype /{subtype} /BaseFont /{} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {desc_obj} 0 R /DW {} /W {w}{gid_map} >>",
                    c.base_font, c.default_width
                )
                .as_bytes(),
            );
            // The descriptor references the file object, so the file's key
            // must be known first; it depends only on the program kind.
            let file_key = match c.program {
                FontProgram::Type1 { .. } => "FontFile",
                FontProgram::Cff(_) => "FontFile3",
                FontProgram::TrueType(_) => "FontFile2",
            };
            d.object(
                desc_obj,
                descriptor_body(
                    &c.descriptor,
                    &c.base_font,
                    &format!(" /{file_key} {file_obj} 0 R"),
                )
                .as_bytes(),
            );
            let written = program_stream(d, file_obj, &c.program, true);
            debug_assert_eq!(written, file_key);
            if !c.to_unicode.is_empty() {
                d.stream(obj + 4, &to_unicode_cmap(&c.to_unicode));
            }
        }
        ExactFont::Simple(s) => {
            let mut next = obj + 1;
            let mut body = format!(
                "<< /Type /Font /Subtype /{} /BaseFont /{}",
                s.subtype, s.base_font
            );
            if !s.widths.is_empty() {
                let _ = write!(
                    body,
                    " /FirstChar {} /LastChar {} /Widths [",
                    s.first_char,
                    s.first_char as usize + s.widths.len() - 1
                );
                for w in &s.widths {
                    let _ = write!(body, " {w}");
                }
                body.push_str(" ]");
            }
            let mut trailing: Vec<(usize, Vec<u8>, Option<String>)> = Vec::new();
            if let Some(desc) = &s.descriptor {
                let desc_obj = next;
                next += 1;
                let file_entry = if let Some(p) = &s.program {
                    let file_obj = next;
                    next += 1;
                    let key = match p {
                        FontProgram::Type1 { .. } => "FontFile",
                        FontProgram::Cff(_) => "FontFile3",
                        FontProgram::TrueType(_) => "FontFile2",
                    };
                    trailing.push((file_obj, Vec::new(), Some("program".into())));
                    format!(" /{key} {file_obj} 0 R")
                } else {
                    String::new()
                };
                let _ = write!(body, " /FontDescriptor {desc_obj} 0 R");
                trailing.insert(
                    0,
                    (
                        desc_obj,
                        descriptor_body(desc, &s.base_font, &file_entry).into_bytes(),
                        None,
                    ),
                );
            }
            match &s.encoding {
                Some(Encoding::Named(n)) => {
                    let _ = write!(body, " /Encoding /{n}");
                }
                Some(Encoding::Differences { base, differences }) => {
                    let enc_obj = next;
                    next += 1;
                    let mut e = String::from("<< /Type /Encoding");
                    if let Some(b) = base {
                        let _ = write!(e, " /BaseEncoding /{b}");
                    }
                    e.push_str(" /Differences [");
                    let mut prev: Option<u8> = None;
                    for (code, name) in differences {
                        if prev.is_none_or(|p| p as u16 + 1 != *code as u16) {
                            let _ = write!(e, " {code}");
                        }
                        let _ = write!(e, " /{name}");
                        prev = Some(*code);
                    }
                    e.push_str(" ] >>");
                    let _ = write!(body, " /Encoding {enc_obj} 0 R");
                    trailing.push((enc_obj, e.into_bytes(), None));
                }
                None => {}
            }
            if s.to_unicode.is_some() {
                let tu_obj = next;
                let _ = write!(body, " /ToUnicode {tu_obj} 0 R");
                trailing.push((tu_obj, Vec::new(), Some("tounicode".into())));
            }
            body.push_str(" >>");
            d.object(obj, body.as_bytes());
            trailing.sort_by_key(|t| t.0);
            for (n, bytes, kind) in trailing {
                match kind.as_deref() {
                    None => d.object(n, &bytes),
                    Some("program") => {
                        program_stream(d, n, s.program.as_ref().expect("checked"), false);
                    }
                    Some(_) => d.stream(n, s.to_unicode.as_ref().expect("checked")),
                }
            }
        }
    }
}

/// A ToUnicode CMap mapping two-byte CIDs to UTF-16BE strings (`bfchar`,
/// so a CID may expand to several scalars).
pub fn to_unicode_cmap(map: &BTreeMap<u16, String>) -> Vec<u8> {
    let mut s = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(&u16, &String)> = map.iter().collect();
    for chunk in entries.chunks(100) {
        let _ = writeln!(s, "{} beginbfchar", chunk.len());
        for (cid, text) in chunk {
            let hex: String = text.encode_utf16().map(|u| format!("{u:04X}")).collect();
            let _ = writeln!(s, "<{cid:04X}> <{hex}>");
        }
        s.push_str("endbfchar\n");
    }
    s.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    s.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_syntax() {
        for ok in ["0", "-0", "12", "12.", ".5", "-.5", "3.1400", "007", "+4"] {
            assert!(Decimal::new(ok).is_ok(), "{ok}");
            assert_eq!(Decimal::new(ok).unwrap().as_str(), ok, "verbatim");
        }
        for bad in ["", ".", "-", "1e5", "1.2.3", "1 2", "abc", "0x10", "--1"] {
            assert!(Decimal::new(bad).is_err(), "{bad}");
        }
        assert_eq!(Decimal::from_ratio(1, 8, 12).unwrap().as_str(), "0.125");
        assert_eq!(Decimal::from_ratio(-1, 8, 12).unwrap().as_str(), "-0.125");
        assert_eq!(
            Decimal::from_ratio(2048 * 1000, 2048, 12).unwrap().as_str(),
            "1000"
        );
        assert_eq!(Decimal::from_ratio(1, 3, 12), None);
    }

    #[test]
    fn parse_and_serialize_round_trip_preserves_operands() {
        let src = b"BT\n/F44 11.9552 Tf 72 708.045 Td [(Hello)-250(w)10(orld.)-310(\\002ts)]TJ\nET\nq\n1 0 0 1 135.015 711.034 cm\n[]0 d 0 J 0.398 w 0 0 m 4.498 0 l S\nQ\n0.0 0.50 1.000 rg 1 2 3 4 re f\n";
        let ops = parse(src).unwrap();
        assert_eq!(ops.len(), 17);
        assert_eq!(
            ops[1],
            Op::Font("F44".into(), Decimal::new("11.9552").unwrap())
        );
        match &ops[3] {
            Op::ShowTextArray(e) => {
                assert_eq!(e[0], TjElement::Text(b"Hello".to_vec()));
                assert_eq!(e[1], TjElement::Adjust(Decimal::new("-250").unwrap()));
                assert_eq!(e[6], TjElement::Text(b"\x02ts".to_vec()));
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(ops[7], Op::Dash(vec![], Decimal::new("0").unwrap()));
        assert_eq!(ops[12], Op::Stroke);
        assert_eq!(ops[13], Op::Restore);
        assert_eq!(
            ops[14],
            Op::FillRgb([
                Decimal::new("0.0").unwrap(),
                Decimal::new("0.50").unwrap(),
                Decimal::new("1.000").unwrap()
            ])
        );
        let bytes = serialize(&ops);
        assert!(
            std::str::from_utf8(&bytes)
                .unwrap()
                .contains("0.0 0.50 1.000 rg\n"),
            "trailing zeros kept"
        );
        assert!(
            std::str::from_utf8(&bytes)
                .unwrap()
                .contains("[(Hello)-250(w)10(orld.)-310(\\002ts)] TJ\n")
        );
        assert_eq!(
            parse(&bytes).unwrap(),
            ops,
            "canonical form re-parses to the same operators"
        );
    }

    #[test]
    fn rejects_operators_outside_the_bounded_set() {
        let e = parse(b"BT /F1 12 Tf ET /GS0 gs").unwrap_err();
        assert!(e.to_string().contains("\"gs\" is outside"), "{e}");
        assert!(
            parse(b"1 2 3 re f")
                .unwrap_err()
                .to_string()
                .contains("re takes 4")
        );
        assert!(
            parse(b"1e5 0 m")
                .unwrap_err()
                .to_string()
                .contains("not a PDF number")
        );
        assert!(parse(b"BI /W 1 ID x EI").is_err());
    }

    #[test]
    fn glyph_run_expands_to_text_matrix_and_hex_codes() {
        let run = GlyphRun {
            font: "F1".into(),
            size: Decimal::new("10").unwrap(),
            glyphs: vec![
                PlacedGlyph {
                    gid: 47,
                    origin: Some((Decimal::new("72").unwrap(), Decimal::new("700.5").unwrap())),
                },
                PlacedGlyph {
                    gid: 72,
                    origin: None,
                },
                PlacedGlyph {
                    gid: 1,
                    origin: Some((
                        Decimal::new("100.25").unwrap(),
                        Decimal::new("700.5").unwrap(),
                    )),
                },
            ],
        };
        let ops = run.to_ops().unwrap();
        assert_eq!(
            ops[2],
            Op::TextMatrix([
                Decimal::from_i64(1),
                Decimal::from_i64(0),
                Decimal::from_i64(0),
                Decimal::from_i64(1),
                Decimal::new("72").unwrap(),
                Decimal::new("700.5").unwrap()
            ])
        );
        assert_eq!(ops[3], Op::ShowText(vec![0, 47, 0, 72]));
        assert_eq!(ops[5], Op::ShowText(vec![0, 1]));
        assert_eq!(ops.last(), Some(&Op::EndText));
        let s = String::from_utf8(serialize(&ops)).unwrap();
        assert!(s.contains("(\\000/\\000H) Tj"), "{s}");
    }

    #[test]
    fn subset_tag_depends_on_glyphs_and_program() {
        let a = subset_tag(&BTreeSet::from([1, 2]), "abc");
        assert_eq!(a.len(), 6);
        assert_eq!(a, subset_tag(&BTreeSet::from([1, 2]), "abc"));
        assert_ne!(a, subset_tag(&BTreeSet::from([1, 3]), "abc"));
        assert_ne!(a, subset_tag(&BTreeSet::from([1, 2]), "abd"));
    }
}

//! Minimal JSON reader/writer.
//!
//! Hand-written on purpose: the crate takes zero external dependencies so the
//! build is offline and deterministic. JSON is transport, not TeX semantics, so
//! implementing it here makes no claim about language compatibility.

use std::collections::BTreeMap;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(BTreeMap<String, Value>),
}

impl Value {
    pub fn obj() -> Value {
        Value::Obj(BTreeMap::new())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(m) => m.get(key),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, v: Value) {
        if let Value::Obj(m) = self {
            m.insert(key.to_string(), v);
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Num(n) if n.is_finite() => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }
}

pub fn num(n: f64) -> Value {
    Value::Num(n)
}
pub fn str_(s: impl Into<String>) -> Value {
    Value::Str(s.into())
}

#[derive(Debug)]
pub struct JsonError(pub String);

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse(input: &str) -> Result<Value, JsonError> {
    let b: Vec<char> = input.chars().collect();
    let mut p = Parser { b, i: 0 };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(JsonError("trailing content after JSON value".into()));
    }
    Ok(v)
}

struct Parser {
    b: Vec<char>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.b.get(self.i).copied()
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t') | Some('\n') | Some('\r')) {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: char) -> Result<(), JsonError> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(JsonError(format!("expected '{}' at position {}", c, self.i)))
        }
    }

    fn value(&mut self) -> Result<Value, JsonError> {
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => Ok(Value::Str(self.string()?)),
            Some('t') => self.lit("true", Value::Bool(true)),
            Some('f') => self.lit("false", Value::Bool(false)),
            Some('n') => self.lit("null", Value::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            Some(c) => Err(JsonError(format!("unexpected character '{}'", c))),
            None => Err(JsonError("unexpected end of input".into())),
        }
    }

    fn lit(&mut self, word: &str, v: Value) -> Result<Value, JsonError> {
        for c in word.chars() {
            self.eat(c)?;
        }
        Ok(v)
    }

    fn object(&mut self) -> Result<Value, JsonError> {
        self.eat('{')?;
        let mut m = BTreeMap::new();
        self.ws();
        if self.peek() == Some('}') {
            self.i += 1;
            return Ok(Value::Obj(m));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.ws();
            self.eat(':')?;
            self.ws();
            let v = self.value()?;
            m.insert(k, v);
            self.ws();
            match self.peek() {
                Some(',') => {
                    self.i += 1;
                }
                Some('}') => {
                    self.i += 1;
                    return Ok(Value::Obj(m));
                }
                _ => return Err(JsonError("expected ',' or '}' in object".into())),
            }
        }
    }

    fn array(&mut self) -> Result<Value, JsonError> {
        self.eat('[')?;
        let mut a = Vec::new();
        self.ws();
        if self.peek() == Some(']') {
            self.i += 1;
            return Ok(Value::Arr(a));
        }
        loop {
            self.ws();
            a.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(',') => {
                    self.i += 1;
                }
                Some(']') => {
                    self.i += 1;
                    return Ok(Value::Arr(a));
                }
                _ => return Err(JsonError("expected ',' or ']' in array".into())),
            }
        }
    }

    fn string(&mut self) -> Result<String, JsonError> {
        self.eat('"')?;
        let mut s = String::new();
        loop {
            let c = self.peek().ok_or_else(|| JsonError("unterminated string".into()))?;
            self.i += 1;
            match c {
                '"' => return Ok(s),
                '\\' => {
                    let e = self.peek().ok_or_else(|| JsonError("unterminated escape".into()))?;
                    self.i += 1;
                    match e {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
                        'b' => s.push('\u{8}'),
                        'f' => s.push('\u{c}'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => s.push(self.unicode_escape()?),
                        other => return Err(JsonError(format!("bad escape '\\{}'", other))),
                    }
                }
                other => s.push(other),
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, JsonError> {
        let mut v = 0u32;
        for _ in 0..4 {
            let c = self.peek().ok_or_else(|| JsonError("short \\u escape".into()))?;
            self.i += 1;
            let d = c.to_digit(16).ok_or_else(|| JsonError("bad hex digit".into()))?;
            v = v * 16 + d;
        }
        Ok(v)
    }

    /// Handles surrogate pairs so non-BMP characters survive the round trip.
    fn unicode_escape(&mut self) -> Result<char, JsonError> {
        let hi = self.hex4()?;
        if (0xD800..0xDC00).contains(&hi) {
            if self.peek() == Some('\\') {
                self.i += 1;
                self.eat('u')?;
                let lo = self.hex4()?;
                if (0xDC00..0xE000).contains(&lo) {
                    let cp = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                    return char::from_u32(cp).ok_or_else(|| JsonError("bad code point".into()));
                }
                return Err(JsonError("unpaired high surrogate".into()));
            }
            return Err(JsonError("unpaired high surrogate".into()));
        }
        char::from_u32(hi).ok_or_else(|| JsonError("bad code point".into()))
    }

    fn number(&mut self) -> Result<Value, JsonError> {
        let start = self.i;
        if self.peek() == Some('-') {
            self.i += 1;
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.i += 1;
        }
        if self.peek() == Some('.') {
            self.i += 1;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.i += 1;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.i += 1;
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        let text: String = self.b[start..self.i].iter().collect();
        text.parse::<f64>()
            .map(Value::Num)
            .map_err(|_| JsonError(format!("bad number '{}'", text)))
    }
}

pub fn write(v: &Value) -> String {
    let mut out = String::new();
    write_into(v, &mut out);
    out
}

fn write_into(v: &Value, out: &mut String) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Num(n) => {
            if n.is_finite() && *n == n.trunc() && n.abs() < 1e15 {
                let _ = write!(out, "{}", *n as i64);
            } else if n.is_finite() {
                let _ = write!(out, "{}", n);
            } else {
                out.push_str("null");
            }
        }
        Value::Str(s) => write_string(s, out),
        Value::Arr(a) => {
            out.push('[');
            for (i, item) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_into(item, out);
            }
            out.push(']');
        }
        Value::Obj(m) => {
            out.push('{');
            for (i, (k, val)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_string(k, out);
                out.push(':');
                write_into(val, out);
            }
            out.push('}');
        }
    }
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse: primitives ──────────────────────────────────────────────────────

    #[test]
    fn parse_null() {
        assert_eq!(parse("null").unwrap(), Value::Null);
    }

    #[test]
    fn parse_bool_true() {
        assert_eq!(parse("true").unwrap(), Value::Bool(true));
    }

    #[test]
    fn parse_bool_false() {
        assert_eq!(parse("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn parse_integer() {
        assert_eq!(parse("42").unwrap(), Value::Num(42.0));
    }

    #[test]
    fn parse_negative_integer() {
        assert_eq!(parse("-7").unwrap(), Value::Num(-7.0));
    }

    #[test]
    fn parse_float() {
        let v = parse("3.14").unwrap();
        if let Value::Num(n) = v {
            assert!((n - 3.14).abs() < 1e-10);
        } else {
            panic!("expected Num");
        }
    }

    #[test]
    fn parse_string_plain() {
        assert_eq!(parse(r#""hello""#).unwrap(), Value::Str("hello".into()));
    }

    #[test]
    fn parse_string_escape_sequences() {
        let v = parse(r#""\"\\\n\r\t""#).unwrap();
        assert_eq!(v, Value::Str("\"\\\n\r\t".into()));
    }

    #[test]
    fn parse_string_unicode_escape_bmp() {
        // A = 'A'
        let v = parse(r#""A""#).unwrap();
        assert_eq!(v, Value::Str("A".into()));
    }

    #[test]
    fn parse_string_surrogate_pair() {
        // 😀 = 😀 (U+1F600)
        let v = parse(r#""😀""#).unwrap();
        assert_eq!(v, Value::Str("😀".into()));
    }

    #[test]
    fn parse_empty_array() {
        assert_eq!(parse("[]").unwrap(), Value::Arr(vec![]));
    }

    #[test]
    fn parse_array_with_elements() {
        let v = parse("[1,2,3]").unwrap();
        assert_eq!(v, Value::Arr(vec![Value::Num(1.0), Value::Num(2.0), Value::Num(3.0)]));
    }

    #[test]
    fn parse_empty_object() {
        assert_eq!(parse("{}").unwrap(), Value::obj());
    }

    #[test]
    fn parse_object_with_fields() {
        let v = parse(r#"{"a":1,"b":"x"}"#).unwrap();
        assert_eq!(v.get("a"), Some(&Value::Num(1.0)));
        assert_eq!(v.get("b"), Some(&Value::Str("x".into())));
    }

    #[test]
    fn parse_whitespace_is_ignored() {
        assert_eq!(parse("  42  ").unwrap(), Value::Num(42.0));
    }

    // ── parse: error cases ─────────────────────────────────────────────────────

    #[test]
    fn parse_error_trailing_content() {
        assert!(parse("1 2").is_err(), "trailing content must be an error");
    }

    #[test]
    fn parse_error_unterminated_string() {
        assert!(parse(r#""hello"#).is_err());
    }

    #[test]
    fn parse_error_bad_escape() {
        assert!(parse(r#""\q""#).is_err());
    }

    #[test]
    fn parse_error_empty_input() {
        assert!(parse("").is_err());
    }

    #[test]
    fn parse_error_unpaired_high_surrogate() {
        assert!(parse(r#""\uD800""#).is_err(), "unpaired high surrogate must fail");
    }

    // ── write: formatting ──────────────────────────────────────────────────────

    #[test]
    fn write_null() {
        assert_eq!(write(&Value::Null), "null");
    }

    #[test]
    fn write_bool() {
        assert_eq!(write(&Value::Bool(true)), "true");
        assert_eq!(write(&Value::Bool(false)), "false");
    }

    #[test]
    fn write_integer_as_integer() {
        // Whole-number f64 values within range are written without decimal point.
        assert_eq!(write(&Value::Num(42.0)), "42");
        assert_eq!(write(&Value::Num(-7.0)), "-7");
        assert_eq!(write(&Value::Num(0.0)), "0");
    }

    #[test]
    fn write_float_preserves_decimal() {
        let s = write(&Value::Num(3.14));
        assert!(s.contains('.'), "float must include decimal point, got {}", s);
    }

    #[test]
    fn write_non_finite_as_null() {
        // JSON has no NaN/Infinity; the spec mandates serialising them as null.
        assert_eq!(write(&Value::Num(f64::NAN)), "null");
        assert_eq!(write(&Value::Num(f64::INFINITY)), "null");
        assert_eq!(write(&Value::Num(f64::NEG_INFINITY)), "null");
    }

    #[test]
    fn write_string_escapes_special_chars() {
        let s = write(&Value::Str("a\nb\tc\"d\\e".into()));
        assert_eq!(s, r#""a\nb\tc\"d\\e""#);
    }

    #[test]
    fn write_control_char_below_0x20() {
        // U+0001 must be escaped as \u0001, not emitted as a raw control char.
        let s = write(&Value::Str("\u{1}".into()));
        assert_eq!(s, "\"\\u0001\"");
    }

    #[test]
    fn write_multibyte_utf8_passthrough() {
        // Non-ASCII printable chars do not need escaping.
        let s = write(&Value::Str("héllo".into()));
        assert_eq!(s, r#""héllo""#);
    }

    #[test]
    fn write_array() {
        let v = Value::Arr(vec![Value::Num(1.0), Value::Null]);
        assert_eq!(write(&v), "[1,null]");
    }

    #[test]
    fn write_object_keys_sorted() {
        // BTreeMap ensures alphabetical key order in output.
        let mut v = Value::obj();
        v.set("z", Value::Num(2.0));
        v.set("a", Value::Num(1.0));
        let s = write(&v);
        let a_pos = s.find("\"a\"").expect("key a");
        let z_pos = s.find("\"z\"").expect("key z");
        assert!(a_pos < z_pos, "BTreeMap must produce alphabetical key order");
    }

    // ── round-trip ─────────────────────────────────────────────────────────────

    #[test]
    fn round_trip_nested_object() {
        let original = r#"{"diags":[{"msg":"oops","code":42}],"ok":true}"#;
        let parsed = parse(original).unwrap();
        let rewritten = write(&parsed);
        let reparsed = parse(&rewritten).unwrap();
        assert_eq!(parsed, reparsed, "parse → write → parse must be stable");
    }

    #[test]
    fn round_trip_string_with_escapes() {
        let s = "line1\nline2\ttab\"quote\\backslash";
        let json = write(&Value::Str(s.into()));
        let back = parse(&json).unwrap();
        assert_eq!(back, Value::Str(s.into()));
    }

    // ── Value helpers ──────────────────────────────────────────────────────────

    #[test]
    fn as_str_returns_some_for_str() {
        assert_eq!(Value::Str("x".into()).as_str(), Some("x"));
    }

    #[test]
    fn as_str_returns_none_for_non_str() {
        assert_eq!(Value::Num(1.0).as_str(), None);
    }

    #[test]
    fn as_i64_returns_int_for_whole_number() {
        assert_eq!(Value::Num(99.0).as_i64(), Some(99i64));
    }

    #[test]
    fn as_i64_returns_none_for_non_finite() {
        assert_eq!(Value::Num(f64::NAN).as_i64(), None);
    }

    #[test]
    fn as_arr_returns_some_for_arr() {
        let v = Value::Arr(vec![Value::Null]);
        assert!(v.as_arr().is_some());
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let v = Value::obj();
        assert!(v.get("missing").is_none());
    }
}

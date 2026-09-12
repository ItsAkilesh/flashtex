//! Hand-written JSON: a serializer for [`Explanation`] and a small parser
//! sufficient for runtime-v1 `compile_result` payloads and round-trips.
//!
//! Output is deterministic: object keys are written in a fixed order, numbers
//! are integers, and strings are escaped per RFC 8259 (control characters as
//! `\uXXXX`). No external crate.

use crate::{
    Category, Confidence, ContextWindow, Diagnostic, Edit, Explanation, Severity, Source,
    Suggestion,
};

// ---- writer ------------------------------------------------------------------

pub fn escape_into(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn key(out: &mut String, first: &mut bool, name: &str) {
    if !*first {
        out.push(',');
    }
    *first = false;
    escape_into(out, name);
    out.push(':');
}

fn opt_str(out: &mut String, v: Option<&str>) {
    match v {
        Some(s) => escape_into(out, s),
        None => out.push_str("null"),
    }
}

pub fn write_edit(out: &mut String, e: &Edit) {
    let mut f = true;
    out.push('{');
    key(out, &mut f, "path");
    escape_into(out, &e.path);
    key(out, &mut f, "start_byte");
    out.push_str(&e.start_byte.to_string());
    key(out, &mut f, "end_byte");
    out.push_str(&e.end_byte.to_string());
    key(out, &mut f, "replacement");
    escape_into(out, &e.replacement);
    out.push('}');
}

pub fn write_suggestion(out: &mut String, s: &Suggestion) {
    let mut f = true;
    out.push('{');
    key(out, &mut f, "text");
    escape_into(out, &s.text);
    key(out, &mut f, "confidence");
    escape_into(out, s.confidence.as_str());
    key(out, &mut f, "edits");
    out.push('[');
    for (i, e) in s.edits.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_edit(out, e);
    }
    out.push(']');
    out.push('}');
}

pub fn write_context(out: &mut String, c: &ContextWindow) {
    let mut f = true;
    out.push('{');
    key(out, &mut f, "path");
    escape_into(out, &c.path);
    key(out, &mut f, "start_byte");
    out.push_str(&c.start_byte.to_string());
    key(out, &mut f, "end_byte");
    out.push_str(&c.end_byte.to_string());
    key(out, &mut f, "text");
    escape_into(out, &c.text);
    key(out, &mut f, "span_start");
    out.push_str(&c.span_start.to_string());
    key(out, &mut f, "span_end");
    out.push_str(&c.span_end.to_string());
    key(out, &mut f, "line");
    out.push_str(&c.line.to_string());
    key(out, &mut f, "column");
    out.push_str(&c.column.to_string());
    key(out, &mut f, "span_in_bounds");
    out.push_str(if c.span_in_bounds { "true" } else { "false" });
    out.push('}');
}

pub fn write_explanation(out: &mut String, x: &Explanation) {
    let mut f = true;
    out.push('{');
    key(out, &mut f, "catalog_id");
    opt_str(out, x.catalog_id.as_deref());
    key(out, &mut f, "title");
    escape_into(out, &x.title);
    key(out, &mut f, "category");
    escape_into(out, x.category.as_str());
    key(out, &mut f, "severity");
    escape_into(out, x.severity.as_str());
    key(out, &mut f, "message");
    escape_into(out, &x.message);
    key(out, &mut f, "why");
    escape_into(out, &x.why);
    key(out, &mut f, "what_happened");
    escape_into(out, &x.what_happened);
    key(out, &mut f, "suggestions");
    out.push('[');
    for (i, s) in x.suggestions.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_suggestion(out, s);
    }
    out.push(']');
    key(out, &mut f, "context");
    match &x.context {
        Some(c) => write_context(out, c),
        None => out.push_str("null"),
    }
    out.push('}');
}

impl Explanation {
    /// Compact JSON with a fixed key order.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        write_explanation(&mut out, self);
        out
    }

    pub fn from_json(s: &str) -> Result<Explanation, JsonError> {
        let v = parse(s)?;
        explanation_from_value(&v)
    }
}

/// Serialize a batch as a JSON array.
pub fn explanations_to_json(xs: &[Explanation]) -> String {
    let mut out = String::from("[");
    for (i, x) in xs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_explanation(&mut out, x);
    }
    out.push(']');
    out
}

// ---- parser ----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, k: &str) -> Option<&Value> {
        match self {
            Value::Obj(fields) => fields.iter().find(|(name, _)| name == k).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            Value::Num(n) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as usize),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError(pub String);

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for JsonError {}

pub fn parse(s: &str) -> Result<Value, JsonError> {
    let mut p = Parser {
        b: s.as_bytes(),
        i: 0,
        s,
    };
    p.ws();
    let v = p.value(0)?;
    p.ws();
    if p.i != p.b.len() {
        return Err(p.err("trailing characters"));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
    s: &'a str,
}

const MAX_DEPTH: usize = 64;

impl Parser<'_> {
    fn err(&self, what: &str) -> JsonError {
        JsonError(format!("{} at byte {}", what, self.i))
    }

    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, JsonError> {
        if depth > MAX_DEPTH {
            return Err(self.err("nesting too deep"));
        }
        match self.b.get(self.i) {
            None => Err(self.err("unexpected end")),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string().map(Value::Str),
            Some(b't') => self.literal("true", Value::Bool(true)),
            Some(b'f') => self.literal("false", Value::Bool(false)),
            Some(b'n') => self.literal("null", Value::Null),
            Some(c) if *c == b'-' || c.is_ascii_digit() => self.number(),
            Some(_) => Err(self.err("unexpected character")),
        }
    }

    fn literal(&mut self, word: &str, v: Value) -> Result<Value, JsonError> {
        if self.s[self.i..].starts_with(word) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(self.err("invalid literal"))
        }
    }

    fn number(&mut self) -> Result<Value, JsonError> {
        let start = self.i;
        if self.b[self.i] == b'-' {
            self.i += 1;
        }
        while self.i < self.b.len()
            && (self.b[self.i].is_ascii_digit()
                || matches!(self.b[self.i], b'.' | b'e' | b'E' | b'+' | b'-'))
        {
            self.i += 1;
        }
        self.s[start..self.i]
            .parse::<f64>()
            .map(Value::Num)
            .map_err(|_| self.err("invalid number"))
    }

    fn string(&mut self) -> Result<String, JsonError> {
        self.i += 1; // opening quote
        let mut out = String::new();
        loop {
            let rest = &self.s[self.i..];
            let mut chars = rest.char_indices();
            let Some((_, c)) = chars.next() else {
                return Err(self.err("unterminated string"));
            };
            match c {
                '"' => {
                    self.i += 1;
                    return Ok(out);
                }
                '\\' => {
                    let Some((_, e)) = chars.next() else {
                        return Err(self.err("unterminated escape"));
                    };
                    self.i += 2;
                    match e {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{08}'),
                        'f' => out.push('\u{0C}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp) {
                                // Surrogate pair.
                                if !self.s[self.i..].starts_with("\\u") {
                                    return Err(self.err("lone high surrogate"));
                                }
                                self.i += 2;
                                let lo = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return Err(self.err("invalid low surrogate"));
                                }
                                let c = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                out.push(
                                    char::from_u32(c)
                                        .ok_or_else(|| self.err("invalid code point"))?,
                                );
                            } else {
                                out.push(
                                    char::from_u32(cp)
                                        .ok_or_else(|| self.err("invalid code point"))?,
                                );
                            }
                        }
                        _ => return Err(self.err("invalid escape")),
                    }
                }
                c if (c as u32) < 0x20 => return Err(self.err("control character in string")),
                c => {
                    out.push(c);
                    self.i += c.len_utf8();
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, JsonError> {
        let hex = self
            .s
            .get(self.i..self.i + 4)
            .ok_or_else(|| self.err("short \\u escape"))?;
        let v = u32::from_str_radix(hex, 16).map_err(|_| self.err("invalid \\u escape"))?;
        self.i += 4;
        Ok(v)
    }

    fn array(&mut self, depth: usize) -> Result<Value, JsonError> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Value::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value(depth + 1)?);
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Value::Arr(items));
                }
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<Value, JsonError> {
        self.i += 1;
        let mut fields = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Value::Obj(fields));
        }
        loop {
            self.ws();
            if self.b.get(self.i) != Some(&b'"') {
                return Err(self.err("expected string key"));
            }
            let k = self.string()?;
            self.ws();
            if self.b.get(self.i) != Some(&b':') {
                return Err(self.err("expected ':'"));
            }
            self.i += 1;
            self.ws();
            let v = self.value(depth + 1)?;
            fields.push((k, v));
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Value::Obj(fields));
                }
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
    }
}

// ---- runtime-v1 diagnostics --------------------------------------------------

fn field<'a>(v: &'a Value, k: &str) -> Result<&'a Value, JsonError> {
    v.get(k)
        .ok_or_else(|| JsonError(format!("missing field '{k}'")))
}

fn str_field(v: &Value, k: &str) -> Result<String, JsonError> {
    field(v, k)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| JsonError(format!("field '{k}' must be a string")))
}

fn usize_field(v: &Value, k: &str) -> Result<usize, JsonError> {
    field(v, k)?
        .as_usize()
        .ok_or_else(|| JsonError(format!("field '{k}' must be a non-negative integer")))
}

pub fn diagnostic_from_value(v: &Value) -> Result<Diagnostic, JsonError> {
    let severity = str_field(v, "severity")?;
    let severity = Severity::parse(&severity)
        .ok_or_else(|| JsonError(format!("unknown severity '{severity}'")))?;
    let message = str_field(v, "message")?;
    let source = match v.get("source") {
        None | Some(Value::Null) => None,
        Some(s) => Some(Source {
            path: str_field(s, "path")?,
            start_byte: usize_field(s, "start_byte")?,
            end_byte: usize_field(s, "end_byte")?,
        }),
    };
    let recovery = match v.get("recovery") {
        None | Some(Value::Null) => None,
        Some(Value::Str(r)) => Some(r.clone()),
        Some(_) => {
            return Err(JsonError(
                "field 'recovery' must be a string or null".into(),
            ));
        }
    };
    Ok(Diagnostic {
        severity,
        message,
        source,
        recovery,
    })
}

/// Diagnostics of a `compile_result`, accepting either the full envelope
/// (`{"type":"compile_result","payload":{...}}`) or the bare payload.
pub fn diagnostics_from_compile_result(json: &str) -> Result<Vec<Diagnostic>, JsonError> {
    let v = parse(json)?;
    let payload = match v.get("payload") {
        Some(p) if v.get("diagnostics").is_none() => p,
        _ => &v,
    };
    let diags = field(payload, "diagnostics")?
        .as_arr()
        .ok_or_else(|| JsonError("'diagnostics' must be an array".into()))?;
    diags.iter().map(diagnostic_from_value).collect()
}

// ---- Explanation from JSON (round-trip) ---------------------------------------------

fn edit_from_value(v: &Value) -> Result<Edit, JsonError> {
    Ok(Edit {
        path: str_field(v, "path")?,
        start_byte: usize_field(v, "start_byte")?,
        end_byte: usize_field(v, "end_byte")?,
        replacement: str_field(v, "replacement")?,
    })
}

fn suggestion_from_value(v: &Value) -> Result<Suggestion, JsonError> {
    let confidence = str_field(v, "confidence")?;
    Ok(Suggestion {
        text: str_field(v, "text")?,
        confidence: Confidence::parse(&confidence)
            .ok_or_else(|| JsonError(format!("unknown confidence '{confidence}'")))?,
        edits: field(v, "edits")?
            .as_arr()
            .ok_or_else(|| JsonError("'edits' must be an array".into()))?
            .iter()
            .map(edit_from_value)
            .collect::<Result<_, _>>()?,
    })
}

fn context_from_value(v: &Value) -> Result<ContextWindow, JsonError> {
    Ok(ContextWindow {
        path: str_field(v, "path")?,
        start_byte: usize_field(v, "start_byte")?,
        end_byte: usize_field(v, "end_byte")?,
        text: str_field(v, "text")?,
        span_start: usize_field(v, "span_start")?,
        span_end: usize_field(v, "span_end")?,
        line: usize_field(v, "line")?,
        column: usize_field(v, "column")?,
        span_in_bounds: field(v, "span_in_bounds")?
            .as_bool()
            .ok_or_else(|| JsonError("'span_in_bounds' must be a boolean".into()))?,
    })
}

pub fn explanation_from_value(v: &Value) -> Result<Explanation, JsonError> {
    let category = str_field(v, "category")?;
    let severity = str_field(v, "severity")?;
    Ok(Explanation {
        catalog_id: match v.get("catalog_id") {
            None | Some(Value::Null) => None,
            Some(Value::Str(s)) => Some(s.clone()),
            Some(_) => return Err(JsonError("'catalog_id' must be a string or null".into())),
        },
        title: str_field(v, "title")?,
        category: Category::parse(&category)
            .ok_or_else(|| JsonError(format!("unknown category '{category}'")))?,
        severity: Severity::parse(&severity)
            .ok_or_else(|| JsonError(format!("unknown severity '{severity}'")))?,
        message: str_field(v, "message")?,
        why: str_field(v, "why")?,
        what_happened: str_field(v, "what_happened")?,
        suggestions: field(v, "suggestions")?
            .as_arr()
            .ok_or_else(|| JsonError("'suggestions' must be an array".into()))?
            .iter()
            .map(suggestion_from_value)
            .collect::<Result<_, _>>()?,
        context: match v.get("context") {
            None | Some(Value::Null) => None,
            Some(c) => Some(context_from_value(c)?),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handles_escapes_and_nesting() {
        let v = parse(r#"{"a":[1,2.5,-3,true,null],"s":"x\"\\\né😀"}"#).unwrap();
        assert_eq!(v.get("s").unwrap().as_str(), Some("x\"\\\né😀"));
        assert_eq!(v.get("a").unwrap().as_arr().unwrap().len(), 5);
        assert!(parse("{").is_err());
        assert!(parse("[1,]").is_err());
        assert!(parse("\"\u{01}\"").is_err());
    }

    #[test]
    fn escape_round_trips_control_characters() {
        let mut out = String::new();
        escape_into(&mut out, "a\u{01}b\t\"\\");
        assert_eq!(out, "\"a\\u0001b\\t\\\"\\\\\"");
        assert_eq!(parse(&out).unwrap().as_str(), Some("a\u{01}b\t\"\\"));
    }
}

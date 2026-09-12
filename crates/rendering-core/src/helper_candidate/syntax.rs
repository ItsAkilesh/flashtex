//! Narrow reuse of the existing runtime serde visitor from 7817e4e8,
//! crates/document-runtime/src/raw_display.rs. Serde owns JSON syntax/depth.
use serde::{
    de::{IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

// Delegate tokenization, escapes, number range and depth checks to serde_json.
pub(super) struct Syntax;
impl<'de> Deserialize<'de> for Syntax {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Check;
        impl<'de> Visitor<'de> for Check {
            type Value = Syntax;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("valid JSON")
            }
            fn visit_bool<E>(self, _: bool) -> Result<Syntax, E> {
                Ok(Syntax)
            }
            fn visit_i64<E>(self, _: i64) -> Result<Syntax, E> {
                Ok(Syntax)
            }
            fn visit_u64<E>(self, _: u64) -> Result<Syntax, E> {
                Ok(Syntax)
            }
            fn visit_f64<E: serde::de::Error>(self, n: f64) -> Result<Syntax, E> {
                if n.is_finite() {
                    Ok(Syntax)
                } else {
                    Err(E::custom("nonfinite JSON number"))
                }
            }
            fn visit_str<E>(self, _: &str) -> Result<Syntax, E> {
                Ok(Syntax)
            }
            fn visit_unit<E>(self) -> Result<Syntax, E> {
                Ok(Syntax)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Syntax, A::Error> {
                while a.next_element::<Syntax>()?.is_some() {}
                Ok(Syntax)
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Syntax, A::Error> {
                while a.next_key::<IgnoredAny>()?.is_some() {
                    a.next_value::<Syntax>()?;
                }
                Ok(Syntax)
            }
        }
        d.deserialize_any(Check)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn syntax_matches_prior_value_gate_on_nested_and_escaped_fields() {
        let mut cases: Vec<String> = [
            "null",
            "true",
            "false",
            "0",
            "-0",
            "1.0",
            "1e0",
            "1e400",
            "-1e400",
            "18446744073709551616",
            "1e-400",
            "[1,true,null]",
            "{\"x\":1,\"x\":2}",
            r#"{"\ud800":0}"#,
            r#"{"\udc00":0}"#,
            r#"{"x":"\ud800"}"#,
            r#"{"extension":{"\ud800":0}}"#,
            r#"{"extension":{"x":"\ud800"}}"#,
            r#"{"\ud83d\ude00":"\u0061"}"#,
            r#"{"x":"\uZZZZ"}"#,
            "[0,]",
            "{}{}",
            "{1:2}",
            "{\"x\":NaN}",
            "\"é\"",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        for depth in [1, 125, 126, 127, 128, 129, 130] {
            cases.push(format!("{}0{}", "[".repeat(depth), "]".repeat(depth)));
            cases.push(format!("{}0{}", "{\"x\":".repeat(depth), "}".repeat(depth)));
        }
        for raw in [b"\"\xff\"".as_slice(), b"{\"\xff\":0}".as_slice()] {
            assert!(serde_json::from_slice::<Syntax>(raw).is_err());
            assert!(serde_json::from_slice::<serde_json::Value>(raw).is_err());
        }
        for raw in cases {
            assert_eq!(
                serde_json::from_str::<Syntax>(&raw).is_ok(),
                serde_json::from_str::<serde_json::Value>(&raw).is_ok(),
                "{raw}"
            );
        }
    }
}

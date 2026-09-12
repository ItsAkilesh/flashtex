//! Experimental syntax-checked raw transport. Rendering fields remain untrusted.
use crate::{Request, SourceBinding};
use serde::{
    de::{IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::value::RawValue;
use std::fmt;

// Delegate tokenization, escapes, number range and depth checks to serde_json.
struct Syntax;
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
// Deserialize recognized metadata and validate all opaque values in one serde pass.
// Field order is unrestricted. Serde owns JSON tokenization and recursion limits.
macro_rules! checked_object {
    ($name:ident { $($field:ident : $ty:ty => $key:literal),* $(,)? }) => {
        struct $name { $( $field: Option<$ty> ),* }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D:Deserializer<'de>>(deserializer:D)->Result<Self,D::Error>{
                struct Object;
                impl<'de> Visitor<'de> for Object {
                    type Value=$name;
                    fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result { f.write_str(stringify!($name)) }
                    fn visit_map<A:MapAccess<'de>>(self,mut map:A)->Result<$name,A::Error>{
                        $(let mut $field=None;)*
                        while let Some(key)=map.next_key::<String>()? {
                            match key.as_str() {
                                $($key => {
                                    if $field.is_some(){return Err(serde::de::Error::duplicate_field($key));}
                                    $field=Some(map.next_value::<$ty>()?);
                                },)*
                                _=>{map.next_value::<Syntax>()?;}
                            }
                        }
                        Ok($name{$($field),*})
                    }
                }
                deserializer.deserialize_map(Object)
            }
        }
    }
}
checked_object!(WireEnvelope { protocol_version:u64=>"protocol_version", id:String=>"id", kind:String=>"type", payload:WirePayload=>"payload" });
checked_object!(WirePayload { project_id:String=>"project_id", revision:u64=>"revision", render_format:String=>"render_format", documents:Vec<WireDocument> =>"documents" });
checked_object!(WireDocument { path:String=>"path", revision:u64=>"revision", sha256:String=>"sha256", byte_length:u64=>"byte_length" });
struct Envelope {
    protocol_version: u64,
    id: String,
    kind: String,
    payload: Payload,
}
struct Payload {
    project_id: String,
    revision: u64,
    render_format: String,
    documents: Vec<Document>,
}
struct Document {
    path: String,
    revision: u64,
    sha256: String,
    byte_length: u64,
}
impl WireEnvelope {
    fn required(self) -> Result<Envelope, String> {
        let p = self.payload.ok_or("missing raw payload")?;
        let documents = p
            .documents
            .ok_or("missing raw documents")?
            .into_iter()
            .map(|d| {
                Ok(Document {
                    path: d.path.ok_or("missing raw path")?,
                    revision: d.revision.ok_or("missing raw document revision")?,
                    sha256: d.sha256.ok_or("missing raw sha256")?,
                    byte_length: d.byte_length.ok_or("missing raw byte_length")?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Envelope {
            protocol_version: self.protocol_version.ok_or("missing raw protocol")?,
            id: self.id.ok_or("missing raw id")?,
            kind: self.kind.ok_or("missing raw type")?,
            payload: Payload {
                project_id: p.project_id.ok_or("missing raw project")?,
                revision: p.revision.ok_or("missing raw revision")?,
                render_format: p.render_format.ok_or("missing raw format")?,
                documents,
            },
        })
    }
}
#[derive(Debug)]
pub struct UntrustedRawDisplayCandidate {
    request_id: String,
    project_id: String,
    revision: u64,
    sources: Vec<SourceBinding>,
    raw: Box<RawValue>,
}
impl UntrustedRawDisplayCandidate {
    pub fn request_id(&self) -> &str {
        &self.request_id
    }
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn sources(&self) -> &[SourceBinding] {
        &self.sources
    }
    pub fn raw(&self) -> &RawValue {
        &self.raw
    }
    pub fn into_raw(self) -> Box<RawValue> {
        self.raw
    }
}
pub(crate) struct Parsed {
    envelope: Envelope,
    raw: Box<RawValue>,
}
#[cfg(test)]
pub(crate) fn is_display(bytes: &[u8]) -> Result<bool, String> {
    serde_json::from_slice::<WireEnvelope>(bytes)
        .map(|e| e.protocol_version == Some(2) && e.kind.as_deref() == Some("display_list"))
        .map_err(|e| e.to_string())
}
pub(crate) type Decoded = (Option<serde_json::Value>, Option<Box<Parsed>>);
pub(crate) fn decode(bytes: Vec<u8>) -> Result<Decoded, String> {
    let text = String::from_utf8(bytes).map_err(|_| "invalid UTF8")?;
    let wire: WireEnvelope =
        serde_json::from_str(&text).map_err(|e| format!("raw syntax and metadata: {e}"))?;
    if wire.protocol_version == Some(2) && wire.kind.as_deref() == Some("display_list") {
        let envelope = wire.required()?;
        let raw = RawValue::from_string(text).map_err(|e| e.to_string())?;
        Ok((None, Some(Box::new(Parsed { envelope, raw }))))
    } else {
        serde_json::from_str(&text)
            .map(|v| (Some(v), None))
            .map_err(|e| e.to_string())
    }
}
impl Parsed {
    #[cfg(test)]
    pub fn parse(bytes: Vec<u8>) -> Result<Self, String> {
        decode(bytes)?
            .1
            .map(|p| *p)
            .ok_or("not a raw display envelope".into())
    }
    pub fn validate(self, request: &Request) -> Result<UntrustedRawDisplayCandidate, String> {
        let e = self.envelope;
        let p = e.payload;
        if e.protocol_version != 2
            || e.kind != "display_list"
            || e.id != request.id
            || p.project_id != request.project_id
            || p.revision != request.revision
            || p.render_format != "display-list-v2"
        {
            return Err("raw sibling correlation mismatch".into());
        }
        if p.documents.len() != request.documents.len() {
            return Err("raw source set mismatch".into());
        }
        let map: std::collections::BTreeMap<_, _> = request
            .documents
            .iter()
            .map(|d| (d.path.as_str(), d.text.as_str()))
            .collect();
        let mut seen = std::collections::BTreeSet::new();
        let mut sources = Vec::with_capacity(p.documents.len());
        for d in p.documents {
            if !seen.insert(d.path.clone()) {
                return Err("duplicate raw source path".into());
            }
            let source = map.get(d.path.as_str()).ok_or("unknown raw source path")?;
            if d.revision != request.revision
                || d.byte_length != source.len() as u64
                || d.sha256 != flashtex_project_files::sha256_hex(source.as_bytes())
            {
                return Err("raw source identity mismatch".into());
            }
            sources.push(SourceBinding {
                path: d.path,
                revision: d.revision,
                sha256: d.sha256,
                byte_length: source.len(),
            });
        }
        Ok(UntrustedRawDisplayCandidate {
            request_id: e.id,
            project_id: p.project_id,
            revision: p.revision,
            sources,
            raw: self.raw,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Vec<u8>, Request) {
        let request: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/display-producer-request.json"))
                .unwrap();
        let text = request["payload"]["documents"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        let r = Request {
            id: "real-1".into(),
            project_id: "p".into(),
            revision: 1,
            entry_path: "main.tex".into(),
            documents: vec![crate::Document {
                path: "main.tex".into(),
                text,
            }],
        };
        (
            include_bytes!("../benchmarks/display-producer-65dbe7d/requested.stdout.jsonl")
                .split(|b| *b == b'\n')
                .nth(1)
                .unwrap()
                .to_vec(),
            r,
        )
    }
    #[test]
    fn exact_bytes_metadata_escapes_and_opaque_duplicates() {
        let (bytes, r) = fixture();
        let text = String::from_utf8(bytes).unwrap();
        for input in [
            text.clone(),
            text.replace("main.tex", "main\\u002etex")
                .replace("Office", "Off\\u0069ce"),
            text.replacen("\"glyph_count\":", "\"glyph_count\":0,\"glyph_count\":", 1),
        ] {
            let c = Parsed::parse(input.as_bytes().to_vec())
                .unwrap()
                .validate(&r)
                .unwrap();
            assert_eq!(c.raw().get(), input);
            assert_eq!(c.request_id(), r.id);
            assert_eq!(c.sources()[0].path, "main.tex");
        }
    }
    #[test]
    fn duplicate_bindings_discriminators_numbers_depth_and_trailing_refuse() {
        let (bytes, r) = fixture();
        let text = String::from_utf8(bytes).unwrap();
        for key in [
            "id",
            "project_id",
            "revision",
            "render_format",
            "documents",
            "path",
            "sha256",
            "byte_length",
        ] {
            let needle = format!("\"{key}\":");
            let bad = text.replacen(&needle, &format!("{needle}null,{needle}"), 1);
            assert!(Parsed::parse(bad.into_bytes()).is_err(), "{key}");
        }
        for input in [
            text.replacen(
                "\"protocol_version\":2",
                "\"protocol_version\":1,\"protocol_version\":2",
                1,
            ),
            text.replacen("\"type\":", "\"type\":\"bogus\",\"type\":", 1),
        ] {
            assert!(is_display(input.as_bytes()).is_err());
        }
        let inputs = [
            format!("{text} {{}}"),
            text.replacen("\"revision\":1", "\"revision\":1.0", 1),
            text.replacen("\"revision\":1", "\"revision\":1e0", 1),
            text.replacen("\"revision\":1", "\"revision\":18446744073709551616", 1),
            text.replacen("\"payload\":", "\"id\":\"conflict\",\"payload\":", 1),
            text.replacen("\"pages\":", "\"extra\":1e400,\"pages\":", 1),
            text.replacen(
                "\"pages\":",
                &format!(
                    "\"extra\":{}0{},\"pages\":",
                    "[".repeat(130),
                    "]".repeat(130)
                ),
                1,
            ),
            text.replacen("\"id\":", "\"\\u0069d\":\"conflict\",\"id\":", 1),
        ];
        for input in inputs {
            assert!(Parsed::parse(input.into_bytes()).is_err());
        }
        let changed = text.replacen("\"byte_length\":72", "\"byte_length\":1", 1);
        if changed != text {
            assert!(Parsed::parse(changed.into_bytes())
                .unwrap()
                .validate(&r)
                .is_err());
        }
    }
    #[test]
    fn nested_unicode_key_validation_matches_value_without_normalization() {
        let (bytes, request) = fixture();
        let text = String::from_utf8(bytes).unwrap();
        for (key, valid) in [(r"\uD800", false), (r"\uD800\uDC00", true)] {
            let input = text.replacen(
                "\"pages\":",
                &format!("\"unknown\":{{\"{key}\":0}},\"pages\":"),
                1,
            );
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&input).is_ok(),
                valid
            );
            assert_eq!(Parsed::parse(input.as_bytes().to_vec()).is_ok(), valid);
            if valid {
                assert_eq!(
                    Parsed::parse(input.as_bytes().to_vec())
                        .unwrap()
                        .validate(&request)
                        .unwrap()
                        .raw()
                        .get(),
                    input
                );
            }
        }
    }
    #[test]
    fn payload_first_order_and_max_exact_revision_preserve_bindings() {
        let (bytes, mut request) = fixture();
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let maximum = 9_007_199_254_740_991u64;
        request.revision = maximum;
        v["payload"]["revision"] = maximum.into();
        v["payload"]["documents"][0]["revision"] = maximum.into();
        let input = format!(
            "{{\"payload\":{},\"id\":{},\"type\":\"display_list\",\"protocol_version\":2}}",
            v["payload"], v["id"]
        );
        let candidate = Parsed::parse(input.as_bytes().to_vec())
            .unwrap()
            .validate(&request)
            .unwrap();
        assert_eq!(candidate.revision(), maximum);
        assert_eq!(candidate.raw().get(), input);
        let invalid = input.replacen(
            &format!("\"byte_length\":{}", request.documents[0].text.len()),
            "\"byte_length\":1",
            1,
        );
        assert_ne!(invalid, input);
        assert!(Parsed::parse(invalid.into_bytes())
            .unwrap()
            .validate(&request)
            .is_err());
    }
}

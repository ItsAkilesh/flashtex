//! Experimental syntax-checked raw transport. Rendering fields remain untrusted.
use crate::{Request, SourceBinding};
use serde::de::DeserializeSeed;
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
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MetadataBudget {
    pub documents: usize,
    pub path_bytes: usize,
}
impl MetadataBudget {
    pub fn from_request(r: &Request) -> Self {
        Self {
            documents: r.documents.len(),
            path_bytes: r.documents.iter().map(|d| d.path.len()).max().unwrap_or(0),
        }
    }
}
struct Text {
    value: Option<String>,
}
struct TextSeed {
    limit: usize,
}
impl<'de> DeserializeSeed<'de> for TextSeed {
    type Value = Text;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Text, D::Error> {
        struct Capture(usize);
        impl Visitor<'_> for Capture {
            type Value = Text;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("metadata string")
            }
            fn visit_str<E>(self, s: &str) -> Result<Text, E> {
                Ok(Text {
                    value: (s.len() <= self.0).then(|| s.to_owned()),
                })
            }
        }
        d.deserialize_str(Capture(self.limit))
    }
}
// Serde handles syntax and decoded duplicate keys; seeds cap retained metadata.
macro_rules! checked_object {
    ($name:ident,$seed:ident,$budget:ident { $($field:ident : $ty:ty => $key:literal => $field_seed:expr),* $(,)? }) => {
        struct $name { $( $field: Option<$ty> ),* }
        struct $seed(MetadataBudget);
        impl<'de> DeserializeSeed<'de> for $seed {
            type Value=$name;
            fn deserialize<D:Deserializer<'de>>(self,deserializer:D)->Result<Self::Value,D::Error>{
                struct Object(MetadataBudget);
                impl<'de> Visitor<'de> for Object {
                    type Value=$name;
                    fn expecting(&self,f:&mut fmt::Formatter)->fmt::Result {f.write_str(stringify!($name))}
                    fn visit_map<A:MapAccess<'de>>(self,mut map:A)->Result<$name,A::Error>{
                        let $budget=self.0;$(let mut $field=None;)*
                        while let Some(key)=map.next_key::<String>()? {
                            match key.as_str() {
                                $($key => {if $field.is_some(){return Err(serde::de::Error::duplicate_field($key));}
                                    $field=Some(map.next_value_seed($field_seed)?);},)*
                                _=>{map.next_value::<Syntax>()?;}
                            }
                        }
                        Ok($name{$($field),*})
                    }
                }
                deserializer.deserialize_map(Object(self.0))
            }
        }
    }
}
checked_object!(WireEnvelope,EnvelopeSeed,budget {
 protocol_version:u64=>"protocol_version"=>std::marker::PhantomData::<u64>,
 id:Text=>"id"=>TextSeed{limit:128}, kind:Text=>"type"=>TextSeed{limit:14},
 payload:WirePayload=>"payload"=>PayloadSeed(budget)
});
checked_object!(WirePayload,PayloadSeed,budget {
 project_id:Text=>"project_id"=>TextSeed{limit:128}, revision:u64=>"revision"=>std::marker::PhantomData::<u64>,
 render_format:Text=>"render_format"=>TextSeed{limit:15}, documents:Documents=>"documents"=>DocumentsSeed(budget)
});
checked_object!(WireDocument,DocumentSeed,budget {
 path:Text=>"path"=>TextSeed{limit:budget.path_bytes}, revision:u64=>"revision"=>std::marker::PhantomData::<u64>,
 sha256:Text=>"sha256"=>TextSeed{limit:64}, byte_length:u64=>"byte_length"=>std::marker::PhantomData::<u64>
});
struct Documents {
    entries: Vec<Document>,
    invalid: bool,
}
struct DocumentsSeed(MetadataBudget);
impl<'de> DeserializeSeed<'de> for DocumentsSeed {
    type Value = Documents;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Documents, D::Error> {
        struct Entries(MetadataBudget);
        impl<'de> Visitor<'de> for Entries {
            type Value = Documents;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("documents array")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Documents, A::Error> {
                let mut entries = Vec::new();
                let mut invalid = false;
                let mut count = 0usize;
                while let Some(d) = seq.next_element_seed(DocumentSeed(self.0))? {
                    count = count.saturating_add(1);
                    match d.complete() {
                        Some(d) if count <= self.0.documents => entries.push(d),
                        _ => invalid = true,
                    }
                }
                Ok(Documents { entries, invalid })
            }
        }
        d.deserialize_seq(Entries(self.0))
    }
}
impl WireDocument {
    fn complete(self) -> Option<Document> {
        Some(Document {
            path: self.path?.value?,
            revision: self.revision?,
            sha256: self.sha256?.value?,
            byte_length: self.byte_length?,
        })
    }
}
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
        let documents = p.documents.ok_or("missing raw documents")?;
        if documents.invalid {
            return Err("raw metadata exceeds request budget or lacks required fields".into());
        }
        Ok(Envelope {
            protocol_version: self.protocol_version.ok_or("missing raw protocol")?,
            id: self
                .id
                .and_then(|v| v.value)
                .ok_or("missing/overlong raw id")?,
            kind: self
                .kind
                .and_then(|v| v.value)
                .ok_or("missing/overlong raw type")?,
            payload: Payload {
                project_id: p
                    .project_id
                    .and_then(|v| v.value)
                    .ok_or("missing/overlong raw project")?,
                revision: p.revision.ok_or("missing raw revision")?,
                render_format: p
                    .render_format
                    .and_then(|v| v.value)
                    .ok_or("missing/overlong raw format")?,
                documents: documents.entries,
            },
        })
    }
}
fn metadata(text: &str, budget: MetadataBudget) -> Result<WireEnvelope, String> {
    let mut d = serde_json::Deserializer::from_str(text);
    let result = EnvelopeSeed(budget)
        .deserialize(&mut d)
        .map_err(|e| format!("raw syntax and metadata: {e}"))?;
    d.end().map_err(|e| e.to_string())?;
    Ok(result)
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
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    metadata(text, MetadataBudget::default()).map(|e| {
        e.protocol_version == Some(2)
            && e.kind.and_then(|v| v.value).as_deref() == Some("display_list")
    })
}
pub(crate) type Decoded = (Option<serde_json::Value>, Option<Box<Parsed>>);
pub(crate) fn decode(bytes: Vec<u8>, budget: MetadataBudget) -> Result<Decoded, String> {
    let text = String::from_utf8(bytes).map_err(|_| "invalid UTF8")?;
    let wire = metadata(&text, budget)?;
    if wire.protocol_version == Some(2)
        && wire.kind.as_ref().and_then(|v| v.value.as_deref()) == Some("display_list")
    {
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
        decode(
            bytes,
            MetadataBudget {
                documents: 4096,
                path_bytes: 4096,
            },
        )?
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
    #[test]
    fn request_budget_bounds_retention_without_weakening_overflow_validation() {
        let empty = MetadataBudget::default();
        let docs = "{},".repeat(50_000) + "{}";
        let v1 = format!(
            r#"{{"payload":{{"documents":[{docs}]}},"type":"compile_result","protocol_version":1}}"#
        );
        let scanned = metadata(&v1, empty).unwrap();
        let captured = scanned.payload.unwrap().documents.unwrap();
        assert!(captured.entries.is_empty());
        assert!(captured.invalid);
        let (value, raw) = decode(v1.as_bytes().to_vec(), empty).unwrap();
        assert!(raw.is_none());
        assert_eq!(
            value.unwrap(),
            serde_json::from_str::<serde_json::Value>(&v1).unwrap()
        );
        let v2 = v1
            .replace("compile_result", "display_list")
            .replace("\"protocol_version\":1", "\"protocol_version\":2");
        assert!(decode(v2.into_bytes(), empty).is_err());
        for entry in [
            r#"{"revision":"wrong"}"#,
            r#"{"path":"a","path":"b"}"#,
            r#"{"opaque":1e400}"#,
        ] {
            let input =
                format!(r#"{{"payload":{{"documents":[{{}},{entry}]}},"protocol_version":1}}"#);
            assert!(decode(input.into_bytes(), empty).is_err());
        }
    }
    #[test]
    fn exact_request_budget_and_long_identity_refusal() {
        let (bytes, request) = fixture();
        let budget = MetadataBudget::from_request(&request);
        assert_eq!(budget.documents, 1);
        assert_eq!(budget.path_bytes, 8);
        assert!(decode(bytes.clone(), MetadataBudget::default()).is_err());
        let candidate = decode(bytes.clone(), budget)
            .unwrap()
            .1
            .unwrap()
            .validate(&request)
            .unwrap();
        assert_eq!(candidate.sources().len(), 1);
        let text = String::from_utf8(bytes).unwrap();
        let long = text.replacen("real-1", &"x".repeat(524288), 1);
        assert!(decode(long.into_bytes(), budget).is_err());
        assert!(decode(text.into_bytes(), budget).unwrap().1.is_some());
    }
    #[test]
    fn budget_does_not_adopt_renderer_path_limit() {
        let path = "a".repeat(5000) + ".tex";
        let request = Request {
            id: "r".into(),
            project_id: "p".into(),
            revision: 1,
            entry_path: path.clone(),
            documents: vec![crate::Document {
                path: path.clone(),
                text: "a".into(),
            }],
        };
        let frame = serde_json::json!({"protocol_version":2,"type":"display_list","id":"r","payload":{"project_id":"p","revision":1,"render_format":"display-list-v2","documents":[{"path":path,"revision":1,"sha256":flashtex_project_files::sha256_hex(b"a"),"byte_length":1}]}});
        let parsed = decode(
            serde_json::to_vec(&frame).unwrap(),
            MetadataBudget::from_request(&request),
        )
        .unwrap()
        .1
        .unwrap();
        assert_eq!(
            parsed.validate(&request).unwrap().sources()[0].path.len(),
            5004
        );
    }
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

//! Conversion-provider seam: which provider `capture_convert` uses, how its
//! credential/model/endpoint are resolved, and the evidence each call leaves.
//!
//! A provider is only ever enabled by an explicit command-line flag
//! (`--conversion-provider <name>`, or the one-release alias `--enable-grok`).
//! The bridge never reads `FLASHTEX_CONVERSION_PROVIDER` itself, so no
//! environment variable alone can make it reach a network. Everything else is
//! resolved per request from the environment the Mac's credential adapter
//! supplies:
//!
//! | variable | xai | openai-compatible |
//! |---|---|---|
//! | `FLASHTEX_AI_API_KEY` | key (then legacy `XAI_API_KEY`) | key; optional for a loopback base URL |
//! | `FLASHTEX_CONVERSION_MODEL` | model (then legacy `FLASHTEX_GROK_MODEL`, then `grok-4.6`) | model, required |
//! | `FLASHTEX_CONVERSION_BASE_URL` | default `https://api.x.ai/v1` | required |
//!
//! A future on-device provider (docs/design/on-device-conversion.md) implements
//! the same `Converter` trait and adds one `ProviderKind` arm; it needs no key
//! and no base URL.
use crate::{grok, openai, BridgeError, Converter, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const KEY_ENV: &str = "FLASHTEX_AI_API_KEY";
pub const LEGACY_XAI_KEY_ENV: &str = "XAI_API_KEY";
pub const MODEL_ENV: &str = "FLASHTEX_CONVERSION_MODEL";
pub const LEGACY_GROK_MODEL_ENV: &str = "FLASHTEX_GROK_MODEL";
pub const BASE_URL_ENV: &str = "FLASHTEX_CONVERSION_BASE_URL";
const MAX_BASE_URL_BYTES: usize = 512;
const MAX_KEY_BYTES: usize = 8192;

/// The providers this bridge can drive. `none` is represented by the absence
/// of a `ProviderKind`, never by a variant, so "disabled" cannot be matched
/// into a network path by mistake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// xAI Responses API (`POST {base}/responses`), the existing Grok path.
    Xai,
    /// Any OpenAI-compatible Chat Completions endpoint
    /// (`POST {base}/chat/completions`): hosted APIs or a local model server.
    OpenAiCompatible,
}

impl ProviderKind {
    /// Parses a `--conversion-provider` value. `Ok(None)` is `none`.
    pub fn parse(name: &str) -> Result<Option<ProviderKind>> {
        match name {
            "none" => Ok(None),
            "xai" | "grok" => Ok(Some(ProviderKind::Xai)),
            "openai-compatible" | "openai" => Ok(Some(ProviderKind::OpenAiCompatible)),
            _ => Err(BridgeError::new(
                "invalid_arguments",
                "--conversion-provider must be none, xai or openai-compatible",
            )),
        }
    }

    /// The canonical name, as reported in evidence and `--help`.
    pub fn name(self) -> &'static str {
        match self {
            ProviderKind::Xai => "xai",
            ProviderKind::OpenAiCompatible => "openai-compatible",
        }
    }
}

/// Resolved configuration for one conversion. `Debug` never prints the key.
#[derive(Clone, PartialEq, Eq)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    pub model: String,
    pub base_url: String,
    key: Option<String>,
}

impl std::fmt::Debug for ProviderConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("kind", &self.kind)
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .field("key", &self.key.as_ref().map(|_| "present"))
            .finish()
    }
}

impl ProviderConfig {
    /// Resolves the configuration from `lookup` (the process environment in
    /// production, a fixture map in tests). Empty values count as absent.
    pub fn resolve(kind: ProviderKind, lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let get = |name: &str| lookup(name).filter(|v| !v.trim().is_empty());
        let base_url = match get(BASE_URL_ENV) {
            Some(raw) => validate_base_url(&raw)?,
            None => match kind {
                ProviderKind::Xai => grok::DEFAULT_BASE_URL.to_string(),
                ProviderKind::OpenAiCompatible => {
                    return Err(BridgeError::new(
                        "provider_base_url_missing",
                        "Set FLASHTEX_CONVERSION_BASE_URL for the openai-compatible provider",
                    ))
                }
            },
        };
        let model = match (get(MODEL_ENV), kind) {
            (Some(model), _) => model,
            (None, ProviderKind::Xai) => {
                get(LEGACY_GROK_MODEL_ENV).unwrap_or_else(|| grok::DEFAULT_MODEL.to_string())
            }
            (None, ProviderKind::OpenAiCompatible) => {
                return Err(BridgeError::new(
                    "invalid_model",
                    "Set FLASHTEX_CONVERSION_MODEL for the openai-compatible provider",
                ))
            }
        };
        if !grok::valid_model(&model) {
            return Err(BridgeError::new(
                "invalid_model",
                "Configure a valid conversion model ID",
            ));
        }
        let key = match kind {
            ProviderKind::Xai => get(KEY_ENV).or_else(|| get(LEGACY_XAI_KEY_ENV)),
            ProviderKind::OpenAiCompatible => get(KEY_ENV),
        };
        if key.as_ref().is_some_and(|k| k.len() > MAX_KEY_BYTES) {
            return Err(BridgeError::new(
                "provider_auth_missing",
                "The configured conversion key exceeds 8 KiB",
            ));
        }
        if key.is_none() && !(kind == ProviderKind::OpenAiCompatible && is_loopback(&base_url)) {
            return Err(BridgeError::new(
                "provider_auth_missing",
                match kind {
                    ProviderKind::Xai => {
                        "Supply the Mac's authorized Grok key through its credential adapter"
                    }
                    ProviderKind::OpenAiCompatible => {
                        "Supply FLASHTEX_AI_API_KEY through the Mac's credential adapter (only a loopback endpoint may omit it)"
                    }
                },
            ));
        }
        Ok(Self {
            kind,
            model,
            base_url,
            key,
        })
    }

    /// Whether a credential was resolved (never the credential itself).
    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    /// Builds the converter. Constructing it sends nothing.
    pub fn build(self) -> Result<Box<dyn Converter>> {
        Ok(match self.kind {
            ProviderKind::Xai => Box::new(grok::GrokClient::with_base_url(
                self.key.unwrap_or_default(),
                self.model,
                &self.base_url,
            )?),
            ProviderKind::OpenAiCompatible => Box::new(openai::OpenAiCompatibleClient::new(
                self.key,
                self.model,
                &self.base_url,
            )?),
        })
    }
}

/// Accepts `https://host[:port][/path]`, or plain `http://` only for loopback
/// hosts (a local model server). Rejects credentials in the URL, queries and
/// fragments. Returns the URL without a trailing slash.
pub fn validate_base_url(raw: &str) -> Result<String> {
    let invalid = || {
        BridgeError::new(
            "invalid_base_url",
            "FLASHTEX_CONVERSION_BASE_URL must be https://…, or http:// on 127.0.0.1, localhost or [::1]",
        )
    };
    let url = raw.trim();
    if url.len() > MAX_BASE_URL_BYTES
        || url.contains(['?', '#', '@', ' ', '\\'])
        || url.chars().any(|c| c.is_control() || !c.is_ascii())
    {
        return Err(invalid());
    }
    let (secure, rest) = if let Some(rest) = url.strip_prefix("https://") {
        (true, rest)
    } else if let Some(rest) = url.strip_prefix("http://") {
        (false, rest)
    } else {
        return Err(invalid());
    };
    let authority = rest.split('/').next().unwrap_or_default();
    let host = host_of(authority);
    if host.is_empty() || (!secure && !is_loopback_host(host)) {
        return Err(invalid());
    }
    Ok(url.trim_end_matches('/').to_string())
}

fn host_of(authority: &str) -> &str {
    if authority.starts_with('[') {
        return authority
            .find(']')
            .map(|end| &authority[..=end])
            .unwrap_or_default();
    }
    authority.split(':').next().unwrap_or_default()
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "[::1]")
}

/// True when `base_url` (already validated) points at this machine.
pub fn is_loopback(base_url: &str) -> bool {
    let rest = base_url
        .strip_prefix("https://")
        .or_else(|| base_url.strip_prefix("http://"))
        .unwrap_or_default();
    is_loopback_host(host_of(rest.split('/').next().unwrap_or_default()))
}

/// Token usage as reported by the provider, when it reports it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

/// What a conversion can prove about the call that produced it: which provider
/// and model answered, the provider's response id (to reconcile with its
/// dashboard/billing), and reported usage. No request content, image data or
/// credentials. Carried in `capture_proposal` and persisted in the journal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderEvidence {
    pub provider: String,
    pub model: String,
    pub response_id: Option<String>,
    pub usage: Option<ProviderUsage>,
}

impl ProviderEvidence {
    /// Extracts evidence from a provider JSON reply. `usage_fields` names the
    /// (input, output, total) token counters in that provider's usage object.
    /// Untrusted strings are bounded; anything malformed is simply omitted.
    pub fn from_reply(
        provider: &str,
        configured_model: &str,
        reply: &Value,
        usage_fields: (&str, &str, &str),
    ) -> Self {
        let bounded = |v: Option<&Value>| {
            v.and_then(Value::as_str)
                .filter(|s| {
                    !s.is_empty() && s.len() <= 256 && s.bytes().all(|b| b.is_ascii_graphic())
                })
                .map(str::to_string)
        };
        let usage = reply.get("usage").filter(|u| u.is_object()).map(|u| {
            let count = |name: &str| u.get(name).and_then(Value::as_u64);
            ProviderUsage {
                input_tokens: count(usage_fields.0),
                output_tokens: count(usage_fields.1),
                total_tokens: count(usage_fields.2),
            }
        });
        Self {
            provider: provider.to_string(),
            model: bounded(reply.get("model")).unwrap_or_else(|| configured_model.to_string()),
            response_id: bounded(reply.get("id")),
            usage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name| map.get(name).cloned()
    }

    #[test]
    fn provider_names_parse_with_aliases_and_none() {
        assert_eq!(ProviderKind::parse("none").unwrap(), None);
        assert_eq!(ProviderKind::parse("xai").unwrap(), Some(ProviderKind::Xai));
        assert_eq!(
            ProviderKind::parse("grok").unwrap(),
            Some(ProviderKind::Xai)
        );
        assert_eq!(
            ProviderKind::parse("openai-compatible").unwrap(),
            Some(ProviderKind::OpenAiCompatible)
        );
        assert_eq!(
            ProviderKind::parse("openai").unwrap(),
            Some(ProviderKind::OpenAiCompatible)
        );
        assert_eq!(
            ProviderKind::parse("XAI").unwrap_err().code,
            "invalid_arguments"
        );
        assert_eq!(
            ProviderKind::parse("").unwrap_err().code,
            "invalid_arguments"
        );
    }

    #[test]
    fn xai_prefers_generic_names_and_falls_back_to_legacy_ones() {
        let c = ProviderConfig::resolve(
            ProviderKind::Xai,
            env(&[
                (KEY_ENV, "generic"),
                (LEGACY_XAI_KEY_ENV, "legacy"),
                (MODEL_ENV, "grok-new"),
                (LEGACY_GROK_MODEL_ENV, "grok-old"),
            ]),
        )
        .unwrap();
        assert_eq!(c.key.as_deref(), Some("generic"));
        assert_eq!(c.model, "grok-new");
        assert_eq!(c.base_url, grok::DEFAULT_BASE_URL);

        let legacy = ProviderConfig::resolve(
            ProviderKind::Xai,
            env(&[
                (LEGACY_XAI_KEY_ENV, "legacy"),
                (LEGACY_GROK_MODEL_ENV, "grok-old"),
            ]),
        )
        .unwrap();
        assert_eq!(legacy.key.as_deref(), Some("legacy"));
        assert_eq!(legacy.model, "grok-old");

        let defaults =
            ProviderConfig::resolve(ProviderKind::Xai, env(&[(KEY_ENV, "k"), (MODEL_ENV, "")]))
                .unwrap();
        assert_eq!(defaults.model, grok::DEFAULT_MODEL);
    }

    #[test]
    fn xai_without_any_key_is_auth_missing() {
        let err = ProviderConfig::resolve(ProviderKind::Xai, env(&[(KEY_ENV, "  ")])).unwrap_err();
        assert_eq!(err.code, "provider_auth_missing");
    }

    #[test]
    fn openai_compatible_requires_base_url_and_model_but_not_legacy_names() {
        let no_base = ProviderConfig::resolve(
            ProviderKind::OpenAiCompatible,
            env(&[(KEY_ENV, "k"), (MODEL_ENV, "m")]),
        )
        .unwrap_err();
        assert_eq!(no_base.code, "provider_base_url_missing");
        let no_model = ProviderConfig::resolve(
            ProviderKind::OpenAiCompatible,
            env(&[
                (KEY_ENV, "k"),
                (BASE_URL_ENV, "https://llm.example/v1"),
                (LEGACY_GROK_MODEL_ENV, "grok-4.6"),
            ]),
        )
        .unwrap_err();
        assert_eq!(no_model.code, "invalid_model");
        let xai_key_is_not_reused = ProviderConfig::resolve(
            ProviderKind::OpenAiCompatible,
            env(&[
                (LEGACY_XAI_KEY_ENV, "xai-only"),
                (BASE_URL_ENV, "https://llm.example/v1"),
                (MODEL_ENV, "m"),
            ]),
        )
        .unwrap_err();
        assert_eq!(xai_key_is_not_reused.code, "provider_auth_missing");
    }

    #[test]
    fn openai_compatible_loopback_endpoint_may_omit_the_key() {
        let c = ProviderConfig::resolve(
            ProviderKind::OpenAiCompatible,
            env(&[
                (BASE_URL_ENV, "http://127.0.0.1:11434/v1/"),
                (MODEL_ENV, "local-vlm"),
            ]),
        )
        .unwrap();
        assert!(!c.has_key());
        assert_eq!(c.base_url, "http://127.0.0.1:11434/v1");
    }

    #[test]
    fn base_urls_are_https_or_loopback_http_only() {
        for ok in [
            "https://api.x.ai/v1",
            "https://llm.example:8443/openai/v1/",
            "http://127.0.0.1:8080/v1",
            "http://localhost:1234",
            "http://[::1]:9000/v1",
        ] {
            assert!(validate_base_url(ok).is_ok(), "{ok}");
        }
        for bad in [
            "http://llm.example/v1",
            "http://127.0.0.1.evil.example/v1",
            "http://localhost@evil.example/",
            "https://user:pass@llm.example/v1",
            "ftp://127.0.0.1",
            "https://",
            "https://llm.example/v1?x=1",
            "https://llm.example/v1#frag",
            "api.x.ai/v1",
        ] {
            assert_eq!(
                validate_base_url(bad).unwrap_err().code,
                "invalid_base_url",
                "{bad}"
            );
        }
    }

    #[test]
    fn debug_output_never_contains_the_key() {
        let c = ProviderConfig::resolve(ProviderKind::Xai, env(&[(KEY_ENV, "sk-fixture-secret")]))
            .unwrap();
        let shown = format!("{c:?}");
        assert!(!shown.contains("sk-fixture-secret"), "{shown}");
        assert!(shown.contains("present"));
    }

    #[test]
    fn evidence_bounds_untrusted_strings_and_reads_usage() {
        let e = ProviderEvidence::from_reply(
            "xai",
            "grok-4.6",
            &json!({"id":"resp_1","model":"grok-4.6-0101","usage":{"input_tokens":10,"output_tokens":5,"total_tokens":15}}),
            ("input_tokens", "output_tokens", "total_tokens"),
        );
        assert_eq!(e.response_id.as_deref(), Some("resp_1"));
        assert_eq!(e.model, "grok-4.6-0101");
        assert_eq!(e.usage.unwrap().total_tokens, Some(15));

        let hostile = ProviderEvidence::from_reply(
            "openai-compatible",
            "configured",
            &json!({"id":"x".repeat(300),"model":"has space","usage":"nope"}),
            ("prompt_tokens", "completion_tokens", "total_tokens"),
        );
        assert_eq!(hostile.response_id, None);
        assert_eq!(hostile.model, "configured");
        assert_eq!(hostile.usage, None);
    }
}

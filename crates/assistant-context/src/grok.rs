//! Optional explicit xAI transport. Construction never reads keys or sends data.
use crate::{Context, ExplanationProposal};
use flashtex_edit_ledger::Document;
use serde_json::{json, Value};
use std::{io::Read, time::Duration};
const ENDPOINT: &str = "https://api.x.ai/v1/responses";
const MAX_RESPONSE: u64 = 512 * 1024;

pub struct GrokClient {
    key: String,
    model: String,
    http: reqwest::blocking::Client,
}
/// Untrusted provider text. Must be validated against the current document after
/// the network call; this type deliberately does not expose an applicable edit.
pub struct ProviderReply {
    bytes: Vec<u8>,
}
impl ProviderReply {
    pub fn validate(
        self,
        context: &Context,
        current: &[Document],
    ) -> Result<ExplanationProposal, String> {
        context.validate_response(&self.bytes, current)
    }
    /// Use registry.receive with these bytes when a cancellable flight owns the
    /// request. That check preserves cancellation/expiry as well as source binding.
    pub fn untrusted_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
fn body(model: &str, context: &Context) -> Value {
    json!({"model":model,"store":false,"stream":false,"max_output_tokens":8192,
        "input":[{"role":"system","content":context.payload().system_instruction},
                 {"role":"user","content":serde_json::to_string(context.payload()).expect("serializable payload")}],
        "text":{"format":{"type":"json_schema","name":"explanation_proposal","strict":true,"schema":{
            "type":"object","additionalProperties":false,"required":["context_id","explanation","edits"],
            "properties":{"context_id":{"type":"string"},"explanation":{"type":"string"},"edits":{"type":"array","items":{
                "type":"object","additionalProperties":false,"required":["location","removed_text","replacement"],
                "properties":{"removed_text":{"type":"string"},"replacement":{"type":"string"},"location":{
                    "type":"object","additionalProperties":false,"required":["path","start_byte","end_byte"],
                    "properties":{"path":{"type":"string"},"start_byte":{"type":"integer"},"end_byte":{"type":"integer"}}}}}}}}}}})
}
fn parse(bytes: &[u8]) -> Result<ProviderReply, String> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|_| "provider returned invalid JSON")?;
    if value["status"] != "completed" {
        return Err("provider response incomplete".into());
    }
    let outputs = value["output"]
        .as_array()
        .ok_or("provider output missing")?;
    let mut text = None;
    for output in outputs {
        if output["type"] != "message" {
            continue;
        }
        if output["role"] != "assistant" {
            return Err("provider message role invalid".into());
        }
        for content in output["content"]
            .as_array()
            .ok_or("provider content missing")?
        {
            if content["type"] == "refusal" {
                return Err("provider refused explanation".into());
            }
            if content["type"] == "output_text" {
                let candidate = content["text"].as_str().ok_or("provider text invalid")?;
                if candidate.len() > 64 * 1024 || text.replace(candidate).is_some() {
                    return Err("provider expected one bounded proposal".into());
                }
            }
        }
    }
    Ok(ProviderReply {
        bytes: text.ok_or("provider proposal missing")?.as_bytes().to_vec(),
    })
}
impl GrokClient {
    pub fn new(key: String, model: String, timeout: Duration) -> Result<Self, String> {
        if key.is_empty() || key.len() > 8192 || key.bytes().any(|b| b.is_ascii_control()) {
            return Err("invalid provider credential".into());
        }
        if model.is_empty()
            || model.len() > 128
            || !model
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
        {
            return Err("invalid explicit model ID".into());
        }
        if timeout.is_zero() || timeout > Duration::from_secs(120) {
            return Err("invalid provider timeout".into());
        }
        let http = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout.min(Duration::from_secs(10)))
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .no_proxy()
            .build()
            .map_err(|_| "provider client initialization failed")?;
        Ok(Self { key, model, http })
    }
    /// One explicitly requested call, without retry, tools or billing fallback.
    /// The caller must validate the reply using a fresh source snapshot afterward.
    pub fn request(
        &self,
        context: &Context,
        current: &[Document],
    ) -> Result<ProviderReply, String> {
        self.request_at(ENDPOINT, context, current)
    }
    fn request_at(
        &self,
        endpoint: &str,
        context: &Context,
        current: &[Document],
    ) -> Result<ProviderReply, String> {
        context.check_current(current)?;
        let bytes = serde_json::to_vec(&body(&self.model, context))
            .map_err(|_| "provider request encoding failed")?;
        if bytes.len() > 256 * 1024 {
            return Err("provider request exceeds256KiB".into());
        }
        let response = self
            .http
            .post(endpoint)
            .bearer_auth(&self.key)
            .header("Content-Type", "application/json")
            .body(bytes)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    "provider timeout; no retry"
                } else {
                    "provider transport failed; no retry"
                }
            })?;
        if !response.status().is_success() {
            return Err(format!(
                "provider HTTP {}; no retry",
                response.status().as_u16()
            ));
        }
        if response.content_length().is_some_and(|n| n > MAX_RESPONSE) {
            return Err("provider response exceeds512KiB".into());
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_RESPONSE + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "provider response read failed")?;
        if bytes.len() as u64 > MAX_RESPONSE {
            return Err("provider response exceeds512KiB".into());
        }
        parse(&bytes)
    }
}

/// Provider bytes tied to the exact dispatched registry request.
pub struct RoutedReply {
    request_id: String,
    context_id: String,
    reply: ProviderReply,
}
impl RoutedReply {
    pub fn receive(
        self,
        registry: &mut crate::ExplanationRegistry,
        current: &[Document],
    ) -> Result<ExplanationProposal, String> {
        registry.receive(
            &self.request_id,
            &self.context_id,
            self.reply.untrusted_bytes(),
            current,
        )
    }
}
impl GrokClient {
    /// Consume the flight's single dispatch lease. Registry cancellation may race
    /// an in-flight call; only RoutedReply::receive can release its proposal.
    pub fn request_lease(
        &self,
        lease: crate::RequestLease,
        current: &[Document],
    ) -> Result<RoutedReply, String> {
        self.request_lease_at(ENDPOINT, lease, current)
    }
    fn request_lease_at(
        &self,
        endpoint: &str,
        lease: crate::RequestLease,
        current: &[Document],
    ) -> Result<RoutedReply, String> {
        lease.check_current(current)?;
        let reply = self.request_at(endpoint, lease.context(), current)?;
        Ok(RoutedReply {
            request_id: lease.request_id().to_owned(),
            context_id: lease.payload().context_id.clone(),
            reply,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CompileBinding;
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
    };
    fn context() -> (Context, Vec<Document>) {
        let docs = vec![Document::new("p".into(), "main.tex".into(), 1, "hello".into()).unwrap()];
        let result = json!({"protocol_version":1,"type":"compile_result","id":"r","payload":{"project_id":"p","revision":1,"status":"ok","pages":[],"diagnostics":[]}});
        (
            Context::build(
                CompileBinding::capture("r", "p", 1, &docs).unwrap(),
                &docs,
                &result,
                "Explain",
                &["main.tex".into()],
            )
            .unwrap(),
            docs,
        )
    }
    fn server(status: u16, body: String) -> (String, std::thread::JoinHandle<Value>) {
        server_mode(status, body, false, Duration::ZERO)
    }
    fn server_mode(
        status: u16,
        body: String,
        chunked: bool,
        delay: Duration,
    ) -> (String, std::thread::JoinHandle<Value>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(socket.try_clone().unwrap());
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().unwrap();
                }
            }
            assert!(length <= 256 * 1024);
            let mut bytes = vec![0; length];
            reader.read_exact(&mut bytes).unwrap();
            std::thread::sleep(delay);
            if chunked {
                let _ = write!(socket, "HTTP/1.1 {status} Test\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n", body.len());
            } else {
                let _ = write!(socket, "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            }
            serde_json::from_slice(&bytes).unwrap()
        });
        (endpoint, handle)
    }
    #[test]
    fn local_transport_preserves_context_and_requires_fresh_validation() {
        let (context, docs) = context();
        let proposal = json!({"context_id":context.payload().context_id,"explanation":"Explanation","edits":[]}).to_string();
        let response = json!({"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":proposal}]}]});
        let (endpoint, handle) = server(200, response.to_string());
        let client = GrokClient::new(
            "test-credential".into(),
            "explicit-test-model".into(),
            Duration::from_secs(2),
        )
        .unwrap();
        let reply = client.request_at(&endpoint, &context, &docs).unwrap();
        let sent = handle.join().unwrap();
        assert_eq!(sent["store"], false);
        assert_eq!(sent["stream"], false);
        assert_eq!(sent["model"], "explicit-test-model");
        assert_eq!(sent["text"]["format"]["type"], "json_schema");
        assert!(context
            .validate_response(reply.untrusted_bytes(), &docs)
            .is_ok());
        let changed =
            vec![Document::new("p".into(), "main.tex".into(), 2, "changed".into()).unwrap()];
        assert!(reply.validate(&context, &changed).is_err());
        // Stale preflight fails before trying this deliberately unreachable URL.
        assert!(client
            .request_at("http://127.0.0.1:1", &context, &changed)
            .err()
            .unwrap()
            .contains("source"));
    }
    #[test]
    fn local_http_errors_and_malformed_output_do_not_produce_proposals() {
        let (context, docs) = context();
        let client = GrokClient::new(
            "test-only-secret".into(),
            "test-model".into(),
            Duration::from_secs(2),
        )
        .unwrap();
        for (status, response) in [
            (429, "secret provider body".to_owned()),
            (401, "secret".to_owned()),
            (200, "malformed".to_owned()),
            (200, "{\"status\":\"incomplete\"}".to_owned()),
            (200, "x".repeat(512 * 1024 + 1)),
        ] {
            let (endpoint, handle) = server(status, response);
            let error = client.request_at(&endpoint, &context, &docs).err().unwrap();
            assert!(!error.contains("secret"));
            handle.join().unwrap();
        }
        assert!(GrokClient::new("secret\n".into(), "test".into(), Duration::from_secs(1)).is_err());
        assert!(GrokClient::new("secret".into(), "".into(), Duration::from_secs(1)).is_err());
    }
    #[test]
    fn leased_http_workflow_validates_only_live_current_flights() {
        for mode in ["success", "cancel", "stale", "expired"] {
            let (bound, docs) = context();
            let context_id = bound.payload().context_id.clone();
            let mut registry =
                crate::ExplanationRegistry::new(format!("workflow_{mode}"), 1, 2).unwrap();
            let timeout = if mode == "expired" {
                Duration::from_millis(20)
            } else {
                Duration::from_secs(2)
            };
            let id = registry.submit(bound, &docs, timeout).unwrap();
            let lease = registry.lease(&id, &docs).unwrap();
            let body = json!({"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":json!({"context_id":context_id,"explanation":"Explain","edits":[]}).to_string()}]}]}).to_string();
            let (endpoint, server) = server_mode(200, body, false, Duration::from_millis(50));
            let client =
                GrokClient::new("dummy".into(), "test".into(), Duration::from_secs(2)).unwrap();
            // request_lease_at performs the real local HTTP exchange using the
            // sole shared Context, with no second context construction.
            let reply = client.request_lease_at(&endpoint, lease, &docs).unwrap();
            server.join().unwrap();
            if mode == "cancel" {
                assert!(registry.cancel(&id));
            }
            let current = if mode == "stale" {
                vec![Document::new("p".into(), "main.tex".into(), 2, "changed".into()).unwrap()]
            } else {
                docs
            };
            assert_eq!(
                reply.receive(&mut registry, &current).is_ok(),
                mode == "success",
                "{mode}"
            );
            assert!(!registry.fail(&id));
        }
    }
    #[test]
    fn delayed_chunked_and_cancelled_responses_are_bounded() {
        let (bound, docs) = context();
        let proposal =
            json!({"context_id":bound.payload().context_id,"explanation":"Explain","edits":[]});
        let response = json!({"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":proposal.to_string()}]}]}).to_string();
        let client =
            GrokClient::new("dummy".into(), "test".into(), Duration::from_millis(50)).unwrap();
        let (endpoint, handle) =
            server_mode(200, response.clone(), false, Duration::from_millis(200));
        let start = std::time::Instant::now();
        let error = client.request_at(&endpoint, &bound, &docs).err().unwrap();
        assert!(error.contains("timeout"), "{error}");
        assert!(start.elapsed() < Duration::from_secs(1));
        handle.join().unwrap();
        let client =
            GrokClient::new("dummy".into(), "test".into(), Duration::from_secs(2)).unwrap();
        let (endpoint, handle) = server_mode(200, "x".repeat(512 * 1024 + 1), true, Duration::ZERO);
        assert!(client
            .request_at(&endpoint, &bound, &docs)
            .err()
            .unwrap()
            .contains("exceeds"));
        handle.join().unwrap();
        let mut registry = crate::ExplanationRegistry::new("cancel_http".into(), 1, 2).unwrap();
        let (flight_context, _) = context();
        let id = registry
            .submit(flight_context, &docs, Duration::from_secs(2))
            .unwrap();
        let context_id = bound.payload().context_id.clone();
        let (endpoint, handle) = server_mode(200, response, true, Duration::from_millis(20));
        let worker =
            std::thread::spawn(move || client.request_at(&endpoint, &bound, &docs).unwrap());
        assert!(registry.cancel(&id));
        let reply = worker.join().unwrap();
        handle.join().unwrap();
        let (_, current) = context();
        assert!(registry
            .receive(&id, &context_id, reply.untrusted_bytes(), &current)
            .is_err());
        assert_eq!(registry.state(&id), Some(crate::FlightState::Cancelled));
    }
}

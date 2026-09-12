//! Persistent original-compiler transport. Poll from an application worker, not
//! the UI thread. Results are never substituted across revisions.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, VecDeque},
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub path: String,
    pub text: String,
}
#[derive(Clone, Debug)]
pub struct Request {
    pub id: String,
    pub project_id: String,
    pub revision: u64,
    pub entry_path: String,
    pub documents: Vec<Document>,
}
#[derive(Clone, Debug)]
pub struct Limits {
    pub max_frame: usize,
    pub max_projects: usize,
    pub max_pending_events: usize,
    pub timeout: Duration,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_frame: 8 * 1024 * 1024,
            max_projects: 32,
            max_pending_events: 256,
            timeout: Duration::from_secs(5),
        }
    }
}
#[derive(Debug)]
pub enum Event {
    Cancelled {
        id: String,
    },
    Superseded {
        id: String,
        by_id: String,
    },
    Preview {
        id: String,
        project_id: String,
        revision: u64,
        result: Value,
        queue_ms: f64,
        compiler_ms: f64,
        total_ms: f64,
    },
    Stale {
        id: String,
        revision: u64,
    },
    Failed {
        id: String,
        reason: String,
    },
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct ResponseProfile {
    pub request_id: String,
    pub response_bytes: usize,
    pub encode_ms: f64,
    pub reader_delivery_wait_ms: f64,
    pub dispatch_to_first_byte_ms: f64,
    pub frame_read_ms: f64,
    pub parse_ms: f64,
    pub validation_ms: f64,
}
struct Pending {
    encode_ms: f64,
    capabilities: Vec<String>,
    cancelled: bool,
    request: Request,
    bytes: Vec<u8>,
    queued: Instant,
    sent: Option<Instant>,
}
enum Input {
    Frame(Vec<u8>, Instant, Instant),
    Failure(String),
}
struct Process {
    child: Child,
    writer: SyncSender<Vec<u8>>,
    reader: Receiver<Input>,
}
impl Process {
    fn spawn(mut command: Command, limit: usize) -> Result<Self, String> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("compiler launch: {e}"))?;
        let mut stdin = child.stdin.take().ok_or("compiler stdin unavailable")?;
        let stdout = child.stdout.take().ok_or("compiler stdout unavailable")?;
        let mut stderr = child.stderr.take().ok_or("compiler stderr unavailable")?;
        let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(1);
        let (out_tx, out_rx) = mpsc::sync_channel(4);
        let failures = out_tx.clone();
        thread::spawn(move || {
            while let Ok(bytes) = rx.recv() {
                if stdin.write_all(&bytes).and_then(|_| stdin.flush()).is_err() {
                    let _ = failures.send(Input::Failure("compiler input closed".into()));
                    break;
                }
            }
        });
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut frame = Vec::new();
                if reader.fill_buf().is_err() {
                    let _ = out_tx.send(Input::Failure("compiler output read failed".into()));
                    break;
                }
                let first_byte = Instant::now();
                let result = reader
                    .by_ref()
                    .take(limit as u64 + 1)
                    .read_until(b'\n', &mut frame);
                match result {
                    Ok(0) => {
                        let _ = out_tx.send(Input::Failure("compiler output closed".into()));
                        break;
                    }
                    Ok(_) if frame.len() <= limit && frame.last() == Some(&b'\n') => {
                        if out_tx
                            .send(Input::Frame(frame, first_byte, Instant::now()))
                            .is_err()
                        {
                            break;
                        }
                    }
                    _ => {
                        let _ = out_tx.send(Input::Failure(
                            "compiler output malformed, truncated or oversized".into(),
                        ));
                        break;
                    }
                }
            }
        });
        // Drain continuously in a fixed buffer. Logs are not retained or exposed to UI.
        thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            while let Ok(n) = stderr.read(&mut buffer) {
                if n == 0 {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            writer: tx,
            reader: out_rx,
        })
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Session {
    process: Option<Process>,
    limits: Limits,
    active: Option<Pending>,
    queue: VecDeque<Pending>,
    latest: BTreeMap<String, (u64, String)>,
    events: VecDeque<Event>,
    last_profile: Option<ResponseProfile>,
}
impl Session {
    pub fn spawn(executable: impl AsRef<Path>, limits: Limits) -> Result<Self, String> {
        Self::spawn_command(Command::new(executable.as_ref()), limits)
    }
    /// Allows explicit original compiler flags/environment without shell parsing.
    pub fn spawn_command(command: Command, limits: Limits) -> Result<Self, String> {
        if limits.max_frame < 128
            || limits.max_frame > 64 * 1024 * 1024
            || limits.max_projects == 0
            || limits.max_pending_events == 0
            || limits.timeout.is_zero()
        {
            return Err("invalid runtime limits".into());
        }
        let process = Process::spawn(command, limits.max_frame)?;
        Ok(Self {
            process: Some(process),
            limits,
            active: None,
            queue: VecDeque::new(),
            latest: BTreeMap::new(),
            events: VecDeque::new(),
            last_profile: None,
        })
    }
    pub fn submit(&mut self, request: Request) -> Result<(), String> {
        self.submit_with_capabilities(request, Vec::new())
    }
    pub fn submit_with_capabilities(
        &mut self,
        request: Request,
        capabilities: Vec<String>,
    ) -> Result<(), String> {
        validate_layout_capabilities(&capabilities)?;
        if self.events.len() >= self.limits.max_pending_events {
            return Err("poll pending events before submitting more edits".into());
        }
        if self.process.is_none() {
            return Err(
                "compiler session failed; create a new session with complete snapshots".into(),
            );
        }
        let encode_start = Instant::now();
        let bytes = encode(&request, self.limits.max_frame, &capabilities)?;
        let encode_ms = encode_start.elapsed().as_secs_f64() * 1000.0;
        if self.latest.values().any(|(_, id)| id == &request.id)
            || self
                .active
                .as_ref()
                .is_some_and(|p| p.request.id == request.id)
            || self.queue.iter().any(|p| p.request.id == request.id)
        {
            return Err("duplicate live request ID".into());
        }
        if let Some((revision, _)) = self.latest.get(&request.project_id) {
            if request.revision <= *revision {
                return Err("revision must advance monotonically".into());
            }
        } else if self.latest.len() >= self.limits.max_projects {
            return Err("project capacity reached".into());
        }
        if let Some(index) = self
            .queue
            .iter()
            .position(|p| p.request.project_id == request.project_id)
        {
            let old = self.queue.remove(index).unwrap();
            self.events.push_back(Event::Superseded {
                id: old.request.id,
                by_id: request.id.clone(),
            });
        }
        self.latest.insert(
            request.project_id.clone(),
            (request.revision, request.id.clone()),
        );
        self.queue.push_back(Pending {
            encode_ms,
            capabilities,
            cancelled: false,
            request,
            bytes,
            queued: Instant::now(),
            sent: None,
        });
        self.dispatch();
        Ok(())
    }
    /// Release a closed project's slot and explicitly cancel accepted work.
    /// An in-flight wire request is still drained before another is dispatched.
    pub fn close_project(&mut self, project_id: &str) -> Result<(), String> {
        if self.events.len() >= self.limits.max_pending_events {
            return Err("poll pending events before closing projects".into());
        }
        self.latest.remove(project_id);
        if let Some(active) = self.active.as_mut() {
            if active.request.project_id == project_id && !active.cancelled {
                active.cancelled = true;
                self.events.push_back(Event::Cancelled {
                    id: active.request.id.clone(),
                });
            }
        }
        let mut retained = VecDeque::new();
        for pending in self.queue.drain(..) {
            if pending.request.project_id == project_id {
                self.events.push_back(Event::Cancelled {
                    id: pending.request.id,
                });
            } else {
                retained.push_back(pending);
            }
        }
        self.queue = retained;
        Ok(())
    }
    fn dispatch(&mut self) {
        if self.active.is_some() || self.process.is_none() {
            return;
        }
        if let Some(mut pending) = self.queue.pop_front() {
            pending.sent = Some(Instant::now());
            let bytes = std::mem::take(&mut pending.bytes);
            self.active = Some(pending);
            if self
                .process
                .as_ref()
                .unwrap()
                .writer
                .try_send(bytes)
                .is_err()
            {
                self.fail("compiler writer unavailable");
            }
        }
    }
    fn fail(&mut self, reason: &str) {
        self.process.take();
        if let Some(p) = self.active.take().filter(|p| !p.cancelled) {
            self.events.push_back(Event::Failed {
                id: p.request.id,
                reason: reason.into(),
            });
        }
        for p in self.queue.drain(..) {
            self.events.push_back(Event::Failed {
                id: p.request.id,
                reason: reason.into(),
            });
        }
    }
    pub fn poll(&mut self) -> Vec<Event> {
        while let Some(process) = self.process.as_ref() {
            let message = process.reader.try_recv();
            match message {
                Ok(Input::Failure(reason)) => {
                    self.fail(&reason);
                    break;
                }
                Ok(Input::Frame(bytes, first_byte, reader_done)) => {
                    let reader_delivery_wait_ms = reader_done.elapsed().as_secs_f64() * 1000.0;
                    let Some(pending) = self.active.as_ref() else {
                        self.fail("unsolicited compiler reply");
                        break;
                    };
                    let parsed_at = Instant::now();
                    // Validate UTF-8 once for the whole frame instead of once per JSON string.
                    // Keep the same Value and semantic validation path below.
                    let parsed: Value = match std::str::from_utf8(&bytes)
                        .map_err(|_| ())
                        .and_then(|text| serde_json::from_str(text).map_err(|_| ()))
                    {
                        Ok(value) => value,
                        Err(_) => {
                            self.fail("compiler returned malformed JSON");
                            break;
                        }
                    };
                    let parse_ms = parsed_at.elapsed().as_secs_f64() * 1000.0;
                    let validate_at = Instant::now();
                    let result =
                        match validate_reply_value(parsed, &pending.request, &pending.capabilities)
                        {
                            Ok(value) => value,
                            Err(reason) => {
                                self.fail(&reason);
                                break;
                            }
                        };
                    self.last_profile = Some(ResponseProfile {
                        request_id: pending.request.id.clone(),
                        response_bytes: bytes.len(),
                        encode_ms: pending.encode_ms,
                        reader_delivery_wait_ms,
                        dispatch_to_first_byte_ms: first_byte
                            .saturating_duration_since(pending.sent.unwrap())
                            .as_secs_f64()
                            * 1000.0,
                        frame_read_ms: reader_done
                            .saturating_duration_since(first_byte)
                            .as_secs_f64()
                            * 1000.0,
                        parse_ms,
                        validation_ms: validate_at.elapsed().as_secs_f64() * 1000.0,
                    });
                    let pending = self.active.take().unwrap();
                    if pending.cancelled {
                        self.dispatch();
                        continue;
                    }
                    let sent = pending.sent.unwrap();
                    let now = Instant::now();
                    if self
                        .latest
                        .get(&pending.request.project_id)
                        .is_some_and(|(rev, id)| {
                            *rev == pending.request.revision && id == &pending.request.id
                        })
                    {
                        self.events.push_back(Event::Preview {
                            id: pending.request.id,
                            project_id: pending.request.project_id,
                            revision: pending.request.revision,
                            result,
                            queue_ms: sent.duration_since(pending.queued).as_secs_f64() * 1000.0,
                            compiler_ms: now.duration_since(sent).as_secs_f64() * 1000.0,
                            total_ms: now.duration_since(pending.queued).as_secs_f64() * 1000.0,
                        });
                    } else {
                        self.events.push_back(Event::Stale {
                            id: pending.request.id,
                            revision: pending.request.revision,
                        });
                    }
                    self.dispatch();
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.fail("compiler readers disconnected");
                    break;
                }
            }
        }
        if self
            .active
            .as_ref()
            .and_then(|p| p.sent)
            .is_some_and(|t| t.elapsed() >= self.limits.timeout)
        {
            self.fail("compiler response timeout");
        }
        self.events.drain(..).collect()
    }
    /// Last fully validated response, including stale/cancelled work. Match its
    /// request ID; these phases do not measure native paint or compiler CPU alone.
    pub fn last_profile(&self) -> Option<&ResponseProfile> {
        self.last_profile.as_ref()
    }
    pub fn is_alive(&self) -> bool {
        self.process.is_some()
    }
}
fn safe_path(p: &str) -> bool {
    !p.is_empty()
        && !p.starts_with('/')
        && !p.contains(['\\', ':', '\0'])
        && !p.split('/').any(|s| s.is_empty() || s == "." || s == "..")
}
fn encode(r: &Request, limit: usize, capabilities: &[String]) -> Result<Vec<u8>, String> {
    if r.id.is_empty()
        || r.id.len() > 128
        || r.project_id.is_empty()
        || r.project_id.len() > 128
        || r.revision > ((1u64 << 53) - 1)
        || !safe_path(&r.entry_path)
    {
        return Err("invalid request identity, revision or entry".into());
    }
    let mut paths = BTreeMap::new();
    let mut size = 0usize;
    for d in &r.documents {
        size = size.saturating_add(d.text.len());
        if size > limit || !safe_path(&d.path) || paths.insert(&d.path, ()).is_some() {
            return Err("invalid/oversized document snapshots".into());
        }
    }
    if !paths.contains_key(&r.entry_path) {
        return Err("entry snapshot missing".into());
    }
    #[derive(Serialize)]
    struct Payload<'a> {
        project_id: &'a str,
        revision: u64,
        entry_path: &'a str,
        documents: &'a [Document],
        #[serde(skip_serializing_if = "<[String]>::is_empty")]
        layout_capabilities: &'a [String],
    }
    #[derive(Serialize)]
    struct Envelope<'a> {
        protocol_version: u8,
        id: &'a str,
        #[serde(rename = "type")]
        kind: &'static str,
        payload: Payload<'a>,
    }
    let envelope = Envelope {
        protocol_version: 1,
        id: &r.id,
        kind: "compile",
        payload: Payload {
            project_id: &r.project_id,
            revision: r.revision,
            entry_path: &r.entry_path,
            documents: &r.documents,
            layout_capabilities: capabilities,
        },
    };
    let mut bytes = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > limit {
        return Err("request frame too large".into());
    }
    Ok(bytes)
}
fn validate_reply(bytes: &[u8], r: &Request, requested: &[String]) -> Result<Value, String> {
    let v: Value = serde_json::from_slice(bytes).map_err(|_| "compiler returned malformed JSON")?;
    validate_reply_value(v, r, requested)
}
fn validate_reply_value(v: Value, r: &Request, requested: &[String]) -> Result<Value, String> {
    if v["protocol_version"] != 1
        || v["id"] != r.id
        || v["type"] != "compile_result"
        || v["payload"]["project_id"] != r.project_id
        || v["payload"]["revision"].as_u64() != Some(r.revision)
    {
        return Err("compiler reply correlation mismatch".into());
    }
    let p = &v["payload"];
    let accepted: Vec<String> = match p.get("layout_capabilities") {
        None => Vec::new(),
        Some(value) => {
            serde_json::from_value(value.clone()).map_err(|_| "invalid accepted capabilities")?
        }
    };
    validate_layout_capabilities(&accepted)?;
    if accepted.iter().any(|cap| {
        !requested.contains(cap) || !matches!(cap.as_str(), "rules-v1" | "font-hints-v1")
    }) {
        return Err("compiler accepted unknown or unrequested capability".into());
    }

    if !matches!(p["status"].as_str(), Some("ok" | "recovered" | "failed"))
        || !p["pages"].is_array()
        || !p["diagnostics"].is_array()
    {
        return Err("invalid compiler result shape".into());
    }
    let documents: BTreeMap<&str, &str> = r
        .documents
        .iter()
        .map(|document| (document.path.as_str(), document.text.as_str()))
        .collect();
    let span = |source: &Value| -> Result<(), String> {
        if source.is_null() {
            return Ok(());
        }
        let path = source["path"].as_str().ok_or("missing source path")?;
        let text = documents.get(path).ok_or("unknown source snapshot")?;
        let start = source["start_byte"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("invalid source start")?;
        let end = source["end_byte"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("invalid source end")?;
        if start > end
            || end > text.len()
            || !text.is_char_boundary(start)
            || !text.is_char_boundary(end)
        {
            return Err("invalid UTF-8 source range".into());
        }
        Ok(())
    };
    for diagnostic in p["diagnostics"].as_array().unwrap() {
        if !matches!(diagnostic["severity"].as_str(), Some("error" | "warning"))
            || !diagnostic["message"].is_string()
            || diagnostic.get("source").is_none()
        {
            return Err("invalid diagnostic".into());
        }
        span(&diagnostic["source"])?;
    }
    for (index, page) in p["pages"].as_array().unwrap().iter().enumerate() {
        if page["number"].as_u64() != Some(index as u64 + 1)
            || !["width_pt", "height_pt"].iter().all(|key| {
                page[*key]
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && n > 0.0)
            })
        {
            return Err("invalid page geometry".into());
        }
        for item in page["items"].as_array().ok_or("missing page items")? {
            if item["kind"] == "rule" {
                if !accepted.iter().any(|cap| cap == "rules-v1")
                    || !["x_pt", "y_pt"].iter().all(|key| {
                        item[*key]
                            .as_f64()
                            .is_some_and(|n| n.is_finite() && n.abs() <= 1_000_000.0)
                    })
                    || !["width_pt", "height_pt"].iter().all(|key| {
                        item[*key]
                            .as_f64()
                            .is_some_and(|n| n.is_finite() && n > 0.0 && n <= 1_000_000.0)
                    })
                    || item["source"].is_null()
                {
                    return Err("unrequested or malformed rule".into());
                }
                span(&item["source"])?;
                continue;
            }
            if let Some(font) = item.get("font") {
                if !accepted.iter().any(|cap| cap == "font-hints-v1")
                    || !font["family"].as_str().is_some_and(|name| {
                        !name.is_empty() && name.len() <= 128 && !name.chars().any(char::is_control)
                    })
                    || !matches!(font["weight"].as_str(), Some("normal" | "bold"))
                    || !matches!(font["style"].as_str(), Some("normal" | "italic"))
                {
                    return Err("unrequested or malformed font hint".into());
                }
            }
            if item["kind"] != "text"
                || !item["text"].is_string()
                || !item["font_size_pt"]
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && n > 0.0)
                || !["x_pt", "baseline_y_pt"]
                    .iter()
                    .all(|key| item[*key].as_f64().is_some_and(f64::is_finite))
                || item.get("source").is_none()
            {
                return Err("unsupported or malformed display item".into());
            }
            span(&item["source"])?;
        }
    }
    Ok(v)
}

pub fn validate_layout_capabilities(capabilities: &[String]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    if capabilities.len() > 16
        || capabilities
            .iter()
            .any(|cap| cap.is_empty() || cap.len() > 64 || !seen.insert(cap))
    {
        return Err("invalid or duplicate layout capabilities".into());
    }
    Ok(())
}

pub mod experimental_chunks;

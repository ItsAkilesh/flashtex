//! Experimental opt-in reassembly only; not connected to the production reader.
use crate::{validate_layout_capabilities, validate_reply, Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chunk {
    pub id: String,
    pub project_id: String,
    pub revision: u64,
    pub sequence: usize,
    pub data: String,
}

pub struct Assembly {
    request: Request,
    capabilities: Vec<String>,
    bytes: Vec<u8>,
    expected_bytes: usize,
    expected_pages: usize,
    next_sequence: usize,
    max_chunk_bytes: usize,
    deadline: Instant,
    failed: bool,
}
impl Assembly {
    /// Explicit experimental opt-in. Announced total is capped at64MiB;
    /// serialized chunks at1MiB. No allocation from untrusted announced length.
    pub fn new(
        request: Request,
        capabilities: Vec<String>,
        expected_bytes: usize,
        expected_pages: usize,
        max_chunk_bytes: usize,
        timeout: Duration,
    ) -> Result<Self, String> {
        validate_layout_capabilities(&capabilities)?;
        if expected_bytes == 0
            || expected_bytes > 64 * 1024 * 1024
            || expected_pages > 10000
            || !(128..=1024 * 1024).contains(&max_chunk_bytes)
            || timeout.is_zero()
            || timeout > Duration::from_secs(60)
        {
            return Err("invalid experimental assembly limits".into());
        }
        Ok(Self {
            request,
            capabilities,
            bytes: Vec::new(),
            expected_bytes,
            expected_pages,
            next_sequence: 0,
            max_chunk_bytes,
            deadline: Instant::now() + timeout,
            failed: false,
        })
    }
    pub fn cancel(&mut self) {
        self.failed = true;
        self.bytes = Vec::new();
    }
    /// Caller supplies the current project revision before every delivery.
    /// A cancelled or failed assembly cannot resume or expose partial pages.
    pub fn push(&mut self, frame: &[u8], current_revision: u64) -> Result<(), String> {
        let result = self.push_checked(frame, current_revision);
        if result.is_err() {
            self.cancel();
        }
        result
    }
    fn push_checked(&mut self, frame: &[u8], current_revision: u64) -> Result<(), String> {
        if self.failed
            || Instant::now() >= self.deadline
            || current_revision != self.request.revision
        {
            return Err("cancelled, stale or expired assembly".into());
        }
        if frame.len() > self.max_chunk_bytes {
            return Err("oversized chunk".into());
        }
        let chunk: Chunk = serde_json::from_slice(frame).map_err(|_| "invalid chunk JSON")?;
        if chunk.id != self.request.id
            || chunk.project_id != self.request.project_id
            || chunk.revision != self.request.revision
            || chunk.sequence != self.next_sequence
            || chunk.data.is_empty()
        {
            return Err("chunk identity, order or content invalid".into());
        }
        if self
            .bytes
            .len()
            .checked_add(chunk.data.len())
            .is_none_or(|n| n > self.expected_bytes)
        {
            return Err("assembly exceeds announced total".into());
        }
        self.bytes.extend_from_slice(chunk.data.as_bytes());
        self.next_sequence += 1;
        Ok(())
    }
    pub fn finish(mut self, current_revision: u64) -> Result<Value, String> {
        if self.failed
            || Instant::now() >= self.deadline
            || current_revision != self.request.revision
            || self.bytes.len() != self.expected_bytes
        {
            self.cancel();
            return Err("incomplete, stale or expired assembly".into());
        }
        let value = validate_reply(&self.bytes, &self.request, &self.capabilities)?;
        if value["payload"]["pages"].as_array().map(Vec::len) != Some(self.expected_pages) {
            return Err("announced page count differs".into());
        }
        Ok(value)
    }
}

/// Alternative prototype: one bounded complete page per message, without JSON
/// string escaping. Oversized individual pages remain explicitly unsupported.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageChunk {
    pub id: String,
    pub project_id: String,
    pub revision: u64,
    pub page: Value,
}
pub struct PageAssembly {
    request: Request,
    capabilities: Vec<String>,
    result: Value,
    header: Value,
    retained_items: usize,
    expected_pages: usize,
    received: usize,
    wire_bytes: usize,
    max_chunk_bytes: usize,
    deadline: Instant,
    failed: bool,
}
impl PageAssembly {
    /// Header is the exact compile_result envelope with an empty pages array.
    pub fn new(
        request: Request,
        capabilities: Vec<String>,
        header: &[u8],
        expected_pages: usize,
        max_chunk_bytes: usize,
        timeout: Duration,
    ) -> Result<Self, String> {
        if !(128..=1024 * 1024).contains(&max_chunk_bytes)
            || header.len() > max_chunk_bytes
            || expected_pages > 10000
            || timeout.is_zero()
            || timeout > Duration::from_secs(60)
        {
            return Err("invalid page assembly limits".into());
        }
        validate_layout_capabilities(&capabilities)?;
        let result = validate_reply(header, &request, &capabilities)?;
        if !result["payload"]["pages"]
            .as_array()
            .is_some_and(Vec::is_empty)
        {
            return Err("page assembly header must contain empty pages".into());
        }
        Ok(Self {
            request,
            capabilities,
            header: result.clone(),
            result,
            retained_items: 0,
            expected_pages,
            received: 0,
            wire_bytes: header.len(),
            max_chunk_bytes,
            deadline: Instant::now() + timeout,
            failed: false,
        })
    }
    pub fn cancel(&mut self) {
        self.failed = true;
        self.result = Value::Null;
        self.header = Value::Null;
        self.retained_items = 0;
        self.received = 0;
        self.wire_bytes = 0;
    }
    pub fn push(&mut self, frame: &[u8], current_revision: u64) -> Result<(), String> {
        let result = self.push_checked(frame, current_revision);
        if result.is_err() {
            self.cancel();
        }
        result
    }
    fn push_checked(&mut self, frame: &[u8], current_revision: u64) -> Result<(), String> {
        if self.failed
            || current_revision != self.request.revision
            || Instant::now() >= self.deadline
        {
            return Err("cancelled, stale or expired page assembly".into());
        }
        if frame.len() > self.max_chunk_bytes
            || self.wire_bytes.saturating_add(frame.len()) > 64 * 1024 * 1024
        {
            return Err("page chunk or total wire budget exceeded".into());
        }
        let mut chunk: PageChunk =
            serde_json::from_slice(frame).map_err(|_| "invalid page chunk")?;
        if chunk.id != self.request.id
            || chunk.project_id != self.request.project_id
            || chunk.revision != self.request.revision
            || self.received >= self.expected_pages
            || chunk.page["number"].as_u64() != Some(self.received as u64 + 1)
        {
            return Err("page identity, order or count invalid".into());
        }
        let items = chunk.page["items"]
            .as_array()
            .ok_or("page items missing")?
            .len();
        if items > 100_000 || self.retained_items.saturating_add(items) > 1_000_000 {
            return Err("page residency item budget exceeded".into());
        }
        // Apply the existing validator to an isolated page before exposing it.
        // Its page-number invariant is local to that one-page envelope.
        let number = chunk.page["number"].take();
        chunk.page["number"] = serde_json::json!(1);
        let mut isolated = self.header.clone();
        isolated["payload"]["pages"] = Value::Array(vec![chunk.page]);
        let mut checked = crate::validate_reply_value(isolated, &self.request, &self.capabilities)?;
        let mut page = checked["payload"]["pages"]
            .as_array_mut()
            .unwrap()
            .pop()
            .unwrap();
        page["number"] = number;
        self.retained_items += items;
        self.result["payload"]["pages"]
            .as_array_mut()
            .unwrap()
            .push(page);
        self.received += 1;
        self.wire_bytes += frame.len();
        Ok(())
    }
    pub fn residency(&self) -> (usize, usize, usize) {
        (self.received, self.retained_items, self.wire_bytes)
    }
    /// The borrowed page is validated but provisional; retain a clone only under
    /// the caller's own memory budget and discard it on cancellation/failure.
    pub fn push_to_sink(
        &mut self,
        frame: &[u8],
        current_revision: u64,
        sink: impl FnOnce(ProvisionalPage<'_>) -> Result<(), String>,
    ) -> Result<(), String> {
        self.push(frame, current_revision)?;
        let event = ProvisionalPage {
            request_id: &self.request.id,
            project_id: &self.request.project_id,
            revision: self.request.revision,
            page: self.result["payload"]["pages"]
                .as_array()
                .unwrap()
                .last()
                .unwrap(),
        };
        if let Err(error) = sink(event) {
            self.cancel();
            return Err(error);
        }
        Ok(())
    }
    /// Required completion path for provisional consumers; no result is returned
    /// unless both full validation and the caller's expected digest match.
    pub fn finish_verified(
        self,
        current_revision: u64,
        expected: [u8; 32],
    ) -> Result<Value, String> {
        let value = self.finish(current_revision)?;
        if canonical_digest(&value)? != expected {
            return Err("completed result digest mismatch".into());
        }
        Ok(value)
    }
    pub fn finish(self, current_revision: u64) -> Result<Value, String> {
        if self.failed
            || current_revision != self.request.revision
            || Instant::now() >= self.deadline
            || self.received != self.expected_pages
        {
            return Err("incomplete, stale or expired page assembly".into());
        }
        crate::validate_reply_value(self.result, &self.request, &self.capabilities)
    }
}

/// This event never authorizes export or marks a revision complete.
pub struct ProvisionalPage<'a> {
    pub request_id: &'a str,
    pub project_id: &'a str,
    pub revision: u64,
    pub page: &'a Value,
}
/// Digest contract: compact serde_json serialization with its ordered map keys.
/// Hashes structure deterministically without allocating a full serialized buffer.
pub fn canonical_digest(value: &Value) -> Result<[u8; 32], String> {
    struct HashWriter(flashtex_project_files::Sha256);
    impl std::io::Write for HashWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.update(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = HashWriter(flashtex_project_files::Sha256::new());
    {
        let mut buffered = std::io::BufWriter::with_capacity(64 * 1024, &mut writer);
        serde_json::to_writer(&mut buffered, value).map_err(|e| e.to_string())?;
        std::io::Write::flush(&mut buffered).map_err(|e| e.to_string())?;
    }
    Ok(writer.0.finalize())
}

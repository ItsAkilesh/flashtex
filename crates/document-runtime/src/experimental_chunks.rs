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
            result,
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
        let chunk: PageChunk = serde_json::from_slice(frame).map_err(|_| "invalid page chunk")?;
        if chunk.id != self.request.id
            || chunk.project_id != self.request.project_id
            || chunk.revision != self.request.revision
            || self.received >= self.expected_pages
            || chunk.page["number"].as_u64() != Some(self.received as u64 + 1)
        {
            return Err("page identity, order or count invalid".into());
        }
        self.result["payload"]["pages"]
            .as_array_mut()
            .unwrap()
            .push(chunk.page);
        self.received += 1;
        self.wire_bytes += frame.len();
        Ok(())
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

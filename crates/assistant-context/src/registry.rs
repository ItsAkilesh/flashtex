//! Caller-owned, bounded lifecycle collection. No provider calls or document edits.
use crate::{Context, ExplanationFlight, ExplanationProposal, FlightState, PromptPayload};
use flashtex_edit_ledger::Document;
use std::{
    collections::{BTreeMap, VecDeque},
    time::Duration,
};

/// Request IDs belong to one registry session. The caller must supply a fresh
/// session identifier after restart and route provider callbacks with this ID.
pub struct ExplanationRegistry {
    session: String,
    sequence: u64,
    pending_limit: usize,
    terminal_limit: usize,
    pending: BTreeMap<String, ExplanationFlight>,
    terminal: VecDeque<(String, FlightState)>,
}
impl ExplanationRegistry {
    pub fn new(
        session: String,
        pending_limit: usize,
        terminal_limit: usize,
    ) -> Result<Self, String> {
        if session.is_empty()
            || session.len() > 128
            || !session
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || !(1..=32).contains(&pending_limit)
            || terminal_limit > 256
        {
            return Err("invalid registry session or retention limits".into());
        }
        Ok(Self {
            session,
            sequence: 0,
            pending_limit,
            terminal_limit,
            pending: BTreeMap::new(),
            terminal: VecDeque::new(),
        })
    }
    fn retire(&mut self, id: String, state: FlightState) {
        self.pending.remove(&id);
        if self.terminal_limit != 0 {
            self.terminal.push_back((id, state));
            while self.terminal.len() > self.terminal_limit {
                self.terminal.pop_front();
            }
        }
    }
    /// Reclaim expired slots. Return IDs so callers can stop their provider tasks.
    pub fn sweep(&mut self) -> Vec<String> {
        let expired: Vec<_> = self
            .pending
            .iter_mut()
            .filter_map(|(id, flight)| (flight.state() == FlightState::Expired).then(|| id.clone()))
            .collect();
        for id in &expired {
            self.retire(id.clone(), FlightState::Expired);
        }
        expired
    }
    pub fn submit(
        &mut self,
        context: Context,
        current: &[Document],
        timeout: Duration,
    ) -> Result<String, String> {
        self.sweep();
        if self.pending.len() >= self.pending_limit {
            return Err("pending explanation limit reached".into());
        }
        let flight = ExplanationFlight::new(context, current, timeout)?;
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or("request sequence exhausted")?;
        let id = format!("{}:{}", self.session, self.sequence);
        self.pending.insert(id.clone(), flight);
        Ok(id)
    }
    pub fn payload(&mut self, id: &str) -> Option<&PromptPayload> {
        self.sweep();
        self.pending.get(id).map(ExplanationFlight::payload)
    }
    pub fn state(&mut self, id: &str) -> Option<FlightState> {
        self.sweep();
        if self.pending.contains_key(id) {
            return Some(FlightState::AwaitingResponse);
        }
        self.terminal
            .iter()
            .find(|(key, _)| key == id)
            .map(|(_, state)| *state)
    }
    pub fn cancel(&mut self, id: &str) -> bool {
        self.sweep();
        if !self.pending.contains_key(id) {
            return false;
        }
        self.retire(id.to_owned(), FlightState::Cancelled);
        true
    }
    /// Only requests for this project are checked. The supplied documents must
    /// be its complete current snapshot; mismatches revoke pending work.
    pub fn revoke_stale(&mut self, project_id: &str, current: &[Document]) -> Vec<String> {
        self.sweep();
        let stale: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, flight)| {
                flight.context.binding.project_id == project_id
                    && flight.context.check_current(current).is_err()
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in &stale {
            self.retire(id.clone(), FlightState::Cancelled);
        }
        stale
    }
    /// Request routing and the immutable context ID are both required. Terminal
    /// callbacks, including duplicate successful responses, cannot yield edits.
    /// The caller remains responsible for explicit approval and ledger validation.
    pub fn receive(
        &mut self,
        id: &str,
        context_id: &str,
        bytes: &[u8],
        current: &[Document],
    ) -> Result<ExplanationProposal, String> {
        self.sweep();
        let flight = self
            .pending
            .get_mut(id)
            .ok_or("unknown or terminal explanation request")?;
        if flight.payload().context_id != context_id {
            return Err("response context routing mismatch".into());
        }
        let result = flight.receive(bytes, current);
        let state = flight.state();
        self.retire(id.to_owned(), state);
        result
    }
    pub fn retained_counts(&self) -> (usize, usize) {
        (self.pending.len(), self.terminal.len())
    }
}

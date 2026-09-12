//! Caller-owned, bounded lifecycle collection. No provider calls or document edits.
use crate::{Context, ExplanationFlight, ExplanationProposal, FlightState, PromptPayload};
use flashtex_edit_ledger::Document;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
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
    leases: BTreeMap<String, Arc<AtomicBool>>,
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
            leases: BTreeMap::new(),
        })
    }
    fn retire(&mut self, id: String, state: FlightState) {
        self.pending.remove(&id);
        if let Some(active) = self.leases.remove(&id) {
            active.store(false, Ordering::Release);
        }
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

/// One transport dispatch per flight. Source context is shared immutably with
/// the registry; dropping/cancelling the registry revokes undispatched leases.
/// A lease is not Clone and does not authorize editing or bypass response checks.
pub struct RequestLease {
    id: String,
    context: Arc<Context>,
    active: Arc<AtomicBool>,
    deadline: Instant,
}
impl RequestLease {
    pub fn request_id(&self) -> &str {
        &self.id
    }
    pub fn payload(&self) -> &PromptPayload {
        self.context.payload()
    }
    pub fn check_current(&self, current: &[Document]) -> Result<(), String> {
        self.check_live()?;
        self.context.check_current(current)
    }
    pub(crate) fn check_live(&self) -> Result<(), String> {
        if !self.active.load(Ordering::Acquire) || Instant::now() >= self.deadline {
            return Err("transport lease revoked or expired".into());
        }
        Ok(())
    }
    #[cfg(feature = "grok")]
    pub(crate) fn context(&self) -> &Context {
        &self.context
    }
}
impl ExplanationRegistry {
    pub fn lease(&mut self, id: &str, current: &[Document]) -> Result<RequestLease, String> {
        self.sweep();
        if self.leases.contains_key(id) {
            return Err("request already leased; no automatic retry".into());
        }
        let flight = self.pending.get(id).ok_or("unknown or terminal request")?;
        flight.context.check_current(current)?;
        let active = Arc::new(AtomicBool::new(true));
        let lease = RequestLease {
            id: id.to_owned(),
            context: Arc::clone(&flight.context),
            active: Arc::clone(&active),
            deadline: flight.deadline,
        };
        self.leases.insert(id.to_owned(), active);
        Ok(lease)
    }
    /// Report a terminal provider failure without retaining untrusted error text.
    pub fn fail(&mut self, id: &str) -> bool {
        self.sweep();
        if !self.pending.contains_key(id) {
            return false;
        }
        self.retire(id.to_owned(), FlightState::Failed);
        true
    }
}
impl Drop for ExplanationRegistry {
    fn drop(&mut self) {
        for active in self.leases.values() {
            active.store(false, Ordering::Release);
        }
    }
}

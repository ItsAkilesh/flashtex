//! Bounded, lossy notifications. State snapshots remain authoritative.
use crate::*;
use std::sync::{
    atomic::AtomicU64,
    mpsc::{sync_channel, Receiver, SyncSender, TrySendError},
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    Queued,
    Running,
    Completed,
    Failed(FailureKind),
    Cancelled,
    RecoveryRequired,
    Progress { percent: u8, message: String },
}
#[derive(Debug, Clone)]
pub struct Event {
    pub sequence: u64,
    pub capture_id: String,
    pub kind: EventKind,
}
#[derive(Debug, Clone)]
pub struct UsageEvidence {
    pub queued: usize,
    pub executing: usize,
    pub retained: usize,
    pub calls_started: u64,
    pub events_dropped: u64,
    pub provider_billing_known: bool,
}
pub(crate) struct Hub {
    subscribers: Mutex<Vec<SyncSender<Event>>>,
    sequence: AtomicU64,
    pub(crate) started: AtomicU64,
    dropped: AtomicU64,
}
impl Hub {
    pub(crate) fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
            sequence: AtomicU64::new(0),
            started: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }
    }
    pub(crate) fn emit(&self, id: &str, kind: EventKind) {
        let mut subscribers = self.subscribers.lock().unwrap();
        let event = Event {
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            capture_id: id.into(),
            kind,
        };
        subscribers.retain(|subscriber| match subscriber.try_send(event.clone()) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(TrySendError::Disconnected(_)) => false,
        });
    }
}
#[derive(Clone)]
pub(crate) struct Reporter {
    pub(crate) id: String,
    pub(crate) hub: Arc<Hub>,
}
impl CancellationToken {
    pub fn progress(&self, percent: u8, message: impl Into<String>) -> Result<(), Error> {
        let message = message.into();
        if percent > 100 || message.len() > 1024 {
            return Err(Error::InvalidProgress);
        }
        if self.is_cancelled() {
            return Err(Error::Stopped);
        }
        if let Some(reporter) = &self.1 {
            reporter
                .hub
                .emit(&reporter.id, EventKind::Progress { percent, message });
        }
        Ok(())
    }
}
impl<T: Send + Sync + 'static> Scheduler<T> {
    pub fn subscribe(&self, capacity: usize) -> Result<Receiver<Event>, Error> {
        if capacity == 0 || capacity > 4096 {
            return Err(Error::InvalidLimits);
        }
        let mut subscribers = self.shared.events.subscribers.lock().unwrap();
        if subscribers.len() >= 64 {
            return Err(Error::SubscriberLimit);
        }
        let (tx, rx) = sync_channel(capacity);
        subscribers.push(tx);
        Ok(rx)
    }
    pub fn usage(&self) -> UsageEvidence {
        let data = self.shared.data.lock().unwrap();
        UsageEvidence {
            queued: data.queue.len(),
            executing: data.jobs.values().filter(|j| j.executing).count(),
            retained: data.jobs.len(),
            calls_started: self.shared.events.started.load(Ordering::Relaxed),
            events_dropped: self.shared.events.dropped.load(Ordering::Relaxed),
            provider_billing_known: false,
        }
    }
}
pub(crate) fn kind<T>(state: &State<T>) -> EventKind {
    match state {
        State::Queued => EventKind::Queued,
        State::Running => EventKind::Running,
        State::Completed(_) => EventKind::Completed,
        State::Failed(f) => EventKind::Failed(f.kind.clone()),
        State::Cancelled => EventKind::Cancelled,
        State::RecoveryRequired(_) => EventKind::RecoveryRequired,
    }
}

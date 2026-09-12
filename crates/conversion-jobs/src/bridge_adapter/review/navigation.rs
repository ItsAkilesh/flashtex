//! Stable filtered navigation and bounded notifications for durable inbox changes.
use super::*;
use std::sync::{
    atomic::AtomicU64,
    mpsc::{sync_channel, Receiver, SyncSender, TrySendError},
};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InboxFilter {
    pub project_id: Option<String>,
    pub undecided_only: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    Previous,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InboxEventKind {
    Added,
    SelectionChanged,
    ContextChanged,
    DecisionRecorded,
    Cancelled,
    Retired,
    Checkpoint,
    PersistenceUncertain,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxEvent {
    pub generation: u64,
    pub capture_id: Option<String>,
    pub project_id: Option<String>,
    pub kind: InboxEventKind,
}
#[derive(Debug, Clone, Copy)]
pub struct EventUsage {
    pub subscribers: usize,
    pub dropped_deliveries: u64,
}
pub(super) struct Hub {
    subscribers: Mutex<Vec<SyncSender<InboxEvent>>>,
    dropped: AtomicU64,
}
impl Hub {
    pub(super) fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new()),
            dropped: AtomicU64::new(0),
        }
    }
    pub(super) fn emit(&self, event: InboxEvent) {
        self.subscribers.lock().unwrap().retain(|subscriber| {
            match subscriber.try_send(event.clone()) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) => {
                    self.dropped.fetch_add(1, Ordering::Relaxed);
                    true
                }
                Err(TrySendError::Disconnected(_)) => false,
            }
        });
    }
}
impl InboxView {
    pub fn visible_ids(&self, filter: &InboxFilter) -> InboxResult<Vec<String>> {
        if let Some(project) = &filter.project_id {
            valid_id(project)?;
        }
        let snapshot = self.snapshot()?;
        let mut entries = snapshot
            .entries
            .values()
            .filter(|entry| {
                filter
                    .project_id
                    .as_ref()
                    .is_none_or(|project| project == &entry.context.project_id)
                    && (!filter.undecided_only || (!entry.cancelled && entry.decision_id.is_none()))
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.sequence);
        Ok(entries
            .into_iter()
            .map(|entry| entry.capture_id.clone())
            .collect())
    }
}
impl ReviewInbox {
    /// Navigation changes selection only, never records an approval. At an end the
    /// selection stays put; an empty filtered view explicitly clears selection.
    pub fn navigate(
        &mut self,
        filter: &InboxFilter,
        direction: Direction,
    ) -> InboxResult<Option<String>> {
        self.ready()?;
        let visible = self.view.visible_ids(filter)?;
        if visible.is_empty() {
            self.select(None)?;
            return Ok(None);
        }
        let position = self
            .state
            .selected_capture
            .as_ref()
            .and_then(|id| visible.iter().position(|candidate| candidate == id));
        let next = match (position, direction) {
            (None, Direction::Next) => Some(0),
            (None, Direction::Previous) => Some(visible.len() - 1),
            (Some(index), Direction::Next) => (index + 1 < visible.len()).then_some(index + 1),
            (Some(index), Direction::Previous) => index.checked_sub(1),
        };
        if let Some(index) = next {
            let id = visible[index].clone();
            self.select(Some(&id))?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
    pub fn subscribe(&self, capacity: usize) -> InboxResult<Receiver<InboxEvent>> {
        if capacity == 0 || capacity > 4096 {
            return Err(InboxError::Invalid);
        }
        let mut subscribers = self.events.subscribers.lock().unwrap();
        if subscribers.len() >= 64 {
            return Err(InboxError::Capacity);
        }
        let (tx, rx) = sync_channel(capacity);
        subscribers.push(tx);
        Ok(rx)
    }
    pub fn event_usage(&self) -> EventUsage {
        EventUsage {
            subscribers: self.events.subscribers.lock().unwrap().len(),
            dropped_deliveries: self.events.dropped.load(Ordering::Relaxed),
        }
    }
}
pub(super) fn describe(old: &InboxSnapshot, next: &InboxSnapshot) -> InboxEvent {
    let (kind, id) = if let Some(id) = next
        .entries
        .keys()
        .find(|id| !old.entries.contains_key(*id))
    {
        (InboxEventKind::Added, Some(id.clone()))
    } else if let Some(id) = old
        .entries
        .keys()
        .find(|id| !next.entries.contains_key(*id))
    {
        (InboxEventKind::Retired, Some(id.clone()))
    } else if old.selected_capture != next.selected_capture {
        (
            InboxEventKind::SelectionChanged,
            next.selected_capture.clone(),
        )
    } else if let Some(decision) = next
        .decisions
        .values()
        .find(|decision| !old.decisions.contains_key(&decision.decision_id))
    {
        (
            InboxEventKind::DecisionRecorded,
            Some(decision.capture_id.clone()),
        )
    } else if let Some(entry) = next.entries.values().find(|entry| {
        old.entries.get(&entry.capture_id).is_some_and(|before| {
            before.current_context != entry.current_context
                || before.context_revoked != entry.context_revoked
        })
    }) {
        (
            InboxEventKind::ContextChanged,
            Some(entry.capture_id.clone()),
        )
    } else if let Some(entry) = next.entries.values().find(|entry| {
        old.entries
            .get(&entry.capture_id)
            .is_some_and(|before| before.cancelled != entry.cancelled)
    }) {
        (InboxEventKind::Cancelled, Some(entry.capture_id.clone()))
    } else {
        (InboxEventKind::Checkpoint, None)
    };
    let project_id = id
        .as_ref()
        .and_then(|id| next.entries.get(id).or_else(|| old.entries.get(id)))
        .map(|entry| entry.context.project_id.clone());
    InboxEvent {
        generation: next.generation,
        capture_id: id,
        project_id,
        kind,
    }
}

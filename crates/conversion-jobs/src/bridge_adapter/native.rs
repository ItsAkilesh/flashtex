//! Typed IPC-facing commands; transport and UI wiring remain external.
use super::*;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, TryRecvError};
pub const MAX_COMMAND_BYTES: usize = 16 * 1024;
pub const MAX_STATUS_BYTES: usize = 512 * 1024;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextIdentity {
    pub project_id: String,
    pub path: String,
    pub revision: u64,
    pub sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub protocol_version: u8,
    pub id: String,
    pub command: Command,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Command {
    Admit {
        capture_id: String,
        context: ContextIdentity,
        supported_features: Vec<String>,
    },
    Start {
        capture_id: String,
        context: ContextIdentity,
    },
    Status {
        capture_id: String,
        context: ContextIdentity,
    },
    Reconcile {
        capture_id: String,
        context: ContextIdentity,
    },
    Cancel {
        capture_id: String,
        context: ContextIdentity,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
}
type WireResult<T> = std::result::Result<T, ProtocolError>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntentState {
    NotRecorded,
    DurableMayHaveStarted,
    ProposalJournaled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionState {
    Admitted,
    Queued,
    Running,
    AwaitingJournal,
    ProposalReady(Proposal),
    Failed { kind: FailureKind, message: String },
    Cancelled,
    RecoveryRequired,
    StaleContext,
    Prepared,
    Applied,
    Rejected,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSnapshot {
    pub capture_id: String,
    pub context: ContextIdentity,
    pub current_context: ContextIdentity,
    pub capture_durably_received: bool,
    pub intent: IntentState,
    pub conversion: ConversionState,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub protocol_version: u8,
    pub id: String,
    pub status: StatusSnapshot,
}
#[derive(Clone)]
struct Binding {
    original: ContextIdentity,
    current: ContextIdentity,
    supported: Vec<String>,
    started: bool,
    intent: IntentState,
    event_floor: u64,
}
#[derive(Clone)]
pub struct NativeStatusHandle {
    status: StatusHandle,
    bindings: Arc<Mutex<BTreeMap<String, Binding>>>,
}
impl NativeStatusHandle {
    /// Pure memory read; native status polling never touches journal/provider IO.
    pub fn snapshot(&self, id: &str, context: &ContextIdentity) -> WireResult<StatusSnapshot> {
        let binding = self
            .bindings
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| error("not_admitted", "Admit the durable capture before dispatch"))?;
        if &binding.original != context {
            return Err(error(
                "context_identity_mismatch",
                "Capture belongs to a different admitted context",
            ));
        }
        let conversion = if !binding.started {
            if matches!(binding.intent, IntentState::DurableMayHaveStarted) {
                ConversionState::RecoveryRequired
            } else {
                ConversionState::Admitted
            }
        } else {
            wire_state(self.status.status(id).map_err(adapter_error)?)
        };
        let intent = if matches!(conversion, ConversionState::ProposalReady(_)) {
            IntentState::ProposalJournaled
        } else {
            binding.intent
        };
        Ok(StatusSnapshot {
            capture_id: id.into(),
            context: binding.original,
            current_context: binding.current,
            capture_durably_received: true,
            intent,
            conversion,
        })
    }
    pub fn encode(
        &self,
        id: &str,
        context: &ContextIdentity,
        maximum: usize,
    ) -> WireResult<Vec<u8>> {
        bounded_json(&self.snapshot(id, context)?, maximum)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeEvent {
    pub sequence: u64,
    pub capture_id: String,
    pub context: ContextIdentity,
    pub event: events::EventKind,
}
pub struct EventSubscription {
    events: Receiver<events::Event>,
    bindings: Arc<Mutex<BTreeMap<String, Binding>>>,
}
impl EventSubscription {
    pub fn try_next(&self) -> WireResult<Option<NativeEvent>> {
        loop {
            match self.events.try_recv() {
                Ok(event) => {
                    let binding = self
                        .bindings
                        .lock()
                        .unwrap()
                        .get(&event.capture_id)
                        .cloned();
                    if let Some(binding) = binding {
                        if event.sequence >= binding.event_floor {
                            return Ok(Some(NativeEvent {
                                sequence: event.sequence,
                                capture_id: event.capture_id,
                                context: binding.original,
                                event: event.kind,
                            }));
                        }
                    }
                }
                Err(TryRecvError::Empty) => return Ok(None),
                Err(TryRecvError::Disconnected) => {
                    return Err(error("events_closed", "Conversion event source stopped"))
                }
            }
        }
    }
}
pub struct NativeService {
    adapter: BridgeAdapter,
    handle: NativeStatusHandle,
}
impl NativeService {
    pub fn new(adapter: BridgeAdapter) -> Self {
        let handle = NativeStatusHandle {
            status: adapter.status_handle(),
            bindings: Arc::new(Mutex::new(BTreeMap::new())),
        };
        Self { adapter, handle }
    }
    pub fn status_handle(&self) -> NativeStatusHandle {
        self.handle.clone()
    }
    pub fn subscribe(&self, capacity: usize) -> WireResult<EventSubscription> {
        Ok(EventSubscription {
            events: self.adapter.scheduler.subscribe(capacity).map_err(|_| {
                error(
                    "event_limit",
                    "Invalid event capacity or subscription limit",
                )
            })?,
            bindings: self.handle.bindings.clone(),
        })
    }
    pub fn context_identity(
        bridge: &Bridge,
        id: &str,
        supported: Vec<String>,
    ) -> WireResult<ContextIdentity> {
        let record = bridge
            .store
            .require(id)
            .map_err(|_| error("capture_missing", "Durable capture is unavailable"))?;
        let context = bridge.context(&record.capture, supported).map_err(|_| {
            error(
                "context_unavailable",
                "Current source context is unavailable",
            )
        })?;
        identity(&context)
    }
    /// Run commands on the background document actor; only status handles are UI-safe.
    pub fn execute(&mut self, bridge: &mut Bridge, request: Request) -> WireResult<Response> {
        if request.protocol_version != 1 {
            return Err(error(
                "unsupported_version",
                "Only protocol_version 1 is accepted",
            ));
        }
        flashtex_bridge::identifier(&request.id)
            .map_err(|_| error("invalid_request_id", "Invalid request correlation ID"))?;
        let (id, expected) = match &request.command {
            Command::Admit {
                capture_id,
                context,
                ..
            }
            | Command::Start {
                capture_id,
                context,
            }
            | Command::Status {
                capture_id,
                context,
            }
            | Command::Reconcile {
                capture_id,
                context,
            }
            | Command::Cancel {
                capture_id,
                context,
            } => (capture_id.clone(), context.clone()),
        };
        flashtex_bridge::identifier(&id)
            .map_err(|_| error("invalid_capture_id", "Invalid capture identity"))?;
        if let Command::Admit {
            supported_features, ..
        } = &request.command
        {
            let current = Self::context_identity(bridge, &id, supported_features.clone())?;
            if current != expected {
                return Err(error(
                    "context_identity_mismatch",
                    "Admission must use the exact current source context",
                ));
            }
            let mut bindings = self.handle.bindings.lock().unwrap();
            if let Some(old) = bindings.get(&id) {
                if old.original != expected {
                    return Err(error(
                        "context_identity_mismatch",
                        "Capture already admitted under another context",
                    ));
                }
            } else {
                if bindings.len() >= self.adapter.retained {
                    return Err(error(
                        "admission_limit",
                        "Retire terminal captures before admitting more",
                    ));
                }
                bindings.insert(
                    id.clone(),
                    Binding {
                        original: expected.clone(),
                        current: expected.clone(),
                        supported: supported_features.clone(),
                        started: false,
                        intent: if self
                            .adapter
                            .directory
                            .join(format!("{id}.intent.json"))
                            .exists()
                        {
                            IntentState::DurableMayHaveStarted
                        } else {
                            IntentState::NotRecorded
                        },
                        event_floor: self
                            .adapter
                            .scheduler
                            .shared
                            .events
                            .sequence
                            .load(Ordering::Relaxed),
                    },
                );
            }
        } else {
            let binding = self
                .handle
                .bindings
                .lock()
                .unwrap()
                .get(&id)
                .cloned()
                .ok_or_else(|| error("not_admitted", "Admit the durable capture first"))?;
            if matches!(request.command, Command::Reconcile { .. }) {
                let current = match Self::context_identity(bridge, &id, binding.supported.clone()) {
                    Ok(current) => current,
                    Err(error) => {
                        if binding.started
                            && expected.project_id == binding.original.project_id
                            && expected.path == binding.original.path
                        {
                            // Invalidation/reselection must revoke exposed status,
                            // even when no new context identity can be assembled.
                            self.adapter.reconcile(bridge, &id).map_err(adapter_error)?;
                        }
                        return Err(error);
                    }
                };
                if current != expected {
                    return Err(error(
                        "context_identity_mismatch",
                        "Reconcile must use the current source context",
                    ));
                }
                self.handle
                    .bindings
                    .lock()
                    .unwrap()
                    .get_mut(&id)
                    .unwrap()
                    .current = current;
                if binding.started {
                    self.adapter.reconcile(bridge, &id).map_err(adapter_error)?;
                }
            } else {
                if expected != binding.original {
                    return Err(error(
                        "context_identity_mismatch",
                        "Command does not match the admitted context",
                    ));
                }
                match request.command {
                    Command::Start { .. } => {
                        let current =
                            Self::context_identity(bridge, &id, binding.supported.clone())?;
                        if current != expected {
                            return Err(error(
                                "context_identity_mismatch",
                                "Source changed after admission; no conversion started",
                            ));
                        }
                        self.adapter
                            .start(bridge, &id, binding.supported)
                            .map_err(adapter_error)?;
                        let durable = self
                            .adapter
                            .directory
                            .join(format!("{id}.intent.json"))
                            .exists();
                        let mut bindings = self.handle.bindings.lock().unwrap();
                        let value = bindings.get_mut(&id).unwrap();
                        value.started = true;
                        value.intent = if durable {
                            IntentState::DurableMayHaveStarted
                        } else {
                            IntentState::NotRecorded
                        };
                    }
                    Command::Cancel { .. } => {
                        if binding.started {
                            self.adapter.cancel(&id).map_err(adapter_error)?;
                        }
                    }
                    Command::Status { .. } => (),
                    _ => unreachable!(),
                }
            }
        }
        let original = self
            .handle
            .bindings
            .lock()
            .unwrap()
            .get(&id)
            .unwrap()
            .original
            .clone();
        Ok(Response {
            protocol_version: 1,
            id: request.id,
            status: self.handle.snapshot(&id, &original)?,
        })
    }
    pub fn execute_bytes(&mut self, bridge: &mut Bridge, bytes: &[u8]) -> WireResult<Vec<u8>> {
        if bytes.len() > MAX_COMMAND_BYTES {
            return Err(error("command_too_large", "Command exceeds byte limit"));
        }
        let request = serde_json::from_slice(bytes)
            .map_err(|_| error("invalid_command", "Malformed or unknown typed command"))?;
        bounded_json(&self.execute(bridge, request)?, MAX_STATUS_BYTES)
    }
    pub fn retire(&mut self, id: &str) -> WireResult<()> {
        self.adapter.retire(id).map_err(adapter_error)?;
        self.handle.bindings.lock().unwrap().remove(id);
        Ok(())
    }
}
fn identity(context: &Context) -> WireResult<ContextIdentity> {
    let fingerprint = fingerprint(context).map_err(adapter_error)?;
    Ok(ContextIdentity {
        project_id: context.project_id.clone(),
        path: context.path.clone(),
        revision: fingerprint.revision,
        sha256: fingerprint.sha256,
    })
}
fn error(code: &str, message: &str) -> ProtocolError {
    ProtocolError {
        code: code.into(),
        message: message.into(),
    }
}
fn adapter_error(_error: AdapterError) -> ProtocolError {
    error(
        "adapter_error",
        "Conversion command failed; inspect the capture state and journal before retry",
    )
}
fn wire_state(state: AdapterState) -> ConversionState {
    match state {
        AdapterState::Queued => ConversionState::Queued,
        AdapterState::Running => ConversionState::Running,
        AdapterState::AwaitingJournal => ConversionState::AwaitingJournal,
        AdapterState::Proposal(p) => ConversionState::ProposalReady((*p).clone()),
        AdapterState::Failed(f) => {
            let mut end = f.message.len().min(2048);
            while !f.message.is_char_boundary(end) {
                end -= 1;
            }
            ConversionState::Failed {
                kind: f.kind,
                message: f.message[..end].into(),
            }
        }
        AdapterState::Cancelled => ConversionState::Cancelled,
        AdapterState::RecoveryRequired => ConversionState::RecoveryRequired,
        AdapterState::StaleContext => ConversionState::StaleContext,
        AdapterState::Prepared => ConversionState::Prepared,
        AdapterState::Applied => ConversionState::Applied,
        AdapterState::Rejected => ConversionState::Rejected,
    }
}
struct BoundedBytes {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("bounded status output exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn bounded_json(value: &impl Serialize, maximum: usize) -> WireResult<Vec<u8>> {
    if maximum == 0 || maximum > MAX_STATUS_BYTES {
        return Err(error(
            "invalid_limit",
            "Status limit outside permitted bounds",
        ));
    }
    let mut out = BoundedBytes {
        bytes: Vec::new(),
        limit: maximum,
    };
    serde_json::to_writer(&mut out, value)
        .map_err(|_| error("status_too_large", "Status exceeds bounded output limit"))?;
    Ok(out.bytes)
}

#[cfg(feature = "bridge-integration")]
pub mod bridge_adapter;
pub mod events;
pub mod snapshot;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextFingerprint {
    pub revision: u64,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureKind {
    Provider,
    AmbiguousProvider,
    StaleContext,
    Panicked,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    pub kind: FailureKind,
    pub message: String,
}
impl Failure {
    pub fn new(kind: FailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}
#[derive(Debug)]
pub enum State<T> {
    Queued,
    Running,
    Completed(Arc<T>),
    Failed(Failure),
    Cancelled,
    RecoveryRequired(snapshot::RecoveryReason),
}
impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Queued => Self::Queued,
            Self::Running => Self::Running,
            Self::Completed(v) => Self::Completed(v.clone()),
            Self::Failed(v) => Self::Failed(v.clone()),
            Self::Cancelled => Self::Cancelled,
            Self::RecoveryRequired(reason) => Self::RecoveryRequired(reason.clone()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidLimits,
    InvalidProgress,
    SubscriberLimit,
    WorkerUnavailable,
    InvalidIdentity,
    InvalidFingerprint,
    QueueFull,
    RetentionFull,
    DuplicateIdentity,
    Missing,
    StillExecuting,
    NotTerminal,
    Stopped,
    RecoveryAuthorizationRequired,
}
#[derive(Clone)]
pub struct CancellationToken(Arc<AtomicBool>, Option<events::Reporter>);
impl CancellationToken {
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
type Task<T> = Box<dyn FnOnce(CancellationToken) -> Result<T, Failure> + Send + 'static>;
struct Job<T> {
    expected: ContextFingerprint,
    current: ContextFingerprint,
    state: State<T>,
    token: CancellationToken,
    task: Option<Task<T>>,
    executing: bool,
}
struct Data<T> {
    jobs: BTreeMap<String, Job<T>>,
    queue: VecDeque<String>,
    stopped: bool,
}
struct Shared<T> {
    data: Mutex<Data<T>>,
    wake: Condvar,
    events: Arc<events::Hub>,
}
/// `max_queued` bounds pending calls; `max_retained` also bounds completed records.
/// Explicit `forget` is required to release terminal identities/results.
pub struct Scheduler<T> {
    shared: Arc<Shared<T>>,
    max_queued: usize,
    max_retained: usize,
}
impl<T: Send + Sync + 'static> Scheduler<T> {
    pub fn new(workers: usize, max_queued: usize, max_retained: usize) -> Result<Self, Error> {
        if workers == 0 || workers > 64 || max_queued == 0 || max_retained == 0 {
            return Err(Error::InvalidLimits);
        }
        let shared = Arc::new(Shared {
            data: Mutex::new(Data {
                jobs: BTreeMap::new(),
                queue: VecDeque::new(),
                stopped: false,
            }),
            wake: Condvar::new(),
            events: Arc::new(events::Hub::new()),
        });
        for _ in 0..workers {
            let shared = shared.clone();
            let worker_shared = shared.clone();
            if thread::Builder::new()
                .name("flashtex-conversion".into())
                .spawn(move || worker(worker_shared))
                .is_err()
            {
                shared.data.lock().unwrap().stopped = true;
                shared.wake.notify_all();
                return Err(Error::WorkerUnavailable);
            }
        }
        Ok(Self {
            shared,
            max_queued,
            max_retained,
        })
    }
    /// Duplicate identity never invokes the provider again, including after failure.
    pub fn submit(
        &self,
        id: impl Into<String>,
        context: ContextFingerprint,
        convert: impl FnOnce(CancellationToken) -> Result<T, Failure> + Send + 'static,
    ) -> Result<(), Error> {
        let id = id.into();
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(Error::InvalidIdentity);
        }
        validate_fingerprint(&context)?;
        let mut data = self.shared.data.lock().unwrap();
        if data.stopped {
            return Err(Error::Stopped);
        }
        if data.jobs.contains_key(&id) {
            return Err(Error::DuplicateIdentity);
        }
        if data.queue.len() >= self.max_queued {
            return Err(Error::QueueFull);
        }
        if data.jobs.len() >= self.max_retained {
            return Err(Error::RetentionFull);
        }
        data.jobs.insert(
            id.clone(),
            Job {
                expected: context.clone(),
                current: context,
                state: State::Queued,
                token: CancellationToken(
                    Arc::new(AtomicBool::new(false)),
                    Some(events::Reporter {
                        id: id.clone(),
                        hub: self.shared.events.clone(),
                    }),
                ),
                task: Some(Box::new(convert)),
                executing: false,
            },
        );
        self.shared.events.emit(&id, events::EventKind::Queued);
        data.queue.push_back(id);
        self.shared.wake.notify_one();
        Ok(())
    }
    pub fn state(&self, id: &str) -> Result<State<T>, Error> {
        self.shared
            .data
            .lock()
            .unwrap()
            .jobs
            .get(id)
            .map(|j| j.state.clone())
            .ok_or(Error::Missing)
    }
    /// The adapter updates this under its document revision transaction, before
    /// consuming a result. Completed outputs are revoked when their context changes.
    pub fn update_context(&self, id: &str, current: ContextFingerprint) -> Result<(), Error> {
        validate_fingerprint(&current)?;
        let mut data = self.shared.data.lock().unwrap();
        let job = data.jobs.get_mut(id).ok_or(Error::Missing)?;
        job.current = current;
        if job.current != job.expected && matches!(job.state, State::Completed(_)) {
            job.state = State::Failed(stale());
            self.shared.events.emit(id, events::kind(&job.state));
        }
        Ok(())
    }
    /// Cooperative cancellation cannot forcibly stop network IO. Running slots and
    /// identities remain reserved until the converter actually returns.
    pub fn cancel(&self, id: &str) -> Result<(), Error> {
        let mut data = self.shared.data.lock().unwrap();
        let job = data.jobs.get_mut(id).ok_or(Error::Missing)?;
        if matches!(job.state, State::Queued | State::Running | State::Cancelled) {
            job.token.0.store(true, Ordering::Release);
            job.state = State::Cancelled;
            self.shared.events.emit(id, events::EventKind::Cancelled);
            job.task = None;
            data.queue.retain(|queued| queued != id);
        }
        Ok(())
    }
    /// Explicit retirement permits identity reuse; caller owns durable deduplication.
    pub fn forget(&self, id: &str) -> Result<State<T>, Error> {
        let mut data = self.shared.data.lock().unwrap();
        let job = data.jobs.get(id).ok_or(Error::Missing)?;
        if job.executing {
            return Err(Error::StillExecuting);
        }
        if matches!(job.state, State::Queued | State::Running) {
            return Err(Error::NotTerminal);
        }
        Ok(data.jobs.remove(id).unwrap().state)
    }
}
impl<T> Drop for Scheduler<T> {
    fn drop(&mut self) {
        let mut data = self.shared.data.lock().unwrap();
        data.stopped = true;
        data.queue.clear();
        for job in data.jobs.values_mut() {
            job.token.0.store(true, Ordering::Release);
            job.task = None;
        }
        self.shared.wake.notify_all();
        // Do not block editor shutdown on an uncooperative provider. Detached
        // worker exits when its bounded converter returns; no new task can start.
    }
}
fn validate_fingerprint(context: &ContextFingerprint) -> Result<(), Error> {
    if context.sha256.len() != 64
        || !context
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Error::InvalidFingerprint);
    }
    Ok(())
}
fn stale() -> Failure {
    Failure::new(
        FailureKind::StaleContext,
        "Conversion context changed; explicit new conversion and review required",
    )
}
fn worker<T: Send + Sync + 'static>(shared: Arc<Shared<T>>) {
    loop {
        let (id, task, token) = {
            let mut data = shared.data.lock().unwrap();
            loop {
                if data.stopped {
                    return;
                }
                if let Some(id) = data.queue.pop_front() {
                    let job = data.jobs.get_mut(&id).unwrap();
                    if job.current != job.expected {
                        job.state = State::Failed(stale());
                        shared.events.emit(&id, events::kind(&job.state));
                        job.task = None;
                        continue;
                    }
                    if let Some(task) = job.task.take() {
                        job.state = State::Running;
                        job.executing = true;
                        shared.events.started.fetch_add(1, Ordering::Relaxed);
                        shared.events.emit(&id, events::EventKind::Running);
                        break (id, task, job.token.clone());
                    }
                } else {
                    data = shared.wake.wait(data).unwrap();
                }
            }
        };
        let result = catch_unwind(AssertUnwindSafe(|| task(token)));
        let mut data = shared.data.lock().unwrap();
        let job = data.jobs.get_mut(&id).unwrap();
        job.executing = false;
        job.state = if job.token.is_cancelled() {
            State::Cancelled
        } else if job.current != job.expected {
            State::Failed(stale())
        } else {
            match result {
                Ok(Ok(value)) => State::Completed(Arc::new(value)),
                Ok(Err(error)) => State::Failed(error),
                Err(_) => State::Failed(Failure::new(
                    FailureKind::Panicked,
                    "Converter panicked; provider completion may be ambiguous, no automatic retry",
                )),
            }
        };
        shared.events.emit(&id, events::kind(&job.state));
    }
}

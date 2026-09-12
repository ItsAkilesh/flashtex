//! Bounded background provider admission. Never retries or applies edits.
use crate::{
    grok::{GrokClient, RoutedReply},
    Context, ExplanationProposal, ExplanationRegistry, FlightState, RequestLease,
};
use flashtex_conversion_jobs::{ContextFingerprint, Failure, FailureKind, Scheduler, State};
use flashtex_edit_ledger::Document;
use flashtex_project_files::sha256_hex;
use std::{collections::BTreeMap, sync::Arc, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Ready,
    Cancelled,
    Failed,
    Expired,
    Consumed,
}
/// Explicit intent supplied by the host after its user/provider billing checks.
/// This is an audit label, not proof of available credit or a dollar-spend cap.
pub struct UsageIntent {
    pub user_requested: bool,
    pub allocation: String,
}
#[derive(serde::Serialize)]
pub struct QueueEntry {
    pub request_id: String,
    pub state: JobStatus,
    pub allocation: String,
}
#[derive(serde::Serialize)]
pub struct QueueSnapshot {
    pub jobs: Vec<QueueEntry>,
    pub queued: usize,
    pub executing: usize,
    pub retained: usize,
    /// Scheduler task starts, including preflight rejection; not HTTP attempts.
    pub scheduler_tasks_started: u64,
    pub provider_billing_known: bool,
}
pub struct ProviderQueue {
    registry: ExplanationRegistry,
    scheduler: Scheduler<RoutedReply>,
    records: BTreeMap<String, (String, String)>, // registry ID -> scheduler ID, allocation
    max_retained: usize,
}
impl ProviderQueue {
    pub fn new(
        session: String,
        workers: usize,
        max_queued: usize,
        max_retained: usize,
    ) -> Result<Self, String> {
        if workers == 0
            || workers > 8
            || max_queued == 0
            || max_queued > 32
            || max_retained == 0
            || max_retained > 32
        {
            return Err("invalid provider queue limits".into());
        }
        Ok(Self {
            registry: ExplanationRegistry::new(session, 32, 64)?,
            scheduler: Scheduler::new(workers, max_queued, max_retained)
                .map_err(|e| format!("scheduler {e:?}"))?,
            records: BTreeMap::new(),
            max_retained,
        })
    }
    pub fn submit(
        &mut self,
        context: Context,
        current: &[Document],
        timeout: Duration,
        intent: UsageIntent,
        client: Arc<GrokClient>,
    ) -> Result<String, String> {
        self.submit_with(context, current, timeout, intent, move |lease| {
            client.request_admitted_lease(lease)
        })
    }
    fn submit_with(
        &mut self,
        context: Context,
        current: &[Document],
        timeout: Duration,
        intent: UsageIntent,
        provider: impl FnOnce(RequestLease) -> Result<RoutedReply, String> + Send + 'static,
    ) -> Result<String, String> {
        if !intent.user_requested
            || intent.allocation.is_empty()
            || intent.allocation.len() > 128
            || intent.allocation.chars().any(char::is_control)
        {
            return Err("explicit bounded usage intent required".into());
        }
        if self.records.len() >= self.max_retained {
            return Err("provider retention full; retire terminal requests".into());
        }
        let fingerprint = ContextFingerprint {
            revision: context.payload().compile_revision,
            sha256: context.payload().context_id.clone(),
        };
        let id = self.registry.submit(context, current, timeout)?;
        let lease = self.registry.lease(&id, current)?;
        // Registry IDs include ':', which the scheduler disallows. Hash the full
        // exact identity, retaining its original separately for response routing.
        let job_id = sha256_hex(id.as_bytes());
        if let Err(error) = self
            .scheduler
            .submit(job_id.clone(), fingerprint, move |token| {
                if token.is_cancelled() {
                    return Err(Failure::new(
                        FailureKind::Provider,
                        "cancelled before provider dispatch",
                    ));
                }
                provider(lease)
                    .map_err(|_| Failure::new(FailureKind::Provider, "provider failed; no retry"))
            })
        {
            self.registry.fail(&id);
            return Err(format!("scheduler admission {error:?}"));
        }
        self.records.insert(id.clone(), (job_id, intent.allocation));
        Ok(id)
    }
    fn job(&self, id: &str) -> Result<&str, String> {
        self.records
            .get(id)
            .map(|r| r.0.as_str())
            .ok_or("unknown provider request".into())
    }
    pub fn allocation(&self, id: &str) -> Option<&str> {
        self.records.get(id).map(|r| r.1.as_str())
    }
    pub fn status(&mut self, id: &str) -> Result<JobStatus, String> {
        let job = self.job(id)?.to_owned();
        match self.registry.state(id) {
            Some(FlightState::Cancelled) => {
                let _ = self.scheduler.cancel(&job);
                return Ok(JobStatus::Cancelled);
            }
            Some(FlightState::Expired) => {
                let _ = self.scheduler.cancel(&job);
                return Ok(JobStatus::Expired);
            }
            Some(FlightState::Completed) => return Ok(JobStatus::Consumed),
            Some(FlightState::Failed) => return Ok(JobStatus::Failed),
            _ => {}
        }
        Ok(
            match self
                .scheduler
                .state(&job)
                .map_err(|e| format!("scheduler {e:?}"))?
            {
                State::Queued => JobStatus::Queued,
                State::Running => JobStatus::Running,
                State::Completed(_) => JobStatus::Ready,
                State::Cancelled => JobStatus::Cancelled,
                State::Failed(_) | State::RecoveryRequired(_) => {
                    self.registry.fail(id);
                    JobStatus::Failed
                }
            },
        )
    }
    pub fn cancel(&mut self, id: &str) -> Result<(), String> {
        let job = self.job(id)?.to_owned();
        self.registry.cancel(id);
        self.scheduler
            .cancel(&job)
            .map_err(|e| format!("scheduler {e:?}"))
    }
    pub fn revoke_stale(&mut self, project: &str, current: &[Document]) -> Vec<String> {
        let ids = self.registry.revoke_stale(project, current);
        for id in &ids {
            if let Ok(job) = self.job(id) {
                let _ = self.scheduler.cancel(job);
            }
        }
        ids
    }
    /// Consume at most one proposal, validating current source at the handoff.
    pub fn receive(
        &mut self,
        id: &str,
        current: &[Document],
    ) -> Result<ExplanationProposal, String> {
        if self.status(id)? != JobStatus::Ready {
            return Err("provider result not ready".into());
        }
        let job = self.job(id)?.to_owned();
        let state = self
            .scheduler
            .forget(&job)
            .map_err(|e| format!("scheduler {e:?}"))?;
        self.records.remove(id);
        match state {
            State::Completed(reply) => Arc::try_unwrap(reply)
                .map_err(|_| "provider result still referenced")?
                .receive(&mut self.registry, current),
            _ => Err("provider result changed before consumption".into()),
        }
    }
    /// Running calls cannot free their slot until their bounded HTTP call exits.
    pub fn retire(&mut self, id: &str) -> Result<(), String> {
        let job = self.job(id)?.to_owned();
        self.scheduler
            .forget(&job)
            .map_err(|e| format!("scheduler {e:?}"))?;
        self.registry.cancel(id);
        self.records.remove(id);
        Ok(())
    }
    /// Bounded native-facing status with no source, proposals, keys or provider
    /// error bodies. Polling also propagates expired registry jobs to scheduler.
    pub fn snapshot(&mut self) -> Result<QueueSnapshot, String> {
        let ids: Vec<_> = self.records.keys().cloned().collect();
        let mut jobs = Vec::with_capacity(ids.len());
        for id in ids {
            let state = self.status(&id)?;
            jobs.push(QueueEntry {
                allocation: self.records[&id].1.clone(),
                request_id: id,
                state,
            });
        }
        let usage = self.scheduler.usage();
        Ok(QueueSnapshot {
            jobs,
            queued: usage.queued,
            executing: usage.executing,
            retained: usage.retained,
            scheduler_tasks_started: usage.calls_started,
            provider_billing_known: false,
        })
    }
    pub fn usage(&self) -> flashtex_conversion_jobs::events::UsageEvidence {
        self.scheduler.usage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc,
        },
        time::Instant,
    };
    fn source() -> Vec<Document> {
        vec![Document::new("p".into(), "main.tex".into(), 1, "hello".into()).unwrap()]
    }
    fn context(docs: &[Document]) -> Context {
        Context::build(crate::CompileBinding::capture("r","p",1,docs).unwrap(),docs,&json!({"protocol_version":1,"type":"compile_result","id":"r","payload":{"project_id":"p","revision":1,"status":"ok","pages":[],"diagnostics":[]}}),"Explain",&["main.tex".into()]).unwrap()
    }
    fn intent() -> UsageIntent {
        UsageIntent {
            user_requested: true,
            allocation: "test-no-billing".into(),
        }
    }
    fn reply(lease: RequestLease) -> Result<RoutedReply, String> {
        let bytes=serde_json::to_vec(&json!({"context_id":lease.payload().context_id,"explanation":"Explanation","edits":[]})).unwrap();
        Ok(RoutedReply::fixture(lease, bytes))
    }
    fn ready(queue: &mut ProviderQueue, id: &str) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if queue.status(id).unwrap() == JobStatus::Ready {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    #[test]
    fn admission_and_consumption_are_bounded_and_source_checked() {
        let docs = source();
        let mut queue = ProviderQueue::new("queue".into(), 1, 1, 1).unwrap();
        assert!(queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                UsageIntent {
                    user_requested: false,
                    allocation: "none".into()
                },
                reply
            )
            .is_err());
        let id = queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                intent(),
                reply,
            )
            .unwrap();
        assert_eq!(queue.allocation(&id), Some("test-no-billing"));
        assert!(queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                intent(),
                reply
            )
            .is_err());
        ready(&mut queue, &id);
        assert!(queue.receive(&id, &docs).is_ok());
        assert!(queue.receive(&id, &docs).is_err());
        let next = queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                intent(),
                reply,
            )
            .unwrap();
        ready(&mut queue, &next);
        let changed = vec![Document::new("p".into(), "main.tex".into(), 2, "new".into()).unwrap()];
        assert!(queue.receive(&next, &changed).is_err());
    }
    #[test]
    fn cancelled_running_calls_keep_slots_and_queued_calls_never_dispatch() {
        let docs = source();
        let mut queue = ProviderQueue::new("cancel_queue".into(), 1, 2, 2).unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (finish_tx, finish_rx) = mpsc::channel();
        let first = queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                intent(),
                move |lease| {
                    started_tx.send(()).unwrap();
                    finish_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                    reply(lease)
                },
            )
            .unwrap();
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let second = queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                intent(),
                move |lease| {
                    observed.fetch_add(1, Ordering::SeqCst);
                    reply(lease)
                },
            )
            .unwrap();
        queue.cancel(&first).unwrap();
        queue.cancel(&second).unwrap();
        assert!(queue.retire(&first).is_err());
        assert_eq!(queue.status(&second).unwrap(), JobStatus::Cancelled);
        queue.retire(&second).unwrap();
        finish_tx.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while queue.retire(&first).is_err() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn terminal_failures_and_stale_revocation_release_only_finished_jobs() {
        let docs = source();
        let mut queue = ProviderQueue::new("failures".into(), 1, 2, 2).unwrap();
        let id = queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                intent(),
                |_| Err("private provider detail".into()),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while queue.status(&id).unwrap() != JobStatus::Failed {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let snapshot = queue.snapshot().unwrap();
        assert_eq!(snapshot.jobs.len(), 1);
        assert!(!snapshot.provider_billing_known);
        assert_eq!(snapshot.scheduler_tasks_started, 1);
        let serialized = serde_json::to_string(&snapshot).unwrap();
        assert!(!serialized.contains("private provider detail"));
        assert!(!serialized.contains("hello"));
        assert!(queue.receive(&id, &docs).is_err());
        queue.retire(&id).unwrap();
        let id = queue
            .submit_with(
                context(&docs),
                &docs,
                Duration::from_secs(2),
                intent(),
                reply,
            )
            .unwrap();
        ready(&mut queue, &id);
        let changed =
            vec![Document::new("p".into(), "main.tex".into(), 2, "changed".into()).unwrap()];
        assert_eq!(queue.revoke_stale("p", &changed), vec![id.clone()]);
        assert_eq!(queue.status(&id).unwrap(), JobStatus::Cancelled);
        assert!(queue.receive(&id, &changed).is_err());
        queue.retire(&id).unwrap();
        assert!(queue.snapshot().unwrap().jobs.is_empty());
    }
}

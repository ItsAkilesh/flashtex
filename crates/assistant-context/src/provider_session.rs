use super::{build_context, Input};
use flashtex_assistant_context::{
    grok::GrokClient,
    provider_queue::{JobStatus, ProviderQueue, UsageIntent},
};
use flashtex_edit_ledger::Document;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    Admit {
        input: Box<Input>,
        timeout_ms: u64,
        user_requested: bool,
        allocation: String,
    },
    Poll {
        request_id: String,
        current_sources: Vec<Document>,
    },
    Cancel {
        request_id: String,
    },
    RevokeStale {
        project_id: String,
        current_sources: Vec<Document>,
    },
    Retire {
        request_id: String,
    },
    Snapshot {},
}
pub(super) struct ProviderHost {
    queue: ProviderQueue,
    client: Arc<GrokClient>,
}
impl ProviderHost {
    pub fn new(session: &str, model: &str, key: String) -> Result<Self, String> {
        Ok(Self {
            queue: ProviderQueue::new(format!("provider_{session}"), 2, 8, 16)?,
            client: Arc::new(GrokClient::new(
                key,
                model.to_owned(),
                Duration::from_secs(90),
            )?),
        })
    }
    pub fn execute(&mut self, value: Value) -> Result<Value, String> {
        let command: Operation =
            serde_json::from_value(value).map_err(|_| "invalid provider command")?;
        match command {
            Operation::Admit {
                input,
                timeout_ms,
                user_requested,
                allocation,
            } => {
                if input.operation != "prepare"
                    || input.response.is_some()
                    || input.current_sources.is_some()
                {
                    return Err("admit requires prepare input".into());
                }
                let context = build_context(&input)?;
                let id = self.queue.submit(
                    context,
                    &input.sources,
                    Duration::from_millis(timeout_ms),
                    UsageIntent {
                        user_requested,
                        allocation,
                    },
                    self.client.clone(),
                )?;
                Ok(json!({"type":"provider_admitted","request_id":id}))
            }
            Operation::Poll {
                request_id,
                current_sources,
            } => {
                let state = self.queue.status(&request_id)?;
                if state == JobStatus::Ready {
                    let proposal = self.queue.receive(&request_id, &current_sources)?;
                    Ok(
                        json!({"type":"validated_proposal","request_id":request_id,"payload":proposal,"applied":false}),
                    )
                } else {
                    Ok(json!({"type":"provider_status","request_id":request_id,"state":state}))
                }
            }
            Operation::Cancel { request_id } => {
                self.queue.cancel(&request_id)?;
                Ok(json!({"type":"provider_cancelled","request_id":request_id}))
            }
            Operation::RevokeStale {
                project_id,
                current_sources,
            } => Ok(
                json!({"type":"provider_revoked","request_ids":self.queue.revoke_stale(&project_id,&current_sources)}),
            ),
            Operation::Retire { request_id } => {
                self.queue.retire(&request_id)?;
                Ok(json!({"type":"provider_retired","request_id":request_id}))
            }
            Operation::Snapshot {} => {
                Ok(json!({"type":"provider_snapshot","payload":self.queue.snapshot()?}))
            }
        }
    }
}

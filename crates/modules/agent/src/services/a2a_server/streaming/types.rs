use axum::response::sse::Event;
use std::sync::Arc;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::{ContextId, TaskId};
use tokio::sync::mpsc::UnboundedSender;

use crate::repository::TaskRepository;
use crate::services::a2a_server::handlers::AgentHandlerState;
use crate::services::a2a_server::processing::message::MessageProcessor;

#[derive(Debug)]
pub struct StreamContext {
    pub tx: UnboundedSender<Event>,
    pub task_id: TaskId,
    pub context_id: ContextId,
    pub message_id: String,
    pub request_id: Option<serde_json::Value>,
    pub log: LogService,
    pub task_repo: TaskRepository,
    pub state: Arc<AgentHandlerState>,
    pub processor: Arc<MessageProcessor>,
}

impl StreamContext {
    pub fn send_event(&self, event: Event) -> bool {
        self.tx.send(event).is_ok()
    }

    pub fn send_json(&self, json: serde_json::Value) -> bool {
        self.tx
            .send(Event::default().data(json.to_string()))
            .is_ok()
    }
}

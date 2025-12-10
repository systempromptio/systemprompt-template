use axum::response::sse::Event;
use serde_json::json;
use systemprompt_identifiers::{ContextId, TaskId};
use tokio::sync::mpsc::UnboundedSender;

pub fn handle_text(
    tx: &UnboundedSender<Event>,
    text: String,
    task_id: &TaskId,
    context_id: &ContextId,
    message_id: &str,
    request_id: &Option<serde_json::Value>,
) {
    let message_event = json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "message",
            "role": "agent",
            "parts": [{
                "kind": "text",
                "text": text
            }],
            "messageId": message_id,
            "taskId": task_id,
            "contextId": context_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        },
        "id": request_id
    });

    let _ = tx.send(Event::default().data(message_event.to_string()));
}

use axum::response::sse::Event;
use serde_json::json;
use systemprompt_core_ai::ToolCall;
use tokio::sync::mpsc::UnboundedSender;

pub fn handle_tool_call(
    tx: &UnboundedSender<Event>,
    tool_call: ToolCall,
    request_id: &Option<serde_json::Value>,
) {
    let tool_event = json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "tool_call",
            "id": tool_call.ai_tool_call_id.as_ref(),
            "name": tool_call.name,
            "arguments": tool_call.arguments
        },
        "id": request_id
    });

    let _ = tx.send(Event::default().data(tool_event.to_string()));
}

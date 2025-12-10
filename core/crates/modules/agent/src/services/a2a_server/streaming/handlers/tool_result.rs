use axum::response::sse::Event;
use rmcp::model::RawContent;
use serde_json::json;
use systemprompt_core_logging::LogService;
use tokio::sync::mpsc::UnboundedSender;

pub async fn handle_tool_result(
    tx: &UnboundedSender<Event>,
    call_id: String,
    result: rmcp::model::CallToolResult,
    request_id: &Option<serde_json::Value>,
    log: &LogService,
) {
    let content_json: Vec<serde_json::Value> = result
        .content
        .iter()
        .map(|c| match &c.raw {
            RawContent::Text(text_content) => {
                json!({"type": "text", "text": text_content.text})
            },
            RawContent::Image(image_content) => {
                json!({"type": "image", "data": image_content.data, "mimeType": image_content.mime_type})
            },
            RawContent::ResourceLink(resource) => {
                json!({"type": "resource", "uri": resource.uri, "mimeType": resource.mime_type})
            },
            _ => json!({"type": "unknown"}),
        })
        .collect();

    let is_error = match result.is_error {
        Some(v) => v,
        None => {
            log.warn(
                "sse_tool_result",
                "Tool result missing is_error flag - treating as error",
            )
            .await
            .ok();
            true
        },
    };

    let result_event = json!({
        "jsonrpc": "2.0",
        "result": {
            "kind": "tool_result",
            "call_id": call_id,
            "content": content_json,
            "is_error": is_error
        },
        "id": request_id
    });

    let _ = tx.send(Event::default().data(result_event.to_string()));
}

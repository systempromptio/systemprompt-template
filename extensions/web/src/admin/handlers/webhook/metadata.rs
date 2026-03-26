use crate::admin::types::{HookEventPayload, StatusLinePayload};

pub(super) fn build_statusline_metadata(payload: &StatusLinePayload) -> serde_json::Value {
    let mut map = serde_json::Map::new();

    if let Some(ref model) = payload.model {
        if let Some(ref id) = model.api_model_id {
            map.insert("model".to_string(), serde_json::json!(id));
        }
    }
    if let Some(ref cost) = payload.cost {
        if let Some(usd) = cost.total_cost_usd {
            map.insert("total_cost_usd".to_string(), serde_json::json!(usd));
        }
    }
    if let Some(ref cw) = payload.context_window {
        if let Some(size) = cw.context_window_size {
            map.insert("context_window_size".to_string(), serde_json::json!(size));
        }
        if let Some(ref usage) = cw.current_usage {
            if let Some(v) = usage.input_tokens {
                map.insert("input_tokens".to_string(), serde_json::json!(v));
            }
            if let Some(v) = usage.output_tokens {
                map.insert("output_tokens".to_string(), serde_json::json!(v));
            }
            if let Some(v) = usage.cache_creation_input_tokens {
                map.insert(
                    "cache_creation_input_tokens".to_string(),
                    serde_json::json!(v),
                );
            }
            if let Some(v) = usage.cache_read_input_tokens {
                map.insert("cache_read_input_tokens".to_string(), serde_json::json!(v));
            }
        }
    }

    if let serde_json::Value::Object(extra) = &payload.extra {
        for (k, v) in extra {
            map.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }

    serde_json::Value::Object(map)
}

pub(super) fn build_metadata(payload: &HookEventPayload) -> serde_json::Value {
    let mut metadata = serde_json::to_value(&payload.extra)
        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
    if let serde_json::Value::Object(ref mut map) = metadata {
        if let Some(ref model) = payload.model {
            map.insert("model".to_string(), serde_json::json!(model));
        }
        if let Some(ref conv_id) = payload.conversation_id {
            map.insert("conversation_id".to_string(), serde_json::json!(conv_id));
        }
        if let Some(ref project_path) = payload.project_path {
            map.insert("project_path".to_string(), serde_json::json!(project_path));
        }
        if let Some(input_size) = payload.tool_input_size {
            map.insert("tool_input_size".to_string(), serde_json::json!(input_size));
        }
        if let Some(output_size) = payload.tool_output_size {
            map.insert(
                "tool_output_size".to_string(),
                serde_json::json!(output_size),
            );
        }
        if let Some(input_tokens) = payload.input_tokens {
            map.insert("input_tokens".to_string(), serde_json::json!(input_tokens));
        }
        if let Some(output_tokens) = payload.output_tokens {
            map.insert(
                "output_tokens".to_string(),
                serde_json::json!(output_tokens),
            );
        }
        if let Some(duration_ms) = payload.duration_ms {
            map.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
        }
        if let Some(success) = payload.success {
            map.insert("success".to_string(), serde_json::json!(success));
        }
        if let Some(ref tool_use_id) = payload.tool_use_id {
            map.insert("tool_use_id".to_string(), serde_json::json!(tool_use_id));
        }
        if let Some(ref tool_input) = payload.tool_input {
            map.insert("tool_input".to_string(), tool_input.clone());
        }
        if let Some(ref tool_response) = payload.tool_response {
            map.insert("tool_response".to_string(), tool_response.clone());
        }
        if let Some(ref error) = payload.error {
            map.insert("error".to_string(), serde_json::json!(error));
        }
        if let Some(ref transcript_path) = payload.transcript_path {
            map.insert(
                "transcript_path".to_string(),
                serde_json::json!(transcript_path),
            );
        }
        if let Some(ref permission_mode) = payload.permission_mode {
            map.insert(
                "permission_mode".to_string(),
                serde_json::json!(permission_mode),
            );
        }
        if let Some(ref last_assistant_message) = payload.last_assistant_message {
            map.insert(
                "last_assistant_message".to_string(),
                serde_json::json!(last_assistant_message),
            );
        }
        if let Some(ref prompt) = payload.prompt {
            map.insert("prompt".to_string(), serde_json::json!(prompt));
        }
        if let Some(ref source) = payload.source {
            map.insert("source".to_string(), serde_json::json!(source));
        }
        if let Some(ref reason) = payload.reason {
            map.insert("reason".to_string(), serde_json::json!(reason));
        }
        if let Some(ref agent_type) = payload.agent_type {
            map.insert("agent_type".to_string(), serde_json::json!(agent_type));
        }
        if let Some(ref agent_id) = payload.agent_id {
            map.insert("agent_id".to_string(), serde_json::json!(agent_id));
        }
    }
    metadata
}

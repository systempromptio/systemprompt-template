use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HookEventPayload {
    pub hook_event_name: Option<String>,
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub conversation_id: Option<String>,
    pub project_path: Option<String>,
    pub tool_input_size: Option<i64>,
    pub tool_output_size: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub duration_ms: Option<i64>,
    pub success: Option<bool>,
    pub tool_use_id: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<serde_json::Value>,
    pub error: Option<String>,
    pub transcript_path: Option<String>,
    pub permission_mode: Option<String>,
    pub last_assistant_message: Option<String>,
    pub prompt: Option<String>,
    pub source: Option<String>,
    pub reason: Option<String>,
    pub agent_type: Option<String>,
    pub agent_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct TrackQuery {
    pub plugin_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GovernQuery {
    pub plugin_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLinePayload {
    pub model: Option<StatusLineModel>,
    pub cost: Option<StatusLineCost>,
    pub context_window: Option<ContextWindow>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct StatusLineModel {
    pub api_model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLineCost {
    pub total_cost_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct ContextWindow {
    pub context_window_size: Option<i64>,
    pub current_usage: Option<ContextWindowUsage>,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct ContextWindowUsage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct StatusLineQuery {
    pub plugin_id: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptPayload {
    pub session_id: Option<String>,
    pub transcript: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptQuery {
    pub plugin_id: Option<String>,
}

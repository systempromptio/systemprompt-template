use serde::Serialize;

use super::common::{EventBreakdownView, NamedEntity};

#[derive(Debug, Clone, Serialize)]
pub struct HookCodeEntry {
    pub matcher: String,
    pub hooks: Vec<HookCodeHook>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HookCodeHook {
    #[serde(rename = "type")]
    pub hook_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(rename = "async", skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HookView {
    pub id: String,
    pub hook_name: String,
    pub description: String,
    pub event_type: String,
    pub hook_type: String,
    pub matcher: String,
    pub url: String,
    pub command: String,
    pub headers: serde_json::Value,
    pub timeout: i32,
    pub is_async: bool,
    pub enabled: bool,
    pub is_default: bool,
    pub plugin_id: Option<String>,
    pub plugin_name: String,
    pub hook_code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HooksStats {
    pub total_count: usize,
    pub enabled_count: usize,
    pub total_events: i64,
    pub total_errors: i64,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub avg_session_quality: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct MyHooksPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub hooks: Vec<HookView>,
    pub has_hooks: bool,
    pub plugins: Vec<NamedEntity>,
    pub stats: HooksStats,
    pub event_breakdown: Vec<EventBreakdownView>,
    pub chart: serde_json::Value,
    pub range: String,
    pub range_24h: bool,
    pub range_7d: bool,
    pub range_14d: bool,
    pub hook_event_types: Vec<&'static str>,
}

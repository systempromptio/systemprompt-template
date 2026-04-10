use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserHook {
    pub id: String,
    pub user_id: String,
    pub plugin_id: Option<String>,
    pub hook_name: String,
    pub description: String,
    pub event_type: String,
    pub matcher: String,
    pub hook_type: String,
    pub url: String,
    pub command: String,
    // JSON: dynamic HTTP headers from user-defined hook config
    pub headers: serde_json::Value,
    pub timeout: i32,
    pub is_async: bool,
    pub enabled: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserHookRequest {
    pub plugin_id: Option<String>,
    pub hook_name: String,
    #[serde(default)]
    pub description: String,
    pub event_type: String,
    #[serde(default = "default_matcher")]
    pub matcher: String,
    #[serde(default = "default_hook_type")]
    pub hook_type: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub command: String,
    #[serde(default = "default_headers")]
    // JSON: dynamic HTTP headers from user-defined hook config
    pub headers: serde_json::Value,
    #[serde(default = "default_timeout")]
    pub timeout: i32,
    #[serde(default)]
    pub is_async: bool,
}

fn default_matcher() -> String {
    "*".to_string()
}
fn default_hook_type() -> String {
    "http".to_string()
}
fn default_headers() -> serde_json::Value {
    serde_json::json!({})
}
const fn default_timeout() -> i32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserHookRequest {
    pub hook_name: Option<String>,
    pub description: Option<String>,
    pub event_type: Option<String>,
    pub matcher: Option<String>,
    pub url: Option<String>,
    pub command: Option<String>,
    // JSON: dynamic HTTP headers from user-defined hook config
    pub headers: Option<serde_json::Value>,
    pub timeout: Option<i32>,
    pub is_async: Option<bool>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HookEventTypeStat {
    pub event_type: String,
    pub event_count: i64,
    pub error_count: i64,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow, Clone, Copy)]
pub struct HookTimeSeriesBucket {
    pub bucket: DateTime<Utc>,
    pub event_count: i64,
    pub error_count: i64,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct HookSummaryStats {
    pub total_events: i64,
    pub total_errors: i64,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
}

#[derive(Debug, Deserialize)]
pub struct HooksQuery {
    #[serde(default = "default_range")]
    pub range: String,
}

fn default_range() -> String {
    "7d".to_string()
}

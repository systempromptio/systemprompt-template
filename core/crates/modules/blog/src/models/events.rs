use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;
use systemprompt_models::SessionId;

/// Blog view event - tracks individual page views
///
/// Note: Session-level attributes (`user_agent`, country, etc.) are NOT stored
/// here       to avoid duplication. Access them via JOIN with `user_sessions`
/// table using       the `session_id` foreign key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewEvent {
    pub id: String,
    pub content_id: String,
    pub user_id: Option<String>,
    pub session_id: SessionId,
    pub referrer_source: Option<String>,
    pub referrer_url: Option<String>,
    pub time_on_page_seconds: i32,
    pub scroll_depth_percent: i32,
    pub viewed_at: DateTime<Utc>,
}

impl ViewEvent {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use systemprompt_core_database::parse_database_datetime;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let content_id = row
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing content_id"))?
            .to_string();

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let session_id = SessionId::new(
            row.get("session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing session_id"))?
                .to_string(),
        );

        let referrer_source = row
            .get("referrer_source")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let referrer_url = row
            .get("referrer_url")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let time_on_page_seconds = row
            .get("time_on_page_seconds")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid time_on_page_seconds"))?
            as i32;

        let scroll_depth_percent = row
            .get("scroll_depth_percent")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing or invalid scroll_depth_percent"))?
            as i32;

        let viewed_at = row
            .get("viewed_at")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid viewed_at"))?;

        Ok(Self {
            id,
            content_id,
            user_id,
            session_id,
            referrer_source,
            referrer_url,
            time_on_page_seconds,
            scroll_depth_percent,
            viewed_at,
        })
    }
}

/// Request payload for tracking a view
/// Note: `session_id` comes from JWT via `RequestContext`, not from client
/// payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewEventRequest {
    pub content_id: String,
    pub user_id: Option<String>,
    pub referrer_source: Option<String>,
    pub referrer_url: Option<String>,
}

/// Request payload for tracking engagement metrics
/// Note: `session_id` comes from JWT via `RequestContext`, not from client
/// payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementEventRequest {
    pub content_id: String,
    pub time_on_page_seconds: Option<i32>,
    pub scroll_depth_percent: Option<i32>,
}

/// Request payload for tracking social shares
/// Note: `session_id` comes from JWT via `RequestContext`, not from client
/// payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareEventRequest {
    pub content_id: String,
    pub platform: String, // 'linkedin', 'twitter', 'facebook'
    pub user_id: Option<String>,
}

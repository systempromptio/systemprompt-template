use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{ContentId, EngagementEventId, SessionId, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EngagementEvent {
    pub id: EngagementEventId,
    pub session_id: SessionId,
    pub user_id: UserId,
    pub page_url: String,
    pub content_id: Option<ContentId>,

    pub time_on_page_ms: i32,
    pub time_to_first_interaction_ms: Option<i32>,
    pub time_to_first_scroll_ms: Option<i32>,
    pub max_scroll_depth: i32,
    pub scroll_velocity_avg: Option<f32>,
    pub scroll_direction_changes: Option<i32>,

    pub click_count: i32,
    pub mouse_move_distance_px: Option<i32>,
    pub keyboard_events: Option<i32>,
    pub copy_events: Option<i32>,

    pub focus_time_ms: i32,
    pub blur_count: i32,
    pub tab_switches: i32,
    pub visible_time_ms: i32,
    pub hidden_time_ms: i32,

    pub is_rage_click: Option<bool>,
    pub is_dead_click: Option<bool>,
    pub reading_pattern: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct EngagementEventRequest {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    #[serde(default)]
    pub page_url: String,
    pub content_id: Option<String>,
    pub data: Option<EngagementData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EngagementData {
    // Event metadata
    pub event_type: Option<EventType>,
    pub referrer: Option<String>,
    pub title: Option<String>,

    // Time metrics
    pub time_on_page_ms: Option<i32>,
    pub time_to_first_interaction_ms: Option<i32>,
    pub time_to_first_scroll_ms: Option<i32>,
    pub focus_time_ms: Option<i32>,
    pub visible_time_ms: Option<i32>,
    pub hidden_time_ms: Option<i32>,

    // Scroll metrics
    pub max_scroll_depth: Option<i32>,
    pub scroll_velocity_avg: Option<i32>,
    pub scroll_direction_changes: Option<i32>,
    pub depth: Option<i32>,
    pub milestone: Option<i32>,
    pub direction: Option<String>,
    pub velocity: Option<i32>,

    // Interaction metrics
    pub click_count: Option<i32>,
    pub mouse_move_distance_px: Option<i32>,
    pub keyboard_events: Option<i32>,
    pub copy_events: Option<i32>,
    pub blur_count: Option<i32>,
    pub tab_switches: Option<i32>,

    // Behavior flags
    pub is_rage_click: Option<bool>,
    pub is_dead_click: Option<bool>,
    pub reading_pattern: Option<String>,

    // Link click data
    pub target_url: Option<String>,
    pub link_text: Option<String>,
    pub is_external: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchEngagementRequest {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub events: Vec<EngagementEventRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngagementEventResponse {
    pub id: String,
    pub page_url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    PageView,
    PageExit,
    Scroll,
    LinkClick,
}

impl EventType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PageView => "page_view",
            Self::PageExit => "page_exit",
            Self::Scroll => "scroll",
            Self::LinkClick => "link_click",
        }
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

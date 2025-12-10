use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ==================== SESSION ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalyticsSession {
    pub session_id: String,
    pub user_id: Option<String>,
    pub fingerprint_hash: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub referrer_url: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub is_bot: bool,
    pub is_scanner: Option<bool>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub request_count: Option<i32>,
    pub task_count: Option<i32>,
    pub ai_request_count: Option<i32>,
    pub message_count: Option<i32>,
}

// ==================== EVENTS ====================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalyticsEvent {
    pub id: String,
    pub event_type: String,
    pub event_category: String,
    pub severity: String,
    pub user_id: String,
    pub session_id: Option<String>,
    pub message: Option<String>,
    pub metadata: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ErrorSummary {
    pub error_type: String,
    pub count: i64,
    pub last_occurred: DateTime<Utc>,
    pub sample_message: Option<String>,
}

// ==================== CORE STATS ====================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRow)]
pub struct PlatformOverview {
    pub total_users: i64,
    pub active_users_24h: i64,
    pub active_users_7d: i64,
    pub total_sessions: i64,
    pub active_sessions: i64,
    pub total_contexts: i64,
    pub total_tasks: i64,
    pub total_ai_requests: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRow)]
pub struct CostOverview {
    pub total_cost: f64,
    pub cost_24h: f64,
    pub cost_7d: f64,
    pub cost_30d: f64,
    pub avg_cost_per_request: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRow)]
pub struct ActivityTrend {
    pub date: DateTime<Utc>,
    pub sessions: i64,
    pub contexts: i64,
    pub tasks: i64,
    pub ai_requests: i64,
    pub tool_executions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TopUser {
    pub user_id: String,
    pub user_name: String,
    pub session_count: i64,
    pub task_count: i64,
    pub ai_request_count: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TopAgent {
    pub agent_name: String,
    pub task_count: i64,
    pub success_rate: f64,
    pub avg_duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TopTool {
    pub tool_name: String,
    pub execution_count: i64,
    pub success_rate: f64,
    pub avg_duration_ms: i64,
}

// ==================== TRAFFIC ====================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRow)]
pub struct TrafficSummary {
    pub total_sessions: i64,
    pub unique_visitors: i64,
    pub page_views: i64,
    pub avg_session_duration_seconds: f64,
    pub bounce_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TrafficSource {
    pub source: String,
    pub sessions: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceBreakdown {
    pub device_type: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GeographicBreakdown {
    pub country: String,
    pub count: i64,
    pub percentage: f64,
}

// ==================== CONVERSATIONS ====================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRow)]
pub struct ConversationSummary {
    pub total_conversations: i64,
    pub active_conversations: i64,
    pub completed_conversations: i64,
    pub avg_messages_per_conversation: f64,
    pub avg_duration_minutes: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromRow)]
pub struct ConversationTrend {
    pub date: DateTime<Utc>,
    pub new_conversations: i64,
    pub completed_conversations: i64,
    pub total_messages: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationByAgent {
    pub agent_name: String,
    pub conversation_count: i64,
    pub avg_messages: f64,
    pub success_rate: f64,
}

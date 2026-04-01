use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{Email, SessionId, UserId};

use super::super::activity;

pub use super::dashboard_enterprise::{
    DepartmentActivity, DepartmentQuery, DepartmentScore, EventTypeBreakdown,
    GovernanceDecisionRow, GovernanceEvent, McpAccessEvent, McpAccessSummary, ModelUsage,
    ProjectActivity, TokenUsageRow,
};
pub use super::dashboard_traffic::{
    ContentPerformanceRow, RealtimePulse, RecentMcpError, TopPageDailyBucket, TrafficCountryBucket,
    TrafficData, TrafficDevice, TrafficGeo, TrafficKpis, TrafficReadingPattern, TrafficSource,
    TrafficTimeBucket, TrafficTopPage,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub timeline: Vec<activity::ActivityTimelineEvent>,
    pub top_users: Vec<TopUser>,
    pub popular_skills: Vec<SkillCount>,
    pub hourly_activity: Vec<HourlyActivity>,
    pub stats: ActivityStats,
    pub usage_timeseries: Vec<TimeSeriesBucket>,
    pub active_users_24h: i64,
    pub tool_success_rates: Vec<ToolSuccessRate>,
    pub traffic: Option<TrafficData>,
    pub recent_mcp_errors: Vec<RecentMcpError>,
    pub top_pages_today: Vec<TrafficTopPage>,
    pub realtime_pulse: Option<RealtimePulse>,
    pub content_performance: Vec<ContentPerformanceRow>,
}


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TimeSeriesBucket {
    pub bucket: DateTime<Utc>,
    pub tool_uses: i64,
    pub prompts: i64,
    pub active_users: i64,
    pub sessions: i64,
    pub errors: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TopUser {
    pub user_id: UserId,
    pub display_name: String,
    pub email: Option<Email>,
    pub logins: i64,
    pub edits: i64,
    pub mcp_calls: i64,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SkillCount {
    pub tool_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HourlyActivity {
    pub hour: i32,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityStats {
    pub events_today: i64,
    pub events_this_week: i64,
    pub total_edits: i64,
    pub mcp_tool_calls: i64,
    pub mcp_errors: i64,
    pub total_logins: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ToolSuccessRate {
    pub tool_name: String,
    pub total: i64,
    pub successes: i64,
    pub failures: i64,
    pub success_pct: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LeaderboardEntry {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub rank_level: i32,
    pub rank_name: String,
    pub total_xp: i64,
    pub events_count: i64,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub achievement_count: i64,
    pub last_active_date: Option<NaiveDate>,
    pub total_sessions: i64,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_subagents: i64,
    pub unique_skills_count: i32,
    pub total_days_active: i32,
    pub period_xp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGamificationProfile {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub rank_level: i32,
    pub rank_name: String,
    pub total_xp: i64,
    pub xp_to_next_rank: i64,
    pub next_rank_name: Option<String>,
    pub events_count: i64,
    pub unique_skills_count: i32,
    pub unique_plugins_count: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub achievements: Vec<UnlockedAchievement>,
    pub rank_position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UnlockedAchievement {
    pub achievement_id: String,
    pub unlocked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AchievementInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub total_unlocked: i64,
    pub unlock_percentage: f64,
}

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    #[serde(default = "default_range")]
    pub range: String,
    #[serde(default = "default_traffic_range")]
    pub traffic_range: String,
    #[serde(default = "default_content_range")]
    pub content_range: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub tab: String,
}

fn default_content_range() -> String {
    "7d".to_string()
}

fn default_range() -> String {
    "7d".to_string()
}

fn default_traffic_range() -> String {
    "today".to_string()
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub search: Option<String>,
    pub event_type: Option<String>,
    #[serde(default = "default_events_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EventRow {
    pub id: String,
    pub user_id: UserId,
    pub display_name: String,
    pub email: Option<Email>,
    pub session_id: SessionId,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EventBreakdown {
    pub event_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<EventRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

fn default_events_limit() -> i64 {
    100
}


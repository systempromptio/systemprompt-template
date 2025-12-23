use chrono::{DateTime, Duration, Utc};

pub fn parse_flexible_timestamp(timestamp_str: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.f") {
        return Some(dt.and_utc());
    }
    DateTime::parse_from_rfc3339(timestamp_str)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn format_timestamp(timestamp_str: &str) -> String {
    match timestamp_str.parse::<DateTime<Utc>>() {
        Ok(dt) => {
            let now = Utc::now();
            let diff = now.signed_duration_since(dt);
            if diff < Duration::zero() {
                dt.format("%b %d, %Y %H:%M UTC").to_string()
            } else if diff.num_seconds() < 60 {
                format!("{} seconds ago", diff.num_seconds())
            } else if diff.num_minutes() < 60 {
                let mins = diff.num_minutes();
                format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
            } else if diff.num_hours() < 24 {
                let hours = diff.num_hours();
                format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
            } else if diff.num_days() < 7 {
                let days = diff.num_days();
                format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
            } else {
                dt.format("%b %d, %Y %H:%M UTC").to_string()
            }
        }
        Err(_) => timestamp_str.to_string(),
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ConversationMetrics {
    pub conversations_24h: i64,
    pub conversations_7d: i64,
    pub conversations_30d: i64,
    pub conversations_prev_24h: i64,
    pub conversations_prev_7d: i64,
    pub conversations_prev_30d: i64,
}

#[derive(Debug)]
pub struct RecentConversation {
    pub context_id: String,
    pub agent_name: String,
    pub started_at: String,
    pub task_started_at: DateTime<Utc>,
    pub task_completed_at: DateTime<Utc>,
    pub status: String,
    pub message_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct TrafficSummary {
    pub total_sessions: i32,
    pub total_requests: i32,
    pub unique_users: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DailyTrend {
    pub date: String,
    pub conversations: i64,
    pub tool_executions: i64,
    pub active_users: i64,
}

#[derive(Debug)]
pub struct ToolUsageData {
    pub agent_data: Vec<AgentUsageRow>,
    pub tool_data: Vec<ToolUsageRow>,
}

#[derive(Debug)]
pub struct AgentUsageRow {
    pub agent_name: String,
    pub hours_24: i64,
    pub days_7: i64,
    pub days_30: i64,
}

#[derive(Debug)]
pub struct ToolUsageRow {
    pub tool_name: String,
    pub hours_24: i64,
    pub days_7: i64,
    pub days_30: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AgentToolUsage {
    pub agent_name: String,
    pub count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct ToolNameUsage {
    pub tool_name: String,
    pub count: i64,
}

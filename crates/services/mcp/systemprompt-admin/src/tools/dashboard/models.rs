use chrono::{DateTime, Utc};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TrafficSummary {
    pub total_sessions: i32,
    pub total_requests: i32,
    pub unique_users: i32,
}

#[derive(Debug)]
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
    pub h24: i64,
    pub d7: i64,
    pub d30: i64,
}

#[derive(Debug)]
pub struct ToolUsageRow {
    pub tool_name: String,
    pub h24: i64,
    pub d7: i64,
    pub d30: i64,
}

#[derive(Debug)]
pub struct AgentToolUsage {
    pub agent_name: String,
    pub count: i64,
}

#[derive(Debug)]
pub struct ToolNameUsage {
    pub tool_name: String,
    pub count: i64,
}

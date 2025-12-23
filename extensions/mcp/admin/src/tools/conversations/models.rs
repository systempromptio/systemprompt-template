#[derive(serde::Serialize, sqlx::FromRow)]
pub struct ConversationSummary {
    pub total_conversations: i32,
    pub total_messages: i32,
    pub avg_messages_per_conversation: f64,
    pub avg_execution_time_ms: f64,
    pub failed_conversations: i32,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct RecentConversation {
    pub context_id: String,
    pub conversation_name: Option<String>,
    pub user_id: String,
    pub user_name: String,
    pub agent_name: String,
    pub started_at: String,
    pub started_at_formatted: Option<String>,
    pub last_updated: String,
    pub last_updated_formatted: Option<String>,
    pub duration_seconds: f64,
    pub duration_status: Option<String>,
    pub status: String,
    pub message_count: i32,
}

#[derive(sqlx::FromRow)]
pub struct ConversationTrendRow {
    pub conversations_1h: i64,
    pub conversations_24h: i64,
    pub conversations_7d: i64,
    pub conversations_30d: i64,
}

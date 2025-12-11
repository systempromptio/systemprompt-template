#[derive(serde::Serialize)]
pub struct ConversationSummary {
    pub total_conversations: i32,
    pub total_messages: i32,
    pub avg_messages_per_conversation: f64,
    pub avg_execution_time_ms: f64,
    pub failed_conversations: i32,
}

#[derive(serde::Serialize)]
pub struct EvaluationStats {
    pub evaluated_conversations: i32,
    pub avg_quality_score: f64,
    pub goal_achievement_rate: f64,
    pub avg_user_satisfaction: f64,
}

#[derive(serde::Serialize)]
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
    pub quality_score: Option<i32>,
    pub goal_achieved: Option<String>,
    pub user_satisfaction: Option<i32>,
    pub primary_category: Option<String>,
    pub topics: Option<String>,
    pub evaluation_summary: Option<String>,
}

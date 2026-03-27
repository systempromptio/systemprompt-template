use serde::Serialize;

#[derive(Serialize, Clone, Default)]
pub struct ToolError {
    pub tool_name: String,
    pub description: String,
}

#[derive(Serialize, Clone, Default)]
pub struct ToolGroup {
    pub name: String,
    pub count: usize,
}

#[derive(Serialize, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Turn {
    pub prompt_text: String,
    pub prompt_time: String,
    pub response_text: String,
    pub response_time: String,
    pub tool_groups: Vec<ToolGroup>,
    pub total_tools: usize,
    pub has_tools: bool,
    pub errors: Vec<ToolError>,
    pub has_errors: bool,
    pub has_prompt: bool,
    pub has_response: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct EntityEntry {
    pub entity_type: String,
    pub entity_name: String,
    pub usage_count: i32,
}

#[derive(Serialize, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct SessionGroupFlags {
    pub has_session_title: bool,
    pub is_active: bool,
    pub has_errors: bool,
    pub has_duration: bool,
    pub has_turns: bool,
    pub has_first_prompt: bool,
    pub has_last_response: bool,
    pub is_analysed: bool,
    pub has_quality_score: bool,
    pub has_recommendations: bool,
    pub has_client_source: bool,
    pub has_permission_mode: bool,
    pub is_plan_mode: bool,
    pub has_model: bool,
    pub has_automated: bool,
    pub has_entities_overflow: bool,
    pub has_entities: bool,
    pub has_rating: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct SessionGroup {
    pub session_id: String,
    pub session_id_short: String,
    pub project_name: String,
    pub session_title: String,
    pub started_at: String,
    pub last_activity_at: String,
    pub status: String,
    pub total_prompts: usize,
    pub total_tools: usize,
    pub total_errors: usize,
    pub turn_count: usize,
    pub duration_display: String,
    pub entity_count: usize,
    pub turns: Vec<Turn>,
    pub first_prompt: String,
    pub last_response: String,
    pub all_errors: Vec<ToolError>,
    pub ai_summary: Option<String>,
    pub ai_tags: Option<String>,
    pub ai_title: Option<String>,
    pub quality_score: i16,
    pub goal_achieved: String,
    pub quality_class: String,
    pub goal_icon: String,
    pub description: Option<String>,
    pub recommendations: Option<String>,
    pub content_bytes: i64,
    pub client_source: String,
    pub client_source_label: String,
    pub client_source_class: String,
    pub permission_mode: String,
    pub model: String,
    pub model_short: String,
    pub user_prompts: i32,
    pub automated_actions: i32,
    pub entities: Vec<EntityEntry>,
    pub entities_preview: Vec<EntityEntry>,
    pub entities_overflow_count: usize,
    pub rating: i16,
    pub outcome: String,
    #[serde(flatten)]
    pub flags: SessionGroupFlags,
}

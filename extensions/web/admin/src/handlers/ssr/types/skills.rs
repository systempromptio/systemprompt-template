use serde::Serialize;
use systemprompt::identifiers::SkillId;

use super::common::NamedEntity;

#[derive(Debug, Clone, Serialize)]
pub struct SkillViewExtra {
    pub usage_count: i64,
    pub content_preview: String,
    pub is_forked: bool,
    pub plugin_names: Vec<NamedEntity>,
    pub total_uses: i64,
    pub sessions_used_in: i64,
    pub avg_effectiveness: String,
    pub scored_sessions: i64,
    pub goal_achievement_pct: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_rating: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_rating_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Copy)]
pub struct SkillStats {
    pub skill_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillsPageData {
    pub page: &'static str,
    pub title: &'static str,
    // JSON: template context for Handlebars rendering
    pub skills: Vec<serde_json::Value>,
    pub all_tags: Vec<String>,
    pub stats: SkillStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillEditView {
    pub skill_id: SkillId,
    pub name: String,
    pub description: String,
    pub content: String,
    pub tags: Vec<String>,
    pub tags_csv: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequiredSecretView {
    pub name: String,
    pub description: String,
    pub optional: bool,
    pub is_configured: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MySkillEditPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub is_edit: bool,
    pub is_forked: bool,
    // JSON: template context for Handlebars rendering
    pub skill: serde_json::Value,
    pub required_secrets: Vec<RequiredSecretView>,
}

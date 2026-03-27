use serde::Serialize;

use super::common::NamedEntity;

#[derive(Debug, Clone, Serialize)]
pub struct SkillWithStats {
    pub id: String,
    pub name: String,
    pub uses: i64,
    pub avg_effectiveness: String,
    pub goal_pct: String,
    pub scored_sessions: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginView {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub base_plugin_id: Option<String>,
    pub author_name: String,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub total_uses: i64,
    pub session_count: i64,
    pub avg_quality_score: String,
    pub goal_achievement_pct: String,
    pub scored_sessions: i64,
    pub skills: Vec<SkillWithStats>,
    pub agents: Vec<NamedEntity>,
    pub mcp_servers: Vec<NamedEntity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginStats {
    pub plugin_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MyPluginsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub plugins: Vec<serde_json::Value>,
    pub has_plugins: bool,
    pub categories: Vec<String>,
    pub stats: PluginStats,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginEditData {
    pub id: Option<String>,
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub category: String,
    pub keywords: Vec<String>,
    pub author_name: String,
    pub base_plugin_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MyPluginEditPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub is_edit: bool,
    pub plugin: PluginEditData,
    pub keywords_csv: String,
    pub skills_list: Vec<super::common::CheckableEntity>,
    pub agents_list: Vec<super::common::CheckableEntity>,
    pub mcp_list: Vec<super::common::CheckableEntity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginDetailView {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub base_plugin_id: Option<String>,
    pub author_name: String,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub hook_count: usize,
    pub skills: Vec<NamedEntity>,
    pub agents: Vec<NamedEntity>,
    pub mcp_servers: Vec<NamedEntity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MyPluginViewPageData {
    pub page: &'static str,
    pub title: String,
    pub plugin: PluginDetailView,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowsePluginView {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub already_added: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowsePluginStats {
    pub total_available: usize,
    pub already_added: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowsePluginsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub plugins: Vec<BrowsePluginView>,
    pub categories: Vec<String>,
    pub stats: BrowsePluginStats,
}

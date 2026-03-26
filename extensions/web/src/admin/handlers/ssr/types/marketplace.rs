use serde::Serialize;

use super::common::NamedEntity;

#[derive(Debug, Clone, Serialize)]
pub struct MarketplacePluginView {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub base_plugin_id: Option<String>,
    pub enabled: bool,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub hook_count: usize,
    pub skills: Vec<NamedEntity>,
    pub agents: Vec<NamedEntity>,
    pub mcp_servers: Vec<NamedEntity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformMarketplacePlugin {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub version: String,
    pub skill_count: usize,
    pub agent_count: usize,
    pub mcp_count: usize,
    pub hook_count: usize,
    pub skills: Vec<NamedEntity>,
    pub agents: Vec<NamedEntity>,
    pub mcp_servers: Vec<NamedEntity>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceStats {
    pub plugin_count: usize,
    pub total_skills: usize,
    pub total_agents: usize,
    pub total_mcp: usize,
    pub total_hooks: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MyMarketplacePageData {
    pub page: &'static str,
    pub title: &'static str,
    pub platform_plugin: serde_json::Value,
    pub plugins: Vec<MarketplacePluginView>,
    pub has_plugins: bool,
    pub stats: MarketplaceStats,
}

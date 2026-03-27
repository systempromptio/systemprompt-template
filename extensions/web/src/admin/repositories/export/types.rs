use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SyncPluginsResponse {
    pub plugins: Vec<PluginBundle>,
    pub marketplace: MarketplaceFile,
    pub totals: ExportTotals,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<ManifestAuthor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestAuthor {
    pub name: String,
    #[serde(default)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpConfigFile {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpServerEntry {
    #[serde(rename = "type")]
    pub server_type: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceManifest {
    pub name: String,
    pub owner: ManifestAuthor,
    pub plugins: Vec<MarketplacePluginEntry>,
}

#[derive(Debug, Serialize)]
pub struct MarketplacePluginEntry {
    pub name: String,
    pub source: String,
    pub description: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<ManifestAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportTotals {
    pub plugins: usize,
    pub files: usize,
    pub skills: usize,
    pub agents: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginBundleCounts {
    pub skills: usize,
    pub agents: usize,
    pub mcp_servers: usize,
    pub scripts: usize,
    pub total_files: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginBundle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub files: Vec<PluginFile>,
    pub counts: PluginBundleCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginFile {
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub executable: bool,
}

#[derive(Debug, Serialize)]
pub struct MarketplaceFile {
    pub path: String,
    pub content: String,
}

use serde::Serialize;

use super::plugin_walker::PluginEntry;

#[derive(Debug, Clone, Serialize)]
pub struct UserSection {
    pub id: String,
    pub name: String,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub file_path: String,
    pub tags: Vec<String>,
    pub sha256: String,
    pub instructions: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentEntry {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub endpoint: String,
    pub enabled: bool,
    pub is_default: bool,
    pub is_primary: bool,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub mcp_servers: Vec<String>,
    pub skills: Vec<String>,
    pub tags: Vec<String>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManagedMcpServer {
    pub name: String,
    pub url: String,
    pub transport: Option<String>,
    pub oauth: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub manifest_version: String,
    pub issued_at: String,
    pub not_before: String,
    pub user_id: String,
    pub tenant_id: Option<String>,
    pub user: UserSection,
    pub plugins: Vec<PluginEntry>,
    pub skills: Vec<SkillEntry>,
    pub agents: Vec<AgentEntry>,
    pub managed_mcp_servers: Vec<ManagedMcpServer>,
    pub revocations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WhoamiResponse {
    pub user: UserSection,
    pub capabilities: Vec<&'static str>,
}

pub const COWORK_CAPABILITIES: &[&str] = &["plugins", "skills", "agents", "mcp", "user"];

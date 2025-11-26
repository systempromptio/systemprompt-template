use serde::{Deserialize, Serialize};

/// MCP server connection information for skill loading
#[derive(Debug, Clone)]
pub struct McpServerConnectionInfo {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub host: String,
    pub port: u16,
}

/// Server status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub name: String,
    pub running: bool,
    pub healthy: bool,
    pub tool_count: usize,
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Skill loading result
#[derive(Debug, Clone)]
pub struct SkillLoadingResult {
    pub server_name: String,
    pub success: bool,
    pub skill_count: usize,
    pub error_message: Option<String>,
    pub load_time_ms: u64,
}

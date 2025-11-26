use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_agent_port_range")]
    pub agent_port_range: (u16, u16),
    #[serde(default = "default_mcp_port_range")]
    pub mcp_port_range: (u16, u16),
    #[serde(default = "default_true")]
    pub auto_start_enabled: bool,
    #[serde(default = "default_true")]
    pub validation_strict: bool,
    #[serde(default = "default_schema_validation_mode")]
    pub schema_validation_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub services_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            agent_port_range: default_agent_port_range(),
            mcp_port_range: default_mcp_port_range(),
            auto_start_enabled: true,
            validation_strict: true,
            schema_validation_mode: default_schema_validation_mode(),
            services_path: std::env::var("SYSTEMPROMPT_SERVICES_PATH").ok(),
            skills_path: std::env::var("SYSTEMPROMPT_SKILLS_PATH").ok(),
            config_path: std::env::var("SYSTEMPROMPT_CONFIG_PATH").ok(),
        }
    }
}

const fn default_agent_port_range() -> (u16, u16) {
    (9000, 9999)
}

const fn default_mcp_port_range() -> (u16, u16) {
    (5000, 5999)
}

const fn default_true() -> bool {
    true
}

fn default_schema_validation_mode() -> String {
    "auto_migrate".to_string()
}

use crate::auth::{JwtAudience, Permission};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub deployments: HashMap<String, Deployment>,
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub binary: Option<String>,
    pub package: Option<String>,
    pub path: Option<String>,
    pub port: u16,
    pub endpoint: String,
    pub enabled: bool,
    pub display_in_web: bool,
    #[serde(default)]
    pub schemas: Vec<SchemaDefinition>,
    pub oauth: OAuthRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub file: String,
    pub table: String,
    pub required_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthRequirement {
    pub required: bool,
    pub scopes: Vec<Permission>,
    pub audience: JwtAudience,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub auto_build: bool,
    pub build_timeout: u64,
    pub health_check_timeout: u64,
    #[serde(default = "default_base_port")]
    pub base_port: u16,
    #[serde(default = "default_working_dir")]
    pub working_dir: String,
}

const fn default_base_port() -> u16 {
    5000
}

fn default_working_dir() -> String {
    "/app".to_string()
}

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::auth::{AuthenticatedUser, Permission};

pub const RUNNING: &str = "running";
pub const ERROR: &str = "error";
pub const STOPPED: &str = "stopped";
pub const STARTING: &str = "starting";

#[derive(Debug, Clone)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub enabled: bool,
    pub display_in_web: bool,
    pub port: u16,
    #[serde(
        serialize_with = "serialize_path",
        deserialize_with = "deserialize_path"
    )]
    pub crate_path: PathBuf,
    pub display_name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub schemas: Vec<super::deployment::SchemaDefinition>,
    pub oauth: super::deployment::OAuthRequirement,
    pub version: String,
    pub host: String,
    pub module_name: String,
    pub protocol: String,
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    path.to_string_lossy().serialize(serializer)
}

fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(PathBuf::from(s))
}

impl McpServerConfig {
    pub fn endpoint(&self) -> String {
        let api_base =
            std::env::var("API_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        format!("{}/api/v1/mcp/{}/mcp", api_base, self.name)
    }

    pub fn from_manifest_and_deployment(
        name: String,
        manifest: &super::registry::ServerManifest,
        deployment: &super::deployment::Deployment,
        crate_path: PathBuf,
    ) -> Self {
        Self {
            name,
            enabled: deployment.enabled,
            display_in_web: deployment.display_in_web,
            port: deployment.port,
            crate_path,
            display_name: manifest.name.clone(),
            description: manifest.description.clone(),
            capabilities: vec!["tools".to_string(), "prompts".to_string()],
            schemas: deployment.schemas.clone(),
            oauth: deployment.oauth.clone(),
            version: manifest.version.clone(),
            host: "127.0.0.1".to_string(),
            module_name: "mcp".to_string(),
            protocol: "mcp".to_string(),
        }
    }

    pub fn with_cargo_target_dir(
        name: String,
        manifest: &super::registry::ServerManifest,
        deployment: &super::deployment::Deployment,
        _cargo_target_dir: &str,
    ) -> Self {
        let services_path = std::env::var("SYSTEMPROMPT_SERVICES_PATH")
            .unwrap_or_else(|_| "crates/services".to_string());
        let crate_path = PathBuf::from(format!("{services_path}/{name}"));

        Self {
            name,
            enabled: deployment.enabled,
            display_in_web: deployment.display_in_web,
            port: deployment.port,
            crate_path,
            display_name: manifest.name.clone(),
            description: manifest.description.clone(),
            capabilities: vec!["tools".to_string(), "prompts".to_string()],
            schemas: deployment.schemas.clone(),
            oauth: deployment.oauth.clone(),
            version: manifest.version.clone(),
            host: "127.0.0.1".to_string(),
            module_name: "mcp".to_string(),
            protocol: "mcp".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserContext {
    Authenticated(AuthenticatedUser),
    Anonymous,
}

impl UserContext {
    pub const fn is_authenticated(&self) -> bool {
        matches!(self, Self::Authenticated(_))
    }

    pub const fn is_anonymous(&self) -> bool {
        matches!(self, Self::Anonymous)
    }

    pub const fn user(&self) -> Option<&AuthenticatedUser> {
        match self {
            Self::Authenticated(user) => Some(user),
            Self::Anonymous => None,
        }
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            Self::Authenticated(user) => user.has_permission(permission),
            Self::Anonymous => permission == Permission::Anonymous,
        }
    }

    pub fn username(&self) -> String {
        match self {
            Self::Authenticated(user) => user.username.clone(),
            Self::Anonymous => "anonymous".to_string(),
        }
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerManifest {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(rename = "websiteUrl")]
    pub website_url: Option<String>,
    pub status: String,
    pub repository: Repository,
    pub packages: Vec<Package>,
    pub remotes: Vec<Remote>,
    #[serde(rename = "_meta")]
    pub meta: RegistryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub url: String,
    pub source: String,
    pub subfolder: Option<String>,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub identifier: String,
    pub version: String,
    #[serde(rename = "runtimeHint")]
    pub runtime_hint: String,
    #[serde(rename = "registryType")]
    pub registry_type: String,
    #[serde(rename = "registryBaseUrl")]
    pub registry_base_url: String,
    #[serde(rename = "fileSha256")]
    pub file_sha256: String,
    #[serde(rename = "environmentVariables")]
    pub environment_variables: Vec<EnvVar>,
    #[serde(rename = "packageArguments")]
    pub arguments: Vec<String>,
    #[serde(rename = "runtimeArguments")]
    pub runtime_arguments: Vec<String>,
    pub transport: Transport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transport {
    pub headers: Vec<Header>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub headers: Vec<Header>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    #[serde(rename = "io.modelcontextprotocol.registry/official")]
    pub official: OfficialMetadata,
    #[serde(rename = "io.modelcontextprotocol.registry/publisher-provided")]
    pub publisher_provided: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialMetadata {
    #[serde(rename = "isLatest")]
    pub is_latest: bool,
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
    #[serde(rename = "versionId")]
    pub version_id: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub servers: Vec<super::server::McpServerConfig>,
    pub registry_url: Option<String>,
    pub cache_dir: Option<String>,
}

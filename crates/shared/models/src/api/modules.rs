use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub api_prefix: String,
    pub docs_url: String,
}

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayRouteView {
    #[serde(default)]
    pub id: String,
    pub model_pattern: String,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_model: Option<String>,
    #[serde(default)]
    pub extra_headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GatewayConfigView {
    pub enabled: bool,
    pub auth_scheme: String,
    pub inference_path_prefix: String,
    pub catalog_path: Option<String>,
    pub routes: Vec<GatewayRouteView>,
    pub profile_path: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGatewaySettingsRequest {
    pub enabled: Option<bool>,
    pub auth_scheme: Option<String>,
    pub inference_path_prefix: Option<String>,
    pub catalog_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRoutesRequest {
    pub order: Vec<usize>,
}

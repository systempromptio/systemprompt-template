//! Gateway route and settings value types.

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
    // Why: opaque passthrough of route blocks the admin UI does not edit
    // (pricing/when/requires) — typed forms live in core; dropping them on a
    // round-trip silently rewrites routing policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GatewayConfigView {
    pub enabled: bool,
    pub auth_scheme: String,
    pub inference_path_prefix: String,
    pub routes: Vec<GatewayRouteView>,
    pub profile_path: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGatewaySettingsRequest {
    pub enabled: Option<bool>,
    pub auth_scheme: Option<String>,
    pub inference_path_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderRoutesRequest {
    pub order: Vec<usize>,
}

use axum::{response::IntoResponse, Json};
use serde::Serialize;
use systemprompt_core_system::Config;

#[derive(Debug, Serialize)]
pub struct OAuthDiscoveryResponse {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: Option<String>,
    pub revocation_endpoint: Option<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
}

pub async fn oauth_discovery() -> impl IntoResponse {
    let config = Config::global();
    let base_url = &config.api_external_url;

    let discovery = OAuthDiscoveryResponse {
        issuer: base_url.clone(),
        authorization_endpoint: format!("{}/api/v1/core/oauth/authorize", base_url),
        token_endpoint: format!("{}/api/v1/core/oauth/token", base_url),
        userinfo_endpoint: Some(format!("{}/api/v1/core/oauth/userinfo", base_url)),
        revocation_endpoint: Some(format!("{}/api/v1/core/oauth/revoke", base_url)),
        response_types_supported: vec!["code".to_string(), "token".to_string()],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ],
    };

    Json(discovery)
}
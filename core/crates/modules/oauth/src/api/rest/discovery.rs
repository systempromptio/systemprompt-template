use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use systemprompt_core_system::AppContext;
use systemprompt_models::oauth::OAuthServerConfig;

#[derive(Debug, Serialize)]
pub struct WellKnownResponse {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub introspection_endpoint: String,
    pub revocation_endpoint: String,
    pub registration_endpoint: Option<String>,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub response_modes_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

pub async fn handle_well_known(State(app_context): State<AppContext>) -> impl IntoResponse {
    let config = OAuthServerConfig::from_api_server_url(&app_context.config().api_external_url);

    let response = WellKnownResponse {
        issuer: config.issuer.clone(),
        authorization_endpoint: format!("{}/api/v1/core/oauth/authorize", config.issuer),
        token_endpoint: format!("{}/api/v1/core/oauth/token", config.issuer),
        userinfo_endpoint: format!("{}/api/v1/core/oauth/userinfo", config.issuer),
        introspection_endpoint: format!("{}/api/v1/core/oauth/introspect", config.issuer),
        revocation_endpoint: format!("{}/api/v1/core/oauth/revoke", config.issuer),
        registration_endpoint: Some(format!("{}/api/v1/core/oauth/register", config.issuer)),
        scopes_supported: config.supported_scopes,
        response_types_supported: config.supported_response_types,
        response_modes_supported: vec!["query".to_string(), "form_post".to_string()],
        grant_types_supported: config.supported_grant_types,
        token_endpoint_auth_methods_supported: vec![
            "client_secret_post".to_string(),
            "client_secret_basic".to_string(),
        ],
        code_challenge_methods_supported: config.supported_code_challenge_methods,
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["HS256".to_string()],
        claims_supported: vec![
            "sub".to_string(),
            "username".to_string(),
            "email".to_string(),
            "user_type".to_string(),
            "roles".to_string(),
            "permissions".to_string(),
            "iat".to_string(),
            "exp".to_string(),
            "iss".to_string(),
            "aud".to_string(),
            "jti".to_string(),
        ],
    };

    (StatusCode::OK, Json(response)).into_response()
}

#[derive(Debug, Serialize)]
pub struct OAuthProtectedResourceResponse {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub bearer_methods_supported: Vec<String>,
    pub resource_documentation: Option<String>,
}

pub async fn handle_oauth_protected_resource(
    State(app_context): State<AppContext>,
) -> impl IntoResponse {
    let config = OAuthServerConfig::from_api_server_url(&app_context.config().api_external_url);

    let response = OAuthProtectedResourceResponse {
        resource: config.issuer.clone(),
        authorization_servers: vec![config.issuer.clone()],
        scopes_supported: config.supported_scopes,
        bearer_methods_supported: vec!["header".to_string(), "body".to_string()],
        resource_documentation: Some(format!("{}/docs", config.issuer)),
    };

    (StatusCode::OK, Json(response)).into_response()
}

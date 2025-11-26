use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthServerConfig {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub registration_endpoint: String,
    pub supported_scopes: Vec<String>,
    pub supported_grant_types: Vec<String>,
    pub supported_response_types: Vec<String>,
    #[serde(default)]
    pub supported_code_challenge_methods: Vec<String>,
    #[serde(default = "default_auth_method")]
    pub token_endpoint_auth_method: String,
    #[serde(default = "default_scope")]
    pub default_scope: String,
    #[serde(default = "default_auth_code_expiry")]
    pub auth_code_expiry_seconds: i32,
    #[serde(default = "default_access_token_expiry")]
    pub access_token_expiry_seconds: i32,
}

fn default_auth_method() -> String {
    "client_secret_basic".to_string()
}

fn default_scope() -> String {
    "openid".to_string()
}

const fn default_auth_code_expiry() -> i32 {
    600
}

const fn default_access_token_expiry() -> i32 {
    3600
}

impl OAuthServerConfig {
    pub fn new(issuer: impl Into<String>) -> Self {
        Self {
            issuer: issuer.into(),
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
            registration_endpoint: String::new(),
            supported_scopes: Vec::new(),
            supported_grant_types: Vec::new(),
            supported_response_types: Vec::new(),
            supported_code_challenge_methods: Vec::new(),
            token_endpoint_auth_method: default_auth_method(),
            default_scope: default_scope(),
            auth_code_expiry_seconds: default_auth_code_expiry(),
            access_token_expiry_seconds: default_access_token_expiry(),
        }
    }

    pub fn from_api_server_url(api_server_url: &str) -> Self {
        Self {
            issuer: api_server_url.to_owned(),
            authorization_endpoint: format!("{api_server_url}/api/v1/core/oauth/authorize"),
            token_endpoint: format!("{api_server_url}/api/v1/core/oauth/token"),
            registration_endpoint: format!("{api_server_url}/api/v1/core/oauth/clients"),
            supported_scopes: vec!["user".to_owned()],
            supported_response_types: vec!["code".to_owned()],
            supported_grant_types: vec!["authorization_code".to_owned()],
            supported_code_challenge_methods: vec!["S256".to_owned()],
            token_endpoint_auth_method: "client_secret_post".to_owned(),
            default_scope: "user".to_owned(),
            auth_code_expiry_seconds: 600,
            access_token_expiry_seconds: 3600,
        }
    }
}

impl Default for OAuthServerConfig {
    fn default() -> Self {
        Self::from_api_server_url("http://localhost:8080")
    }
}

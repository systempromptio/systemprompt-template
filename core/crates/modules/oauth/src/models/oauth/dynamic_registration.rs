use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct DynamicRegistrationRequest {
    pub client_name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub grant_types: Option<Vec<String>>,
    pub response_types: Option<Vec<String>>,
    pub scope: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub contacts: Option<Vec<String>>,

    pub software_statement: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DynamicRegistrationResponse {
    pub client_id: String,
    pub client_secret: String,
    pub client_name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub scope: String,
    pub token_endpoint_auth_method: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<String>>,

    pub client_secret_expires_at: u64,

    #[serde(with = "chrono::serde::ts_seconds")]
    pub client_id_issued_at: DateTime<Utc>,

    // RFC 7591 Required Fields for Client Configuration Management
    pub registration_access_token: String,
    pub registration_client_uri: String,
}

impl DynamicRegistrationRequest {
    pub fn get_client_name(&self) -> Result<String, String> {
        self.client_name
            .as_ref()
            .filter(|n| !n.is_empty())
            .ok_or_else(|| "client_name is required for client registration".to_string())
            .cloned()
    }

    pub fn get_redirect_uris(&self) -> Result<Vec<String>, String> {
        self.redirect_uris
            .as_ref()
            .filter(|uris| !uris.is_empty())
            .ok_or_else(|| "redirect_uris are required for client registration".to_string())
            .cloned()
    }

    pub fn get_grant_types(&self) -> Result<Vec<String>, String> {
        self.grant_types
            .as_ref()
            .filter(|types| !types.is_empty())
            .ok_or_else(|| "grant_types are required for client registration".to_string())
            .cloned()
    }

    pub fn get_response_types(&self) -> Result<Vec<String>, String> {
        self.response_types
            .as_ref()
            .filter(|types| !types.is_empty())
            .ok_or_else(|| "response_types are required for client registration".to_string())
            .cloned()
    }

    pub fn get_scopes(&self) -> Vec<String> {
        self.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(ToString::to_string).collect())
            .unwrap_or_default()
    }

    pub fn get_token_endpoint_auth_method(&self) -> Result<String, String> {
        self.token_endpoint_auth_method
            .as_ref()
            .filter(|m| !m.is_empty())
            .ok_or_else(|| {
                "token_endpoint_auth_method is required for client registration".to_string()
            })
            .cloned()
    }
}

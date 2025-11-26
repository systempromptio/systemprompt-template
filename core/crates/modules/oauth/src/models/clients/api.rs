use serde::{Deserialize, Serialize};

use super::OAuthClient;

#[derive(Debug, Deserialize)]
pub struct CreateOAuthClientRequest {
    pub client_id: String,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOAuthClientRequest {
    pub name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct OAuthClientResponse {
    pub client_id: String,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub scopes: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<OAuthClient> for OAuthClientResponse {
    fn from(client: OAuthClient) -> Self {
        Self {
            client_id: client.client_id,
            name: client.name.unwrap_or(client.client_name),
            redirect_uris: client.redirect_uris,
            scopes: client.scopes,
            created_at: client.created_at,
        }
    }
}

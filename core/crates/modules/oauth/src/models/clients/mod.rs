pub mod api;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OAuthClientRow {
    pub client_id: String,
    pub client_secret_hash: Option<String>,
    pub client_name: String,
    pub name: Option<String>,
    pub token_endpoint_auth_method: Option<String>,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub is_active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret_hash: Option<String>,
    pub client_name: String,
    pub name: Option<String>,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub response_types: Vec<String>,
    pub scopes: Vec<String>,
    pub token_endpoint_auth_method: String,
    pub client_uri: Option<String>,
    pub logo_uri: Option<String>,
    pub contacts: Option<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OAuthClient {
    pub fn from_row_with_relations(
        row: OAuthClientRow,
        redirect_uris: Vec<String>,
        grant_types: Vec<String>,
        response_types: Vec<String>,
        scopes: Vec<String>,
        contacts: Option<Vec<String>>,
    ) -> Self {
        Self {
            client_id: row.client_id,
            client_secret_hash: row.client_secret_hash,
            client_name: row.client_name,
            name: row.name,
            redirect_uris,
            grant_types,
            response_types,
            scopes,
            token_endpoint_auth_method: row
                .token_endpoint_auth_method
                .unwrap_or_else(|| "client_secret_post".to_string()),
            client_uri: row.client_uri,
            logo_uri: row.logo_uri,
            contacts,
            is_active: row.is_active.unwrap_or(true),
            created_at: row.created_at.unwrap_or_else(Utc::now),
            updated_at: row.updated_at.unwrap_or_else(Utc::now),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.client_id.is_empty() {
            return Err(anyhow::anyhow!("client_id cannot be empty"));
        }
        if self.client_name.is_empty() {
            return Err(anyhow::anyhow!("client_name cannot be empty"));
        }
        if self.redirect_uris.is_empty() {
            return Err(anyhow::anyhow!("redirect_uris cannot be empty"));
        }
        if self.grant_types.is_empty() {
            return Err(anyhow::anyhow!("grant_types cannot be empty"));
        }
        if self.response_types.is_empty() {
            return Err(anyhow::anyhow!("response_types cannot be empty"));
        }
        if self.scopes.is_empty() {
            return Err(anyhow::anyhow!("scopes cannot be empty"));
        }
        Ok(())
    }
}

pub mod api;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;

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

    pub fn from_row_with_relations(
        row: &JsonRow,
        redirect_uris: Vec<String>,
        grant_types: Vec<String>,
        response_types: Vec<String>,
        scopes: Vec<String>,
        contacts: Option<Vec<String>>,
    ) -> Result<Self> {
        use anyhow::anyhow;

        let client_id = row
            .get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_id"))?
            .to_string();

        let client_secret_hash = row
            .get("client_secret_hash")
            .and_then(|v| v.as_str())
            .map(String::from);

        let client_name = row
            .get("client_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_name"))?
            .to_string();

        let name = row.get("name").and_then(|v| v.as_str()).map(String::from);

        let token_endpoint_auth_method = row
            .get("token_endpoint_auth_method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing token_endpoint_auth_method"))?
            .to_string();

        let client_uri = row
            .get("client_uri")
            .and_then(|v| v.as_str())
            .map(String::from);

        let logo_uri = row
            .get("logo_uri")
            .and_then(|v| v.as_str())
            .map(String::from);

        let is_active = row
            .get("is_active")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|i| i != 0)))
            .unwrap_or(false);

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid updated_at"))?;

        Ok(Self {
            client_id,
            client_secret_hash,
            client_name,
            name,
            redirect_uris,
            grant_types,
            response_types,
            scopes,
            token_endpoint_auth_method,
            client_uri,
            logo_uri,
            contacts,
            is_active,
            created_at,
            updated_at,
        })
    }
}

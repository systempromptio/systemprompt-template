use crate::models::OAuthClient;
use anyhow::Result;
use chrono;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbValue, JsonRow};
use systemprompt_core_system::DbPool;

#[derive(Clone, Debug)]
pub struct ClientRepository {
    db_pool: DbPool,
}

impl ClientRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_by_client_id(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        let query = DatabaseQueryEnum::GetClientByClientId.get(self.db_pool.as_ref());
        let row = self.db_pool.fetch_optional(&query, &[&client_id]).await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let redirect_uris = self.load_redirect_uris(client_id).await?;
        let grant_types = self.load_grant_types(client_id).await?;
        let response_types = self.load_response_types(client_id).await?;
        let scopes = self.load_scopes(client_id).await?;
        let contacts = self.load_contacts(client_id).await?;

        Ok(Some(OAuthClient::from_row_with_relations(
            &row,
            redirect_uris,
            grant_types,
            response_types,
            scopes,
            contacts,
        )?))
    }

    pub async fn get_by_client_id_any(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        let query = DatabaseQueryEnum::GetClientByClientId.get(self.db_pool.as_ref());
        let row = self.db_pool.fetch_optional(&query, &[&client_id]).await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let redirect_uris = self.load_redirect_uris(client_id).await?;
        let grant_types = self.load_grant_types(client_id).await?;
        let response_types = self.load_response_types(client_id).await?;
        let scopes = self.load_scopes(client_id).await?;
        let contacts = self.load_contacts(client_id).await?;

        Ok(Some(OAuthClient::from_row_with_relations(
            &row,
            redirect_uris,
            grant_types,
            response_types,
            scopes,
            contacts,
        )?))
    }

    pub async fn create(
        &self,
        client_id: &str,
        client_secret_hash: &str,
        client_name: &str,
        redirect_uris: &[String],
        grant_types: Option<&[String]>,
        response_types: Option<&[String]>,
        scopes: &[String],
        token_endpoint_auth_method: Option<&str>,
        client_uri: Option<&str>,
        logo_uri: Option<&str>,
        contacts: Option<&[String]>,
    ) -> Result<OAuthClient> {
        let auth_method = token_endpoint_auth_method.unwrap_or("client_secret_post");

        let mut tx = self.db_pool.begin_transaction().await?;

        tx.execute(
            &DatabaseQueryEnum::InsertClientBase.get(self.db_pool.as_ref()),
            &[
                &client_id,
                &client_secret_hash,
                &client_name,
                &auth_method,
                &client_uri,
                &logo_uri,
            ],
        )
        .await?;

        for (idx, uri) in redirect_uris.iter().enumerate() {
            let is_primary = idx == 0;
            tx.execute(
                &DatabaseQueryEnum::InsertRedirectUri.get(self.db_pool.as_ref()),
                &[&client_id, &uri, &is_primary],
            )
            .await?;
        }

        let default_grant_types = vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ];
        let grant_types_list = grant_types.unwrap_or(&default_grant_types);
        for grant_type in grant_types_list {
            tx.execute(
                &DatabaseQueryEnum::InsertGrantType.get(self.db_pool.as_ref()),
                &[&client_id, &grant_type],
            )
            .await?;
        }

        let default_response_types = vec!["code".to_string()];
        let response_types_list = response_types.unwrap_or(&default_response_types);
        for response_type in response_types_list {
            tx.execute(
                &DatabaseQueryEnum::InsertResponseType.get(self.db_pool.as_ref()),
                &[&client_id, &response_type],
            )
            .await?;
        }

        for scope in scopes {
            tx.execute(
                &DatabaseQueryEnum::InsertScope.get(self.db_pool.as_ref()),
                &[&client_id, &scope],
            )
            .await?;
        }

        if let Some(contact_list) = contacts {
            for contact in contact_list {
                tx.execute(
                    &DatabaseQueryEnum::InsertContact.get(self.db_pool.as_ref()),
                    &[&client_id, &contact],
                )
                .await?;
            }
        }

        tx.commit().await?;

        self.get_by_client_id(client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Failed to load created client"))
    }

    pub async fn update(
        &self,
        client_id: &str,
        client_name: &str,
        redirect_uris: &[String],
        grant_types: Option<&[String]>,
        response_types: Option<&[String]>,
        scopes: &[String],
        token_endpoint_auth_method: Option<&str>,
        client_uri: Option<&str>,
        logo_uri: Option<&str>,
        contacts: Option<&[String]>,
    ) -> Result<Option<OAuthClient>> {
        let auth_method = token_endpoint_auth_method.unwrap_or("client_secret_post");

        let mut tx = self.db_pool.begin_transaction().await?;

        let affected = tx
            .execute(
                &DatabaseQueryEnum::UpdateClient.get(self.db_pool.as_ref()),
                &[
                    &client_name,
                    &auth_method,
                    &client_uri,
                    &logo_uri,
                    &client_id,
                ],
            )
            .await?;

        if affected == 0 {
            return Ok(None);
        }

        tx.execute(
            &DatabaseQueryEnum::DeleteRedirectUris.get(self.db_pool.as_ref()),
            &[&client_id],
        )
        .await?;
        tx.execute(
            &DatabaseQueryEnum::DeleteGrantTypes.get(self.db_pool.as_ref()),
            &[&client_id],
        )
        .await?;
        tx.execute(
            &DatabaseQueryEnum::DeleteResponseTypes.get(self.db_pool.as_ref()),
            &[&client_id],
        )
        .await?;
        tx.execute(
            &DatabaseQueryEnum::DeleteScopes.get(self.db_pool.as_ref()),
            &[&client_id],
        )
        .await?;
        tx.execute(
            &DatabaseQueryEnum::DeleteContacts.get(self.db_pool.as_ref()),
            &[&client_id],
        )
        .await?;

        for (idx, uri) in redirect_uris.iter().enumerate() {
            let is_primary = idx == 0;
            tx.execute(
                &DatabaseQueryEnum::InsertRedirectUri.get(self.db_pool.as_ref()),
                &[&client_id, &uri, &is_primary],
            )
            .await?;
        }

        let default_grant_types = vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ];
        let grant_types_list = grant_types.unwrap_or(&default_grant_types);
        for grant_type in grant_types_list {
            tx.execute(
                &DatabaseQueryEnum::InsertGrantType.get(self.db_pool.as_ref()),
                &[&client_id, &grant_type],
            )
            .await?;
        }

        let default_response_types = vec!["code".to_string()];
        let response_types_list = response_types.unwrap_or(&default_response_types);
        for response_type in response_types_list {
            tx.execute(
                &DatabaseQueryEnum::InsertResponseType.get(self.db_pool.as_ref()),
                &[&client_id, &response_type],
            )
            .await?;
        }

        for scope in scopes {
            tx.execute(
                &DatabaseQueryEnum::InsertScope.get(self.db_pool.as_ref()),
                &[&client_id, &scope],
            )
            .await?;
        }

        if let Some(contact_list) = contacts {
            for contact in contact_list {
                tx.execute(
                    &DatabaseQueryEnum::InsertContact.get(self.db_pool.as_ref()),
                    &[&client_id, &contact],
                )
                .await?;
            }
        }

        tx.commit().await?;

        self.get_by_client_id(client_id).await
    }

    pub async fn update_secret(
        &self,
        client_id: &str,
        client_secret_hash: &str,
    ) -> Result<Option<OAuthClient>> {
        let affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::UpdateClientSecret.get(self.db_pool.as_ref()),
                &[&client_secret_hash, &client_id],
            )
            .await?;

        if affected == 0 {
            return Ok(None);
        }

        self.get_by_client_id(client_id).await
    }

    pub async fn delete(&self, client_id: &str) -> Result<u64> {
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeleteClient.get(self.db_pool.as_ref()),
                &[&client_id],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn deactivate(&self, client_id: &str) -> Result<u64> {
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeactivateClient.get(self.db_pool.as_ref()),
                &[&client_id],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn activate(&self, client_id: &str) -> Result<u64> {
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::ActivateClient.get(self.db_pool.as_ref()),
                &[&client_id],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn list(&self) -> Result<Vec<OAuthClient>> {
        let base_rows = self
            .db_pool
            .fetch_all(
                &DatabaseQueryEnum::ListClients.get(self.db_pool.as_ref()),
                &[],
            )
            .await?;

        let mut clients = Vec::new();
        for row in base_rows {
            let client_id = row
                .get("client_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing client_id"))?;

            let redirect_uris = self.load_redirect_uris(client_id).await?;
            let grant_types = self.load_grant_types(client_id).await?;
            let response_types = self.load_response_types(client_id).await?;
            let scopes = self.load_scopes(client_id).await?;
            let contacts = self.load_contacts(client_id).await?;

            clients.push(OAuthClient::from_row_with_relations(
                &row,
                redirect_uris,
                grant_types,
                response_types,
                scopes,
                contacts,
            )?);
        }

        Ok(clients)
    }

    pub async fn list_paginated(&self, limit: i32, offset: i32) -> Result<Vec<OAuthClient>> {
        let base_rows = self
            .db_pool
            .fetch_all(
                &DatabaseQueryEnum::ListClients.get(self.db_pool.as_ref()),
                &[&limit, &offset],
            )
            .await?;

        let mut clients = Vec::new();
        for row in base_rows {
            let client_id = row
                .get("client_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing client_id"))?;

            let redirect_uris = self.load_redirect_uris(client_id).await?;
            let grant_types = self.load_grant_types(client_id).await?;
            let response_types = self.load_response_types(client_id).await?;
            let scopes = self.load_scopes(client_id).await?;
            let contacts = self.load_contacts(client_id).await?;

            clients.push(OAuthClient::from_row_with_relations(
                &row,
                redirect_uris,
                grant_types,
                response_types,
                scopes,
                contacts,
            )?);
        }

        Ok(clients)
    }

    pub async fn count(&self) -> Result<i64> {
        let value = self
            .db_pool
            .fetch_scalar_value(
                &DatabaseQueryEnum::CountClients.get(self.db_pool.as_ref()),
                &[],
            )
            .await?;
        match value {
            DbValue::Int(i) => Ok(i),
            _ => Err(anyhow::anyhow!("Expected integer count, got {value:?}")),
        }
    }

    async fn load_redirect_uris(&self, client_id: &str) -> Result<Vec<String>> {
        let query = DatabaseQueryEnum::LoadRedirectUris.get(self.db_pool.as_ref());
        let rows = self.db_pool.fetch_all(&query, &[&client_id]).await?;

        Ok(rows
            .iter()
            .filter_map(|r| {
                r.get("redirect_uri")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect())
    }

    async fn load_grant_types(&self, client_id: &str) -> Result<Vec<String>> {
        let query = DatabaseQueryEnum::LoadGrantTypes.get(self.db_pool.as_ref());
        let rows = self.db_pool.fetch_all(&query, &[&client_id]).await?;

        Ok(rows
            .iter()
            .filter_map(|r| {
                r.get("grant_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect())
    }

    async fn load_response_types(&self, client_id: &str) -> Result<Vec<String>> {
        let query = DatabaseQueryEnum::LoadResponseTypes.get(self.db_pool.as_ref());
        let rows = self.db_pool.fetch_all(&query, &[&client_id]).await?;

        Ok(rows
            .iter()
            .filter_map(|r| {
                r.get("response_type")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect())
    }

    async fn load_scopes(&self, client_id: &str) -> Result<Vec<String>> {
        let query = DatabaseQueryEnum::LoadScopes.get(self.db_pool.as_ref());
        let rows = self.db_pool.fetch_all(&query, &[&client_id]).await?;

        Ok(rows
            .iter()
            .filter_map(|r| r.get("scope").and_then(|v| v.as_str()).map(String::from))
            .collect())
    }

    async fn load_contacts(&self, client_id: &str) -> Result<Option<Vec<String>>> {
        let query = DatabaseQueryEnum::LoadContacts.get(self.db_pool.as_ref());
        let rows = self.db_pool.fetch_all(&query, &[&client_id]).await?;

        let contacts: Vec<String> = rows
            .iter()
            .filter_map(|r| {
                r.get("contact_email")
                    .and_then(|v| v.as_str())
                    .map(String::from)
            })
            .collect();

        if contacts.is_empty() {
            Ok(None)
        } else {
            Ok(Some(contacts))
        }
    }

    pub async fn cleanup_inactive(&self) -> Result<u64> {
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeleteInactiveClients.get(self.db_pool.as_ref()),
                &[],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn cleanup_old_test(&self, days_old: u32) -> Result<u64> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeleteOldTestClients.get(self.db_pool.as_ref()),
                &[&cutoff_timestamp],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn deactivate_old_test(&self, days_old: u32) -> Result<u64> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeactivateOldTestClients.get(self.db_pool.as_ref()),
                &[&cutoff_timestamp],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn delete_unused(&self, never_used_before: i64) -> Result<u64> {
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeleteUnusedClients.get(self.db_pool.as_ref()),
                &[&never_used_before],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn delete_stale(&self, last_used_before: i64) -> Result<u64> {
        let rows_affected = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeleteStaleClients.get(self.db_pool.as_ref()),
                &[&last_used_before],
            )
            .await?;
        Ok(rows_affected)
    }

    pub async fn list_inactive(&self) -> Result<Vec<ClientSummary>> {
        let rows = self
            .db_pool
            .fetch_all(
                &DatabaseQueryEnum::ListInactiveClients.get(self.db_pool.as_ref()),
                &[],
            )
            .await?;

        rows.into_iter()
            .map(|row| ClientSummary::from_json_row(&row))
            .collect()
    }

    pub async fn list_old(&self, older_than_timestamp: i64) -> Result<Vec<ClientSummary>> {
        let rows = self
            .db_pool
            .fetch_all(
                &DatabaseQueryEnum::ListOldClients.get(self.db_pool.as_ref()),
                &[&older_than_timestamp],
            )
            .await?;

        rows.into_iter()
            .map(|row| ClientSummary::from_json_row(&row))
            .collect()
    }

    pub async fn list_unused(&self, never_used_before: i64) -> Result<Vec<ClientUsageSummary>> {
        let rows = self
            .db_pool
            .fetch_all(
                &DatabaseQueryEnum::ListUnusedClients.get(self.db_pool.as_ref()),
                &[&never_used_before],
            )
            .await?;

        rows.into_iter()
            .map(|row| ClientUsageSummary::from_json_row(&row))
            .collect()
    }

    pub async fn list_stale(&self, last_used_before: i64) -> Result<Vec<ClientUsageSummary>> {
        let rows = self
            .db_pool
            .fetch_all(
                &DatabaseQueryEnum::ListStaleClients.get(self.db_pool.as_ref()),
                &[&last_used_before],
            )
            .await?;

        rows.into_iter()
            .map(|row| ClientUsageSummary::from_json_row(&row))
            .collect()
    }

    pub async fn update_last_used(&self, client_id: &str, timestamp: i64) -> Result<()> {
        self.db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::UpdateClientLastUsed.get(self.db_pool.as_ref()),
                &[&timestamp, &client_id],
            )
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClientSummary {
    pub client_id: String,
    pub client_name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ClientSummary {
    fn extract_timestamp(value: &serde_json::Value) -> Option<i64> {
        if let Some(n) = value.as_i64() {
            Some(n)
        } else if let Some(n) = value.as_u64() {
            Some(n as i64)
        } else if let Some(s) = value.as_str() {
            s.parse::<i64>().ok()
        } else { value.as_f64().map(|n| n as i64) }
    }

    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let client_id = row
            .get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_id"))?
            .to_string();

        let client_name = row
            .get("client_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_name"))?
            .to_string();

        let created_at = row
            .get("created_at")
            .and_then(Self::extract_timestamp)
            .ok_or_else(|| anyhow!("Missing created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(Self::extract_timestamp)
            .ok_or_else(|| anyhow!("Missing updated_at"))?;

        Ok(Self {
            client_id,
            client_name,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug)]
pub struct ClientUsageSummary {
    pub client_id: String,
    pub client_name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub last_used_at: Option<i64>,
}

impl ClientUsageSummary {
    fn extract_timestamp(value: &serde_json::Value) -> Option<i64> {
        if let Some(n) = value.as_i64() {
            Some(n)
        } else if let Some(n) = value.as_u64() {
            Some(n as i64)
        } else if let Some(s) = value.as_str() {
            s.parse::<i64>().ok()
        } else { value.as_f64().map(|n| n as i64) }
    }

    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let client_id = row
            .get("client_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_id"))?
            .to_string();

        let client_name = row
            .get("client_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing client_name"))?
            .to_string();

        let created_at = row
            .get("created_at")
            .and_then(Self::extract_timestamp)
            .ok_or_else(|| anyhow!("Missing created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(Self::extract_timestamp)
            .ok_or_else(|| anyhow!("Missing updated_at"))?;

        let last_used_at = row.get("last_used_at").and_then(Self::extract_timestamp);

        Ok(Self {
            client_id,
            client_name,
            created_at,
            updated_at,
            last_used_at,
        })
    }
}

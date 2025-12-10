use super::ClientRepository;
use crate::models::{OAuthClient, OAuthClientRow};
use anyhow::Result;

impl ClientRepository {
    pub async fn get_by_client_id(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        let row = sqlx::query_as!(
            OAuthClientRow,
            "SELECT client_id, client_secret_hash, client_name, name, token_endpoint_auth_method,
                    client_uri, logo_uri, is_active, created_at, updated_at
             FROM oauth_clients WHERE client_id = $1 AND is_active = true",
            client_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        match row {
            Some(row) => {
                let client = self.load_client_with_relations(row).await?;
                Ok(Some(client))
            },
            None => Ok(None),
        }
    }

    pub async fn get_by_client_id_any(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        let row = sqlx::query_as!(
            OAuthClientRow,
            "SELECT client_id, client_secret_hash, client_name, name, token_endpoint_auth_method,
                    client_uri, logo_uri, is_active, created_at, updated_at
             FROM oauth_clients WHERE client_id = $1",
            client_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        match row {
            Some(row) => {
                let client = self.load_client_with_relations(row).await?;
                Ok(Some(client))
            },
            None => Ok(None),
        }
    }

    pub async fn list(&self) -> Result<Vec<OAuthClient>> {
        let rows = sqlx::query_as!(
            OAuthClientRow,
            "SELECT client_id, client_secret_hash, client_name, name, token_endpoint_auth_method,
                    client_uri, logo_uri, is_active, created_at, updated_at
             FROM oauth_clients WHERE is_active = true ORDER BY created_at DESC"
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut clients = Vec::new();
        for row in rows {
            let client = self.load_client_with_relations(row).await?;
            clients.push(client);
        }

        Ok(clients)
    }

    pub async fn list_paginated(&self, limit: i32, offset: i32) -> Result<Vec<OAuthClient>> {
        let limit_i64 = i64::from(limit);
        let offset_i64 = i64::from(offset);
        let rows = sqlx::query_as!(
            OAuthClientRow,
            "SELECT client_id, client_secret_hash, client_name, name, token_endpoint_auth_method,
                    client_uri, logo_uri, is_active, created_at, updated_at
             FROM oauth_clients WHERE is_active = true ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
            limit_i64,
            offset_i64
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut clients = Vec::new();
        for row in rows {
            let client = self.load_client_with_relations(row).await?;
            clients.push(client);
        }

        Ok(clients)
    }

    pub async fn count(&self) -> Result<i64> {
        let result =
            sqlx::query_scalar!("SELECT COUNT(*) FROM oauth_clients WHERE is_active = true")
                .fetch_one(&*self.pool)
                .await?;
        Ok(result.unwrap_or(0))
    }
}

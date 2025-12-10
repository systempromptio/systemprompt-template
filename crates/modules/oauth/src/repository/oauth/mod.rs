mod auth_code;
mod refresh_token;
mod scopes;
mod user;

use super::ClientRepository;
use crate::models::OAuthClient;
use crate::services::generate_client_secret;
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct OAuthRepository {
    pool: Arc<PgPool>,
    db: DbPool,
    log: LogService,
}

impl RepositoryTrait for OAuthRepository {
    type Pool = Arc<PgPool>;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }
}

impl OAuthRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        let log = LogService::system(db.clone());
        Self { pool, db, log }
    }

    pub fn pool_ref(&self) -> &PgPool {
        &self.pool
    }

    pub const fn log_ref(&self) -> &LogService {
        &self.log
    }

    pub async fn create_client(
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
        let start_time = Instant::now();

        let _ = self
            .log
            .info(
                "oauth_repo",
                &format!("Creating OAuth client: {client_id} (name: {client_name})"),
            )
            .await;

        let client_repo = ClientRepository::new(self.db.clone());
        match client_repo
            .create(
                client_id,
                client_secret_hash,
                client_name,
                redirect_uris,
                grant_types,
                response_types,
                scopes,
                token_endpoint_auth_method,
                client_uri,
                logo_uri,
                contacts,
            )
            .await
        {
            Ok(client) => {
                let duration = start_time.elapsed();

                self.log
                    .log(
                        LogLevel::Info,
                        "oauth_repo",
                        &format!("OAuth client created: {client_id}"),
                        Some(serde_json::json!({
                            "client_id": client_id,
                            "client_name": client_name,
                            "scopes": scopes,
                            "redirect_uris": redirect_uris,
                            "created_in_ms": duration.as_millis()
                        })),
                    )
                    .await
                    .ok();

                let _ = self
                    .log
                    .info(
                        "oauth_repo",
                        &format!("OAuth client {client_id} created successfully in {duration:?}"),
                    )
                    .await;

                if duration.as_millis() > 500 {
                    let _ = self
                        .log
                        .warn(
                            "oauth_repo",
                            &format!("Slow OAuth client creation: {client_id} took {duration:?}"),
                        )
                        .await;
                }

                Ok(client)
            },
            Err(e) => {
                let duration = start_time.elapsed();
                let _ = self
                    .log
                    .error(
                        "oauth_repo",
                        &format!(
                            "OAuth client creation failed: {client_id} after {duration:?} - {e:?}"
                        ),
                    )
                    .await;
                Err(e)
            },
        }
    }

    pub async fn list_clients(&self) -> Result<Vec<OAuthClient>> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.list().await
    }

    pub async fn list_clients_paginated(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<OAuthClient>> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.list_paginated(limit, offset).await
    }

    pub async fn count_clients(&self) -> Result<i64> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.count().await
    }

    pub async fn find_client_by_id(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.get_by_client_id(client_id).await
    }

    pub async fn get_client(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        self.find_client_by_id(client_id).await
    }

    pub async fn update_client(
        &self,
        client_id: &str,
        name: Option<&str>,
        redirect_uris: Option<&[String]>,
        scopes: Option<&[String]>,
    ) -> Result<OAuthClient> {
        let updated_name = match name {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => {
                return Err(anyhow::anyhow!("Client name is required for update"));
            },
        };

        let updated_redirect_uris = redirect_uris
            .filter(|uris| !uris.is_empty())
            .ok_or_else(|| anyhow::anyhow!("At least one redirect URI required"))?;

        let updated_scopes = scopes
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow::anyhow!("At least one scope required"))?;

        let client = self
            .get_client(client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        let client_repo = ClientRepository::new(self.db.clone());
        let updated = client_repo
            .update(
                client_id,
                &updated_name,
                updated_redirect_uris,
                Some(&client.grant_types),
                Some(&client.response_types),
                updated_scopes,
                Some(&client.token_endpoint_auth_method),
                client.client_uri.as_deref(),
                client.logo_uri.as_deref(),
                client.contacts.as_deref(),
            )
            .await?;

        updated.ok_or_else(|| anyhow::anyhow!("Client not found"))
    }

    pub async fn update_client_full(&self, client: &OAuthClient) -> Result<OAuthClient> {
        let client_repo = ClientRepository::new(self.db.clone());
        let updated = client_repo
            .update(
                &client.client_id,
                &client.client_name,
                &client.redirect_uris,
                Some(&client.grant_types),
                Some(&client.response_types),
                &client.scopes,
                Some(&client.token_endpoint_auth_method),
                client.client_uri.as_deref(),
                client.logo_uri.as_deref(),
                client.contacts.as_deref(),
            )
            .await?;

        updated.ok_or_else(|| anyhow::anyhow!("Client not found"))
    }

    pub async fn delete_client(&self, client_id: &str) -> Result<bool> {
        let client_repo = ClientRepository::new(self.db.clone());
        let rows_affected = client_repo.delete(client_id).await?;
        Ok(rows_affected > 0)
    }

    #[must_use]
    pub fn generate_client_secret() -> String {
        generate_client_secret()
    }

    #[must_use]
    pub fn generate_client_id() -> String {
        format!("client_{}", Uuid::new_v4().simple())
    }

    pub async fn find_client(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        self.find_client_by_id(client_id).await
    }

    pub async fn cleanup_inactive_clients(&self) -> Result<u64> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.cleanup_inactive().await
    }

    pub async fn cleanup_old_test_clients(&self, days_old: u32) -> Result<u64> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.cleanup_old_test(days_old).await
    }

    pub async fn cleanup_unused_clients(&self, days_old: u32) -> Result<u64> {
        let cutoff_timestamp = Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.delete_unused(cutoff_timestamp).await
    }

    pub async fn cleanup_stale_clients(&self, days_unused: u32) -> Result<u64> {
        let cutoff_timestamp = Utc::now().timestamp() - (i64::from(days_unused) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.delete_stale(cutoff_timestamp).await
    }

    pub async fn get_unused_clients(
        &self,
        days_old: u32,
    ) -> Result<Vec<super::ClientUsageSummary>> {
        let cutoff_timestamp = Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.list_unused(cutoff_timestamp).await
    }

    pub async fn get_stale_clients(
        &self,
        days_unused: u32,
    ) -> Result<Vec<super::ClientUsageSummary>> {
        let cutoff_timestamp = Utc::now().timestamp() - (i64::from(days_unused) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.list_stale(cutoff_timestamp).await
    }

    pub async fn deactivate_old_test_clients(&self, days_old: u32) -> Result<u64> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.deactivate_old_test(days_old).await
    }

    pub async fn get_inactive_clients(&self) -> Result<Vec<super::ClientSummary>> {
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.list_inactive().await
    }

    pub async fn get_old_clients(&self, days_old: u32) -> Result<Vec<super::ClientSummary>> {
        let cutoff_timestamp = Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.list_old(cutoff_timestamp).await
    }

    pub async fn update_client_last_used(&self, client_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();
        let client_repo = ClientRepository::new(self.db.clone());
        client_repo.update_last_used(client_id, now).await
    }
}

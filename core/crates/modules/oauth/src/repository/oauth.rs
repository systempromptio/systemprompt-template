use super::ClientRepository;
use crate::models::OAuthClient;
use crate::services::generate_client_secret;
use anyhow::Result;
use base64::Engine;
use chrono;
use std::str::FromStr;
use std::time::Instant;
use systemprompt_models::auth::Permission;
use systemprompt_core_database::{parse_database_datetime, DatabaseProvider, DatabaseQueryEnum};
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::DbPool;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct OAuthRepository {
    pub(crate) db_pool: DbPool,
    log: LogService,
}

impl RepositoryTrait for OAuthRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl OAuthRepository {
    pub fn new(db_pool: DbPool) -> Self {
        let log = LogService::system(db_pool.clone());
        Self { db_pool, log }
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
                &format!(
                    "Creating OAuth client: {client_id} (name: {client_name})"
                ),
            )
            .await;

        let client_repo = ClientRepository::new(self.db_pool.clone());
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
                        &format!(
                            "OAuth client {client_id} created successfully in {duration:?}"
                        ),
                    )
                    .await;

                if duration.as_millis() > 500 {
                    let _ = self
                        .log
                        .warn(
                            "oauth_repo",
                            &format!(
                                "Slow OAuth client creation: {client_id} took {duration:?}"
                            ),
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
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.list().await
    }

    pub async fn list_clients_paginated(
        &self,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<OAuthClient>> {
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.list_paginated(limit, offset).await
    }

    pub async fn count_clients(&self) -> Result<i64> {
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.count().await
    }

    pub async fn find_client_by_id(&self, client_id: &str) -> Result<Option<OAuthClient>> {
        let client_repo = ClientRepository::new(self.db_pool.clone());
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

        let client_repo = ClientRepository::new(self.db_pool.clone());
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
        let client_repo = ClientRepository::new(self.db_pool.clone());
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
        let client_repo = ClientRepository::new(self.db_pool.clone());
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

    pub async fn store_authorization_code(
        &self,
        code: &str,
        client_id: &str,
        user_id: &str,
        redirect_uri: &str,
        scope: &str,
        code_challenge: Option<&str>,
        code_challenge_method: Option<&str>,
    ) -> Result<()> {
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(600); // 10 minutes

        self.db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::InsertAuthorizationCode.get(self.db_pool.as_ref()),
                &[
                    &code,
                    &client_id,
                    &user_id,
                    &redirect_uri,
                    &scope,
                    &expires_at,
                    &code_challenge,
                    &code_challenge_method,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_client_id_from_auth_code(&self, code: &str) -> Result<Option<String>> {
        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(
                &DatabaseQueryEnum::GetAuthorizationCode.get(self.db_pool.as_ref()),
                &[&code],
            )
            .await?;

        Ok(row.and_then(|r| {
            r.get("client_id")
                .and_then(|v| v.as_str())
                .map(ToString::to_string)
        }))
    }

    pub async fn validate_authorization_code(
        &self,
        code: &str,
        client_id: &str,
        redirect_uri: Option<&str>,
        code_verifier: Option<&str>,
    ) -> Result<(String, String)> {
        let now = chrono::Utc::now();

        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(
                &DatabaseQueryEnum::GetAuthorizationCode.get(self.db_pool.as_ref()),
                &[&code, &client_id],
            )
            .await?;

        let row = row.ok_or_else(|| anyhow::anyhow!("Invalid authorization code"))?;

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?
            .to_string();

        let scope = row
            .get("scope")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing scope"))?
            .to_string();

        let expires_at = row
            .get("expires_at")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow::anyhow!("Missing expires_at"))?;

        let redirect_uri_db = row
            .get("redirect_uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing redirect_uri"))?
            .to_string();

        let used_at = row.get("used_at").and_then(parse_database_datetime);

        let code_challenge = row
            .get("code_challenge")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let code_challenge_method = row
            .get("code_challenge_method")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        if used_at.is_some() {
            return Err(anyhow::anyhow!("Authorization code already used"));
        }

        if expires_at < now {
            return Err(anyhow::anyhow!("Authorization code expired"));
        }

        if let Some(expected_uri) = redirect_uri {
            if redirect_uri_db != expected_uri {
                return Err(anyhow::anyhow!("Redirect URI mismatch"));
            }
        }

        if let Some(challenge) = code_challenge {
            let verifier =
                code_verifier.ok_or_else(|| anyhow::anyhow!("code_verifier required for PKCE"))?;

            let method = code_challenge_method
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("code_challenge_method required for PKCE"))?;

            let computed_challenge = match method.as_str() {
                "S256" => {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(verifier.as_bytes());
                    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize())
                },
                "plain" => {
                    return Err(anyhow::anyhow!(
                        "PKCE method 'plain' is not allowed. Only 'S256' is supported for security."
                    ));
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported code_challenge_method: {method}. Only 'S256' is allowed."
                    ))
                },
            };

            if computed_challenge != challenge {
                return Err(anyhow::anyhow!("PKCE validation failed"));
            }
        }

        self.db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::MarkAuthorizationCodeUsed.get(self.db_pool.as_ref()),
                &[&now, &code],
            )
            .await?;

        Ok((user_id, scope))
    }

    pub async fn store_refresh_token(
        &self,
        token_id: &str,
        client_id: &str,
        user_id: &str,
        scope: &str,
        expires_at: i64,
    ) -> Result<()> {
        let expires_at_dt = chrono::DateTime::from_timestamp(expires_at, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp for expires_at"))?;

        self.db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::InsertRefreshToken.get(self.db_pool.as_ref()),
                &[&token_id, &client_id, &user_id, &scope, &expires_at_dt],
            )
            .await?;

        Ok(())
    }

    pub async fn validate_refresh_token(
        &self,
        token_id: &str,
        client_id: &str,
    ) -> Result<(String, String)> {
        let now = chrono::Utc::now();

        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(
                &DatabaseQueryEnum::GetRefreshToken.get(self.db_pool.as_ref()),
                &[&token_id, &client_id],
            )
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid refresh token"))?;

        let expires_at = row
            .get("expires_at")
            .and_then(parse_database_datetime)
            .ok_or_else(|| anyhow::anyhow!("Missing expires_at"))?;

        if expires_at < now {
            return Err(anyhow::anyhow!("Refresh token expired"));
        }

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing user_id"))?
            .to_string();

        let scope = row
            .get("scope")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing scope"))?
            .to_string();

        Ok((user_id, scope))
    }

    pub async fn consume_refresh_token(
        &self,
        token_id: &str,
        client_id: &str,
    ) -> Result<(String, String)> {
        let (user_id, scope) = self.validate_refresh_token(token_id, client_id).await?;

        self.db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::RevokeRefreshToken.get(self.db_pool.as_ref()),
                &[&token_id],
            )
            .await?;

        Ok((user_id, scope))
    }

    pub async fn revoke_refresh_token(&self, token_id: &str) -> Result<bool> {
        let result = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::RevokeRefreshToken.get(self.db_pool.as_ref()),
                &[&token_id],
            )
            .await?;

        Ok(result > 0)
    }

    pub async fn cleanup_expired_refresh_tokens(&self) -> Result<u64> {
        let now = chrono::Utc::now();

        let result = self
            .db_pool
            .as_ref()
            .execute(
                &DatabaseQueryEnum::DeleteExpiredRefreshTokens.get(self.db_pool.as_ref()),
                &[&now],
            )
            .await?;

        Ok(result)
    }

    pub async fn validate_scopes(&self, requested_scopes: &[String]) -> Result<Vec<String>> {
        if requested_scopes.is_empty() {
            return Ok(vec![]);
        }

        let mut valid_scopes = Vec::new();
        let mut invalid_scopes = Vec::new();

        for scope in requested_scopes {
            match self.scope_exists(scope).await {
                Ok(true) => valid_scopes.push(scope.clone()),
                Ok(false) => invalid_scopes.push(scope.clone()),
                Err(e) => return Err(e),
            }
        }

        if !invalid_scopes.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid scopes (roles): {}",
                invalid_scopes.join(", ")
            ));
        }

        Ok(valid_scopes)
    }

    pub async fn get_available_scopes(&self) -> Result<Vec<(String, Option<String>)>> {
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&DatabaseQueryEnum::GetRoles.get(self.db_pool.as_ref()), &[])
            .await?;

        let scopes: Vec<(String, Option<String>)> = rows
            .iter()
            .filter_map(|row| {
                let scope_name = row.get("name")?.as_str()?.to_string();
                let description = row
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
                Some((scope_name, description))
            })
            .collect();

        Ok(scopes)
    }

    pub async fn scope_exists(&self, scope_name: &str) -> Result<bool> {
        let row = self
            .db_pool
            .as_ref()
            .fetch_one(
                &DatabaseQueryEnum::CheckRoleExists.get(self.db_pool.as_ref()),
                &[&scope_name],
            )
            .await?;

        let count = row
            .get("count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("Missing count"))?;

        Ok(count > 0)
    }

    pub fn parse_scopes(scope_string: &str) -> Vec<String> {
        scope_string
            .split_whitespace()
            .map(ToString::to_string)
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn format_scopes(scopes: &[String]) -> String {
        scopes.join(" ")
    }

    pub async fn get_default_roles(&self) -> Result<Vec<String>> {
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(
                &DatabaseQueryEnum::GetDefaultRoles.get(self.db_pool.as_ref()),
                &[],
            )
            .await?;

        let default_roles: Vec<String> = rows
            .iter()
            .filter_map(|row| {
                row.get("name")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
            .collect();

        Ok(default_roles)
    }

    pub async fn cleanup_inactive_clients(&self) -> Result<u64> {
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.cleanup_inactive().await
    }

    pub async fn cleanup_old_test_clients(&self, days_old: u32) -> Result<u64> {
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.cleanup_old_test(days_old).await
    }

    pub async fn cleanup_unused_clients(&self, days_old: u32) -> Result<u64> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.delete_unused(cutoff_timestamp).await
    }

    pub async fn cleanup_stale_clients(&self, days_unused: u32) -> Result<u64> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (i64::from(days_unused) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.delete_stale(cutoff_timestamp).await
    }

    pub async fn get_unused_clients(
        &self,
        days_old: u32,
    ) -> Result<Vec<super::ClientUsageSummary>> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.list_unused(cutoff_timestamp).await
    }

    pub async fn get_stale_clients(
        &self,
        days_unused: u32,
    ) -> Result<Vec<super::ClientUsageSummary>> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (i64::from(days_unused) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.list_stale(cutoff_timestamp).await
    }

    pub async fn deactivate_old_test_clients(&self, days_old: u32) -> Result<u64> {
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.deactivate_old_test(days_old).await
    }

    pub async fn get_inactive_clients(&self) -> Result<Vec<super::ClientSummary>> {
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.list_inactive().await
    }

    pub async fn get_old_clients(&self, days_old: u32) -> Result<Vec<super::ClientSummary>> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (i64::from(days_old) * 24 * 60 * 60);
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.list_old(cutoff_timestamp).await
    }

    pub async fn update_client_last_used(&self, client_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        let client_repo = ClientRepository::new(self.db_pool.clone());
        client_repo.update_last_used(client_id, now).await
    }

    pub async fn get_authenticated_user(
        &self,
        user_id: &str,
    ) -> Result<systemprompt_models::auth::AuthenticatedUser> {
        let query = DatabaseQueryEnum::GetAuthenticatedUser.get(self.db_pool.as_ref());
        let row = self
            .db_pool
            .as_ref()
            .fetch_optional(&query, &[&user_id])
            .await?
            .ok_or_else(|| anyhow::anyhow!("User not found: {user_id}"))?;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing user id"))?
            .to_string();

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing user name"))?
            .to_string();

        let email = row
            .get("email")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        let role_strings: Vec<String> = row
            .get("roles")
            .ok_or_else(|| anyhow::anyhow!("User has no roles configured"))
            .and_then(|v| {
                if let Some(arr) = v.as_array() {
                    arr.iter()
                        .map(|item| {
                            item.as_str()
                                .map(ToString::to_string)
                                .ok_or_else(|| anyhow::anyhow!("Role item is not a string"))
                        })
                        .collect::<Result<Vec<String>>>()
                } else if let Some(s) = v.as_str() {
                    serde_json::from_str::<Vec<String>>(s)
                        .map_err(|e| anyhow::anyhow!("Failed to parse user roles JSON: {e}"))
                } else {
                    Err(anyhow::anyhow!("Roles is neither array nor JSON string"))
                }
            })?;

        let permissions: Vec<Permission> = role_strings
            .iter()
            .filter_map(|s| Permission::from_str(s).ok())
            .collect();

        if permissions.is_empty() {
            return Err(anyhow::anyhow!(
                "User has no valid permissions after parsing"
            ));
        }

        let user_uuid =
            Uuid::parse_str(&id).map_err(|_| anyhow::anyhow!("Invalid user UUID: {id}"))?;

        Ok(systemprompt_models::auth::AuthenticatedUser::new(
            user_uuid,
            name,
            email,
            permissions,
        ))
    }
}

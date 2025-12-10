use super::ClientRepository;
use crate::models::OAuthClient;
use anyhow::Result;
use chrono::Utc;

impl ClientRepository {
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
        let now = Utc::now();

        let mut tx = self.pool.as_ref().begin().await?;

        sqlx::query!(
            "INSERT INTO oauth_clients (client_id, client_secret_hash, client_name,
                                       token_endpoint_auth_method, client_uri, logo_uri,
                                       is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, true, $7, $7)",
            client_id,
            client_secret_hash,
            client_name,
            auth_method,
            client_uri,
            logo_uri,
            now
        )
        .execute(&mut *tx)
        .await?;

        for (idx, uri) in redirect_uris.iter().enumerate() {
            let is_primary = idx == 0;
            sqlx::query!(
                "INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
                 VALUES ($1, $2, $3)",
                client_id,
                uri,
                is_primary
            )
            .execute(&mut *tx)
            .await?;
        }

        let default_grant_types = vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ];
        let grant_types_list = grant_types.unwrap_or(&default_grant_types);
        for grant_type in grant_types_list {
            sqlx::query!(
                "INSERT INTO oauth_client_grant_types (client_id, grant_type) VALUES ($1, $2)",
                client_id,
                grant_type
            )
            .execute(&mut *tx)
            .await?;
        }

        let default_response_types = vec!["code".to_string()];
        let response_types_list = response_types.unwrap_or(&default_response_types);
        for response_type in response_types_list {
            sqlx::query!(
                "INSERT INTO oauth_client_response_types (client_id, response_type) VALUES ($1, \
                 $2)",
                client_id,
                response_type
            )
            .execute(&mut *tx)
            .await?;
        }

        for scope in scopes {
            sqlx::query!(
                "INSERT INTO oauth_client_scopes (client_id, scope) VALUES ($1, $2)",
                client_id,
                scope
            )
            .execute(&mut *tx)
            .await?;
        }

        if let Some(contact_list) = contacts {
            for contact in contact_list {
                sqlx::query!(
                    "INSERT INTO oauth_client_contacts (client_id, contact_email) VALUES ($1, $2)",
                    client_id,
                    contact
                )
                .execute(&mut *tx)
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
        let now = Utc::now();

        let mut tx = self.pool.as_ref().begin().await?;

        let result = sqlx::query!(
            "UPDATE oauth_clients SET client_name = $1, token_endpoint_auth_method = $2,
                                      client_uri = $3, logo_uri = $4, updated_at = $5
             WHERE client_id = $6",
            client_name,
            auth_method,
            client_uri,
            logo_uri,
            now,
            client_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        sqlx::query!(
            "DELETE FROM oauth_client_redirect_uris WHERE client_id = $1",
            client_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM oauth_client_grant_types WHERE client_id = $1",
            client_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM oauth_client_response_types WHERE client_id = $1",
            client_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM oauth_client_scopes WHERE client_id = $1",
            client_id
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "DELETE FROM oauth_client_contacts WHERE client_id = $1",
            client_id
        )
        .execute(&mut *tx)
        .await?;

        for (idx, uri) in redirect_uris.iter().enumerate() {
            let is_primary = idx == 0;
            sqlx::query!(
                "INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
                 VALUES ($1, $2, $3)",
                client_id,
                uri,
                is_primary
            )
            .execute(&mut *tx)
            .await?;
        }

        let default_grant_types = vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
        ];
        let grant_types_list = grant_types.unwrap_or(&default_grant_types);
        for grant_type in grant_types_list {
            sqlx::query!(
                "INSERT INTO oauth_client_grant_types (client_id, grant_type) VALUES ($1, $2)",
                client_id,
                grant_type
            )
            .execute(&mut *tx)
            .await?;
        }

        let default_response_types = vec!["code".to_string()];
        let response_types_list = response_types.unwrap_or(&default_response_types);
        for response_type in response_types_list {
            sqlx::query!(
                "INSERT INTO oauth_client_response_types (client_id, response_type) VALUES ($1, \
                 $2)",
                client_id,
                response_type
            )
            .execute(&mut *tx)
            .await?;
        }

        for scope in scopes {
            sqlx::query!(
                "INSERT INTO oauth_client_scopes (client_id, scope) VALUES ($1, $2)",
                client_id,
                scope
            )
            .execute(&mut *tx)
            .await?;
        }

        if let Some(contact_list) = contacts {
            for contact in contact_list {
                sqlx::query!(
                    "INSERT INTO oauth_client_contacts (client_id, contact_email) VALUES ($1, $2)",
                    client_id,
                    contact
                )
                .execute(&mut *tx)
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
        let now = Utc::now();
        let result = sqlx::query!(
            "UPDATE oauth_clients SET client_secret_hash = $1, updated_at = $2 WHERE client_id = \
             $3",
            client_secret_hash,
            now,
            client_id
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_by_client_id(client_id).await
    }

    pub async fn delete(&self, client_id: &str) -> Result<u64> {
        let result = sqlx::query!("DELETE FROM oauth_clients WHERE client_id = $1", client_id)
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn deactivate(&self, client_id: &str) -> Result<u64> {
        let now = Utc::now();
        let result = sqlx::query!(
            "UPDATE oauth_clients SET is_active = false, updated_at = $1 WHERE client_id = $2",
            now,
            client_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn activate(&self, client_id: &str) -> Result<u64> {
        let now = Utc::now();
        let result = sqlx::query!(
            "UPDATE oauth_clients SET is_active = true, updated_at = $1 WHERE client_id = $2",
            now,
            client_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

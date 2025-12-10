use super::OAuthRepository;
use anyhow::Result;
use chrono::Utc;

impl OAuthRepository {
    pub async fn store_refresh_token(
        &self,
        token_id: &str,
        client_id: &str,
        user_id: &str,
        scope: &str,
        expires_at: i64,
    ) -> Result<()> {
        let expires_at_dt = chrono::DateTime::<Utc>::from_timestamp(expires_at, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp for expires_at"))?;
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO oauth_refresh_tokens (token_id, client_id, user_id, scope, expires_at, \
             created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            token_id,
            client_id,
            user_id,
            scope,
            expires_at_dt,
            now
        )
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    pub async fn validate_refresh_token(
        &self,
        token_id: &str,
        client_id: &str,
    ) -> Result<(String, String)> {
        let now = Utc::now();

        let row = sqlx::query!(
            "SELECT user_id, scope, expires_at FROM oauth_refresh_tokens
             WHERE token_id = $1 AND client_id = $2",
            token_id,
            client_id
        )
        .fetch_optional(self.pool_ref())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid refresh token"))?;

        if row.expires_at < now {
            return Err(anyhow::anyhow!("Refresh token expired"));
        }

        Ok((row.user_id, row.scope))
    }

    pub async fn consume_refresh_token(
        &self,
        token_id: &str,
        client_id: &str,
    ) -> Result<(String, String)> {
        let (user_id, scope) = self.validate_refresh_token(token_id, client_id).await?;

        sqlx::query!(
            "DELETE FROM oauth_refresh_tokens WHERE token_id = $1",
            token_id
        )
        .execute(self.pool_ref())
        .await?;

        Ok((user_id, scope))
    }

    pub async fn revoke_refresh_token(&self, token_id: &str) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM oauth_refresh_tokens WHERE token_id = $1",
            token_id
        )
        .execute(self.pool_ref())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn cleanup_expired_refresh_tokens(&self) -> Result<u64> {
        let now = Utc::now();

        let result = sqlx::query!(
            "DELETE FROM oauth_refresh_tokens WHERE expires_at < $1",
            now
        )
        .execute(self.pool_ref())
        .await?;

        Ok(result.rows_affected())
    }
}

use super::OAuthRepository;
use anyhow::Result;
use base64::Engine;
use chrono::Utc;

impl OAuthRepository {
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
        let expires_at = Utc::now() + chrono::Duration::seconds(600);
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO oauth_auth_codes
             (code, client_id, user_id, redirect_uri, scope, expires_at, code_challenge,
             code_challenge_method, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            code,
            client_id,
            user_id,
            redirect_uri,
            scope,
            expires_at,
            code_challenge,
            code_challenge_method,
            now
        )
        .execute(self.pool_ref())
        .await?;

        Ok(())
    }

    pub async fn get_client_id_from_auth_code(&self, code: &str) -> Result<Option<String>> {
        let result = sqlx::query_scalar!(
            "SELECT client_id FROM oauth_auth_codes WHERE code = $1",
            code
        )
        .fetch_optional(self.pool_ref())
        .await?;

        Ok(result)
    }

    pub async fn validate_authorization_code(
        &self,
        code: &str,
        _client_id: &str,
        redirect_uri: Option<&str>,
        code_verifier: Option<&str>,
    ) -> Result<(String, String)> {
        let now = Utc::now();

        let row = sqlx::query!(
            "SELECT user_id, scope, expires_at, redirect_uri, used_at, code_challenge,
             code_challenge_method
             FROM oauth_auth_codes WHERE code = $1",
            code
        )
        .fetch_optional(self.pool_ref())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Invalid authorization code"))?;

        if row.used_at.is_some() {
            return Err(anyhow::anyhow!("Authorization code already used"));
        }

        if row.expires_at < now {
            return Err(anyhow::anyhow!("Authorization code expired"));
        }

        if let Some(expected_uri) = redirect_uri {
            if row.redirect_uri != expected_uri {
                return Err(anyhow::anyhow!("Redirect URI mismatch"));
            }
        }

        if let Some(ref challenge) = row.code_challenge {
            let verifier =
                code_verifier.ok_or_else(|| anyhow::anyhow!("code_verifier required for PKCE"))?;

            let method = row
                .code_challenge_method
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
                        "PKCE method 'plain' is not allowed. Only 'S256' is supported for \
                         security."
                    ));
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported code_challenge_method: {method}. Only 'S256' is allowed."
                    ))
                },
            };

            if computed_challenge != *challenge {
                return Err(anyhow::anyhow!("PKCE validation failed"));
            }
        }

        sqlx::query!(
            "UPDATE oauth_auth_codes SET used_at = $1 WHERE code = $2",
            now,
            code
        )
        .execute(self.pool_ref())
        .await?;

        Ok((row.user_id, row.scope))
    }
}

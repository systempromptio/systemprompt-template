//! Access-token queries: every issued personal access token with its owner,
//! plus the user options used by the token-management page.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// Raw token row joined to its owner, ordered so a user's tokens are
/// contiguous (active before revoked).
#[derive(Debug, sqlx::FromRow)]
pub struct AccessTokenRowDb {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub user_id: UserId,
    pub user_email: Option<String>,
    pub department: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// One selectable token owner (user id + email + display name).
#[derive(Debug, sqlx::FromRow)]
pub struct TokenUserRow {
    pub uid: String,
    pub email: Option<String>,
    pub display: Option<String>,
}

pub async fn list_access_tokens(pool: &PgPool) -> Result<Vec<AccessTokenRowDb>, sqlx::Error> {
    sqlx::query_as!(
        AccessTokenRowDb,
        r#"
        SELECT
            ak.id AS "id!",
            ak.name AS "name!",
            ak.key_prefix AS "key_prefix!",
            ak.user_id AS "user_id!: UserId",
            u.email::TEXT AS "user_email?",
            NULLIF(upe.department, '') AS "department?",
            ak.last_used_at AS "last_used_at?",
            ak.expires_at AS "expires_at?",
            ak.created_at AS "created_at?",
            ak.revoked_at AS "revoked_at?"
        FROM user_api_keys ak
        LEFT JOIN users u ON u.id = ak.user_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        ORDER BY ak.revoked_at IS NOT NULL,
                 COALESCE(u.email::TEXT, ak.user_id::TEXT),
                 ak.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_token_user_options(pool: &PgPool) -> Result<Vec<TokenUserRow>, sqlx::Error> {
    sqlx::query_as!(
        TokenUserRow,
        r#"
        SELECT u.id::TEXT AS "uid!",
               u.email::TEXT AS "email?",
               COALESCE(NULLIF(u.display_name, ''), NULLIF(u.full_name, ''), NULLIF(u.name, '')) AS "display?"
        FROM users u
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        ORDER BY COALESCE(NULLIF(u.display_name, ''), u.email::TEXT, u.id::TEXT)
        "#,
    )
    .fetch_all(pool)
    .await
}

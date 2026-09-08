//! Per-user settings records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingsRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Why: the write shape, separate from the row. It carries no `user_id` — the
// handler takes that from the validated session, so there is no field a caller
// could set to write somebody else's settings.
#[derive(Debug, Clone, Deserialize)]
pub struct UserSettingsInput {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".to_owned()
}

pub async fn find_user_settings(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<UserSettingsRow>, sqlx::Error> {
    let id = user_id.as_str();
    sqlx::query_as!(
        UserSettingsRow,
        r#"SELECT
             user_id AS "user_id!: UserId",
             display_name,
             avatar_url,
             timezone,
             created_at,
             updated_at
           FROM user_settings WHERE user_id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}

// Why: an upsert rather than an update. The row is created lazily — a person
// who has never saved has no `user_settings` row at all, which is why the page
// renders defaults — so the first save has nothing to update.
pub async fn update_user_settings(
    pool: &PgPool,
    user_id: &UserId,
    settings: &UserSettingsInput,
) -> Result<UserSettingsRow, sqlx::Error> {
    sqlx::query_as!(
        UserSettingsRow,
        r#"INSERT INTO user_settings (user_id, display_name, avatar_url, timezone, updated_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (user_id) DO UPDATE
             SET display_name = EXCLUDED.display_name,
                 avatar_url   = EXCLUDED.avatar_url,
                 timezone     = EXCLUDED.timezone,
                 updated_at   = NOW()
           RETURNING
             user_id AS "user_id!: UserId",
             display_name,
             avatar_url,
             timezone,
             created_at,
             updated_at"#,
        user_id.as_str(),
        settings.display_name.as_deref(),
        settings.avatar_url.as_deref(),
        settings.timezone.as_str(),
    )
    .fetch_one(pool)
    .await
}

// Why: `user_settings.user_id` carries no foreign key to `users`, so deleting
// an account leaves this row behind unless it is removed explicitly.
pub async fn delete_user_settings(pool: &PgPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM user_settings WHERE user_id = $1",
        user_id.as_str()
    )
    .execute(pool)
    .await
    .map(|_| ())
}

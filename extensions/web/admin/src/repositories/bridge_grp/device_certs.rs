use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::error::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct DeviceCertRow {
    pub id: String,
    pub fingerprint: String,
    pub label: String,
    pub enrolled_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

pub async fn list_device_certs_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<DeviceCertRow>> {
    let rows = sqlx::query_as!(
        DeviceCertRow,
        r#"
        SELECT id, fingerprint, label, enrolled_at, revoked_at
        FROM user_device_certs
        WHERE user_id = $1
        ORDER BY enrolled_at DESC
        "#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn revoke_device_cert(pool: &PgPool, user_id: &UserId, id: &str) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE user_device_certs
        SET revoked_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
        "#,
        id,
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

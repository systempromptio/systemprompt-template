//! Device telemetry for the enrolled desktop bridges of one user.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// App-link telemetry for a single user's enrolled devices, keyed by device id.
#[derive(Debug, sqlx::FromRow)]
pub struct DeviceAppLinkRow {
    // Why: no typed-ID equivalent for device id in systemprompt-identifiers
    pub device_id: String,
    pub app_platform: String,
    pub app_version: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

pub async fn list_device_app_links(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<DeviceAppLinkRow>, sqlx::Error> {
    sqlx::query_as!(
        DeviceAppLinkRow,
        r#"SELECT device_id AS "device_id!",
                  app_platform AS "app_platform!",
                  app_version AS "app_version!",
                  last_seen_at
             FROM device_app_links
             WHERE user_id = $1"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

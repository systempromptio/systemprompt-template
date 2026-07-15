//! Device-fleet queries: enrolled API-key devices, their app-link telemetry,
//! and the user options used by the device-management pages.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// Raw device row joined to owner + app-link telemetry, ordered so a user's
/// devices are contiguous (active before revoked).
#[derive(Debug, sqlx::FromRow)]
pub struct DeviceRowDb {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub user_id: UserId,
    pub user_email: Option<String>,
    pub department: Option<String>,
    pub platform: Option<String>,
    pub app_version: Option<String>,
    pub hostname: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub enrolled_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// One selectable device owner (user id + email + display name).
#[derive(Debug, sqlx::FromRow)]
pub struct DeviceUserRow {
    pub uid: String,
    pub email: Option<String>,
    pub display: Option<String>,
}

/// App-link telemetry for a single user's enrolled devices, keyed by device id.
#[derive(Debug, sqlx::FromRow)]
pub struct DeviceAppLinkRow {
    pub device_id: String,
    pub app_platform: String,
    pub app_version: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

/// Load every enrolled device with owner + telemetry for the fleet view.
pub async fn list_devices(pool: &PgPool) -> Result<Vec<DeviceRowDb>, sqlx::Error> {
    sqlx::query_as!(
        DeviceRowDb,
        r#"
        SELECT
            ak.id AS "id!",
            ak.name AS "name!",
            ak.key_prefix AS "key_prefix!",
            ak.user_id AS "user_id!: UserId",
            u.email::TEXT AS "user_email?",
            NULLIF(upe.department, '') AS "department?",
            dal.app_platform AS "platform?",
            NULLIF(dal.app_version, '') AS "app_version?",
            NULLIF(dal.hostname, '') AS "hostname?",
            COALESCE(dal.last_seen_at, ak.last_used_at) AS "last_seen_at?",
            dal.enrolled_at AS "enrolled_at?",
            ak.expires_at AS "expires_at?",
            ak.created_at AS "created_at?",
            ak.revoked_at AS "revoked_at?"
        FROM user_api_keys ak
        LEFT JOIN users u ON u.id = ak.user_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        LEFT JOIN device_app_links dal ON dal.device_id = ak.id
        ORDER BY ak.revoked_at IS NOT NULL,
                 COALESCE(u.email::TEXT, ak.user_id::TEXT),
                 ak.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// List human users eligible as device owners in the assignment dropdown.
pub async fn list_device_user_options(pool: &PgPool) -> Result<Vec<DeviceUserRow>, sqlx::Error> {
    sqlx::query_as!(
        DeviceUserRow,
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

/// Load app-link telemetry for a single user's enrolled devices.
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

//! Device-fleet view model for `/admin/management/devices`.
//!
//! Loads enrolled API-key devices joined to their owners + app-link telemetry,
//! reshapes them into template rows, computes the per-owner rowspans that group
//! a user's devices in the table, and counts "online" (seen <5m ago) devices.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct DeviceRowDb {
    id: String,
    name: String,
    key_prefix: String,
    user_id: String,
    user_email: Option<String>,
    department: Option<String>,
    platform: Option<String>,
    app_version: Option<String>,
    hostname: Option<String>,
    last_seen_at: Option<DateTime<Utc>>,
    enrolled_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub(super) struct DeviceRow {
    id: String,
    name: String,
    key_prefix: String,
    user_id: String,
    user_email: Option<String>,
    department: Option<String>,
    platform: Option<String>,
    app_version: Option<String>,
    hostname: Option<String>,
    last_seen_at: Option<DateTime<Utc>>,
    enrolled_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
    revoked: bool,
    owner_rowspan: u32,
    group_start: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct DeviceUserOption {
    user_id: String,
    label: String,
}

pub(super) async fn load_devices(pool: &PgPool) -> Vec<DeviceRowDb> {
    sqlx::query_as!(
        DeviceRowDb,
        r#"
        SELECT
            ak.id AS "id!",
            ak.name AS "name!",
            ak.key_prefix AS "key_prefix!",
            ak.user_id AS "user_id!",
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
    .inspect_err(|e| tracing::warn!(error = %e, "ssr_management: load devices failed"))
    .unwrap_or_default()
}

pub(super) fn build_device_rows(rows: Vec<DeviceRowDb>) -> (Vec<DeviceRow>, usize) {
    let now = Utc::now();
    let mut devices = Vec::with_capacity(rows.len());
    let mut online = 0usize;
    for r in rows {
        let revoked = r.revoked_at.is_some();
        if !revoked {
            if let Some(ts) = r.last_seen_at {
                if (now - ts).num_minutes() < 5 {
                    online += 1;
                }
            }
        }
        devices.push(DeviceRow {
            id: r.id,
            name: r.name,
            key_prefix: r.key_prefix,
            user_id: r.user_id,
            user_email: r.user_email,
            department: r.department,
            platform: r.platform,
            app_version: r.app_version,
            hostname: r.hostname,
            last_seen_at: r.last_seen_at,
            enrolled_at: r.enrolled_at,
            expires_at: r.expires_at,
            created_at: r.created_at,
            revoked,
            owner_rowspan: 0,
            group_start: false,
        });
    }
    (devices, online)
}

pub(super) async fn load_device_user_options(pool: &PgPool) -> Vec<DeviceUserOption> {
    sqlx::query!(
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
    .inspect_err(|e| tracing::warn!(error = %e, "ssr_management: load device user options failed"))
    .unwrap_or_default()
    .into_iter()
    .map(|r| {
        let label = match (r.display.as_deref(), r.email.as_deref()) {
            (Some(d), Some(e)) => format!("{d} ({e})"),
            (Some(d), None) => d.to_string(),
            (None, Some(e)) => e.to_string(),
            (None, None) => r.uid.clone(),
        };
        DeviceUserOption {
            user_id: r.uid,
            label,
        }
    })
    .collect()
}

fn owner_key(d: &DeviceRow) -> &str {
    d.user_email.as_deref().unwrap_or(d.user_id.as_str())
}

pub(super) fn compute_owner_rowspans(devices: &mut [DeviceRow]) {
    let mut i = 0;
    while i < devices.len() {
        let key = owner_key(&devices[i]).to_owned();
        let mut j = i + 1;
        while j < devices.len() && owner_key(&devices[j]) == key {
            j += 1;
        }
        let span = u32::try_from(j - i).unwrap_or(1);
        devices[i].owner_rowspan = span;
        devices[i].group_start = true;
        i = j;
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ManagementDevicesPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub devices: Vec<DeviceRow>,
    pub total: usize,
    pub online: usize,
    pub user_options: Vec<DeviceUserOption>,
    pub department_options: Vec<String>,
}

//! Device telemetry for the enrolled desktop bridges of one user.

use chrono::{DateTime, Utc};

/// App-link telemetry for a single user's enrolled devices, keyed by device id.
#[derive(Debug, sqlx::FromRow)]
pub struct DeviceAppLinkRow {
    // Why: no typed-ID equivalent for device id in systemprompt-identifiers
    pub device_id: String,
    pub app_platform: String,
    pub app_version: String,
    pub last_seen_at: Option<DateTime<Utc>>,
}

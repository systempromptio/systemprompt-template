//! The fleet totals the page shows before any tab is chosen.
//!
//! One query per table rather than one query with four subselects: the four
//! numbers are independent, none of them constrains another, and a single
//! statement joining unrelated tables would only be harder to read for the
//! same four scans.
//!
//! None of these tiles carries a period-on-period delta, and none can. A
//! bridge session keeps only its latest heartbeat, so the table records what
//! is alive now and nothing about what was alive a week ago; the obvious
//! query — heartbeats falling inside the previous window — counts the bridges
//! that went quiet during it, not the bridges that were running. There is no
//! history table behind this one to join to, so a delta here would be a number
//! that looks comparable and is not.

use sqlx::PgPool;

use super::STALE_AFTER_DAYS;

#[derive(Debug, Clone, Copy, Default)]
pub struct FleetStats {
    pub bridges_total: i64,
    pub bridges_active: i64,
    pub bridges_stale: i64,
    pub versions: i64,
    pub pats_active: i64,
    pub pats_total: i64,
    pub certs_active: i64,
    pub certs_total: i64,
    pub links_pending: i64,
    pub links_expired: i64,
}

pub async fn get_fleet_stats(pool: &PgPool) -> Result<FleetStats, sqlx::Error> {
    let days = i32::try_from(STALE_AFTER_DAYS).unwrap_or(7);
    let bridges = sqlx::query!(
        r#"SELECT COUNT(*) AS "total!",
                  COUNT(*) FILTER (
                      WHERE last_heartbeat_at >= NOW() - make_interval(days => $1)
                  ) AS "active!",
                  COUNT(DISTINCT bridge_version) AS "versions!"
             FROM bridge_sessions"#,
        days,
    )
    .fetch_one(pool)
    .await?;

    let creds = sqlx::query!(
        r#"SELECT
             (SELECT COUNT(*) FROM user_api_keys) AS "pats_total!",
             (SELECT COUNT(*) FROM user_api_keys WHERE revoked_at IS NULL)
                 AS "pats_active!",
             (SELECT COUNT(*) FROM user_device_certs) AS "certs_total!",
             (SELECT COUNT(*) FROM user_device_certs WHERE revoked_at IS NULL)
                 AS "certs_active!",
             (SELECT COUNT(*) FROM bridge_exchange_codes WHERE consumed_at IS NULL)
                 AS "links_pending!",
             (SELECT COUNT(*) FROM bridge_exchange_codes
               WHERE consumed_at IS NULL AND expires_at < NOW())
                 AS "links_expired!""#
    )
    .fetch_one(pool)
    .await?;

    Ok(FleetStats {
        bridges_total: bridges.total,
        bridges_active: bridges.active,
        bridges_stale: bridges.total - bridges.active,
        versions: bridges.versions,
        pats_active: creds.pats_active,
        pats_total: creds.pats_total,
        certs_active: creds.certs_active,
        certs_total: creds.certs_total,
        links_pending: creds.links_pending,
        links_expired: creds.links_expired,
    })
}

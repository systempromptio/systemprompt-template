//! Bridge sessions, read as people and the machines they run.
//!
//! A session is never deleted, only left to go quiet, and every restart of a
//! desktop client writes a new one. Listed raw, the table is a restart log:
//! one person six times, one laptop three times. So the fleet is read in two
//! steps — a page of people with their totals, then one row per machine for
//! the people on that page, each machine represented by its most recent
//! session and carrying the count of the sessions folded beneath it.
//!
//! Staleness is evaluated in SQL and returned on the row, so the filter, the
//! count and the badge are one expression rather than three kept in step. The
//! sort key is a bound parameter rather than an interpolated column name: it
//! arrives from the query string, and a static statement that cannot be
//! reshaped by its own arguments is the only form in which that is safe.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::STALE_AFTER_DAYS;

#[derive(Debug, Clone)]
pub struct BridgeUserSessionsRow {
    pub user_id: UserId,
    pub user_name: String,
    pub hosts: i64,
    pub sessions: i64,
    pub latest_version: String,
    pub first_started_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
    pub forwarded_total: i64,
    pub tokens_total: i64,
    pub any_active: bool,
}

#[derive(Debug, Clone)]
pub struct BridgeHostRow {
    pub user_id: UserId,
    pub hostname: String,
    pub os: String,
    pub bridge_version: String,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
    pub forwarded_total: i64,
    pub tokens_total: i64,
    pub session_count: i64,
    pub is_stale: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionQuery<'a> {
    pub stale_only: bool,
    pub sort: &'a str,
    pub dir: &'a str,
    pub limit: i64,
    pub offset: i64,
}

fn stale_days() -> i32 {
    i32::try_from(STALE_AFTER_DAYS).unwrap_or(7)
}

pub async fn list_bridge_users_paged(
    pool: &PgPool,
    query: SessionQuery<'_>,
) -> Result<(Vec<BridgeUserSessionsRow>, i64), sqlx::Error> {
    let days = stale_days();
    let rows = sqlx::query_as!(
        BridgeUserSessionsRow,
        r#"SELECT b.user_id AS "user_id!: UserId", u.name AS "user_name!",
                  COUNT(DISTINCT b.hostname) AS "hosts!",
                  COUNT(*) AS "sessions!",
                  MAX(b.bridge_version) AS "latest_version!",
                  MIN(b.started_at) AS "first_started_at!",
                  MAX(b.last_heartbeat_at) AS "last_heartbeat_at!",
                  SUM(b.forwarded_total)::BIGINT AS "forwarded_total!",
                  SUM(b.tokens_in_total + b.tokens_out_total)::BIGINT AS "tokens_total!",
                  BOOL_OR(b.last_heartbeat_at >= NOW() - make_interval(days => $1))
                      AS "any_active!"
             FROM bridge_sessions b
             JOIN users u ON u.id = b.user_id
            WHERE NOT $2::BOOLEAN
               OR b.last_heartbeat_at < NOW() - make_interval(days => $1)
            GROUP BY b.user_id, u.name
            ORDER BY
              CASE WHEN $4::TEXT = 'desc' THEN
                CASE $3::TEXT WHEN 'started' THEN EXTRACT(EPOCH FROM MIN(b.started_at))
                        WHEN 'forwarded' THEN SUM(b.forwarded_total)::FLOAT8
                        WHEN 'tokens' THEN
                             SUM(b.tokens_in_total + b.tokens_out_total)::FLOAT8
                        ELSE EXTRACT(EPOCH FROM MAX(b.last_heartbeat_at)) END
              END DESC NULLS LAST,
              CASE WHEN $4::TEXT = 'asc' THEN
                CASE $3::TEXT WHEN 'started' THEN EXTRACT(EPOCH FROM MIN(b.started_at))
                        WHEN 'forwarded' THEN SUM(b.forwarded_total)::FLOAT8
                        WHEN 'tokens' THEN
                             SUM(b.tokens_in_total + b.tokens_out_total)::FLOAT8
                        ELSE EXTRACT(EPOCH FROM MAX(b.last_heartbeat_at)) END
              END ASC NULLS LAST,
              CASE WHEN $3::TEXT = 'version' AND $4::TEXT = 'desc'
                   THEN MAX(b.bridge_version) END DESC,
              CASE WHEN $3::TEXT = 'version' AND $4::TEXT = 'asc'
                   THEN MAX(b.bridge_version) END ASC,
              b.user_id
            LIMIT $5 OFFSET $6"#,
        days,
        query.stale_only,
        query.sort,
        query.dir,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT b.user_id) AS "total!"
             FROM bridge_sessions b
            WHERE NOT $2::BOOLEAN
               OR b.last_heartbeat_at < NOW() - make_interval(days => $1)"#,
        days,
        query.stale_only,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

// Why: one row per machine, not per session. The machine's identity is the
// most recent session's (version, OS, heartbeat), because that is the state
// the bridge is in now; the totals are summed over every session, because a
// restart does not un-forward the requests that went before it.
pub async fn list_bridge_hosts_for_users(
    pool: &PgPool,
    user_ids: &[String],
    stale_only: bool,
) -> Result<Vec<BridgeHostRow>, sqlx::Error> {
    let days = stale_days();
    sqlx::query_as!(
        BridgeHostRow,
        r#"SELECT h.user_id AS "user_id!: UserId", h.hostname AS "hostname!",
                  l.os AS "os!", l.bridge_version AS "bridge_version!",
                  l.started_at AS "started_at!",
                  l.last_heartbeat_at AS "last_heartbeat_at!",
                  h.forwarded_total AS "forwarded_total!",
                  h.tokens_total AS "tokens_total!",
                  h.session_count AS "session_count!",
                  (l.last_heartbeat_at < NOW() - make_interval(days => $1))
                      AS "is_stale!"
             FROM (SELECT user_id, hostname,
                          SUM(forwarded_total)::BIGINT AS forwarded_total,
                          SUM(tokens_in_total + tokens_out_total)::BIGINT AS tokens_total,
                          COUNT(*) AS session_count
                     FROM bridge_sessions
                    WHERE user_id = ANY($2::TEXT[])
                      AND (NOT $3::BOOLEAN
                           OR last_heartbeat_at < NOW() - make_interval(days => $1))
                    GROUP BY user_id, hostname) h
             JOIN LATERAL (SELECT b.os, b.bridge_version, b.started_at, b.last_heartbeat_at
                             FROM bridge_sessions b
                            WHERE b.user_id = h.user_id AND b.hostname = h.hostname
                            ORDER BY b.last_heartbeat_at DESC
                            LIMIT 1) l ON TRUE
            ORDER BY h.user_id, l.last_heartbeat_at DESC, h.hostname"#,
        days,
        user_ids,
        stale_only,
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone)]
pub struct VersionCount {
    pub bridge_version: String,
    pub devices: i64,
}

// Why: the histogram counts machines, not sessions, for the same reason the
// table does — a laptop restarted three times is one machine to upgrade. It
// is capped because it is a shape, not an inventory: a fleet mid-rollout has
// two or three versions that matter and a long tail nobody has touched.
pub async fn list_version_counts(pool: &PgPool) -> Result<Vec<VersionCount>, sqlx::Error> {
    sqlx::query_as!(
        VersionCount,
        r#"SELECT bridge_version AS "bridge_version!", COUNT(*) AS "devices!"
             FROM (SELECT DISTINCT ON (user_id, hostname) bridge_version
                     FROM bridge_sessions
                    ORDER BY user_id, hostname, last_heartbeat_at DESC) latest
            GROUP BY bridge_version
            ORDER BY COUNT(*) DESC, bridge_version DESC
            LIMIT 12"#
    )
    .fetch_all(pool)
    .await
}

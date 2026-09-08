//! The paged secret-access trail from `secret_audit_log`.
//!
//! Plaintext never reaches the database, so this table is the whole record of
//! what happened to a credential: who created it, who read it, who rotated it.
//! `user_id` is the secret's owner and `actor_id` the person who acted, and
//! they differ exactly when an administrator touched someone else's credential
//! — which is the row an auditor is here to find.
//!
//! `repositories::secrets::secret_audit` owns the per-plugin trail the profile
//! page shows. This is the console-wide view, so it lives beside the other
//! governance readers rather than inside the crypto module.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::util::time_range::TimeRange;

/// One recorded action against one secret.
#[derive(Debug, Clone)]
pub struct SecretAuditRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub action: String,
    pub var_name: String,
    pub plugin_id: String,
    pub user_id: UserId,
    pub actor_id: UserId,
    pub ip_address: Option<String>,
}

/// The audit KPI strip.
#[derive(Debug, Clone, Copy, Default)]
pub struct SecretAuditStats {
    pub entries: i64,
    pub variables: i64,
    pub actors: i64,
    pub accesses: i64,
    pub rotations: i64,
    pub third_party: i64,
}

/// What the toolbar narrowed the trail to.
#[derive(Debug, Clone, Default)]
pub struct SecretAuditFilter {
    pub action: Option<String>,
    pub search: Option<String>,
}

pub async fn list_secret_audit_paged(
    pool: &PgPool,
    range: TimeRange,
    filter: &SecretAuditFilter,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SecretAuditRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        SecretAuditRow,
        r#"SELECT s.id, s.created_at, s.action, s.var_name, s.plugin_id,
                  s.user_id AS "user_id!: UserId", s.actor_id AS "actor_id!: UserId",
                  NULLIF(s.ip_address, '') AS ip_address
           FROM secret_audit_log s
           WHERE s.created_at >= $1 AND s.created_at < $2
             AND ($3::TEXT IS NULL OR s.action = $3)
             AND ($4::TEXT IS NULL
                  OR s.var_name ILIKE '%' || $4 || '%'
                  OR s.plugin_id ILIKE '%' || $4 || '%'
                  OR s.user_id ILIKE '%' || $4 || '%'
                  OR s.actor_id ILIKE '%' || $4 || '%')
           ORDER BY s.created_at DESC, s.id
           LIMIT $5 OFFSET $6"#,
        range.from,
        range.to,
        filter.action.as_deref(),
        filter.search.as_deref(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "n!"
           FROM secret_audit_log s
           WHERE s.created_at >= $1 AND s.created_at < $2
             AND ($3::TEXT IS NULL OR s.action = $3)
             AND ($4::TEXT IS NULL
                  OR s.var_name ILIKE '%' || $4 || '%'
                  OR s.plugin_id ILIKE '%' || $4 || '%'
                  OR s.user_id ILIKE '%' || $4 || '%'
                  OR s.actor_id ILIKE '%' || $4 || '%')"#,
        range.from,
        range.to,
        filter.action.as_deref(),
        filter.search.as_deref(),
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

// Why: `third_party` counts the rows where the actor is not the owner. It is
// the one number on the page that cannot be read off the table at a glance and
// the one an auditor asks for first.
pub async fn get_secret_audit_stats(
    pool: &PgPool,
    range: TimeRange,
) -> Result<SecretAuditStats, sqlx::Error> {
    sqlx::query_as!(
        SecretAuditStats,
        r#"SELECT
             COUNT(*)::BIGINT AS "entries!",
             COUNT(DISTINCT var_name)::BIGINT AS "variables!",
             COUNT(DISTINCT actor_id)::BIGINT AS "actors!",
             COUNT(*) FILTER (WHERE action = 'accessed')::BIGINT AS "accesses!",
             COUNT(*) FILTER (WHERE action = 'rotated')::BIGINT AS "rotations!",
             COUNT(*) FILTER (WHERE actor_id <> user_id)::BIGINT AS "third_party!"
           FROM secret_audit_log
           WHERE created_at >= $1 AND created_at < $2"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await
}

// Why: as with the categories above, the select lists the actions this
// window recorded rather than every action the column may hold.
pub async fn list_secret_audit_actions(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT action AS "action!"
           FROM secret_audit_log
           WHERE created_at >= $1 AND created_at < $2
           GROUP BY action
           ORDER BY COUNT(*) DESC, action"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await
}

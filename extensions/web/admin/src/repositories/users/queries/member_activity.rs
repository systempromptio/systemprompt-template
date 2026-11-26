//! The optional members' share of a user's activity on the user pages.
//!
//! The base listing and detail reads cover what every build installs: plugin
//! events, console activity and the bytes rollup. The gateway member's
//! requests and the MCP member's tool executions are tables only the builds
//! that link those members install, so their latest timestamp (and the
//! gateway's request count) are read here, keyed by the user ids the base read
//! returned, and folded into the same rows.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::types::UserSummary;

pub(super) async fn merge_member_activity(
    pool: &PgPool,
    rows: &mut [UserSummary],
) -> Result<(), sqlx::Error> {
    let ids: Vec<String> = rows.iter().map(|r| r.user_id.as_str().to_owned()).collect();
    #[cfg(feature = "gateway")]
    {
        let requests = latest_requests(pool, &ids).await?;
        for row in rows.iter_mut() {
            if let Some((last, count)) = requests.get(row.user_id.as_str()) {
                row.last_active = row.last_active.max(Some(*last));
                row.total_events += count;
            }
        }
    }
    #[cfg(feature = "mcp")]
    {
        let executions = latest_executions(pool, &ids).await?;
        for row in rows.iter_mut() {
            if let Some(last) = executions.get(row.user_id.as_str()) {
                row.last_active = row.last_active.max(Some(*last));
            }
        }
    }
    Ok(())
}

#[cfg(feature = "gateway")]
async fn latest_requests(
    pool: &PgPool,
    ids: &[String],
) -> Result<HashMap<String, (DateTime<Utc>, i64)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT user_id AS "user_id!", MAX(created_at) AS "last_request!",
                  COUNT(*)::BIGINT AS "request_count!"
           FROM ai_requests
           WHERE user_id = ANY($1)
           GROUP BY user_id"#,
        ids,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.user_id, (r.last_request, r.request_count)))
        .collect())
}

#[cfg(feature = "mcp")]
async fn latest_executions(
    pool: &PgPool,
    ids: &[String],
) -> Result<HashMap<String, DateTime<Utc>>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT user_id AS "user_id!", MAX(created_at) AS "last_mcp!"
           FROM mcp_tool_executions
           WHERE user_id = ANY($1)
           GROUP BY user_id"#,
        ids,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| (r.user_id, r.last_mcp)).collect())
}

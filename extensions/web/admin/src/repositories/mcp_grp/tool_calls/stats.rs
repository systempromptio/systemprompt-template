//! Per-tool aggregates for the per-tool drill page: summary stats, the top
//! deny reasons, and the top actors (users or agents) by deny volume.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

/// Per-tool aggregates used by the per-tool drill page.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ToolDetailStats {
    pub tool_name: String,
    pub total_calls: i64,
    pub allow_count: i64,
    pub deny_count: i64,
    pub distinct_users: i64,
    pub distinct_agents: i64,
    pub total_bytes_in: i64,
    pub total_bytes_out: i64,
}

pub async fn fetch_tool_detail_stats(
    pool: &PgPool,
    tool_name: &str,
    range: TimeRange,
) -> Result<ToolDetailStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH calls AS (
            SELECT e.id, e.user_id, e.session_id,
                   COALESCE(e.content_input_bytes, 0)::bigint AS content_input_bytes,
                   COALESCE(e.content_output_bytes, 0)::bigint AS content_output_bytes
            FROM plugin_usage_events e
            WHERE e.created_at >= $1 AND e.created_at < $2
              AND e.event_type ILIKE '%ToolUse%'
              AND e.tool_name = $3
        ),
        verdicts AS (
            SELECT DISTINCT c.id,
                   COALESCE((
                     SELECT g.decision FROM governance_decisions g
                     WHERE g.session_id = c.session_id AND g.tool_name = $3
                     ORDER BY g.created_at DESC LIMIT 1
                   ), 'unknown') AS decision,
                   (SELECT g.agent_id FROM governance_decisions g
                    WHERE g.session_id = c.session_id AND g.tool_name = $3
                    ORDER BY g.created_at DESC LIMIT 1) AS agent_id
            FROM calls c
        )
        SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE v.decision = 'allow')::bigint AS "allow!",
            COUNT(*) FILTER (WHERE v.decision = 'deny')::bigint AS "deny!",
            COUNT(DISTINCT c.user_id)::bigint AS "distinct_users!",
            COUNT(DISTINCT v.agent_id) FILTER (WHERE v.agent_id IS NOT NULL)::bigint
                AS "distinct_agents!",
            COALESCE(SUM(c.content_input_bytes), 0)::bigint AS "bytes_in!",
            COALESCE(SUM(c.content_output_bytes), 0)::bigint AS "bytes_out!"
        FROM calls c
        LEFT JOIN verdicts v ON v.id = c.id"#,
        range.from,
        range.to,
        tool_name,
    )
    .fetch_one(pool)
    .await?;

    Ok(ToolDetailStats {
        tool_name: tool_name.to_owned(),
        total_calls: row.total,
        allow_count: row.allow,
        deny_count: row.deny,
        distinct_users: row.distinct_users,
        distinct_agents: row.distinct_agents,
        total_bytes_in: row.bytes_in,
        total_bytes_out: row.bytes_out,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolDenyReason {
    pub reason: String,
    pub policy: String,
    pub count: i64,
}

pub async fn fetch_tool_deny_reasons(
    pool: &PgPool,
    tool_name: &str,
    range: TimeRange,
    limit: i64,
) -> Result<Vec<ToolDenyReason>, sqlx::Error> {
    sqlx::query_as!(
        ToolDenyReason,
        r#"SELECT
            reason as "reason!",
            policy as "policy!",
            COUNT(*)::bigint AS "count!"
          FROM governance_decisions
          WHERE created_at >= $1 AND created_at < $2
            AND tool_name = $3
            AND decision = 'deny'
          GROUP BY reason, policy
          ORDER BY COUNT(*) DESC
          LIMIT $4"#,
        range.from,
        range.to,
        tool_name,
        limit,
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolTopActor {
    pub identity_id: String,
    pub label: String,
    pub deny_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum ToolActorGroup {
    User,
    Agent,
}

pub async fn fetch_tool_top_actors(
    pool: &PgPool,
    tool_name: &str,
    range: TimeRange,
    group: ToolActorGroup,
    limit: i64,
) -> Result<Vec<ToolTopActor>, sqlx::Error> {
    match group {
        ToolActorGroup::User => {
            sqlx::query_as!(
                ToolTopActor,
                r#"SELECT
                g.user_id as "identity_id!",
                COALESCE(u.display_name, u.full_name, u.name, u.email, g.user_id)
                    as "label!",
                COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint as "deny_count!",
                COUNT(*)::bigint as "total_count!"
              FROM governance_decisions g
              LEFT JOIN users u ON u.id = g.user_id
              WHERE g.created_at >= $1 AND g.created_at < $2
                AND g.tool_name = $3
              GROUP BY g.user_id, u.display_name, u.full_name, u.name, u.email
              ORDER BY COUNT(*) FILTER (WHERE g.decision = 'deny') DESC, COUNT(*) DESC
              LIMIT $4"#,
                range.from,
                range.to,
                tool_name,
                limit,
            )
            .fetch_all(pool)
            .await
        },
        ToolActorGroup::Agent => {
            sqlx::query_as!(
                ToolTopActor,
                r#"SELECT
                COALESCE(g.agent_id, '')   as "identity_id!",
                COALESCE(g.agent_id, '—')  as "label!",
                COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint as "deny_count!",
                COUNT(*)::bigint as "total_count!"
              FROM governance_decisions g
              WHERE g.created_at >= $1 AND g.created_at < $2
                AND g.tool_name = $3
                AND g.agent_id IS NOT NULL
              GROUP BY g.agent_id
              ORDER BY COUNT(*) FILTER (WHERE g.decision = 'deny') DESC, COUNT(*) DESC
              LIMIT $4"#,
                range.from,
                range.to,
                tool_name,
                limit,
            )
            .fetch_all(pool)
            .await
        },
    }
}

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

#[derive(Debug)]
pub struct TraceSessionRow {
    pub session_id: SessionId,
    pub event_count: i64,
    pub tool_uses: i64,
    pub errors: i64,
    pub first_at: DateTime<Utc>,
    pub last_at: DateTime<Utc>,
}

pub async fn list_recent_traces(pool: &PgPool) -> Result<Vec<TraceSessionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            session_id AS "session_id!: SessionId",
            COUNT(*)::bigint AS "event_count!",
            COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')::bigint AS "tool_uses!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!",
            MIN(created_at) AS "first_at!",
            MAX(created_at) AS "last_at!"
        FROM plugin_usage_events
        WHERE session_id IS NOT NULL
          AND created_at >= NOW() - INTERVAL '7 days'
        GROUP BY session_id
        ORDER BY MAX(created_at) DESC
        LIMIT 50"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| TraceSessionRow {
            session_id: r.session_id,
            event_count: r.event_count,
            tool_uses: r.tool_uses,
            errors: r.errors,
            first_at: r.first_at,
            last_at: r.last_at,
        })
        .collect())
}

#[derive(Debug)]
pub struct BenchRunRow {
    pub session_id: SessionId,
    pub total_decisions: i64,
    pub allowed: i64,
    pub denied: i64,
    pub duration_seconds: f64,
    pub first_at: DateTime<Utc>,
}

pub async fn list_bench_runs(pool: &PgPool) -> Result<Vec<BenchRunRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            session_id AS "session_id!: SessionId",
            COUNT(*)::bigint AS "total_decisions!",
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            GREATEST(
                EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))),
                0.001
            )::float8 AS "duration_seconds!",
            MIN(created_at) AS "first_at!"
        FROM governance_decisions
        WHERE session_id LIKE 'bench-%'
          OR session_id LIKE '%-govern'
          OR session_id LIKE '%-track'
        GROUP BY session_id
        HAVING COUNT(*) >= 10
        ORDER BY MIN(created_at) DESC
        LIMIT 10"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| BenchRunRow {
            session_id: r.session_id,
            total_decisions: r.total_decisions,
            allowed: r.allowed,
            denied: r.denied,
            duration_seconds: r.duration_seconds,
            first_at: r.first_at,
        })
        .collect())
}

#[derive(Debug, Clone, Copy)]
pub struct HourThroughputRow {
    pub bucket: DateTime<Utc>,
    pub decisions: i64,
}

pub async fn list_hourly_throughput(pool: &PgPool) -> Result<Vec<HourThroughputRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            date_trunc('hour', created_at) AS "bucket!",
            COUNT(*)::bigint AS "decisions!"
        FROM governance_decisions
        WHERE created_at >= NOW() - INTERVAL '24 hours'
        GROUP BY date_trunc('hour', created_at)
        ORDER BY date_trunc('hour', created_at) ASC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| HourThroughputRow {
            bucket: r.bucket,
            decisions: r.decisions,
        })
        .collect())
}

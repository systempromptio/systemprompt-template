//! Traffic-distribution read models for the Evals page.
//!
//! Answers "what has actually gone through the gateway": which models, which
//! users, what kinds of prompt, and what each cost. The KPI strip, latency
//! histogram, and cost time series are *not* duplicated here — the Evals page
//! reuses [`crate::repositories::analytics::request_stats`] for those.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::util::time_range::TimeRange;

/// One model's share of traffic over the window.
#[derive(Debug, Clone, Serialize)]
pub struct EvalModelDistributionRow {
    pub provider: String,
    pub model: String,
    pub request_count: i64,
    pub user_count: i64,
    pub error_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_microdollars: i64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
}

pub async fn list_eval_model_distribution(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<EvalModelDistributionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            provider AS "provider!",
            model AS "model!",
            COUNT(*)::bigint AS "request_count!",
            COUNT(DISTINCT user_id)::bigint AS "user_count!",
            COUNT(*) FILTER (WHERE status NOT IN ('completed', 'pending', 'streaming'))::bigint
                AS "error_count!",
            COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
            COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!",
            COALESCE(percentile_cont(0.50) WITHIN GROUP (ORDER BY latency_ms), 0)::float8 AS "p50!",
            COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms), 0)::float8 AS "p95!"
          FROM ai_requests
          WHERE created_at >= $1 AND created_at < $2
          GROUP BY provider, model
          ORDER BY COUNT(*) DESC"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| EvalModelDistributionRow {
            provider: r.provider,
            model: r.model,
            request_count: r.request_count,
            user_count: r.user_count,
            error_count: r.error_count,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost_microdollars: r.cost_microdollars,
            p50_latency_ms: r.p50,
            p95_latency_ms: r.p95,
        })
        .collect())
}

/// One user's share of traffic over the window.
#[derive(Debug, Clone, Serialize)]
pub struct UserDistributionRow {
    pub user_id: UserId,
    pub user_label: Option<String>,
    pub request_count: i64,
    pub session_count: i64,
    pub model_count: i64,
    pub cost_microdollars: i64,
    pub error_count: i64,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

pub async fn list_user_distribution(
    pool: &PgPool,
    range: TimeRange,
    limit: i64,
) -> Result<Vec<UserDistributionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            ar.user_id AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email) AS user_label,
            COUNT(*)::bigint AS "request_count!",
            COUNT(DISTINCT ar.session_id)::bigint AS "session_count!",
            COUNT(DISTINCT ar.model)::bigint AS "model_count!",
            COALESCE(SUM(ar.cost_microdollars), 0)::bigint AS "cost_microdollars!",
            COUNT(*) FILTER (WHERE ar.status NOT IN ('completed', 'pending', 'streaming'))::bigint
                AS "error_count!",
            MAX(ar.created_at) AS "last_seen!"
          FROM ai_requests ar
          LEFT JOIN users u ON u.id = ar.user_id
          WHERE ar.created_at >= $1 AND ar.created_at < $2
          GROUP BY ar.user_id, u.display_name, u.full_name, u.name, u.email
          ORDER BY COUNT(*) DESC
          LIMIT $3"#,
        range.from,
        range.to,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UserDistributionRow {
            user_id: r.user_id,
            user_label: r.user_label,
            request_count: r.request_count,
            session_count: r.session_count,
            model_count: r.model_count,
            cost_microdollars: r.cost_microdollars,
            error_count: r.error_count,
            last_seen: r.last_seen,
        })
        .collect())
}

/// A recurring prompt shape, keyed on the opening words of the stored request
/// excerpt. Crude on purpose: it is a distribution hint, not a classifier, and
/// it costs nothing to compute.
#[derive(Debug, Clone, Serialize)]
pub struct PromptTopicRow {
    pub topic: String,
    pub request_count: i64,
    pub distinct_models: i64,
    pub cost_microdollars: i64,
    pub sample_excerpt: String,
}

pub async fn list_prompt_topics(
    pool: &PgPool,
    range: TimeRange,
    limit: i64,
) -> Result<Vec<PromptTopicRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH excerpts AS (
            SELECT
                ar.model,
                ar.cost_microdollars,
                p.request_excerpt,
                LOWER(
                    regexp_replace(
                        LEFT(COALESCE(NULLIF(p.request_excerpt, ''), '(no prompt recorded)'), 48),
                        '\s+', ' ', 'g'
                    )
                ) AS topic
            FROM ai_requests ar
            JOIN ai_request_payloads p ON p.ai_request_id = ar.id
            WHERE ar.created_at >= $1 AND ar.created_at < $2
        )
        SELECT
            topic AS "topic!",
            COUNT(*)::bigint AS "request_count!",
            COUNT(DISTINCT model)::bigint AS "distinct_models!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!",
            COALESCE(MIN(request_excerpt), '') AS "sample_excerpt!"
        FROM excerpts
        GROUP BY topic
        ORDER BY COUNT(*) DESC, topic
        LIMIT $3"#,
        range.from,
        range.to,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PromptTopicRow {
            topic: r.topic,
            request_count: r.request_count,
            distinct_models: r.distinct_models,
            cost_microdollars: r.cost_microdollars,
            sample_excerpt: r.sample_excerpt,
        })
        .collect())
}

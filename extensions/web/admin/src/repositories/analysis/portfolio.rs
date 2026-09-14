//! Adapter from Systemprompt's authenticated hook evidence to core resource metrics.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::analytics::resource_metrics::ResourceFact;
use systemprompt::identifiers::{AiRequestId, ResourceInvocationId, SessionId, UserId};

#[derive(Debug)]
pub struct PortfolioFact {
    pub skill: String,
    pub plugin: String,
    pub fact: ResourceFact,
}

pub async fn list_portfolio_facts(
    pool: &PgPool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<PortfolioFact>, sqlx::Error> {
    let rows = sqlx::query!(r#"SELECT e.invocation_id AS "invocation_id!",e.user_id AS "user_id!",e.session_id AS "session_id!",e.skill AS "skill!",e.plugin_id AS "plugin_id!",e.invoked_at AS "invoked_at!",e.attribution_status AS "attribution_status!",
        r.id AS "request_id?",r.input_tokens::BIGINT AS "input_tokens?",r.output_tokens::BIGINT AS "output_tokens?",
        r.cache_read_tokens::BIGINT AS "cache_read_tokens?",r.cache_creation_tokens::BIGINT AS "cache_creation_tokens?",
        NULLIF(r.cost_microdollars,0) AS "cost_microdollars?",r.latency_ms::BIGINT AS "latency_ms?",r.status AS "status?",
        a.quality_score::FLOAT8 AS "quality_score?",a.outcome AS "outcome?"
        FROM analysis_skill_version_events e
        LEFT JOIN ai_requests r ON r.user_id=e.user_id AND r.client_session_id=e.session_id
            AND r.created_at >= $1 AND r.created_at < $2 AND NOT r.synthetic
            AND NOT EXISTS(SELECT 1 FROM eval_request_reservations er WHERE er.request_id=r.id)
            AND NOT EXISTS(SELECT 1 FROM eval_session_bindings b WHERE b.session_id=r.session_id)
        LEFT JOIN session_analyses a ON a.user_id=e.user_id AND a.session_id=e.session_id
        WHERE e.invoked_at >= $1 AND e.invoked_at < $2 AND e.traffic_class='production'
          AND NOT EXISTS(SELECT 1 FROM eval_session_bindings b WHERE b.session_id=e.session_id)
        ORDER BY e.invoked_at DESC,e.invocation_id,r.id LIMIT 100001"#, start, end).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| PortfolioFact {
            skill: row.skill,
            plugin: row.plugin_id,
            fact: ResourceFact {
                invocation_id: ResourceInvocationId::new(row.invocation_id),
                user_id: UserId::new(row.user_id),
                session_id: SessionId::new(row.session_id),
                invoked_at: row.invoked_at,
                request_id: row.request_id.map(AiRequestId::new),
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cache_read_tokens: row.cache_read_tokens,
                cache_creation_tokens: row.cache_creation_tokens,
                cost_microdollars: row.cost_microdollars,
                latency_ms: row.latency_ms,
                failed: row.status.as_deref() == Some("failed"),
                quality_score: row.quality_score,
                successful: row
                    .outcome
                    .map(|value| matches!(value.as_str(), "success" | "successful" | "completed")),
                revision_verified: row.attribution_status == "verified",
            },
        })
        .collect())
}

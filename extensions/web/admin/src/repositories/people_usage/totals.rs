//! Every container's totals in one pass, with the traffic that belongs to no
//! container reported rather than dropped.
//!
//! Under exclusive attribution these rows partition the instance: each person
//! counts in one container, and the `unattributed` row carries what is left —
//! a request whose user matches no account, a non-user actor such as a service
//! token, and anyone no primary container covers. Summing the rows therefore
//! reproduces the instance total, which is the property that makes the split
//! worth reading.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::scope::membership::UNATTRIBUTED;
use crate::repositories::scope::{Attribution, ScopeKind};

/// One container's slice of the window, or the unattributed remainder.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeTotalRow {
    pub scope_id: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

pub async fn list_scope_totals(
    pool: &PgPool,
    kind: ScopeKind,
    attribution: Attribution,
    window_days: i32,
) -> Result<Vec<ScopeTotalRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT COALESCE(m.scope_id, $3) AS "scope_id!",
                  COUNT(*)::BIGINT AS "requests!",
                  COALESCE(SUM(COALESCE(r.input_tokens, 0) + COALESCE(r.output_tokens, 0)), 0)::BIGINT AS "tokens!",
                  COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!"
           FROM ai_requests r
           LEFT JOIN membership m
             ON m.user_id = r.user_id AND r.actor_kind = 'user'
           WHERE r.created_at >= NOW() - make_interval(days => $4)
           GROUP BY 1
           ORDER BY 2 DESC, 1"#,
        kind.as_str(),
        attribution.is_exclusive(),
        UNATTRIBUTED,
        window_days
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ScopeTotalRow {
            scope_id: row.scope_id,
            requests: row.requests,
            tokens: row.tokens,
            cost_microdollars: row.cost_microdollars,
        })
        .collect())
}

//! Top containers by spend, under exclusive attribution.
//!
//! Exclusive is not a default that happened: it is what makes these two tables
//! readable side by side. Each person's traffic lands in exactly one group and
//! one project, so the rows partition the instance and the leaderboard cannot
//! show a total larger than the KPI strip above it.
//!
//! Traffic that matches no container is reported as its own row rather than
//! dropped — a rejected request with no user, a job or MCP actor, or a person
//! no primary container covers. Dropping it would make the leaderboard quietly
//! disagree with the cost KPI.

use sqlx::PgPool;

use crate::repositories::scope::membership::UNATTRIBUTED;
use crate::repositories::scope::{Attribution, ScopeKind};
use crate::util::time_range::TimeRange;

/// One container's slice of the window, or the unattributed remainder.
#[derive(Debug, Clone)]
pub struct ScopeCostRow {
    pub scope_id: String,
    pub label: String,
    pub requests: i64,
    pub cost_microdollars: i64,
}

impl ScopeCostRow {
    #[must_use]
    pub fn is_unattributed(&self) -> bool {
        self.scope_id == UNATTRIBUTED
    }
}

pub async fn list_top_scopes_by_cost(
    pool: &PgPool,
    kind: ScopeKind,
    range: TimeRange,
    limit: i64,
) -> Result<Vec<ScopeCostRow>, sqlx::Error> {
    // Why: the flag is derived outside the statement so the attribution mode
    // is named once, in code, rather than buried in a macro argument list.
    let exclusive = Attribution::Exclusive.is_exclusive();
    let rows = crate::scoped_query!(
        r#"SELECT COALESCE(m.scope_id, $3) AS "scope_id!",
                  COALESCE(g.name, p.name, m.scope_id, $3) AS "label!",
                  COUNT(*)::BIGINT AS "requests!",
                  COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost_microdollars!"
           FROM ai_requests r
           LEFT JOIN membership m
             ON m.user_id = r.user_id AND r.actor_kind = 'user'
           LEFT JOIN groups g ON g.id = m.scope_id AND $1 = 'group'
           LEFT JOIN projects p ON p.id = m.scope_id AND $1 = 'project'
           WHERE NOT r.synthetic
             AND r.created_at >= $4
             AND r.created_at < $5
           GROUP BY 1, 2
           ORDER BY 4 DESC, 3 DESC, 1
           LIMIT $6"#,
        kind.as_str(),
        exclusive,
        UNATTRIBUTED,
        range.from,
        range.to,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ScopeCostRow {
            scope_id: row.scope_id,
            label: row.label,
            requests: row.requests,
            cost_microdollars: row.cost_microdollars,
        })
        .collect())
}

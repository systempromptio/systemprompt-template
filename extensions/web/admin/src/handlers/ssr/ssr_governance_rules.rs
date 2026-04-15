use std::collections::BTreeMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext, DECISION_DENY};

use super::ACCESS_DENIED_HTML;

pub async fn governance_rules_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let rows = repositories::governance::list_governance_decisions(&pool, None)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch governance rule evaluations");
            vec![]
        });

    // Aggregate per policy (rule)
    let mut agg: BTreeMap<String, (i64, i64, i64)> = BTreeMap::new(); // (total, allow, deny)
    for r in &rows {
        let entry = agg.entry(r.policy.clone()).or_insert((0, 0, 0));
        entry.0 += 1;
        if r.decision == DECISION_DENY {
            entry.2 += 1;
        } else {
            entry.1 += 1;
        }
    }

    let rules: Vec<serde_json::Value> = agg
        .iter()
        .map(|(policy, (total, allow, deny))| {
            json!({
                "policy": policy,
                "total": total,
                "allowed": allow,
                "denied": deny,
            })
        })
        .collect();

    let recent: Vec<serde_json::Value> = rows
        .iter()
        .take(50)
        .map(|r| {
            json!({
                "tool_name": r.tool_name,
                "agent_id": r.agent_id,
                "agent_scope": r.agent_scope,
                "decision": r.decision,
                "is_denied": r.decision == DECISION_DENY,
                "policy": r.policy,
                "reason": r.reason,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let data = json!({
        "page": "governance-rules",
        "title": "Governance Rules",
        "hero_title": "Governance Rule Evaluation",
        "hero_subtitle": "Per-rule evaluation detail (scope, secret, rate)",
        "cli_command": "systemprompt infra db query \"SELECT policy, decision, COUNT(*) FROM governance_decisions GROUP BY policy, decision ORDER BY policy\"",
        "has_rules": !rules.is_empty(),
        "rules": rules,
        "has_recent": !recent.is_empty(),
        "recent": recent,
    });

    super::render_page(&engine, "governance-rules", &data, &user_ctx, &mkt_ctx)
}

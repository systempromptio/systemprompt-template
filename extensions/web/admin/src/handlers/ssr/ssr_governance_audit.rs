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

pub async fn governance_audit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let (rows, counts) = tokio::join!(
        repositories::governance::list_governance_decisions(&pool, None),
        repositories::governance::fetch_governance_counts(&pool),
    );
    let rows = rows.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch audit trail");
        vec![]
    });
    let counts = counts.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch governance counts");
        repositories::governance::GovernanceCounts {
            total: 0,
            allowed: 0,
            denied: 0,
            secret_breaches: 0,
        }
    });

    let entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "user_id": r.user_id,
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
        "page": "governance-audit",
        "title": "Governance Audit Trail",
        "hero_title": "Governance Audit Trail",
        "hero_subtitle": "Full audit trail with policy + reason for each decision",
        "cli_command": "systemprompt infra db query \"SELECT decision, tool_name, agent_id, agent_scope, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 5\"",
        "total": counts.total,
        "allowed": counts.allowed,
        "denied": counts.denied,
        "secret_breaches": counts.secret_breaches,
        "has_entries": !entries.is_empty(),
        "entries": entries,
    });

    super::render_page(&engine, "governance-audit", &data, &user_ctx, &mkt_ctx)
}

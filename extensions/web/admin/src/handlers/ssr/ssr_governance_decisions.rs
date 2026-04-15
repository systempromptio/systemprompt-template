use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext, DECISION_DENY, POLICY_SECRET_INJECTION};

use super::ACCESS_DENIED_HTML;

#[derive(Debug, Deserialize)]
pub struct DecisionsQuery {
    pub outcome: Option<String>,
}

pub async fn governance_decisions_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DecisionsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let outcome_filter = query.outcome.as_deref().unwrap_or("all");

    let (rows, counts) = tokio::join!(
        repositories::governance::list_governance_decisions(&pool, None),
        repositories::governance::fetch_governance_counts(&pool),
    );
    let rows = rows.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch governance decisions");
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

    let filtered: Vec<serde_json::Value> = rows
        .iter()
        .filter(|r| match outcome_filter {
            "allow" => r.decision != DECISION_DENY,
            "deny" => r.decision == DECISION_DENY,
            _ => true,
        })
        .map(|r| {
            json!({
                "id": r.id,
                "user_id": r.user_id,
                "tool_name": r.tool_name,
                "agent_id": r.agent_id,
                "agent_scope": r.agent_scope,
                "decision": r.decision,
                "is_denied": r.decision == DECISION_DENY,
                "is_secret_breach": r.policy == POLICY_SECRET_INJECTION,
                "policy": r.policy,
                "reason": r.reason,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let data = json!({
        "page": "governance-decisions",
        "title": "Governance Decisions",
        "hero_title": "Governance Decisions",
        "hero_subtitle": "Allow/deny decision log for all governed tool calls",
        "cli_command": "systemprompt infra db query \"SELECT decision, tool_name, agent_id, policy, reason FROM governance_decisions ORDER BY created_at DESC LIMIT 50\"",
        "total": counts.total,
        "allowed": counts.allowed,
        "denied": counts.denied,
        "outcome": outcome_filter,
        "has_decisions": !filtered.is_empty(),
        "decisions": filtered,
    });

    super::render_page(&engine, "governance-decisions", &data, &user_ctx, &mkt_ctx)
}

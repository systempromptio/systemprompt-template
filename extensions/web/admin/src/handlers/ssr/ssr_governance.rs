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
pub struct GovernanceQuery {
    pub q: Option<String>,
}

pub async fn governance_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GovernanceQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let search = query.q.as_deref();
    let (rows, counts) = tokio::join!(
        repositories::governance::list_governance_decisions(&pool, search),
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
    let total = counts.total;
    let denied = counts.denied;
    let allowed = counts.allowed;
    let secret_breaches = counts.secret_breaches;

    let decisions_json: Vec<serde_json::Value> = rows
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
                "is_secret_breach": r.policy == POLICY_SECRET_INJECTION,
                "policy": r.policy,
                "reason": r.reason,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let data = json!({
        "page": "governance",
        "title": "Governance",
        "total": total,
        "denied": denied,
        "allowed": allowed,
        "secret_breaches": secret_breaches,
        "decisions": decisions_json,
        "has_decisions": !decisions_json.is_empty(),
        "search_query": search.unwrap_or(""),
    });

    super::render_page(&engine, "governance", &data, &user_ctx, &mkt_ctx)
}

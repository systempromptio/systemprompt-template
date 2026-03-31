use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

#[derive(Debug, Deserialize)]
pub struct GovernanceQuery {
    pub q: Option<String>,
}

pub(crate) async fn governance_page(
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
    let rows = repositories::governance::list_governance_decisions(&pool, search)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch governance decisions");
            vec![]
        });

    let total = i64::try_from(rows.len()).unwrap_or(0);
    let denied: i64 = i64::try_from(rows.iter().filter(|r| r.decision == "deny").count()).unwrap_or(0);
    let allowed = total - denied;
    let secret_breaches: i64 = i64::try_from(rows
        .iter()
        .filter(|r| r.policy == "secret_injection")
        .count()).unwrap_or(0);

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
                "is_denied": r.decision == "deny",
                "is_secret_breach": r.policy == "secret_injection",
                "policy": r.policy,
                "reason": r.reason,
                "created_at": r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
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

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
use crate::types::{MarketplaceContext, UserContext, POLICY_SECRET_INJECTION};

use super::ACCESS_DENIED_HTML;

pub async fn governance_violations_page(
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
        tracing::warn!(error = %e, "Failed to fetch violations");
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

    let violations: Vec<serde_json::Value> = rows
        .iter()
        .filter(|r| {
            r.policy == POLICY_SECRET_INJECTION
                || r.reason.to_lowercase().contains("secret")
                || r.reason.to_lowercase().contains("aws")
                || r.reason.to_lowercase().contains("ghp_")
                || r.reason.to_lowercase().contains("private key")
        })
        .map(|r| {
            let reason_lc = r.reason.to_lowercase();
            let kind = if reason_lc.contains("aws") || reason_lc.contains("akia") {
                "AWS Key"
            } else if reason_lc.contains("ghp_") || reason_lc.contains("github") {
                "GitHub Token"
            } else if reason_lc.contains("private key") || reason_lc.contains("pem") || reason_lc.contains("rsa") {
                "PEM Private Key"
            } else {
                "Secret"
            };
            json!({
                "id": r.id,
                "user_id": r.user_id,
                "tool_name": r.tool_name,
                "agent_id": r.agent_id,
                "policy": r.policy,
                "reason": r.reason,
                "kind": kind,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let data = json!({
        "page": "governance-violations",
        "title": "Governance Violations",
        "hero_title": "Secret Breach Events",
        "hero_subtitle": "AWS keys, GitHub tokens, and PEM private keys detected in tool inputs",
        "cli_command": "systemprompt infra db query \"SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE policy = 'secret_injection' ORDER BY created_at DESC\"",
        "total": counts.total,
        "secret_breaches": counts.secret_breaches,
        "has_violations": !violations.is_empty(),
        "violations": violations,
    });

    super::render_page(&engine, "governance-violations", &data, &user_ctx, &mkt_ctx)
}

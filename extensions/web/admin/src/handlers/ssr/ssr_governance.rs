//! `/admin/governance` — Policies dashboard.
//!
//! Lists every policy registered via `inventory::submit!` together with its
//! enabled state (from `services/governance/config.yaml`) and recent
//! allow/deny counts from `governance_decisions`. This is the "modular
//! framework" view: the user sees the full chain at a glance and can drill
//! into any policy to inspect or tune it.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::handlers::webhook::governance;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

pub async fn governance_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let (counts, per_policy) = tokio::join!(
        repositories::governance::fetch_governance_counts(&pool),
        repositories::governance::fetch_per_policy_counts(&pool),
    );
    let counts = counts.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "governance counts query failed");
        repositories::governance::GovernanceCounts::default()
    });
    let per_policy_rows = per_policy.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "per-policy counts query failed");
        Vec::new()
    });
    let mut by_id: HashMap<String, repositories::governance::PerPolicyCounts> = per_policy_rows
        .into_iter()
        .map(|r| (r.policy.clone(), r))
        .collect();

    let policies_json: Vec<serde_json::Value> = {
        let chain = governance::chain();
        chain
            .iter()
            .map(|(cfg, p)| {
                let stats = by_id.remove(p.id());
                let allowed = stats.as_ref().map_or(0, |s| s.allowed);
                let denied = stats.as_ref().map_or(0, |s| s.denied);
                let last_at = stats
                    .as_ref()
                    .and_then(|s| s.last_at)
                    .map(|t| {
                        t.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                    })
                    .unwrap_or_default();
                json!({
                    "id": p.id(),
                    "name": p.name(),
                    "description": p.description(),
                    "enabled": cfg.enabled,
                    "allowed": allowed,
                    "denied": denied,
                    "has_recent_denies": denied > 0,
                    "last_at": last_at,
                    "edit_url": format!("/admin/governance/{}", p.id()),
                })
            })
            .collect()
    };

    // Anything left in `by_id` is a policy that produced decisions in the
    // past but is no longer registered (renamed / removed). Surface it so
    // operators don't lose sight of it.
    let orphan_json: Vec<serde_json::Value> = by_id
        .values()
        .map(|s| {
            json!({
                "id": s.policy,
                "allowed": s.allowed,
                "denied": s.denied,
                "last_at": s.last_at
                    .map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
            })
        })
        .collect();

    let data = json!({
        "page": "governance",
        "title": "Governance Policies",
        "total": counts.total,
        "allowed": counts.allowed,
        "denied": counts.denied,
        "secret_breaches": counts.secret_breaches,
        "policies": policies_json,
        "has_policies": !policies_json.is_empty(),
        "orphans": orphan_json,
        "has_orphans": !orphan_json.is_empty(),
        "config_path": "services/governance/config.yaml",
    });

    super::render_page(&engine, "governance", &data, &user_ctx, &mkt_ctx)
}

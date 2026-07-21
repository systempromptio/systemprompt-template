//! `/admin/governance` — Policies dashboard.
//!
//! Lists every policy registered via `inventory::submit!` together with its
//! enabled state, per-policy params from `services/governance/config.yaml`,
//! the source file the impl lives in, and 24h enforcement counts pulled from
//! `governance_decisions`. The page is the front door to the modular policy
//! framework — operators land here to see (a) what policies exist as code,
//! (b) what config they're running with, and (c) what they're actually doing
//! at runtime.

use crate::error::AdminError;
use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};


mod context;
mod data;
mod view;

use context::GovernancePageContext;

const WINDOW_24H_SECS: i64 = 86_400;
const TOP_POLICIES_LIMIT: i64 = 10;
const TOP_ACTORS_LIMIT: i64 = 10;

pub(crate) async fn governance_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let mut fetched = data::fetch_governance_data(&pool).await;

    let policies_json =
        view::build_policies_json(&mut fetched.lifetime_by_id, &mut fetched.window_by_id);
    let orphan_json = view::build_orphans_json(&fetched.lifetime_by_id);
    let (enforcement_json, any_enforcement_activity) = view::build_enforcement_json(&policies_json);
    let top_tools_json = view::build_top_tools_json(&fetched.top_tools);
    let top_actors_json = view::build_top_actors_json(&fetched.top_actors);

    let ctx = GovernancePageContext {
        page: "governance",
        title: "Governance Policies",
        lifetime_total: fetched.lifetime.total,
        lifetime_allowed: fetched.lifetime.allowed,
        lifetime_denied: fetched.lifetime.denied,
        window_total: fetched.window.total,
        window_allowed: fetched.window.allowed,
        window_denied: fetched.window.denied,
        window_breaches: fetched.window.secret_breaches,
        has_policies: !policies_json.is_empty(),
        policies: policies_json,
        enforcement: enforcement_json,
        has_enforcement_activity: any_enforcement_activity,
        has_top_tools: !top_tools_json.is_empty(),
        top_tools: top_tools_json,
        has_top_actors: !top_actors_json.is_empty(),
        top_actors: top_actors_json,
        has_orphans: !orphan_json.is_empty(),
        orphans_count: orphan_json.len(),
        orphans: orphan_json,
        config_path: "services/governance/config.yaml",
    };

    Ok(super::render_typed_page(
        &engine,
        "governance",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

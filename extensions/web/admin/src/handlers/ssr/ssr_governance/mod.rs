//! `/admin/governance` — Policies dashboard.
//!
//! Lists every policy the core `GovernanceEngine` carries together with its
//! enabled state, per-policy params from `services/governance/config.yaml`
//! and 24h enforcement counts pulled from `governance_decisions`. The page is
//! the front door to the modular policy framework — operators land here to
//! see what policies exist as code, what config they run with, and what they
//! are actually doing at runtime.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::types::BreadcrumbView;
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

    let policies = view::build_policies(&mut fetched.lifetime_by_id, &mut fetched.window_by_id).map_err(AdminError::internal)?;
    let orphans = view::build_orphans(&fetched.lifetime_by_id);
    let top_tools = view::build_top_tools(&fetched.top_tools);
    let top_actors = view::build_top_actors(&fetched.top_actors);
    let has_enforcement_activity = policies.iter().any(|p| p.window_evaluations > 0);

    let ctx = GovernancePageContext {
        page: "governance",
        title: "Policies",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", "/admin/governance"),
            BreadcrumbView::current("Policies"),
        ],
        kpis: view::build_kpis(&fetched.window, &fetched.lifetime),
        policy_count: policies.len(),
        has_policies: !policies.is_empty(),
        policies,
        has_enforcement_activity,
        has_top_tools: !top_tools.is_empty(),
        top_tools,
        has_top_actors: !top_actors.is_empty(),
        top_actors,
        has_orphans: !orphans.is_empty(),
        orphans_count: orphans.len(),
        orphans,
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

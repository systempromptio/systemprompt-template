//! Analysis routes: conversations, skills, marketplace versions and the
//! publication decisions taken from a marketplace's Distribution view.
//!
//! Every page is `/analysis/<noun>` with the entity's identity as the next
//! segment — the Platform section's `/<noun>/{id}` shape, one level in.

use crate::handlers;
use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/analysis/inventory/sync",
            post(handlers::ssr::analysis::inventory_actions::sync_inventory),
        )
        .route(
            "/analysis/skills",
            get(handlers::ssr::analysis::skills::page),
        )
        .route(
            "/analysis/skills/{skill}",
            get(handlers::ssr::analysis::skill_page),
        )
        .route(
            "/analysis/inventory/{entry_id}/bind",
            post(handlers::ssr::analysis::inventory_actions::bind),
        )
        .route(
            "/analysis/versions",
            get(handlers::ssr::analysis::marketplace_versions::landing_page),
        )
        .route(
            "/analysis/versions/{marketplace_id}",
            get(handlers::ssr::analysis::marketplace_versions::detail_page),
        )
        .route(
            "/analysis/revisions/{id}",
            get(handlers::ssr::analysis::revisions::revision_page),
        )
        .route(
            "/analysis/conversations",
            get(handlers::ssr::analysis::conversations::page),
        )
        .route(
            "/analysis/reports",
            get(handlers::ssr::analysis::reports::page)
                .post(handlers::ssr::analysis::reports::create),
        )
        .route(
            "/analysis/reports/{id}",
            get(handlers::ssr::analysis::reports::report_page),
        )
        .route(
            "/analysis/reports/{id}/regenerate",
            post(handlers::ssr::analysis::reports::regenerate),
        )
        .route(
            "/analysis/reports/{id}/status",
            get(handlers::ssr::analysis::reports::status),
        )
        .route(
            "/analysis/conversations/judge",
            post(handlers::ssr::analysis::conversations::judge_all),
        )
        .route(
            "/analysis/conversations/{context_id}",
            get(handlers::ssr::analysis::conversation_detail::page),
        )
        .route(
            "/analysis/conversations/{context_id}/judge",
            post(handlers::ssr::analysis::conversation_detail::judge_now),
        )
        .route(
            "/analysis/publications/review",
            post(handlers::ssr::analysis::lifecycle::review),
        )
        .route(
            "/analysis/publications/withdrawals",
            post(handlers::ssr::analysis::lifecycle::decide_withdrawal),
        )
}

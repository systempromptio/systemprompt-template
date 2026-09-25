//! The legacy-path redirects the SSR router points at.
//!
//! They live beside the router rather than in it because the router file is a
//! table of route literals — the admin contract suite reads it as one — and a
//! handler body in the middle of that table is the thing that makes it stop
//! reading like a table.
//!
//! Every route in [`legacy_routes`] is a path the dashboard used to serve and
//! no longer does. They answer `308`, never HTML: one home per page, so a
//! bookmark or a stale link lands on the new page rather than on a second copy
//! of it that could drift. They are a one-release courtesy and are expected to
//! be deleted, which is why they are one table rather than scattered.

use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, RawQuery};
use axum::response::Redirect;
use axum::routing::get;
use sqlx::PgPool;

// Why: `Redirect::permanent` is a 308, which preserves the method and the
// body. A 301 would let an intermediary rewrite a POST into a GET, and the
// device endpoints under these prefixes are POSTs.
fn moved(target: &str) -> Redirect {
    Redirect::permanent(target)
}

fn with_query(base: &str, query: Option<String>) -> Redirect {
    match query {
        Some(q) if !q.is_empty() => moved(&format!("{base}?{q}")),
        _ => moved(base),
    }
}

// Why: the analytics dashboard's tabs absorbed four standalone pages, so their
// redirect target is a tab rather than a path. An existing query string is
// dropped here on purpose: those pages' filters do not exist on the tab.
fn to_tab(tab: &str) -> Redirect {
    moved(&format!("/admin/analytics?tab={tab}"))
}

// Why: the old paths, each 308ing to its new home.
pub(super) fn legacy_routes() -> Router<Arc<PgPool>> {
    people_redirects().merge(catalog_redirects()).merge(
        entity_redirects()
            .merge(demo_and_report_redirects())
            .merge(governance_redirects()),
    )
}

fn people_redirects() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/access/users", get(access_users))
        .route("/access/user", get(access_user))
        // Why: mounted before the `{group_id}` form below reads it as an id.
        // Axum gives a static segment priority over a dynamic one, so the
        // order here is documentation rather than load-bearing.
        .route("/access/groups/unassigned", get(access_unassigned))
        .route("/access/groups", get(access_groups))
        .route("/access/groups/{group_id}", get(access_group_detail))
        .route("/access/projects", get(access_projects))
        .route("/access/projects/{project_id}", get(access_project_detail))
        .route("/access/control", get(access_control))
}

fn catalog_redirects() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/catalog", get(catalog))
        .route("/catalog/marketplace", get(catalog_marketplaces))
        .route("/catalog/marketplaces", get(catalog_marketplaces))
        .route(
            "/catalog/marketplaces/{marketplace_id}",
            get(catalog_marketplace_detail),
        )
        .route("/catalog/plugins", get(catalog_plugins))
        .route("/catalog/plugins/{plugin_id}", get(catalog_plugin_detail))
        .route("/catalog/skills", get(catalog_skills))
        .route("/catalog/skills/{skill_id}", get(catalog_skill_detail))
        .route("/catalog/mcp", get(catalog_mcp))
        .route("/catalog/mcp/{mcp_id}", get(catalog_mcp_detail))
        .route("/catalog/access-control", get(access_control))
}

fn entity_redirects() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/entities/requests", get(entities_requests))
        .route("/entities/requests/{request_id}", get(entities_request))
        .route("/entities/sessions", get(entities_sessions))
        .route("/entities/sessions/{session_id}", get(entities_session))
        .route("/entities/traces", get(entities_traces))
        .route("/entities/traces/{trace_id}", get(entities_trace))
        .route("/entities/contexts", get(entities_contexts))
        .route("/entities/contexts/{context_id}", get(entities_context))
        .route("/entities/skills", get(entities_skills))
}

fn demo_and_report_redirects() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/demo/skills", get(demo_skills))
        .route("/demo/tools", get(demo_tools))
        .route("/reports/internal", get(reports))
        .route("/reports/customer", get(reports))
        .route("/analytics/users/{user_id}", get(analytics_user))
}

fn governance_redirects() -> Router<Arc<PgPool>> {
    Router::new().route("/governance/warnings", get(governance_warnings))
}

async fn access_users(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/users", q)
}

async fn access_user(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/user", q)
}

// Why: "unassigned" stopped being a group and became a filter on the roster —
// there was never a group row behind it, only a query that read one.
async fn access_unassigned() -> Redirect {
    moved("/admin/users?filter=unassigned")
}

async fn access_groups(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/groups", q)
}

async fn access_group_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/groups/{id}"), q)
}

async fn access_projects(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/projects", q)
}

async fn access_project_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/projects/{id}"), q)
}

async fn access_control(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/access-control", q)
}

async fn catalog() -> Redirect {
    moved("/admin/plugins")
}

async fn catalog_marketplaces(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/marketplaces", q)
}

async fn catalog_marketplace_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/marketplaces/{id}"), q)
}

async fn catalog_plugins(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/plugins", q)
}

async fn catalog_plugin_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/plugins/{id}"), q)
}

async fn catalog_skills(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/skills", q)
}

async fn catalog_skill_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/skills/{id}"), q)
}

async fn catalog_mcp(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/mcp", q)
}

async fn catalog_mcp_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/mcp/{id}"), q)
}

async fn entities_requests(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/requests", q)
}

async fn entities_request(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/requests/{id}"), q)
}

async fn entities_sessions(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/sessions", q)
}

async fn entities_session(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/sessions/{id}"), q)
}

async fn entities_traces(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/traces", q)
}

async fn entities_trace(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/traces/{id}"), q)
}

async fn entities_contexts(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/contexts", q)
}

async fn entities_context(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/contexts/{id}"), q)
}

async fn entities_skills() -> Redirect {
    Redirect::to("/admin/skills")
}

async fn demo_skills() -> Redirect {
    Redirect::to("/admin/skills")
}

async fn demo_tools() -> Redirect {
    to_tab("tools")
}

async fn reports() -> Redirect {
    to_tab("cost")
}

// Why: per-user analytics became a tab on the user's own detail page — the
// question "what has this person spent" belongs beside who they are.
async fn analytics_user(Path(user_id): Path<String>) -> Redirect {
    moved(&format!("/admin/users/{user_id}?tab=usage"))
}

async fn governance_warnings(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/governance", q)
}

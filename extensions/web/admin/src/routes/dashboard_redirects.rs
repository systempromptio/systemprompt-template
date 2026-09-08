//! Compatibility redirects for the newly shared dashboard surfaces.

use axum::Router;
use axum::extract::{Path, RawQuery};
use axum::response::Redirect;
use axum::routing::get;
use sqlx::PgPool;
use std::sync::Arc;

pub(super) fn legacy_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/access/groups/unassigned", get(access_unassigned))
        .route("/access/groups", get(access_groups))
        .route("/access/groups/{group_id}", get(access_group_detail))
        .route("/access/projects", get(access_projects))
        .route("/access/projects/{project_id}", get(access_project_detail))
        .route("/access/control", get(access_control))
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
        .route("/entities/skills", get(entities_skills))
        .route("/demo/skills", get(demo_skills))
        .route("/demo/tools", get(demo_tools))
        .route("/reports/internal", get(reports))
        .route("/reports/customer", get(reports))
        .route("/analytics/users/{user_id}", get(analytics_user))
        .route("/governance/warnings", get(governance_warnings))
}

async fn access_control(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/access-control", q)
}

async fn access_group_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/groups/{id}"), q)
}

async fn access_groups(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/groups", q)
}

async fn access_project_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/projects/{id}"), q)
}

async fn access_projects(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/projects", q)
}

async fn access_unassigned() -> Redirect {
    moved("/admin/users?filter=unassigned")
}

async fn analytics_user(Path(user_id): Path<String>) -> Redirect {
    moved(&format!("/admin/users/{user_id}?tab=usage"))
}

async fn catalog() -> Redirect {
    moved("/admin/plugins")
}

async fn catalog_marketplace_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/marketplaces/{id}"), q)
}

async fn catalog_marketplaces(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/marketplaces", q)
}

async fn catalog_mcp(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/mcp", q)
}

async fn catalog_mcp_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/mcp/{id}"), q)
}

async fn catalog_plugin_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/plugins/{id}"), q)
}

async fn catalog_plugins(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/plugins", q)
}

async fn catalog_skill_detail(Path(id): Path<String>, RawQuery(q): RawQuery) -> Redirect {
    with_query(&format!("/admin/skills/{id}"), q)
}

async fn catalog_skills(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/skills", q)
}

async fn demo_skills() -> Redirect {
    to_tab("skills")
}

async fn demo_tools() -> Redirect {
    to_tab("tools")
}

async fn entities_skills() -> Redirect {
    to_tab("skills")
}

async fn governance_warnings(RawQuery(q): RawQuery) -> Redirect {
    with_query("/admin/governance", q)
}

fn moved(target: &str) -> Redirect {
    Redirect::permanent(target)
}

async fn reports() -> Redirect {
    to_tab("cost")
}

fn to_tab(tab: &str) -> Redirect {
    moved(&format!("/admin/analytics?tab={tab}"))
}

fn with_query(base: &str, query: Option<String>) -> Redirect {
    match query {
        Some(q) if !q.is_empty() => moved(&format!("{base}?{q}")),
        _ => moved(base),
    }
}

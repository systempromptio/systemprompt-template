//! View-model types and assembly helpers for the catalog pages.
//!
//! Holds the `Serialize` row/page structs the catalog templates consume plus
//! the small builders that map repository records into them. The axum handlers
//! in the parent module own the request flow; this module owns the shaping.

use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub(super) struct CatalogRow {
    pub(super) entity_type: &'static str,
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) enabled: bool,
    pub(super) source_path: String,
    pub(super) assignment_count: i64,
    pub(super) matrix_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct MarketplacePageData {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) skills: Vec<CatalogRow>,
    pub(super) plugins: Vec<CatalogRow>,
    pub(super) mcp_servers: Vec<CatalogRow>,
    pub(super) skills_count: usize,
    pub(super) plugins_count: usize,
    pub(super) mcp_servers_count: usize,
    pub(super) total: usize,
    pub(super) types: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct A2aPageData {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) agents: Vec<CatalogRow>,
    pub(super) agents_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct ExternalAgentRow {
    pub(super) id: String,
    pub(super) display_name: String,
    pub(super) kind: String,
    pub(super) enabled: bool,
    pub(super) description: String,
    pub(super) platforms: Vec<String>,
    pub(super) docs_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct ExternalPageData {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) agents: Vec<ExternalAgentRow>,
    pub(super) agents_count: usize,
    pub(super) enabled_count: usize,
}

fn matrix_url(entity_type: &str, entity_id: &str) -> String {
    format!("/admin/access/matrix?entity_type={entity_type}&entity_id={entity_id}")
}

pub(super) async fn assignment_counts_by_type(
    pool: &PgPool,
    entity_type: &str,
) -> HashMap<String, i64> {
    let rows = sqlx::query!(
        r#"SELECT entity_id, COUNT(*)::BIGINT AS "count!"
           FROM access_control_rules
           WHERE entity_type = $1
           GROUP BY entity_id"#,
        entity_type,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, entity_type, "Failed to load assignment counts");
        Vec::new()
    });
    rows.into_iter().map(|r| (r.entity_id, r.count)).collect()
}

pub(super) fn build_row(
    entity_type: &'static str,
    id: String,
    name: String,
    description: String,
    enabled: bool,
    source_path: String,
    assignment_count: i64,
) -> CatalogRow {
    CatalogRow {
        entity_type,
        matrix_url: matrix_url(entity_type, &id),
        id,
        name,
        description,
        enabled,
        source_path,
        assignment_count,
    }
}

pub(super) fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Html("<h1>Access Denied</h1><p>Admin access required.</p>"),
    )
        .into_response()
}

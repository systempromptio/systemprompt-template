//! View-model types and assembly helpers for the catalog pages.
//!
//! Three entity families — plugins, skills, MCP servers — each get a list page
//! and a detail page. Plugins are collections that reference skills, MCP
//! servers, agents, and hooks; the detail pages surface those links in both
//! directions (a plugin lists its members; a skill/server lists the plugins
//! that include it). The handlers own the request flow and data loading; this
//! module owns the row/page shaping and the cross-link URL construction.

use std::collections::HashMap;

use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize)]
pub(super) struct LinkedEntity {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct HookRef {
    pub(super) id: String,
    pub(super) event: String,
    pub(super) matcher: String,
    pub(super) command: String,
    pub(super) is_async: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct PluginListRow {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) category: String,
    pub(super) version: String,
    pub(super) enabled: bool,
    pub(super) skills_count: usize,
    pub(super) mcp_count: usize,
    pub(super) agents_count: usize,
    pub(super) assignment_count: i64,
    pub(super) source_path: String,
    pub(super) detail_url: String,
    pub(super) matrix_url: String,
    pub(super) visibility: super::visibility::VisibilityView,
}

// Why: A headline figure on a catalog list page.
#[derive(Debug, Clone, Serialize)]
pub(super) struct CatalogKpiView {
    pub(super) label: &'static str,
    pub(super) value: String,
    pub(super) sub: String,
    pub(super) tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct PluginsPageData {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) breadcrumbs: Vec<crate::handlers::ssr::types::BreadcrumbView>,
    pub(super) kpis: Vec<CatalogKpiView>,
    pub(super) sort_headers: Vec<super::sorting::SortHeaderView>,
    pub(super) plugins: Vec<PluginListRow>,
    pub(super) plugins_count: usize,
    pub(super) access_control_url: &'static str,
    pub(super) search: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillListRow {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) enabled: bool,
    pub(super) plugin_count: usize,
    pub(super) assignment_count: i64,
    pub(super) source_path: String,
    pub(super) detail_url: String,
    pub(super) matrix_url: String,
    pub(super) visibility: super::visibility::VisibilityView,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillsPageData {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) breadcrumbs: Vec<crate::handlers::ssr::types::BreadcrumbView>,
    pub(super) kpis: Vec<CatalogKpiView>,
    pub(super) sort_headers: Vec<super::sorting::SortHeaderView>,
    pub(super) skills: Vec<SkillListRow>,
    pub(super) skills_count: usize,
    pub(super) access_control_url: &'static str,
    pub(super) search: String,
}

#[derive(Debug, Serialize)]
pub(super) struct PluginDetailData {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) breadcrumbs: Vec<crate::handlers::ssr::types::BreadcrumbView>,
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) version: String,
    pub(super) category: String,
    pub(super) enabled: bool,
    pub(super) author_name: String,
    pub(super) keywords: Vec<String>,
    pub(super) roles: Vec<String>,
    pub(super) source_path: String,
    pub(super) matrix_url: String,
    pub(super) assignment_count: i64,
    pub(super) skills: Vec<LinkedEntity>,
    pub(super) mcp_servers: Vec<LinkedEntity>,
    pub(super) agents: Vec<LinkedEntity>,
    pub(super) hooks: Vec<HookRef>,
    pub(super) skills_count: usize,
    pub(super) mcp_count: usize,
    pub(super) agents_count: usize,
    pub(super) hooks_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillDetailData {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) breadcrumbs: Vec<crate::handlers::ssr::types::BreadcrumbView>,
    pub(super) activity_url: String,
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) enabled: bool,
    pub(super) source_path: String,
    pub(super) matrix_url: String,
    pub(super) assignment_count: i64,
    pub(super) included_by: Vec<LinkedEntity>,
    pub(super) included_by_count: usize,
}

pub(super) fn matrix_url(entity_type: &str, entity_id: &str) -> String {
    format!("/admin/access-control?entity_type={entity_type}&entity_id={entity_id}")
}

pub(super) fn plugin_url(id: &str) -> String {
    format!("/admin/plugins/{id}")
}

pub(super) fn skill_url(id: &str) -> String {
    format!("/admin/skills/{id}")
}

pub(super) fn mcp_url(id: &str) -> String {
    format!("/admin/mcp/{id}")
}

pub(super) async fn assignment_counts_by_type(
    pool: &PgPool,
    entity_type: &str,
) -> HashMap<String, i64> {
    crate::repositories::users::access_control::count_assignments_by_entity_type(pool, entity_type)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, entity_type, "Failed to load assignment counts");
            HashMap::new()
        })
}

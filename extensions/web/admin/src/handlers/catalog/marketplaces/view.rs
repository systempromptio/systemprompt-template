//! View-model types for the marketplace catalog pages.
//!
//! Two shapes matter here. The *declared* audience is what the manifest on
//! disk says — role and group chips, rendered verbatim so an operator can see
//! the intent. The *audience matrix* is what the resolver actually decides for
//! each group and role against each marketplace, which is a different fact: a
//! deny rule written elsewhere can close a marketplace the manifest offers.
//! Showing only one of the two is how an access surprise goes unnoticed.
//! The detail page answers the same question for one marketplace through
//! the shared "Who gets this" panel.

use serde::Serialize;

use crate::handlers::ssr::entity_panel::EntityAccessView;
use crate::handlers::ssr::sync_plane::HashView;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_shared::GroupId;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MarketplaceCardView {
    pub id: MarketplaceId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub visibility: String,
    pub detail_url: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub projects: Vec<String>,
    pub plugin_count: usize,
    pub skill_count: usize,
    pub mcp_count: usize,
    pub default_included: bool,
    pub assigned_groups: Vec<GroupId>,
    pub assigned_group_count: usize,
    pub allowed_subjects: usize,
    // Why: provenance — which source ships it — and the content hash that
    // is its version, from the same record the Versions page reads.
    pub source: String,
    pub source_tone: &'static str,
    pub content_hash: Option<HashView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AudienceCellView {
    pub marketplace_id: MarketplaceId,
    pub effective: String,
    pub is_allow: bool,
    pub layer: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AudienceRowView {
    pub subject: String,
    pub label: String,
    pub kind: &'static str,
    pub cells: Vec<AudienceCellView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AudienceMatrixView {
    pub columns: Vec<AudienceColumnView>,
    pub rows: Vec<AudienceRowView>,
    pub has_rows: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AudienceColumnView {
    pub id: MarketplaceId,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct MarketplacesPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub breadcrumbs: Vec<crate::handlers::ssr::types::BreadcrumbView>,
    pub marketplaces: Vec<MarketplaceCardView>,
    pub marketplaces_count: usize,
    pub audience: AudienceMatrixView,
    pub kpis: Vec<MarketplaceKpiView>,
    pub access_control_url: &'static str,
    pub search: String,
    pub tabs: Vec<crate::handlers::ssr::types::TabLinkView>,
    pub show_audience: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MarketplaceKpiView {
    pub label: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MemberLinkView {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MarketplaceDetailData {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<crate::handlers::ssr::types::BreadcrumbView>,
    pub id: MarketplaceId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub enabled: bool,
    pub visibility: String,
    pub source_path: String,
    pub plugins: Vec<MemberLinkView>,
    pub skills: Vec<MemberLinkView>,
    pub mcp_servers: Vec<MemberLinkView>,
    pub plugins_count: usize,
    pub skills_count: usize,
    pub mcp_count: usize,
    pub access: EntityAccessView,
}

#[must_use]
pub(crate) fn marketplace_url(id: &str) -> String {
    format!("/admin/marketplaces/{id}")
}

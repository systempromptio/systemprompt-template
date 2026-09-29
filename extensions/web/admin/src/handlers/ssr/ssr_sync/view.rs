//! View types for `/admin/sync` — Code sync.

use serde::Serialize;

use crate::handlers::ssr::sync_plane::HashView;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::sync::history::SyncHistoryRow;
use crate::repositories::sync::sources::SourcesView;

// Why: one row of the sync trail — who did what to which plane, and for
// which entities when the apply was scoped.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncHistoryView {
    pub id: String,
    pub actor: String,
    pub actor_url: String,
    pub action: String,
    pub plane: String,
    pub description: String,
    pub marketplaces: Vec<String>,
    pub at: String,
}

impl From<SyncHistoryRow> for SyncHistoryView {
    fn from(row: SyncHistoryRow) -> Self {
        Self {
            id: row.id,
            actor: row.display_name,
            actor_url: format!("/admin/users/{}", urlencoding::encode(row.user_id.as_str())),
            action: row.action,
            plane: row.plane.unwrap_or_default(),
            description: row.description,
            marketplaces: row.marketplaces,
            at: row.created_at.to_rfc3339(),
        }
    }
}

// Why: one plane as the export card lists it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExportPlaneView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub owner_url: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SyncPageData {
    pub page: &'static str,
    pub title: &'static str,
    // Why: `can_write` is the administrator's whole-plane and archive
    // authority; `can_refresh` is the source refresh, which needs the same
    // manage tier here (no participant tier), so it follows `can_write`.
    pub can_write: bool,
    pub can_refresh: bool,
    pub history: Vec<SyncHistoryView>,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub show_sources: bool,
    pub show_access: bool,
    pub access_review: Option<super::access_review::AccessReviewView>,
    pub docs_url: &'static str,
    pub configuration_url: &'static str,
    pub observability_url: &'static str,
    pub export_zip_url: &'static str,
    pub import_url: &'static str,
    pub sources: SourcesView,
    pub composed_hash: Option<HashView>,
    pub last_reconciled_hash: Option<HashView>,
    pub base_tree_hash: Option<HashView>,
    pub sources_unreadable: Option<String>,
    pub export_planes: Vec<ExportPlaneView>,
    pub stage_ttl_minutes: i64,
}

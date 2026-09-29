//! View types for `/admin/sync/import/{stage}` — the staged-archive preview.

use serde::Serialize;

use crate::handlers::ssr::sync_plane::{HashView, PlaneCardView};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::sync::archive::staging::OtherEntry;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ManifestView {
    pub release: String,
    pub exported_at: String,
    pub exported_by: String,
    pub base_tree_hash: Option<HashView>,
    pub composed_hash: Option<HashView>,
    pub planes: usize,
}

// Why: a plane the instance projects that the archive did not carry.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AbsentPlaneView {
    pub label: &'static str,
    pub source_file: &'static str,
    pub owner_url: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PreviewKpiView {
    pub label: &'static str,
    pub value: usize,
    pub note: &'static str,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportPreviewPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub stage_id: String,
    pub staged_by: String,
    pub staged_at: String,
    pub expires_at: String,
    pub manifest: Option<ManifestView>,
    pub manifest_error: Option<String>,
    pub kpis: Vec<PreviewKpiView>,
    pub planes: Vec<PlaneCardView>,
    pub absent: Vec<AbsentPlaneView>,
    pub other: Vec<OtherEntry>,
    // Why: handlebars-rust has no `.length`, so the section takes the count as a field.
    pub other_count: usize,
    pub apply_all_url: String,
    pub discard_url: String,
    pub sync_url: &'static str,
    pub docs_url: &'static str,
}

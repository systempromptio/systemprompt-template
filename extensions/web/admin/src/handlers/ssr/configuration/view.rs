//! View types for `/admin/configuration`.

use serde::Serialize;

use super::retention::RetentionView;
use crate::handlers::ssr::sync_plane::{AppliedView, HashView};
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};

// Why: one kind of configuration as the table prints it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConfigRowView {
    pub id: &'static str,
    pub label: &'static str,
    pub purpose: &'static str,
    pub path: String,
    pub is_dir: bool,
    pub files: usize,
    pub source: String,
    pub source_tone: &'static str,
    pub hash: Option<HashView>,
    pub is_projected: bool,
    pub projection: Option<&'static str>,
    pub state: &'static str,
    pub state_tone: &'static str,
    pub state_note: String,
    pub applied: Option<AppliedView>,
    pub link: Option<String>,
    pub link_label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConfigKpiView {
    pub label: &'static str,
    pub value: usize,
    pub note: &'static str,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigurationPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub release: &'static str,
    pub base_tree_hash: Option<HashView>,
    pub composed_hash: Option<HashView>,
    pub provenance: &'static str,
    pub bundle_count: usize,
    pub sources_unreadable: Option<String>,
    pub kpis: Vec<ConfigKpiView>,
    pub rows: Vec<ConfigRowView>,
    pub retention: RetentionView,
    pub sync_url: &'static str,
    pub export_zip_url: &'static str,
    pub docs_url: &'static str,
}

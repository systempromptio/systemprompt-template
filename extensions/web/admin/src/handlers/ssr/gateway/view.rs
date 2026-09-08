//! View-model types for `/admin/gateway`.
//!
//! Two route lists, deliberately kept apart. **Declared** routes are the
//! `gateway.routes:` sequence in `services/ai/gateway.yaml`, in file order,
//! addressed by index — that index is what the edit, delete and reorder
//! endpoints take, so it is the page's primary key. **Resolved** routes are
//! what the dispatcher will actually match, which includes the synthesized
//! catch-all the file never lists. A route present in one and not the other is
//! the fact this page exists to show.

use serde::Serialize;

use crate::handlers::ssr::types::BreadcrumbView;

#[derive(Debug, Clone, Serialize)]
pub(super) struct GatewayRouteRow {
    // Why: the position in the YAML sequence, which is what every write
    // endpoint addresses and what reordering permutes. Not a display detail.
    pub index: usize,
    pub id: String,
    pub model_pattern: String,
    pub provider: String,
    pub upstream_model: String,
    pub surface: String,
    pub surface_tone: &'static str,
    pub requires: String,
    pub when: String,
    pub has_pricing: bool,
    pub header_count: usize,
    pub grants: i64,
    pub dispatchable: bool,
    pub is_first: bool,
    pub is_last: bool,
    pub matrix_url: String,
}

// Why: A route the dispatcher will match that the file does not list.
#[derive(Debug, Clone, Serialize)]
pub(super) struct ResolvedOnlyRow {
    pub id: String,
    pub model_pattern: String,
    pub provider: String,
    pub upstream_model: String,
    pub matrix_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProviderOptionView {
    pub name: String,
    pub surface: String,
    pub model_count: usize,
    pub advertised: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct GatewayKpiView {
    pub label: &'static str,
    pub value: String,
    pub sub: String,
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProbeAccountView {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub(super) struct GatewayPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub enabled: bool,
    pub auth_scheme: String,
    pub inference_path_prefix: String,
    pub source_path: String,
    pub kpis: Vec<GatewayKpiView>,
    pub routes: Vec<GatewayRouteRow>,
    pub routes_count: usize,
    pub resolved_only: Vec<ResolvedOnlyRow>,
    pub resolved_only_count: usize,
    pub providers: Vec<ProviderOptionView>,
    pub providers_count: usize,
    pub probe_users: Vec<ProbeAccountView>,
    // Why: the page cannot mutate a file it could not read. The banner carries
    // the loader's own message rather than an empty table, because "no routes"
    // and "the gateway file is unreadable" are opposite operator situations.
    pub load_error: String,
    pub catalog_error: String,
    pub tabs: Vec<crate::handlers::ssr::types::TabLinkView>,
    pub show_resolved: bool,
}

//! Template context for `/admin/devices`.
//!
//! One outer row per person, holding the rows of whichever tab is open. The
//! four inner row types stay separate structs rather than one flattened row
//! because the four tables share only a person: a bridge has a heartbeat and
//! no state, a token has a state and no heartbeat, and a rendering that
//! pretended otherwise would have to invent a value for half its own columns.
//! The person's row carries its aggregate cells as a list in header order, so
//! the template draws them with one loop and cannot skew against the headers.

use serde::Serialize;

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{BreadcrumbView, FilterChipView, SortHeaderView, TabLinkView};

#[derive(Debug, Serialize)]
pub(super) struct DevicesPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) tabs: Vec<TabLinkView>,
    pub(super) tab: &'static str,
    pub(super) stats: FleetStatsView,
    pub(super) versions: Vec<VersionBarView>,
    pub(super) has_versions: bool,
    pub(super) filters: Vec<FilterChipView>,
    pub(super) items_label: &'static str,
    pub(super) sort_headers: Vec<SortHeaderView>,
    pub(super) groups: Vec<UserGroupView>,
    pub(super) count_label: String,
    pub(super) has_rows: bool,
    pub(super) empty_message: &'static str,
    pub(super) pagination: Pagination,
    // Why: the revoke buttons, not the page. A project manager reads the
    // fleet; only the admin roles may disable a credential, and the API
    // refuses them anyway — this stops the console offering a button that
    // would come back 403.
    pub(super) can_manage: bool,
    // Why: two of the four tabs offer no row action, and an actions column
    // with nothing in it is a column of whitespace taken from the columns that
    // do carry content. The header and the cells are drawn from this together,
    // so they cannot disagree about how many columns the row has.
    pub(super) has_row_actions: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct FleetStatsView {
    pub(super) bridges_active: String,
    pub(super) bridges_stale: String,
    pub(super) bridges_total_sub: String,
    pub(super) stale_tone: &'static str,
    pub(super) stale_url: String,
    pub(super) stale_active: bool,
    pub(super) versions: String,
    pub(super) versions_sub: String,
    pub(super) pats_active: String,
    pub(super) pats_sub: String,
    pub(super) certs_active: String,
    pub(super) certs_sub: String,
    pub(super) links_pending: String,
    pub(super) links_sub: String,
    pub(super) links_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct VersionBarView {
    pub(super) label: String,
    pub(super) devices: i64,
    // Why: a percentage rather than a pixel width, so the bar is drawn from
    // the same number the label states and cannot drift from it.
    pub(super) pct: i64,
    pub(super) is_latest: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct UserGroupView {
    pub(super) user_name: String,
    pub(super) user_url: String,
    pub(super) detail_id: String,
    pub(super) count_label: String,
    pub(super) summary: String,
    pub(super) cells: Vec<CellView>,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) sessions: Vec<BridgeHostRowView>,
    pub(super) pats: Vec<ApiKeyRowView>,
    pub(super) certs: Vec<CertRowView>,
    pub(super) links: Vec<LinkRowView>,
}

#[derive(Debug, Serialize)]
pub(super) struct CellView {
    pub(super) value: String,
    pub(super) class: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct BridgeHostRowView {
    // Why: no session id. A machine is the unit here, and its sessions are
    // folded into a count; the console has no page for a bridge session
    // anyway — `/admin/sessions/{id}` is a different id space.
    pub(super) hostname: String,
    pub(super) os: String,
    pub(super) version: String,
    pub(super) started_display: String,
    pub(super) heartbeat_display: String,
    pub(super) forwarded_display: String,
    pub(super) tokens_display: String,
    pub(super) sessions_display: Option<String>,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ApiKeyRowView {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) key_prefix: String,
    pub(super) created_display: String,
    pub(super) used_display: String,
    pub(super) expires_display: String,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) can_revoke: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct CertRowView {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) fingerprint: String,
    pub(super) enrolled_display: String,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) can_revoke: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct LinkRowView {
    pub(super) created_display: String,
    pub(super) expires_display: String,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
}

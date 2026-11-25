//! The one place admin entity-detail URLs are spelled out.
//!
//! Every entity-detail page is mounted flat under `/admin/` (see
//! `routes::ssr`). Hand-written links drifted from the mount point and 404'd,
//! so each prefix is stated exactly once here and every link builder goes
//! through these helpers.

use systemprompt::identifiers::{AiRequestId, ContextId, SessionId, TraceId};

pub(crate) fn session_detail_url(session: &SessionId) -> String {
    format!("/admin/sessions/{}", urlencoding::encode(session.as_str()))
}

pub(crate) fn context_detail_url(context: &ContextId) -> String {
    format!("/admin/contexts/{}", urlencoding::encode(context.as_str()))
}

pub(crate) fn request_detail_url(request: &AiRequestId) -> String {
    format!("/admin/requests/{}", urlencoding::encode(request.as_str()))
}

pub(crate) fn trace_detail_url(trace: &TraceId) -> String {
    format!("/admin/traces/{}", urlencoding::encode(trace.as_str()))
}

// Why: the page every "access" link lands on — the entity's own panel. A
// kind with no catalog detail page in this build has none.
#[must_use]
pub(crate) fn entity_access_url(entity_type: &str, entity_id: &str) -> Option<String> {
    let base = entity_access_base(entity_type)?;
    Some(format!("{base}/{}#access", urlencoding::encode(entity_id)))
}

const ENTITY_ACCESS_PAGES: &[(&str, &str)] = &[
    #[cfg(feature = "mcp")]
    ("marketplace", "/admin/marketplaces"),
    #[cfg(feature = "mcp")]
    ("plugin", "/admin/plugins"),
    #[cfg(feature = "mcp")]
    ("skill", "/admin/skills"),
    #[cfg(feature = "mcp")]
    ("mcp_server", "/admin/mcp"),
    #[cfg(feature = "gateway")]
    ("gateway_route", "/admin/gateway/routes"),
];

fn entity_access_base(entity_type: &str) -> Option<&'static str> {
    ENTITY_ACCESS_PAGES
        .iter()
        .find(|(kind, _)| *kind == entity_type)
        .map(|(_, base)| *base)
}

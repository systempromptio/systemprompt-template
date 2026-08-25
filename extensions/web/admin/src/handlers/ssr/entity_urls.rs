//! The one place admin entity-detail URLs are spelled out.
//!
//! Every entity-detail page is mounted under `/admin/entities/` (see
//! `routes::ssr`). Hand-written `/admin/sessions/{id}` style links drifted from
//! that mount point and 404'd, so each prefix is stated exactly once here and
//! every link builder goes through these helpers.

use systemprompt::identifiers::{AiRequestId, ContextId, SessionId, TraceId};

pub(crate) fn session_detail_url(session: &SessionId) -> String {
    format!(
        "/admin/entities/sessions/{}",
        urlencoding::encode(session.as_str())
    )
}

pub(crate) fn context_detail_url(context: &ContextId) -> String {
    format!(
        "/admin/entities/contexts/{}",
        urlencoding::encode(context.as_str())
    )
}

pub(crate) fn request_detail_url(request: &AiRequestId) -> String {
    format!(
        "/admin/entities/requests/{}",
        urlencoding::encode(request.as_str())
    )
}

pub(crate) fn trace_detail_url(trace: &TraceId) -> String {
    format!(
        "/admin/entities/traces/{}",
        urlencoding::encode(trace.as_str())
    )
}

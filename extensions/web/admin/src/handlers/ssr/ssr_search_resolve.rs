//! `GET /admin/api/search/resolve?q=<id>` — global header-search resolver.
//!
//! Inspects an opaque id and returns the URL of the appropriate detail page:
//! the request audit page for request/decision ids, the trace waterfall for
//! trace/session ids. Returns `{kind: "none"}` when the id does not resolve.
//!
//! This is a JSON endpoint that happens to live beside the SSR pages it feeds,
//! so its failures are `AdminError`, not the HTML-rendering face.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::governance::resolve::{ResolvedKind, resolve_id};
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SearchResponse {
    pub kind: &'static str,
    pub url: Option<String>,
}

pub(crate) async fn search_resolve(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SearchQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    let raw = query.q.unwrap_or_default();
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return Ok(unresolved());
    }

    let Some(r) = resolve_id(&pool, trimmed).await? else {
        return Ok(unresolved());
    };

    let encoded = urlencoding::encode(&r.id);
    let (kind, url) = match r.kind {
        ResolvedKind::Request => ("request", format!("/admin/requests/{encoded}")),
        ResolvedKind::Trace => ("trace", format!("/admin/traces/{encoded}")),
        ResolvedKind::Session => ("session", format!("/admin/sessions/{encoded}")),
        ResolvedKind::Context => ("context", format!("/admin/contexts/{encoded}")),
    };
    Ok(Json(SearchResponse {
        kind,
        url: Some(url),
    })
    .into_response())
}

fn unresolved() -> Response {
    // lint-ok: http-error — a successful "no match", not a failure
    Json(SearchResponse {
        kind: "none",
        url: None,
    })
    .into_response()
}

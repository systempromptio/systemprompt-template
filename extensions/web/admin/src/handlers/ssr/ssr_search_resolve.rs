//! `GET /admin/api/search/resolve?q=<id>` — global header-search resolver.
//!
//! Inspects an opaque id and returns the URL of the appropriate detail page:
//! the request audit page for request/decision ids, the trace waterfall for
//! trace/session ids. Returns `{kind: "none"}` when the id does not resolve.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::repositories::governance_grp::resolve::{resolve_id, ResolvedKind};
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub kind: &'static str,
    pub url: Option<String>,
}

pub async fn search_resolve(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SearchQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    let raw = query.q.unwrap_or_default();
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 128 {
        return Json(SearchResponse {
            kind: "none",
            url: None,
        })
        .into_response();
    }

    match resolve_id(&pool, trimmed).await {
        Ok(Some(r)) => {
            let encoded = urlencoding::encode(&r.id);
            let (kind, url) = match r.kind {
                ResolvedKind::Request => ("request", format!("/admin/requests/{encoded}")),
                ResolvedKind::Trace => ("trace", format!("/admin/traces/{encoded}")),
                ResolvedKind::Session => ("session", format!("/admin/sessions/{encoded}")),
                ResolvedKind::Context => ("context", format!("/admin/contexts/{encoded}")),
            };
            Json(SearchResponse {
                kind,
                url: Some(url),
            })
            .into_response()
        }
        Ok(None) => Json(SearchResponse {
            kind: "none",
            url: None,
        })
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, q = %trimmed, "search_resolve failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

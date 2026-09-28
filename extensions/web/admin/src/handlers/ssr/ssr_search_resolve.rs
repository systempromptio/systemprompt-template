//! `GET /admin/api/search/resolve?q=<id>` — global header-search resolver.
//!
//! Inspects an opaque id and returns the URL of the appropriate detail page:
//! the request audit page for request/decision ids, the trace waterfall for
//! trace/session ids. An exact id resolves outright. Anything else is treated
//! as a prefix — every list page shows ids as `short_id` (twelve characters
//! and an ellipsis), so a pasted value is usually a prefix — and the matches
//! come back as `matches` for the box to offer; a lone match also fills `url`
//! so Enter jumps to it. Returns `{kind: "none"}` when nothing resolves.
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
use crate::handlers::ssr::entity_urls::{
    context_detail_url, request_detail_url, session_detail_url, trace_detail_url,
};
use crate::repositories::governance::resolve::{ResolvedId, ResolvedKind, resolve_id};
use crate::repositories::governance::suggest::list_id_matches;
use crate::types::UserContext;
use systemprompt::identifiers::{AiRequestId, ContextId, SessionId, TraceId};

const MAX_MATCHES: i64 = 8;

#[derive(Debug, Deserialize)]
pub(crate) struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SearchMatch {
    pub kind: &'static str,
    pub id: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SearchResponse {
    pub kind: &'static str,
    pub url: Option<String>,
    pub matches: Vec<SearchMatch>,
}

pub(crate) async fn search_resolve(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SearchQuery>,
) -> AdminResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    let raw = query.q.unwrap_or_default();
    let trimmed = normalise(&raw);
    if trimmed.is_empty() || trimmed.len() > 128 {
        return Ok(unresolved());
    }

    if let Some(r) = resolve_id(&pool, trimmed).await? {
        let hit = to_match(r)?;
        return Ok(Json(SearchResponse {
            kind: hit.kind,
            url: Some(hit.url.clone()),
            matches: vec![hit],
        })
        .into_response());
    }

    let matches = list_id_matches(&pool, trimmed, MAX_MATCHES)
        .await?
        .into_iter()
        .map(to_match)
        .collect::<Result<Vec<_>, _>>()?;
    let (kind, url) = match matches.as_slice() {
        [only] => (only.kind, Some(only.url.clone())),
        [] => ("none", None),
        _ => ("many", None),
    };
    Ok(Json(SearchResponse { kind, url, matches }).into_response())
}

// Why: the copied text carries the `short_id` ellipsis and whatever
// whitespace the clipboard added; neither is part of the id.
fn normalise(raw: &str) -> &str {
    raw.trim().trim_end_matches('…').trim()
}

fn to_match(r: ResolvedId) -> AdminResult<SearchMatch> {
    // Why: The resolver picks the entity kind at runtime from an opaque id, so
    // the typed newtype is only known here, in the arm that matched.
    let (kind, url) = match r.kind {
        ResolvedKind::Request => ("request", request_detail_url(&AiRequestId::new(&r.id))),
        ResolvedKind::Trace => ("trace", trace_detail_url(&TraceId::new(&r.id))),
        ResolvedKind::Session => ("session", session_detail_url(&SessionId::new(&r.id))),
        ResolvedKind::Context => (
            "context",
            context_detail_url(&ContextId::try_new(&r.id).map_err(AdminError::internal)?),
        ),
    };
    Ok(SearchMatch {
        kind,
        id: r.id,
        url,
    })
}

fn unresolved() -> Response {
    // Why: lint-ok: http-error — a successful "no match", not a failure
    Json(SearchResponse {
        kind: "none",
        url: None,
        matches: Vec::new(),
    })
    .into_response()
}

//! A personal access token in place of a browser session, on the export
//! routes only.
//!
//! A kit repository's CI reads its release figures from
//! `/admin/export/{dataset}`, and an analysis job reads conversation records
//! from `/admin/export/transcripts`; both hold a PAT, not a browser session.
//! The PAT resolves to its owner and then passes through exactly the gates a
//! browser session does, so it reads what its owner could export and no more
//! — a transcript outside the owner's view is a 404, the same as in the
//! console. Every other admin route still refuses it.

use axum::Json;
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::PgPool;
use systemprompt::identifiers::Email;

use crate::handlers::extract_token_from_headers;
use crate::repositories::bridge::{API_KEY_PREFIX, find_api_key_user};
use crate::types::CookieSession;

pub(super) enum PatSession {
    NotApplicable,
    Accepted(CookieSession),
    Rejected(Response),
}

// Why: `/export/{dataset}`, `/export/{dataset}/preview`, and the transcript
// routes — `/export/transcripts`, `/export/transcripts/preview` and
// `/export/transcripts/{context_id}` — with or without the `/admin` mount
// prefix. Nothing deeper, and nothing outside `/export/`.
fn is_export(path: &str) -> bool {
    let path = path.strip_prefix("/admin").unwrap_or(path);
    let Some(rest) = path.strip_prefix("/export/") else {
        return false;
    };
    let mut parts = rest.split('/');
    let dataset = parts.next().unwrap_or_default();
    let tail = parts.next();
    let tail_ok = match tail {
        None | Some("preview") => true,
        Some(id) => dataset == "transcripts" && !id.is_empty(),
    };
    !dataset.is_empty() && tail_ok && parts.next().is_none()
}

fn rejected(message: &str) -> PatSession {
    PatSession::Rejected(
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "unauthorized", "message": message })),
        )
            .into_response(),
    )
}

// Why: everything the check needs is read out of the request before the
// lookup awaits, so the middleware future holds no borrow of the body.
pub(super) fn pat_request(request: &Request) -> Option<PatRequest> {
    let token = extract_token_from_headers(request.headers()).ok()?;
    token.starts_with(API_KEY_PREFIX).then(|| PatRequest {
        token,
        allowed: request.method() == Method::GET && is_export(request.uri().path()),
        path: request.uri().path().to_owned(),
    })
}

pub(super) struct PatRequest {
    token: String,
    allowed: bool,
    path: String,
}

pub(super) async fn pat_session(pool: &PgPool, pat: Option<PatRequest>) -> PatSession {
    let Some(PatRequest {
        token,
        allowed,
        path,
    }) = pat
    else {
        return PatSession::NotApplicable;
    };
    if !allowed {
        return rejected("A personal access token is accepted only on GET /admin/export/…");
    }
    let user = match find_api_key_user(pool, &token).await {
        Ok(Some(user)) => user,
        Ok(None) => return rejected("Unknown, revoked or expired personal access token"),
        Err(e) => {
            tracing::error!(error = %e, "PAT lookup failed");
            return rejected("The personal access token could not be verified");
        },
    };
    let Ok(email) = Email::try_new(user.email) else {
        return rejected("The token's owner has no valid email");
    };
    tracing::info!(key_id = %user.key_id, user_id = %user.user_id, path = %path, "export read with a personal access token");
    PatSession::Accepted(CookieSession {
        user_id: user.user_id,
        username: user.username,
        email,
        session_id: None,
    })
}

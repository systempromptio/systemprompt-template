//! `GET /admin/export/transcripts/{context_id}` — one conversation as JSON
//! or Markdown — and `GET /admin/export/transcripts` — a selected or
//! filtered set as JSON Lines, one bundle per line, with the `/preview` the
//! export dialog counts from.
//!
//! Visibility is the detail pages' rule: a context outside the caller's
//! scope answers 404, the same as a missing id, so the URL is no oracle for
//! other people's conversation ids. A viewer without a console seat gets the
//! owner-facing rendering: gateway framing stripped and credentials redacted,
//! exactly as `/admin/history` shows them.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, OriginalUri, Path, State};
use axum::http::header;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use super::bundle::{ConversationBundle, load_conversation_bundle};
use super::markdown;
use super::selection::{MAX_CONVERSATIONS, TranscriptSource, resolve_selection, source_of};
use crate::error::{AdminError, AdminResult};
use crate::export::model::ExportContext;
use crate::handlers::ssr::transcript_view::TranscriptOptions;
use crate::repositories::analytics::context_detail::find_context_header;
use crate::repositories::analytics::conversations::history_scope_for;
use crate::repositories::scope::visibility::may_view;
use crate::types::UserContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DocumentFormat {
    Json,
    Markdown,
}

#[derive(Debug, Default, Deserialize)]
struct DocumentQuery {
    format: Option<DocumentFormat>,
}

fn not_found() -> AdminError {
    AdminError::NotFound("No conversation matches that id.".to_owned())
}

// Why: console readers see the stored text as it is; everyone else is an
// owner reading their own conversation and gets the owner-facing treatment.
// This instance has no participant tier, so "console" is `is_console`.
fn options_for(user: &UserContext) -> TranscriptOptions {
    if user.is_console {
        TranscriptOptions::default()
    } else {
        TranscriptOptions::owner_facing()
    }
}

async fn visible(pool: &PgPool, user: &UserContext, context_id: &ContextId) -> AdminResult<bool> {
    let Some(header) = find_context_header(pool, context_id).await? else {
        return Ok(false);
    };
    let owner = header.user_id.as_ref();
    if user.is_console {
        return Ok(may_view(pool, user, owner).await?);
    }
    Ok(owner.is_some_and(|o| history_scope_for(user).may_view(o)))
}

fn attachment(content_type: &str, filename: &str, body: String) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(body.into())
        .unwrap_or_default()
}

pub(crate) async fn export_conversation(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(context_id): Path<String>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    let Ok(context_id) = ContextId::try_new(context_id.trim()) else {
        return Err(not_found());
    };
    if !visible(&pool, &user, &context_id).await? {
        return Err(not_found());
    }
    let query = axum::extract::Query::<DocumentQuery>::try_from_uri(&uri)
        .map(|q| q.0)
        .map_err(|e| AdminError::BadRequest(e.body_text()))?;
    let bundle = load_conversation_bundle(&pool, &context_id, options_for(&user))
        .await?
        .ok_or_else(not_found)?;
    let short = context_id.as_str().get(..8).unwrap_or_default();
    Ok(match query.format.unwrap_or(DocumentFormat::Json) {
        DocumentFormat::Json => attachment(
            "application/json; charset=utf-8",
            &format!("conversation-{short}.json"),
            serde_json::to_string_pretty(&bundle).map_err(AdminError::internal)?,
        ),
        DocumentFormat::Markdown => attachment(
            "text/markdown; charset=utf-8",
            &format!("conversation-{short}.md"),
            markdown::render(&bundle),
        ),
    })
}

// Why: a set is read through a list page's scoped query; the console pages
// need a console seat, and "My conversations" is every signed-in user's own.
fn may_export_set(user: &UserContext, ctx: &ExportContext<'_>) -> AdminResult<()> {
    let own_history = matches!(source_of(ctx), Ok(TranscriptSource::History));
    if user.is_console || own_history {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Console access required.".to_owned()))
    }
}

pub(crate) async fn export_conversations(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Response> {
    let ctx = ExportContext::new(&pool, &user, uri);
    may_export_set(&user, &ctx)?;
    let selection = resolve_selection(&pool, &user, &ctx).await?;
    let opts = options_for(&user);
    let mut body = String::new();
    let mut skipped = 0usize;
    for context_id in &selection.context_ids {
        if !visible(&pool, &user, context_id).await? {
            skipped += 1;
            continue;
        }
        if let Some(bundle) = load_conversation_bundle(&pool, context_id, opts).await? {
            push_line(&mut body, &bundle)?;
        }
    }
    let filename = selection.window.map_or_else(
        || {
            format!(
                "transcripts-{}.jsonl",
                chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
            )
        },
        |w| {
            format!(
                "transcripts-{}-{}.jsonl",
                w.from.format("%Y%m%d"),
                w.to.format("%Y%m%d")
            )
        },
    );
    let mut response = attachment("application/x-ndjson; charset=utf-8", &filename, body);
    let headers = response.headers_mut();
    if selection.capped() {
        headers.insert("x-export-capped", header::HeaderValue::from_static("true"));
    }
    if skipped > 0 {
        headers.insert("x-export-skipped", header::HeaderValue::from(skipped));
    }
    Ok(response)
}

#[derive(Debug, Serialize)]
pub(crate) struct TranscriptsPreview {
    rows: i64,
    columns: usize,
    cells: i64,
    capped: bool,
    cap: i64,
    from: Option<String>,
    to: Option<String>,
    clamped: bool,
}

// Why: the dialog's count for a transcript set — how many conversations the
// file will hold, whether the 500 ceiling bit, and the window it resolved.
pub(crate) async fn export_conversations_preview(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    OriginalUri(uri): OriginalUri,
) -> AdminResult<Json<TranscriptsPreview>> {
    let ctx = ExportContext::new(&pool, &user, uri);
    may_export_set(&user, &ctx)?;
    let selection = resolve_selection(&pool, &user, &ctx).await?;
    Ok(Json(TranscriptsPreview {
        rows: selection.total.min(MAX_CONVERSATIONS),
        columns: 0,
        cells: 0,
        capped: selection.capped(),
        cap: MAX_CONVERSATIONS,
        from: selection.window.map(|w| w.from.to_rfc3339()),
        to: selection.window.map(|w| w.to.to_rfc3339()),
        clamped: selection.window.is_some_and(|w| w.clamped),
    }))
}

fn push_line(body: &mut String, bundle: &ConversationBundle) -> AdminResult<()> {
    body.push_str(&serde_json::to_string(bundle).map_err(AdminError::internal)?);
    body.push('\n');
    Ok(())
}

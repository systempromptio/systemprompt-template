//! `/admin/artifacts/{artifact_id}/preview` — the artifact rendered as the
//! host would render it, for the sandboxed frame on the detail page.
//!
//! The body comes from the content-addressed store, so what is previewed is
//! exactly what was scanned and stored. Typed artifacts go through core's
//! renderer registry; anything it cannot render is shown as its JSON. The
//! page is served for same-origin framing only and with a sandboxing CSP,
//! because an artifact body is third-party content.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::{Html, IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::extension::FrameOptionsOverride;
use systemprompt::identifiers::{ArtifactId, ContextId};
use systemprompt::manifest::profile::FrameOptions;
use systemprompt::mcp::services::ui_renderer::{RenderTarget, artifact_ui_resource};

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::page::Page;
use crate::repositories::analytics::artifacts::find_artifact;

pub(crate) async fn artifact_preview(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(artifact_id): Path<ArtifactId>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let detail = find_artifact(&pool, &artifact_id)
        .await
        .map_err(AdminError::from)?
        .ok_or_else(|| AdminError::NotFound(format!("Artifact {artifact_id} not found")))?;
    let Some(body) = detail.body.as_ref() else {
        return Err(AdminError::NotFound("Artifact body is not retained".to_owned()).into());
    };

    let context_id = detail
        .context_id
        .clone()
        .unwrap_or_else(ContextId::generate);
    let target = RenderTarget {
        artifact_id: &artifact_id,
        artifact_type: &detail.artifact_type,
        payload: body,
        context_id,
        title: detail.artifact_title.clone(),
    };
    let html = match artifact_ui_resource(&target) {
        Ok(resource) => resource.html,
        Err(error) => {
            tracing::debug!(%error, artifact_id = %artifact_id, "artifact has no UI renderer; previewing as JSON");
            json_preview(body)
        },
    };

    let mut response = Html(html).into_response();
    response
        .extensions_mut()
        .insert(FrameOptionsOverride(FrameOptions::SameOrigin));
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("sandbox allow-scripts; frame-ancestors 'self'"),
    );
    Ok(response)
}

// JSON: renders an arbitrary stored body as escaped, pretty-printed text.
fn json_preview(body: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string());
    let escaped = pretty
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Artifact</title>\
         <style>body{{margin:0;padding:16px;font:13px/1.5 ui-monospace,monospace;\
         background:#0b0e14;color:#e6e6e6}}pre{{white-space:pre-wrap;word-break:break-word}}</style>\
         </head><body><pre>{escaped}</pre></body></html>"
    )
}

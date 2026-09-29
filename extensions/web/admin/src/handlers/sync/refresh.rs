//! `POST /sync/sources/refresh` — re-resolve the pinned sources through
//! core's own refresh pipeline, in-process.
//!
//! Core owns the fetch-verify-compose pipeline and the in-place import that
//! follows it (authz reconcile + inventory refresh); an extension has
//! neither. Core hands every extension router a `ServicesRefresh` handle to
//! that pipeline, so this handler runs it directly for a caller it has
//! authorised by its own rule — an administrator — with no `restart` (a
//! marketplace-only kit is served the moment core recomposes) and no admin
//! token minted. Core's response is returned verbatim.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::api::routes::admin::services::ServicesRefresh;

use crate::activity::{self, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::types::UserContext;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CoreRefreshResponse {
    pub changed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composed_hash: Option<String>,
    // JSON: core's per-source views, relayed as-is
    #[serde(default)]
    pub sources: Vec<serde_json::Value>,
    #[serde(default)]
    pub reconciled: bool,
    #[serde(default)]
    pub restart_recommended: bool,
    pub restarting: bool,
}

pub(crate) async fn refresh_sources_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Extension(refresh): Extension<ServicesRefresh>,
) -> AdminResult<Response> {
    let result = refresh.run(&user_ctx.user_id).await.map_err(|e| {
        let err = e.into_inner();
        match err.code.status_code() {
            StatusCode::CONFLICT => AdminError::Conflict(err.message),
            _ => AdminError::Upstream(format!("services refresh failed: {}", err.message)),
        }
    })?;
    // JSON: core's typed reply, relayed through our own shape so the page's
    // contract does not follow core's field set
    let sources = result
        .sources
        .iter()
        .map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null))
        .collect();
    let parsed = CoreRefreshResponse {
        changed: result.changed,
        composed_hash: result.composed_hash,
        sources,
        reconciled: result.reconciled,
        restart_recommended: result.restart_recommended,
        restarting: result.restarting,
    };
    activity::record(
        &pool,
        NewActivity::sources_refreshed(&user_ctx.user_id, parsed.changed, parsed.reconciled),
    )
    .await;
    Ok(Json(parsed).into_response())
}

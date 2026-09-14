//! `POST /admin/connectors/{provider}/reprovision` — an org was refreshed or
//! its app re-created, so every stored grant against it is void.
//!
//! Salesforce sandbox refreshes delete the External Client App and with it
//! every user's consent. Without this, each user discovers it alone as a
//! `grant_rejected` on their next tool call. Resetting the whole provider in
//! one transaction moves every account to `reconnect_required` with a named
//! reason, so the profile page explains it before anyone tries a tool.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::users::{connector_accounts, connector_credentials};
use crate::services::connector_oauth::Provider;
use crate::types::UserContext;

const SANDBOX_REFRESHED: &str = "sandbox_refreshed";

#[derive(Debug, Serialize)]
pub(crate) struct ReprovisionResponse {
    pub provider: String,
    pub accounts_reset: u64,
    pub credentials_deleted: u64,
    pub error_code: &'static str,
}

pub(crate) async fn reprovision_connector_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(provider): Path<String>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let provider = Provider::try_from(provider).map_err(AdminError::BadRequest)?;
    if !provider.is_salesforce() || provider.salesforce_org().is_err() {
        return Err(AdminError::NotFound(
            "No Salesforce org with this id".to_owned(),
        ));
    }

    let mut tx = pool.begin().await?;
    let accounts_reset =
        connector_accounts::reprovision_provider(&mut tx, provider.slug(), SANDBOX_REFRESHED)
            .await?;
    let credentials_deleted =
        connector_credentials::delete_all_for_provider(&mut tx, provider.slug()).await?;
    connector_accounts::delete_all_pending(&mut tx, provider.slug()).await?;
    tx.commit().await?;

    tracing::info!(
        actor = %user_ctx.user_id, provider = provider.slug(), accounts_reset,
        "Salesforce connector reprovisioned"
    );
    Ok(Json(ReprovisionResponse {
        provider: provider.slug().to_owned(),
        accounts_reset,
        credentials_deleted,
        error_code: SANDBOX_REFRESHED,
    })
    .into_response())
}

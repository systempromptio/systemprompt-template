//! Explicit binding of a configured inventory entry to the managed resource
//! that shares its name, so retained snapshots and coverage can attach to it,
//! and the on-demand record of the services tree's marketplace versions.
//!
//! The inventory refresh and the publication of every configured skill are
//! core's `inventory_refresh` job: its refresher reads the bundle sources
//! and the active services root, which an install member does not hold, so a
//! binding or a sync here takes effect on that job's next run.

use crate::env::AdminEnv;

use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::Extension;
use axum::extract::{Form, Path, State};
use axum::http::HeaderMap;
use axum::response::Redirect;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{InventoryEntryId, ManagedResourceId};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BindForm {
    resource_id: ManagedResourceId,
}

pub(crate) async fn bind(
    Extension(env): Extension<AdminEnv>,
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(entry): Path<InventoryEntryId>,
    headers: HeaderMap,
    Form(form): Form<BindForm>,
) -> AdminHtmlResult<Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    crate::handlers::shared::require_write_origin(&env, &headers)?;
    state
        .repository
        .bind_inventory_resource(&state.owner, &user.user_id, &entry, &form.resource_id)
        .await?;
    Ok(Redirect::to(SKILLS_PAGE))
}

pub(crate) async fn sync_inventory(
    Extension(env): Extension<AdminEnv>,
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> AdminHtmlResult<Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    crate::handlers::shared::require_write_origin(&env, &headers)?;
    // Why: a sync is also when a marketplace's bytes may have moved, so the
    // version record is refreshed here and not only at boot.
    crate::repositories::sync::sources_db::record_service_sources(&env, &pool)
        .await
        .map_err(AdminError::internal)?;
    Ok(Redirect::to(SKILLS_PAGE))
}

const SKILLS_PAGE: &str = crate::handlers::ssr::analysis_urls::ANALYSIS_SKILLS_URL;

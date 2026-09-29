//! Explicit binding of a configured inventory entry to the managed resource
//! that shares its name, so retained snapshots and coverage can attach to it,
//! and the on-demand inventory sync that publishes every configured skill.

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
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(entry): Path<InventoryEntryId>,
    headers: HeaderMap,
    Form(form): Form<BindForm>,
) -> AdminHtmlResult<Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    crate::handlers::shared::require_write_origin(&headers)?;
    state
        .repository
        .bind_inventory_resource(&state.owner, &user.user_id, &entry, &form.resource_id)
        .await?;
    let root = crate::handlers::shared::get_services_path()?;
    let services = systemprompt::loader::ConfigLoader::load().map_err(AdminError::internal)?;
    systemprompt::marketplace::inventory::InventoryService::new(state.repository.clone())
        .refresh(&state.owner, &root, &services)
        .await?;
    Ok(Redirect::to(SKILLS_PAGE))
}

// Why: the same pass the scheduler runs each minute, on demand — refresh the
// inventory, then publish the latest configured revision of every skill.
pub(crate) async fn sync_inventory(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> AdminHtmlResult<Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator access required".to_owned()).into());
    }
    crate::handlers::shared::require_write_origin(&headers)?;
    let root = crate::handlers::shared::get_services_path()?;
    let services = systemprompt::loader::ConfigLoader::load().map_err(AdminError::internal)?;
    let service =
        systemprompt::marketplace::inventory::InventoryService::new(state.repository.clone());
    service.refresh(&state.owner, &root, &services).await?;
    service
        .publish_latest(
            &systemprompt::marketplace::inventory::BaselineScope {
                owner: &state.owner,
                actor: &user.user_id,
                root: &root,
                services: &services,
            },
            &mut systemprompt::marketplace::inventory::PublishGuard::default(),
        )
        .await?;
    // Why: a sync is also when a marketplace's bytes may have moved, so the
    // version record is refreshed here and not only at boot.
    crate::repositories::sync::sources_db::record_service_sources(&pool)
        .await
        .map_err(AdminError::internal)?;
    Ok(Redirect::to(SKILLS_PAGE))
}

const SKILLS_PAGE: &str = crate::handlers::ssr::analysis_urls::ANALYSIS_SKILLS_URL;

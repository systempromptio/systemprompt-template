//! Server-rendered admin dashboard router.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;

use systemprompt::config::ProfileBootstrap;
use systemprompt::models::AppPaths;

use super::pools::DbHandles;
use systemprompt_web_site::config_loader;

use crate::admin;

pub(crate) fn build(db: &DbHandles) -> Option<Router> {
    let admin_dir = admin_template_dir()?;
    let branding = config_loader::branding_config();
    let engine = admin::templates::AdminTemplateEngine::new(&admin_dir)
        .map_err(|e| tracing::error!(error = %e, "Failed to initialize admin template engine"))
        .ok()?
        .with_branding(branding);
    Some(admin::admin_ssr_router(Arc::clone(&db.read), engine))
}

fn admin_template_dir() -> Option<PathBuf> {
    let profile = ProfileBootstrap::get()
        .map_err(|e| tracing::error!(error = %e, "Profile unavailable for admin template dir"))
        .ok()?;
    let paths = AppPaths::from_profile(&profile.paths, profile.path_resolution())
        .map_err(|e| tracing::error!(error = %e, "App paths unavailable for admin template dir"))
        .ok()?;
    Some(paths.storage().files().join("admin"))
}

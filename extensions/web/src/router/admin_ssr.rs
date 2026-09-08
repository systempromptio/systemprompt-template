//! Server-rendered admin dashboard router.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;

use systemprompt::config::ProfileBootstrap;
use systemprompt::models::AppPaths;

use super::pools::DbHandles;
use systemprompt_web_site::config_loader;

use crate::admin;

// Why: The dashboard and the desktop-bridge sign-in flow, built together
// because both need the template engine, and either one missing is the same
// misconfiguration.
pub(crate) struct SsrRouters {
    pub admin: Router,
    pub bridge_auth: Router,
}

pub(crate) fn build(db: &DbHandles) -> Option<SsrRouters> {
    let admin_dir = admin_template_dir()?;
    let branding = config_loader::branding_config();
    let engine = admin::templates::AdminTemplateEngine::new(&admin_dir)
        .map_err(|e| tracing::error!(error = %e, "Failed to initialize admin template engine"))
        .ok()?
        .with_branding(branding);
    Some(SsrRouters {
        bridge_auth: admin::bridge_auth_ssr_router(Arc::clone(&db.write), engine.clone()),
        admin: admin::admin_ssr_router(Arc::clone(&db.write), engine),
    })
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

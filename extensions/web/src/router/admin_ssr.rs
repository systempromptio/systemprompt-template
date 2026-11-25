//! Server-rendered admin dashboard router.

use std::sync::Arc;

use axum::Router;


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

pub(crate) fn build(
    db: &DbHandles,
    sso_deps: admin::AdfsDeps,
    settings: &crate::WebSettings,
    env: &systemprompt_web_admin::env::AdminEnv,
) -> Option<SsrRouters> {
    let admin_dir = settings.admin_template_dir.clone();
    let branding = config_loader::branding_config(&settings.services_path);
    let engine = admin::templates::AdminTemplateEngine::new(&admin_dir)
        .map_err(|e| tracing::error!(error = %e, "Failed to initialize admin template engine"))
        .ok()?
        .with_branding(branding);
    Some(SsrRouters {
        bridge_auth: admin::bridge_auth_ssr_router(Arc::clone(&db.read), engine.clone()),
        admin: admin::admin_ssr_router(
            &db.db,
            Arc::clone(&db.read),
            db.owner.clone(),
            admin::AdminSsrShell {
                engine,
                sso_deps,
                env,
            },
        ),
    })
}

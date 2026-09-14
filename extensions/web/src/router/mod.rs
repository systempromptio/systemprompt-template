//! HTTP router assembly for the web extension.
//!
//! Split by concern: [`pools`] extracts the read/write database handles from
//! the type-erased extension context, [`api`] builds the JSON/webhook plane,
//! and [`admin_ssr`] builds the server-rendered dashboard plus the
//! desktop-bridge sign-in flow. `build` composes them; if the admin template
//! engine cannot initialise, the API plane still mounts at its normal prefix
//! and only the SSR nests are skipped, so a misconfigured deploy degrades
//! loudly instead of moving routes.

mod admin_ssr;
mod api;
mod pools;

use std::sync::Arc;

use axum::Router;

use systemprompt::extension::prelude::{ExtensionContext, ExtensionRouter};

use crate::admin;
use pools::DbHandles;

pub(crate) fn build(ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
    let db = DbHandles::from_context(ctx)?;
    let session_service = pools::build_session_service(&db)?;

    let sso_deps = admin::AdfsDeps {
        config: crate::extension::WebExtension::adfs_config()
            .unwrap_or_else(|| Arc::new(admin::AdfsConfig::disabled())),
        write_pool: Arc::clone(&db.write),
        session_service: Arc::clone(&session_service),
    };

    let api_router = api::build(&db, &session_service);
    let share_api = api::share(&db);

    let mut combined = Router::new()
        .merge(share_api)
        .nest("/api/public", api_router);

    match admin_ssr::build(&db, sso_deps) {
        Some(ssr) => {
            combined = Router::new()
                .nest_service("/admin", ssr.admin)
                .nest_service("/bridge-auth", ssr.bridge_auth)
                .merge(combined);
        },
        None => {
            tracing::error!(
                "Admin template engine failed to initialise; serving API routes without the SSR dashboard"
            );
        },
    }

    Some(ExtensionRouter::public(combined, "/"))
}

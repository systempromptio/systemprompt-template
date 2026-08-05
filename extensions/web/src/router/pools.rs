//! Database handle extraction from the type-erased extension context.

use std::sync::Arc;


use systemprompt::analytics::AnalyticsService;
use systemprompt::database::{Database, PgPool};
use systemprompt::extension::prelude::ExtensionContext;
use systemprompt::oauth::SessionCreationService;
use systemprompt::users::UserService;

pub(crate) struct DbHandles {
    pub read: Arc<PgPool>,
    pub write: Arc<PgPool>,
}

impl DbHandles {
    pub(crate) fn from_context(ctx: &dyn ExtensionContext) -> Option<Self> {
        let db_handle = ctx.database();
        let db = db_handle.as_any().downcast_ref::<Database>()?;
        let read = db.pool()?;
        let write = db.write_pool_arc().unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get write pool, falling back to read pool");
            Arc::clone(&read)
        });
        Some(Self { read, write })
    }

    fn database(&self) -> Arc<Database> {
        Arc::new(Database::from_pools(
            Arc::clone(&self.read),
            Some(Arc::clone(&self.write)),
        ))
    }
}

pub(crate) fn build_session_service(db: &DbHandles) -> Option<Arc<SessionCreationService>> {
    let dbpool = db.database();
    let user_repo = systemprompt::users::UserRepository::new(&dbpool)
        .map_err(|e| tracing::error!(error = %e, "Failed to build user repository"))
        .ok()?;
    let user = UserService::new(Arc::new(user_repo));
    let analytics_repos = systemprompt::analytics::repository::AnalyticsRepositories::new(&dbpool)
        .map_err(|e| tracing::error!(error = %e, "Failed to build analytics repositories"))
        .ok()?;
    let analytics = AnalyticsService::new(None, None, &analytics_repos);
    Some(Arc::new(SessionCreationService::new(
        Arc::new(analytics),
        Arc::new(user),
    )))
}

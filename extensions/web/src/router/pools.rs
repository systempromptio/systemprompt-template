//! The database handles and system owner the web routers are built from,
//! resolved in the member's provide fn.

use std::sync::Arc;

use systemprompt::database::{Database, DbPool};
use systemprompt::extension::prelude::{MissingDependency, Resolved};
use systemprompt::manifest::services::SystemAdmin;
use systemprompt::oauth::SessionCreationService;
use systemprompt::prelude::PgPool;
use systemprompt::users::{SessionRepository, UserService};

pub(crate) struct DbHandles {
    pub owner: systemprompt::identifiers::UserId,
    pub read: Arc<PgPool>,
    pub write: Arc<PgPool>,
    // Why: core repositories open their own handles from the shared database,
    // so the routers receive it alongside the raw pools the handlers still use.
    pub db: DbPool,
}

impl DbHandles {
    pub(crate) fn resolve(r: &Resolved<'_>) -> Result<Self, MissingDependency> {
        let database = r.get::<DbPool>()?;
        let db = database.as_ref();
        let read = db.pool();
        let write = db.write_pool();
        let db = Arc::new(Database::from_pools(
            Arc::clone(&read),
            Some(Arc::clone(&write)),
        ));
        Some(Self {
            read,
            write,
            db,
            owner: r.get::<SystemAdmin>()?.id().clone(),
        })
    }
}

pub(crate) fn build_session_service(db: &DbHandles) -> Arc<SessionCreationService> {
    let dbpool = Arc::clone(&db.db);
    let user_repo = systemprompt::users::UserRepository::new(&dbpool);
    let user = UserService::new(Arc::new(user_repo));
    let sessions = SessionRepository::new(&dbpool);
    Arc::new(SessionCreationService::new(
        Arc::new(sessions),
        Arc::new(user),
    ))
}

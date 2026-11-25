//! The database handles and system owner the web routers are built from,
//! resolved in the member's provide fn.

use std::sync::Arc;

use systemprompt::database::{Database, DbPool};
use systemprompt::extension::prelude::{MissingDependency, Resolved};
use systemprompt::manifest::services::SystemAdmin;
use systemprompt::prelude::PgPool;

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
        Ok(Self {
            read,
            write,
            db,
            owner: r.get::<SystemAdmin>()?.id().clone(),
        })
    }
}

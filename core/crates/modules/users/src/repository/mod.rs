mod session_queries;
mod user_operations;
mod user_queries;

use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

#[derive(Debug, Clone)]
pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }
}

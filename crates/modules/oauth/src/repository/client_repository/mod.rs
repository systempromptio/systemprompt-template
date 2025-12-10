mod cleanup;
mod mutations;
mod queries;
mod relations;

use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

#[derive(Clone, Debug)]
pub struct ClientRepository {
    pool: Arc<PgPool>,
}

impl ClientRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClientSummary {
    pub client_id: String,
    pub client_name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ClientUsageSummary {
    pub client_id: String,
    pub client_name: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub last_used_at: Option<chrono::DateTime<Utc>>,
}

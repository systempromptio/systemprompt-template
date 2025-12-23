use anyhow::Result;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use systemprompt_core_database::{Database, DbPool};

static TEST_DB: OnceCell<DbPool> = OnceCell::new();

pub struct TestDb {
    pool: DbPool,
}

impl TestDb {
    pub async fn new() -> Result<Self> {
        let pool = get_or_init_db().await?;
        Ok(Self { pool })
    }

    pub fn db_pool(&self) -> DbPool {
        self.pool.clone()
    }
}

async fn get_or_init_db() -> Result<DbPool> {
    if let Some(pool) = TEST_DB.get() {
        return Ok(pool.clone());
    }

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test".to_string());

    let db = Database::new_postgres(&database_url).await?;
    let pool: DbPool = Arc::new(db);

    let _ = TEST_DB.set(pool.clone());

    Ok(pool)
}

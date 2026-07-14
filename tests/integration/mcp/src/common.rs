//! Throwaway-database harness. Each test provisions its own database on the
//! server named by `DATABASE_URL`, applies the minimal schema the audit writers
//! touch (`users` + `user_activity`), and drops it on teardown — the shared
//! application tables are never involved.

use std::sync::Arc;

use sqlx::PgPool;
use sqlx::AssertSqlSafe;
use systemprompt::database::{Database, DbPool};
use url::Url;

pub struct TempDb {
    pub pool: DbPool,
    pg: PgPool,
    admin_url: String,
    db_name: String,
}

fn database_url() -> String {
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for the integration tests")
}

/// Rebuild a connection URL against the same server but a different database.
fn with_database(base: &str, db_name: &str) -> String {
    let mut url = Url::parse(base).expect("DATABASE_URL is a valid URL");
    url.set_path(&format!("/{db_name}"));
    url.into()
}

impl TempDb {
    pub async fn create() -> Self {
        let base = database_url();
        // Maintenance connection lives on the `postgres` database; CREATE
        // DATABASE cannot run inside a transaction, so use a plain autocommit
        // execute here.
        let admin_url = with_database(&base, "postgres");
        let db_name = format!("mcp_ext_test_{}", uuid::Uuid::new_v4().simple());

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
        // Name is a UUID-derived literal, not user input — safe to interpolate.
        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{db_name}\"")))
            .execute(&admin)
            .await
            .expect("create throwaway database");
        admin.close().await;

        let child_url = with_database(&base, &db_name);
        let pg = PgPool::connect(&child_url)
            .await
            .expect("connect to throwaway database");

        // Minimal schema: only the columns the audit writers read/write.
        sqlx::query(
            r"CREATE TABLE users (
                id         TEXT PRIMARY KEY,
                email      TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(&pg)
        .await
        .expect("create users table");

        sqlx::query(
            r"CREATE TABLE user_activity (
                id          TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
                user_id     TEXT NOT NULL,
                category    TEXT NOT NULL,
                action      TEXT NOT NULL,
                entity_type TEXT,
                entity_id   TEXT,
                entity_name TEXT,
                description TEXT NOT NULL,
                metadata    JSONB DEFAULT '{}'::jsonb,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(&pg)
        .await
        .expect("create user_activity table");

        let pool: DbPool = Arc::new(Database::from_pools(Arc::new(pg.clone()), None));

        Self {
            pool,
            pg,
            admin_url,
            db_name,
        }
    }

    /// Insert a user row with the given id/email. Used to seed (or omit) the
    /// reserved anonymous principal.
    pub async fn insert_user(&self, id: &str, email: &str) {
        sqlx::query("INSERT INTO users (id, email) VALUES ($1, $2)")
            .bind(id)
            .bind(email)
            .execute(&self.pg)
            .await
            .expect("seed user");
    }

    /// All `mcp_access` rows recorded for a given `entity_name` (server or
    /// tool), returned as (user_id, action, entity_type, description).
    pub async fn mcp_rows(
        &self,
        entity_name: &str,
    ) -> Vec<(String, String, Option<String>, String)> {
        sqlx::query_as::<_, (String, String, Option<String>, String)>(
            r"SELECT user_id, action, entity_type, description
              FROM user_activity
              WHERE category = 'mcp_access' AND entity_name = $1",
        )
        .bind(entity_name)
        .fetch_all(&self.pg)
        .await
        .expect("query recorded activity")
    }

    pub async fn cleanup(self) {
        self.pg.close().await;
        let admin = PgPool::connect(&self.admin_url)
            .await
            .expect("reconnect maintenance database for drop");
        // Force-drop even if a stray connection lingers.
        sqlx::query(AssertSqlSafe(format!(
            "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
            self.db_name
        )))
        .execute(&admin)
        .await
        .expect("drop throwaway database");
        admin.close().await;
    }
}

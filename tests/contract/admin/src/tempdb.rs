//! Throwaway database with the full extension schema applied.
//!
//! Unlike the MCP integration harness, which hand-creates the two tables its
//! audit writers touch, the contract suite needs the real schema: it drives
//! every admin route, and a missing table is indistinguishable from the
//! handler bug the suite exists to catch. The schema therefore comes from the
//! same `install_extension_schemas` path the server runs at startup.
//!
//! Migrations are collected from `inventory` registrations, which only exist
//! for crates actually linked into this binary — hence the `use ... as _`
//! below. Dropping one silently yields a partial schema rather than an error.

use std::sync::Arc;

use sqlx::{AssertSqlSafe, PgPool};
use systemprompt::ExtensionRegistry;
use systemprompt::database::{Database, install_extension_schemas};
use url::Url;

use systemprompt_web_extension as _;

pub struct TempDb {
    pub pool: Arc<PgPool>,
    admin_url: String,
    db_name: String,
}

/// Maintenance-server URL, or `None` so the suite self-skips in environments
/// with no Postgres (the same contract the MCP integration harness offers).
///
/// Only throwaway `admin_contract_<uuid>` databases are ever created or
/// dropped; the database named in the URL is used solely to reach the server.
fn server_url() -> Option<String> {
    std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
}

fn with_database(base: &str, db_name: &str) -> String {
    let mut url = Url::parse(base).expect("DATABASE_URL is a valid URL");
    url.set_path(&format!("/{db_name}"));
    url.into()
}

impl TempDb {
    pub async fn create() -> Option<Self> {
        let base = server_url()?;
        let admin_url = with_database(&base, "postgres");
        let db_name = format!("admin_contract_{}", uuid::Uuid::new_v4().simple());

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
        // Name is a UUID-derived literal, not user input — safe to interpolate.
        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{db_name}\"")))
            .execute(&admin)
            .await
            .expect("create throwaway database");
        admin.close().await;

        let pool = Arc::new(
            PgPool::connect(&with_database(&base, &db_name))
                .await
                .expect("connect to throwaway database"),
        );

        let database = Database::from_pools(Arc::clone(&pool), Some(Arc::clone(&pool)));
        let registry = ExtensionRegistry::discover().expect("discover extension registrations");
        assert!(
            !registry.is_empty(),
            "no extensions registered — the contract binary must link the crates whose \
             `register_extension!` supplies the migrations"
        );
        install_extension_schemas(&registry, database.write())
            .await
            .expect("install extension schemas");

        Some(Self {
            pool,
            admin_url,
            db_name,
        })
    }

    pub async fn cleanup(self) {
        self.pool.close().await;
        let admin = PgPool::connect(&self.admin_url)
            .await
            .expect("reconnect maintenance database for drop");
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

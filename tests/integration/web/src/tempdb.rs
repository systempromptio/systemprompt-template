//! Throwaway database with the full extension schema applied.
//!
//! The content repositories query core's content-domain tables
//! (`markdown_content`, `campaign_links`, `link_clicks`), so a hand-rolled
//! minimal schema would drift from the real one the queries were compiled
//! against. The schema therefore comes from the same
//! `install_extension_schemas` path the server runs at startup.
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

// Maintenance-server URL, or `None` so the suite self-skips in environments
// with no Postgres.
//
// Refuses to run against a development database: only database names `test`,
// `postgres`, or `*_test` are accepted, so a stray `DATABASE_URL` pointing at
// a dev server's live database panics instead of being used as the
// maintenance connection. Only throwaway `web_int_test_<uuid>` databases are
// ever created or dropped.
fn server_url() -> Option<String> {
    let raw = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()?;
    let parsed = Url::parse(&raw).expect("test database URL must be a valid URL");
    let db_name = parsed.path().trim_start_matches('/');
    let allowed = db_name == "test" || db_name == "postgres" || db_name.ends_with("_test");
    assert!(
        allowed,
        "Refusing to run integration tests against database '{db_name}'. Set \
         SYSTEMPROMPT_TEST_DATABASE_URL to a database reserved for tests: the database name \
         must be 'test', 'postgres', or end in '_test'."
    );
    Some(raw)
}

fn with_database(base: &str, db_name: &str) -> String {
    let mut url = Url::parse(base).expect("DATABASE_URL is a valid URL");
    url.set_path(&format!("/{db_name}"));
    url.into()
}

impl TempDb {
    pub async fn create() -> Option<Self> {
        let base = server_url()?;
        // CREATE DATABASE cannot run inside a transaction, so the maintenance
        // connection lives on `postgres` and executes autocommit.
        let admin_url = with_database(&base, "postgres");
        let db_name = format!("web_int_test_{}", uuid::Uuid::new_v4().simple());

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
            "no extensions registered — the integration binary must link the crates whose \
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

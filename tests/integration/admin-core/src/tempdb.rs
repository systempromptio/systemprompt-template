//! Throwaway database with the full extension schema applied.
//!
//! The admin repositories query core's identity and gateway tables (`users`,
//! `ai_requests`, `governance_decisions`) alongside the web extension's own
//! (`user_profile_ext`, `departments`, `plugin_usage_*`), so a
//! hand-rolled minimal schema would drift from the real one the queries were
//! compiled against. The schema therefore comes from the same
//! `install_extension_schemas` path the server runs at startup — migrations
//! included, which is why the resulting database is *seeded*, not empty (see
//! `fixtures`).
//!
//! Migrations are collected from `inventory` registrations, which only exist
//! for crates actually linked into this binary — hence the `use ... as _`
//! below. Dropping one silently yields a partial schema rather than an error.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use sqlx::{AssertSqlSafe, PgPool};
use systemprompt::ExtensionRegistry;
use systemprompt::database::{Database, install_extension_schemas};
use url::Url;

use systemprompt_web_admin as _;
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
// maintenance connection. Only throwaway `admin_core_test_<uuid>` databases
// are ever created or dropped.
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

// Why: installing the schema is a full migration run plus the seed rows those
// migrations insert — around a second of work, paid once per test, for a
// result that is byte-identical every time. `CREATE DATABASE ... TEMPLATE` is
// a file copy of an already-installed database instead, which is the same
// schema and the same seeds for a fraction of the cost.
//
// The template has to live in Postgres rather than in a process-local cache:
// nextest runs every test in its own process, so a cache that did not outlive
// the process would never once be hit.
const TEMPLATE_PREFIX: &str = "admin_core_test_tpl_";

// Why: the schema is compiled into this binary, so a rebuild is exactly the
// event that can invalidate a template — and keying on the binary rather than
// on the schema text means a change to the schema cannot be missed, only
// over-reported. An over-report costs one rebuild of the template.
fn template_name() -> String {
    let exe = std::env::current_exe().expect("locate the running test binary");
    let meta = std::fs::metadata(&exe).expect("stat the running test binary");
    let mut hasher = DefaultHasher::new();
    meta.len().hash(&mut hasher);
    meta.modified().ok().hash(&mut hasher);
    format!("{TEMPLATE_PREFIX}{:016x}", hasher.finish())
}

// Why: a 64-bit key for `pg_advisory_lock`, so concurrent test processes that
// find the template missing build it once between them rather than racing to
// `CREATE DATABASE` the same name.
fn advisory_key(template: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    template.hash(&mut hasher);
    hasher.finish() as i64
}

async fn database_exists(conn: &mut sqlx::PgConnection, name: &str) -> bool {
    sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
        .bind(name)
        .fetch_one(conn)
        .await
        .expect("query pg_database")
}

// Why: templates from earlier binaries are dead weight, but a concurrent run
// may still be copying one, and `DROP DATABASE` on a database in use is an
// error rather than a wait. Failures are therefore ignored: the next run
// collects whatever this one could not.
async fn drop_stale_templates(conn: &mut sqlx::PgConnection, keep: &str) {
    let Ok(stale) = sqlx::query_scalar::<_, String>(
        // `LIKE` would read the underscores in the prefix as wildcards, so the
        // match is a plain prefix comparison instead.
        "SELECT datname FROM pg_database WHERE left(datname, length($1)) = $1 AND datname <> $2",
    )
    .bind(TEMPLATE_PREFIX)
    .bind(keep)
    .fetch_all(&mut *conn)
    .await
    else {
        return;
    };
    for name in stale {
        let _ = sqlx::query(AssertSqlSafe(format!("DROP DATABASE IF EXISTS \"{name}\"")))
            .execute(&mut *conn)
            .await;
    }
}

async fn ensure_template(admin: &PgPool, base: &str, template: &str) {
    let mut conn = admin.acquire().await.expect("maintenance connection");
    // The lock is session-scoped, so every statement below has to run on this
    // one connection — a pool would hand the unlock to a different session.
    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(advisory_key(template))
        .execute(&mut *conn)
        .await
        .expect("take the template build lock");

    if !database_exists(&mut conn, template).await {
        sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{template}\"")))
            .execute(&mut *conn)
            .await
            .expect("create the template database");
        let pool = Arc::new(
            PgPool::connect(&with_database(base, template))
                .await
                .expect("connect to the template database"),
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
        // The copy refuses to run while any session holds the template open, so
        // this close is load-bearing, not tidiness.
        pool.close().await;
        drop_stale_templates(&mut conn, template).await;
    }

    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(advisory_key(template))
        .execute(&mut *conn)
        .await
        .expect("release the template build lock");
}

// Why: `CREATE DATABASE ... TEMPLATE` fails outright, rather than waiting, if
// any session is still connected to the source. The closing connection of a
// process that has just built the template can linger for a few milliseconds,
// so the copy is retried briefly before it is called a failure.
async fn copy_template(admin: &PgPool, template: &str, db_name: &str) {
    let statement = format!("CREATE DATABASE \"{db_name}\" TEMPLATE \"{template}\"");
    let mut last = None;
    for _ in 0..50 {
        match sqlx::query(AssertSqlSafe(statement.clone()))
            .execute(admin)
            .await
        {
            Ok(_) => return,
            Err(e) => {
                last = Some(e);
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            },
        }
    }
    panic!(
        "could not copy the template database: {}",
        last.expect("a failure after the retry budget")
    );
}

impl TempDb {
    pub async fn create() -> Option<Self> {
        let base = server_url()?;
        // CREATE DATABASE cannot run inside a transaction, so the maintenance
        // connection lives on `postgres` and executes autocommit.
        let admin_url = with_database(&base, "postgres");
        let db_name = format!("admin_core_test_{}", uuid::Uuid::new_v4().simple());

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
        let template = template_name();
        ensure_template(&admin, &base, &template).await;
        // Name is a UUID-derived literal, not user input — safe to interpolate.
        copy_template(&admin, &template, &db_name).await;
        admin.close().await;

        let pool = Arc::new(
            PgPool::connect(&with_database(&base, &db_name))
                .await
                .expect("connect to throwaway database"),
        );

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

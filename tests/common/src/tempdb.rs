//! Throwaway database with the full extension schema applied.
//!
//! Every DB-backed suite in this repository used to carry its own copy of this
//! file. They drifted: four applied the real schema and one hand-rolled two
//! tables, and only one of the five refused to skip under `CI` -- so four
//! tiers could report green with no Postgres at all. This is the single copy,
//! and the `CI` guard is its only behaviour on a missing database.
//!
//! Each suite still gets its own databases and its own template: the
//! name prefix is derived from the running test binary, so two suites cannot
//! collide and neither can two builds of the same suite.
//!
//! The schema comes from the same `install_extension_schemas` path the server
//! runs at startup, so it cannot drift from the one the queries were compiled
//! against. Migrations are collected from `inventory` registrations, which
//! exist only for crates linked into the binary -- hence the `use ... as _`
//! below, and the extension dependencies in this crate's manifest.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use sqlx::{AssertSqlSafe, PgPool};
use systemprompt::database::{Database, DbPool, install_extension_schemas};
use systemprompt::extension::ExtensionRegistry;
use url::Url;

use systemprompt_web_admin as _;
use systemprompt_web_extension as _;

use crate::skip::skip_or_panic;

pub struct TempDb {
    pub pool: Arc<PgPool>,
    admin_url: String,
    db_name: String,
}

// Maintenance-server URL, or `None` so a developer machine with no Postgres
// still runs the rest of the suite. Under `CI` the absence is a panic.
//
// Refuses to run against a development database: only database names `test`,
// `postgres`, or `*_test` are accepted, so a stray `DATABASE_URL` pointing at
// a dev server's live database panics instead of being used as the
// maintenance connection. Only throwaway `<prefix>_<uuid>` databases are ever
// created or dropped.
fn server_url() -> Option<String> {
    let raw = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|u| !u.trim().is_empty());
    let Some(raw) = raw else {
        skip_or_panic(
            "SYSTEMPROMPT_TEST_DATABASE_URL",
            "DB-backed suites need a maintenance Postgres URL",
        );
        return None;
    };
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
// migrations insert -- around a second of work, paid once per test, for a
// result that is byte-identical every time. `CREATE DATABASE ... TEMPLATE` is
// a file copy of an already-installed database instead, which is the same
// schema and the same seeds for a fraction of the cost.
//
// The template has to live in Postgres rather than in a process-local cache:
// nextest runs every test in its own process, so a cache that did not outlive
// the process would never once be hit.
//
// Why: the schema is compiled into this binary, so a rebuild is exactly the
// event that can invalidate a template -- and keying on the binary rather than
// on the schema text means a change to the schema cannot be missed, only
// over-reported. An over-report costs one rebuild of the template.
fn template_name(prefix: &str) -> String {
    let exe = std::env::current_exe().expect("locate the running test binary");
    let meta = std::fs::metadata(&exe).expect("stat the running test binary");
    let mut hasher = DefaultHasher::new();
    meta.len().hash(&mut hasher);
    meta.modified().ok().hash(&mut hasher);
    format!("{prefix}_tpl_{:016x}", hasher.finish())
}

// Why: the prefix names the suite, so no call site has to pass one and no two
// suites can share a template. Cargo appends a build hash to the binary name,
// which is dropped: it changes on every rebuild, and the template already
// keys on the binary's own fingerprint.
fn suite_prefix() -> String {
    let exe = std::env::current_exe().expect("locate the running test binary");
    let stem = exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("systemprompt_test")
        .to_owned();
    let stem = stem
        .rsplit_once('-')
        .filter(|(_, hash)| hash.len() == 16 && hash.chars().all(|c| c.is_ascii_hexdigit()))
        .map_or(stem.as_str(), |(head, _)| head);
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    cleaned.chars().take(24).collect()
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
async fn drop_stale_templates(conn: &mut sqlx::PgConnection, prefix: &str, keep: &str) {
    let Ok(stale) = sqlx::query_scalar::<_, String>(
        // `LIKE` would read the underscores in the prefix as wildcards, so the
        // match is a plain prefix comparison instead.
        "SELECT datname FROM pg_database WHERE left(datname, length($1)) = $1 AND datname <> $2",
    )
    .bind(format!("{prefix}_tpl_"))
    .bind(keep)
    .fetch_all(&mut *conn)
    .await
    else {
        return; // skip-ok: a stale template that cannot be listed is collected next run
    };
    for name in stale {
        let _ = sqlx::query(AssertSqlSafe(format!("DROP DATABASE IF EXISTS \"{name}\"")))
            .execute(&mut *conn)
            .await;
    }
}

async fn ensure_template(admin: &PgPool, base: &str, prefix: &str, template: &str) {
    // A dependency alone does not keep an inventory section alive under link-time
    // garbage collection. Referencing the extension value makes its submitted
    // evaluation migrations part of every shared test-schema binary.
    let mut conn = admin.acquire().await.expect("maintenance connection");
    // The lock is session-scoped, so every statement below has to run on this
    // one connection -- a pool would hand the unlock to a different session.
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
            "no extensions registered — the test binary must link the crates whose \
             `register_extension!` supplies the migrations"
        );
        install_extension_schemas(&registry, database.write())
            .await
            .expect("install extension schemas");
        // The copy refuses to run while any session holds the template open, so
        // this close is load-bearing, not tidiness.
        pool.close().await;
        drop_stale_templates(&mut conn, prefix, template).await;
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
        let prefix = &suite_prefix();
        let base = server_url()?;
        // CREATE DATABASE cannot run inside a transaction, so the maintenance
        // connection lives on `postgres` and executes autocommit.
        let admin_url = with_database(&base, "postgres");
        let db_name = format!("{prefix}_{}", uuid::Uuid::new_v4().simple());

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
        let template = template_name(prefix);
        ensure_template(&admin, &base, prefix, &template).await;
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

    // Why: a database with no schema at all, for the suites that install their
    // own. Skipping the template is the point -- an empty database is faster to
    // make than a copy, and the suite's assertions are about the tables it
    // creates itself.
    pub async fn create_empty() -> Option<Self> {
        let prefix = &suite_prefix();
        let base = server_url()?;
        let admin_url = with_database(&base, "postgres");
        let db_name = format!("{prefix}_{}", uuid::Uuid::new_v4().simple());

        let admin = PgPool::connect(&admin_url)
            .await
            .expect("connect to maintenance database");
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

        Some(Self {
            pool,
            admin_url,
            db_name,
        })
    }

    #[must_use]
    pub fn db_pool(&self) -> DbPool {
        Arc::new(Database::from_pools(Arc::clone(&self.pool), None))
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

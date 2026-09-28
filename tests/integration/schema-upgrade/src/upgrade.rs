//! Restore each released schema on the ladder, seed its hot tables, run the
//! current installer over it, and compare the result with a fresh install.
//!
//! The seed is what makes this the path a real instance takes: a schema-only
//! rung ran no backfill against rows and fired no trigger, so an upgrade that
//! passed here could still fail on the first populated database.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::database::install_extension_schemas;
use systemprompt::extension::ExtensionRegistry;
use systemprompt_marketplace as _;
use systemprompt_users as _;
use systemprompt_web_content as _;
use template_test_common::{TempDb, db_or_skip, empty_db_or_skip, repo_path};

use crate::catalog;

const FIXTURE_DIR: &str = "tests/fixtures/schema";
const FIXTURE_PREFIX: &str = "release-baseline-";
const SEED_HOT_TABLES: &str = include_str!("seed_hot_tables.sql");
const SEEDED_ROWS: i64 = 2000;

struct Rung {
    version: String,
    sql: String,
}

fn strip_meta_commands(raw: &str) -> String {
    // Why: pg_dump emits `\restrict` / `\unrestrict` psql meta-commands and a
    // `set_config('search_path', '')` that would stick to the connection the
    // installer then uses; the recorder strips both, this is belt and braces.
    raw.lines()
        .filter(|l| !l.starts_with('\\') && !l.contains("set_config('search_path'"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn semver_key(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|p| p.parse::<u64>().expect("release version component"))
        .collect()
}

// Why: every release since the floor is a rung; the gate
// (scripts/check-schema-baseline.sh) keeps the set complete, this reads it.
fn ladder() -> Vec<Rung> {
    let dir: PathBuf = repo_path(FIXTURE_DIR);
    let mut rungs: Vec<Rung> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|entry| {
            let path = entry.expect("fixture dir entry").path();
            let name = path.file_name()?.to_str()?.to_owned();
            let version = name
                .strip_prefix(FIXTURE_PREFIX)?
                .strip_suffix(".sql")?
                .to_owned();
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            Some(Rung {
                version,
                sql: strip_meta_commands(&raw),
            })
        })
        .collect();
    assert!(
        !rungs.is_empty(),
        "no release-baseline-*.sql under {} — run 'just schema-baseline'",
        dir.display()
    );
    rungs.sort_by_key(|r| semver_key(&r.version));
    rungs
}

// Why: core's content and managed-resources extensions register their
// migrations through the injected list rather than `inventory`, so the
// installer sees the set the server runs at boot only once they are injected.
fn discover_registry() -> ExtensionRegistry {
    let _ = std::hint::black_box(systemprompt_content::ContentExtension);
    let _ = systemprompt::extension::runtime_config::set_injected_extensions(
        systemprompt::extension::runtime_config::InjectedExtensions {
            extensions: vec![
                Arc::new(systemprompt_content::ContentExtension),
                Arc::new(systemprompt_marketplace::ManagedResourcesExtension),
            ],
            ..Default::default()
        },
    );
    let registry = ExtensionRegistry::discover().expect("discover extension registrations");
    assert!(
        !registry.is_empty(),
        "no extensions linked into the test binary"
    );
    registry
}

// Why: restore the rung, seed its hot tables and upgrade it with the current
// installer; the caller owns the database.
async fn restore_and_upgrade(db: &TempDb, rung: &Rung) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(rung.sql.clone()))
        .execute(&*db.pool)
        .await
        .unwrap_or_else(|e| panic!("restore the release {} schema: {e}", rung.version));
    sqlx::raw_sql(SEED_HOT_TABLES)
        .execute(&*db.pool)
        .await
        .unwrap_or_else(|e| panic!("seed the release {} hot tables: {e}", rung.version));
    let registry = discover_registry();
    let database = db.db_pool();
    if let Err(e) = install_extension_schemas(&registry, database.write()).await {
        panic!(
            "the current installer cannot upgrade a database left by release {}:\n{e}\n\n\
             This is the path every deployed instance takes and no other tier exercises. \
             Fix the declarative schema or add the migration the established database needs.",
            rung.version
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_release_schema_upgrades_with_the_current_installer() {
    for rung in ladder() {
        let db = empty_db_or_skip!();
        restore_and_upgrade(&db, &rung).await;
        for table in ["ai_requests", "ai_request_messages", "plugin_usage_events"] {
            let rows: i64 =
                sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
                    .fetch_one(&*db.pool)
                    .await
                    .unwrap_or_else(|e| {
                        panic!("count {table} after upgrading {}: {e}", rung.version)
                    });
            assert_eq!(
                rows, SEEDED_ROWS,
                "upgrading release {} changed the row count of {table}",
                rung.version
            );
        }
        db.cleanup().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_upgraded_schema_matches_a_fresh_install() {
    let fresh = db_or_skip!();
    let fresh_shape = catalog::snapshot(&fresh.pool).await;
    fresh.cleanup().await;

    for rung in ladder() {
        let upgraded = empty_db_or_skip!();
        restore_and_upgrade(&upgraded, &rung).await;
        let upgraded_shape = catalog::snapshot(&upgraded.pool).await;
        upgraded.cleanup().await;

        let diff = catalog::diff(&upgraded_shape, &fresh_shape);
        assert!(
            diff.is_empty(),
            "a database upgraded from release {} differs from a fresh install \
             (- only after upgrade, + only on fresh):\n{diff}\n\
             A migration and the declarative baseline disagree — fix the migration (existing \
             databases) or the baseline (fresh installs); never both by hand.",
            rung.version
        );
    }
}

#[test]
fn every_rung_header_names_its_release() {
    for rung in ladder() {
        let recorded = rung
            .sql
            .lines()
            .next()
            .and_then(|l| l.split(" release-baseline: ").nth(1))
            .and_then(|l| l.split_whitespace().next())
            .unwrap_or_default();
        assert_eq!(
            recorded, rung.version,
            "release-baseline-{}.sql records a different release in its header",
            rung.version
        );
    }
}

//! Restore the previous release's schema, run the current installer over it,
//! and compare the result with a fresh install.

use systemprompt::extension::ExtensionRegistry;
use systemprompt::database::install_extension_schemas;
use template_test_common::{TempDb, db_or_skip, empty_db_or_skip, repo_path};

use crate::catalog;

const FIXTURE: &str = "tests/fixtures/schema/release-baseline.sql";

fn fixture_sql() -> String {
    let path = repo_path(FIXTURE);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e} — run 'just schema-baseline'", path.display()));
    // Why: pg_dump emits `\restrict` / `\unrestrict` psql meta-commands and a
    // `set_config('search_path', '')` that would stick to the connection the
    // installer then uses; the recorder strips both, this is belt and braces.
    raw.lines()
        .filter(|l| !l.starts_with('\\') && !l.contains("set_config('search_path'"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fixture_version() -> String {
    fixture_sql()
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("-- systemprompt-systemprompt release-baseline: "))
        .and_then(|l| l.split_whitespace().next())
        .map(str::to_owned)
        .expect("fixture header names the release it records")
}

// Why: restore the fixture and upgrade it with the current installer; the
// caller owns the database.
async fn restore_and_upgrade(db: &TempDb) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(fixture_sql()))
        .execute(&*db.pool)
        .await
        .expect("restore the previous release's schema");
    let registry = ExtensionRegistry::discover().expect("discover extension registrations");
    assert!(
        !registry.is_empty(),
        "no extensions linked into the test binary"
    );
    let database = db.db_pool();
    if let Err(e) = install_extension_schemas(&registry, database.write()).await {
        panic!(
            "the current installer cannot upgrade a database left by release {}:\n{e}\n\n\
             This is the path every deployed instance takes and no other tier exercises. \
             Fix the declarative schema or add the migration the established database needs.",
            fixture_version()
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn previous_release_schema_upgrades_with_the_current_installer() {
    let db = empty_db_or_skip!();
    restore_and_upgrade(&db).await;
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn upgraded_schema_matches_a_fresh_install() {
    let upgraded = empty_db_or_skip!();
    restore_and_upgrade(&upgraded).await;
    let fresh = db_or_skip!();

    let upgraded_shape = catalog::snapshot(&upgraded.pool).await;
    let fresh_shape = catalog::snapshot(&fresh.pool).await;
    let diff = catalog::diff(&upgraded_shape, &fresh_shape);

    upgraded.cleanup().await;
    fresh.cleanup().await;

    assert!(
        diff.is_empty(),
        "a database upgraded from release {} differs from a fresh install \
         (- only after upgrade, + only on fresh):\n{diff}\n\
         A migration and the declarative baseline disagree — fix the migration (existing \
         databases) or the baseline (fresh installs); never both by hand.",
        fixture_version()
    );
}

#[test]
fn fixture_header_names_a_release() {
    let version = fixture_version();
    assert!(
        version.split('.').count() == 3 && version.chars().all(|c| c.is_ascii_digit() || c == '.'),
        "{version}"
    );
}

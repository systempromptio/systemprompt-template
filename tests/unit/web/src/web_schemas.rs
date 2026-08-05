//! Schema DDL and migrations are the only things that run against a live
//! database before any request is served, so a mistake here is a boot failure
//! rather than a page bug. `include_str!` makes an empty or missing `.sql` file
//! compile fine, and migration versions are discovered by `build.rs` — both are
//! silent, so emptiness and version uniqueness are asserted directly.

use std::collections::HashSet;
use systemprompt_web_extension::schemas::{migrations, schema_definitions};

#[test]
fn every_schema_definition_carries_non_empty_ddl() {
    let definitions = schema_definitions();
    assert!(!definitions.is_empty(), "no schema DDL is registered");

    for definition in &definitions {
        assert!(
            !definition.sql.trim().is_empty(),
            "a schema definition embedded an empty .sql file"
        );
    }
}

#[test]
fn schema_ddl_is_not_registered_twice() {
    let definitions = schema_definitions();
    let unique: HashSet<&str> = definitions.iter().map(|d| d.sql.as_str()).collect();

    assert_eq!(
        unique.len(),
        definitions.len(),
        "the same DDL file is registered under two entries"
    );
}

#[test]
fn migrations_have_unique_increasing_versions_and_non_empty_sql() {
    let migrations = migrations();

    let versions: Vec<u32> = migrations.iter().map(|m| m.version).collect();
    let unique: HashSet<u32> = versions.iter().copied().collect();
    assert_eq!(
        unique.len(),
        versions.len(),
        "two migrations share a version: {versions:?}"
    );

    let mut sorted = versions.clone();
    sorted.sort_unstable();
    assert_eq!(
        versions, sorted,
        "migrations must be handed to the runner in version order"
    );

    let names: HashSet<&str> = migrations.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names.len(), migrations.len(), "two migrations share a name");

    for migration in &migrations {
        assert!(
            !migration.sql.trim().is_empty(),
            "migration {} ({}) has no SQL",
            migration.version,
            migration.name
        );
    }
}

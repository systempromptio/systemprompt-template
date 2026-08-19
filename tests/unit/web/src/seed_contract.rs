//! Boot-seed contract checks for `extensions/web/schema/seeds/*.sql`.
//!
//! The installer applies these on every boot and rejects anything that is not
//! an idempotent INSERT (with ON CONFLICT), UPDATE, or MERGE. Catching a
//! violation here fails `cargo test` instead of failing the next boot.

const ADMIN_OAUTH_CLIENT: &str =
    include_str!("../../../../extensions/web/schema/seeds/admin_oauth_client.sql");
const MARKETPLACE_PLANS: &str =
    include_str!("../../../../extensions/web/schema/seeds/marketplace_plans.sql");
const DEFAULT_DEPARTMENT: &str =
    include_str!("../../../../extensions/web/schema/seeds/default_department.sql");

const ALL_SEEDS: [(&str, &str); 3] = [
    ("admin_oauth_client", ADMIN_OAUTH_CLIENT),
    ("marketplace_plans", MARKETPLACE_PLANS),
    ("default_department", DEFAULT_DEPARTMENT),
];

fn statements(sql: &str) -> Vec<String> {
    let without_comments: String = sql
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    without_comments
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_uppercase)
        .collect()
}

#[test]
fn every_seed_is_idempotent_sql() {
    for (id, sql) in ALL_SEEDS {
        let stmts = statements(sql);
        assert!(!stmts.is_empty(), "seed {id} has no statements");
        for stmt in &stmts {
            assert!(
                stmt.starts_with("INSERT")
                    || stmt.starts_with("UPDATE")
                    || stmt.starts_with("MERGE"),
                "seed {id}: statements must be INSERT/UPDATE/MERGE, found: {}...",
                &stmt[..stmt.len().min(60)]
            );
            if stmt.starts_with("INSERT") {
                assert!(
                    stmt.contains("ON CONFLICT"),
                    "seed {id}: INSERT without ON CONFLICT is not idempotent"
                );
            }
        }
    }
}

#[test]
fn plan_and_department_seeds_never_overwrite_operator_edits() {
    for (id, sql) in [
        ("marketplace_plans", MARKETPLACE_PLANS),
        ("default_department", DEFAULT_DEPARTMENT),
    ] {
        for stmt in statements(sql) {
            assert!(
                stmt.contains("DO NOTHING"),
                "seed {id}: inserts must be insert-if-absent (ON CONFLICT ... DO NOTHING) so an \
                 operator's edits survive every boot"
            );
        }
    }
}

#[test]
fn child_table_inserts_guard_on_client_existence() {
    let upper = ADMIN_OAUTH_CLIENT.to_uppercase();
    for table in [
        "OAUTH_CLIENT_GRANT_TYPES",
        "OAUTH_CLIENT_RESPONSE_TYPES",
        "OAUTH_CLIENT_SCOPES",
        "OAUTH_CLIENT_REDIRECT_URIS",
    ] {
        let Some(idx) = upper.find(table) else {
            continue;
        };
        let tail = &upper[idx..];
        let stmt_end = tail.find(';').unwrap_or(tail.len());
        assert!(
            tail[..stmt_end].contains("WHERE EXISTS"),
            "insert into {table} must guard on the parent client row existing, or boot fails \
             before any admin user is created"
        );
    }
}

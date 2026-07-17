//! Boot-seed contract checks for `extensions/web/schema/seeds/*.sql`.
//!
//! The installer applies these on every boot and rejects anything that is not
//! an idempotent INSERT (with ON CONFLICT), UPDATE, or MERGE. Catching a
//! violation here fails `cargo test` instead of failing the next boot.

const ADMIN_OAUTH_CLIENT: &str =
    include_str!("../../../../extensions/web/schema/seeds/admin_oauth_client.sql");

fn statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_uppercase)
        .collect()
}

#[test]
fn admin_oauth_client_seed_is_idempotent_sql() {
    let stmts = statements(ADMIN_OAUTH_CLIENT);
    assert!(!stmts.is_empty());
    for stmt in &stmts {
        assert!(
            stmt.starts_with("INSERT") || stmt.starts_with("UPDATE") || stmt.starts_with("MERGE"),
            "seed statements must be INSERT/UPDATE/MERGE, found: {}...",
            &stmt[..stmt.len().min(60)]
        );
        if stmt.starts_with("INSERT") {
            assert!(
                stmt.contains("ON CONFLICT"),
                "INSERT without ON CONFLICT is not idempotent"
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

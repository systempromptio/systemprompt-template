//! Fail-closed rejection audit: `record_mcp_access_rejected` writes a row only
//! when the reserved `%@anonymous.local` principal exists, and drops it (no
//! row, no panic) otherwise — never attributing a rejection to an arbitrary
//! user.

use systemprompt_mcp_shared::{MAX_REASON_LEN, record_mcp_access_rejected};

use crate::common::TempDb;

const SERVER: &str = "systemprompt";
const TOOL: &str = "dangerous_tool";

#[tokio::test]
async fn drops_row_when_no_anonymous_principal_exists() {
    let db = TempDb::create().await;
    // No users at all — the anonymous lookup returns None.

    record_mcp_access_rejected(&db.pool, SERVER, TOOL, "scope denied").await;

    let rows = db.mcp_rows(SERVER).await;
    assert!(
        rows.is_empty(),
        "rejection must be dropped when no anonymous principal exists, got {rows:?}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn does_not_fall_back_to_an_arbitrary_user() {
    let db = TempDb::create().await;
    // A real user exists, but NOT an anonymous one — the row must still drop.
    db.insert_user("real-user", "person@example.com").await;

    record_mcp_access_rejected(&db.pool, SERVER, TOOL, "blocklist hit").await;

    let rows = db.mcp_rows(SERVER).await;
    assert!(
        rows.is_empty(),
        "must not attribute a rejection to a non-anonymous user, got {rows:?}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn attributes_row_to_anonymous_principal_when_present() {
    let db = TempDb::create().await;
    db.insert_user("anon-1", "fp_abc@anonymous.local").await;

    record_mcp_access_rejected(&db.pool, SERVER, TOOL, "scope denied").await;

    let rows = db.mcp_rows(SERVER).await;
    assert_eq!(rows.len(), 1, "expected one attributed rejection row");
    let (user_id, action, entity_type, description) = &rows[0];
    assert_eq!(user_id, "anon-1", "attributed to the anonymous principal");
    assert_eq!(action, "rejected");
    assert_eq!(entity_type.as_deref(), Some("mcp_server"));
    assert_eq!(description, "Access rejected on systemprompt: scope denied");

    db.cleanup().await;
}

#[tokio::test]
async fn long_reason_is_truncated_in_the_description() {
    let db = TempDb::create().await;
    db.insert_user("anon-2", "fp_def@anonymous.local").await;

    let reason = "x".repeat(MAX_REASON_LEN + 40);
    record_mcp_access_rejected(&db.pool, SERVER, TOOL, &reason).await;

    let rows = db.mcp_rows(SERVER).await;
    assert_eq!(rows.len(), 1);
    let description = &rows[0].3;
    let prefix = "Access rejected on systemprompt: ";
    let reason_part = description
        .strip_prefix(prefix)
        .expect("description carries the fixed prefix");
    assert!(reason_part.ends_with("..."), "long reason should be suffixed");
    // Reason body capped at MAX_REASON_LEN, plus the "..." suffix.
    assert_eq!(reason_part.len(), MAX_REASON_LEN + 3);

    db.cleanup().await;
}

//! `record_mcp_access` writes an attributed `mcp_access` audit row, shaping
//! `entity_type`/`entity_name` by action.

use systemprompt_mcp_shared::record_mcp_access;

use crate::common::TempDb;

#[tokio::test]
async fn used_action_records_tool_scoped_row() {
    let db = TempDb::create().await;
    db.insert_user("user-1", "dev@example.com").await;

    record_mcp_access(&db.pool, "user-1", "systemprompt", "list_skills", "used").await;

    let rows = db.mcp_rows("list_skills").await;
    assert_eq!(rows.len(), 1, "exactly one activity row expected");
    let (user_id, action, entity_type, description) = &rows[0];
    assert_eq!(user_id, "user-1");
    assert_eq!(action, "used");
    assert_eq!(entity_type.as_deref(), Some("tool"));
    assert_eq!(description, "Executed 'list_skills' on systemprompt");

    db.cleanup().await;
}

#[tokio::test]
async fn authenticated_action_records_server_scoped_row() {
    let db = TempDb::create().await;
    db.insert_user("user-2", "dev2@example.com").await;

    record_mcp_access(&db.pool, "user-2", "systemprompt", "list_skills", "authenticated").await;

    // For non-"used" actions the row is attributed to the server, not the tool.
    let rows = db.mcp_rows("systemprompt").await;
    assert_eq!(rows.len(), 1);
    let (user_id, action, entity_type, description) = &rows[0];
    assert_eq!(user_id, "user-2");
    assert_eq!(action, "authenticated");
    assert_eq!(entity_type.as_deref(), Some("mcp_server"));
    assert_eq!(description, "Authenticated to systemprompt for 'list_skills'");

    db.cleanup().await;
}

//! An intent the client never reported back has no execution, so
//! `tool_activity.execution_status` is NULL. The Tools page derives its
//! `failed` flag from that column, and SQL three-valued logic turns
//! `FALSE OR NULL` into NULL — a null the page's row type decodes into a
//! plain `bool` and cannot. `/admin/tools` sorts newest first, so one such
//! row on the first page took the whole page down with a 500.
//!
//! The statement itself never errors; only the Rust decode does. That is why
//! this asserts through `load_tool_activity_page` rather than over SQL.

use chrono::Utc;

use systemprompt_web_admin::repositories::analysis::tools::{
    ToolActivityFilter, ToolActivityPage, ToolBreakdownBy, ToolSort, ToolState,
    load_tool_activity_page,
};

use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn page() -> ToolActivityPage {
    ToolActivityPage {
        sort: ToolSort::Time,
        descending: true,
        limit: 50,
        offset: 0,
        breakdown: ToolBreakdownBy::Tool,
    }
}

#[tokio::test]
async fn an_intent_without_an_execution_decodes_as_not_failed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("tool-intent"), &unclaimed_email("intent")).await;
    let request_id = unique("req");
    insert_request(&db.pool, &RequestSpec::completed(&request_id, &user)).await;

    let tool_use_id = unique("toolu");
    sqlx::query(
        "INSERT INTO ai_request_tool_calls
             (request_id, tool_name, tool_input, ai_tool_call_id, sequence_number, created_at)
         VALUES ($1, 'Bash', '{\"command\":\"ls\"}', $2, 1, $3)",
    )
    .bind(&request_id)
    .bind(&tool_use_id)
    .bind(Utc::now())
    .execute(&*db.pool)
    .await
    .expect("insert tool intent");

    let data = load_tool_activity_page(&db.pool, &ToolActivityFilter::default(), page())
        .await
        .expect("the tools page decodes a row whose execution never landed");

    let row = data
        .rows
        .iter()
        .find(|r| r.ai_tool_call_id.as_deref() == Some(tool_use_id.as_str()))
        .expect("the intent is on the page");
    assert_eq!(row.state, "intended");
    assert!(!row.failed, "an intent that never ran has not failed");
    // The schema install seeds rows of its own, so this asserts on the row it
    // inserted and on the presence of its state, never on an empty table.
    assert!(data.totals.intended >= 1);
}

#[tokio::test]
async fn the_failed_filter_still_selects_a_failed_execution() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("tool-failed"), &unclaimed_email("failed")).await;
    let execution_id = unique("exec");
    sqlx::query(
        "INSERT INTO mcp_tool_executions
             (mcp_execution_id, tool_name, server_name, started_at, completed_at,
              execution_time_ms, input, status, user_id, source)
         VALUES ($1, 'search', 'atlassian', $2, $2, 12, '{}', 'failed', $3, 'proxy')",
    )
    .bind(&execution_id)
    .bind(Utc::now())
    .bind(user.as_str())
    .execute(&*db.pool)
    .await
    .expect("insert failed execution");

    let only_failed = load_tool_activity_page(
        &db.pool,
        &ToolActivityFilter {
            state: Some(ToolState::Failed),
            ..ToolActivityFilter::default()
        },
        page(),
    )
    .await
    .expect("load tools page filtered to failures");

    let row = only_failed
        .rows
        .iter()
        .find(|r| r.mcp_execution_id.as_deref() == Some(execution_id.as_str()))
        .expect("the failed execution is on the page");
    assert!(
        row.failed,
        "a timed-out or failed execution still reads failed"
    );
}

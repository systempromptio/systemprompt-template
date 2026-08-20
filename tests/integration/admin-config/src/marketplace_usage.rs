//! `repositories::marketplace::webhook` — plugin usage events.

use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_web_admin::repositories::marketplace::webhook::{
    UsageEventParams, insert_plugin_usage_event,
};

use crate::fixtures::{unique, user_id};
use crate::tempdb::TempDb;

fn usage_params<'a>(
    user: &'a UserId,
    session: &'a SessionId,
    metadata: &'a serde_json::Value,
    dedup_key: &'a str,
) -> UsageEventParams<'a> {
    UsageEventParams {
        user_id: user,
        session_id: session,
        event_type: "PostToolUse",
        tool_name: Some("Bash"),
        metadata,
        description: Some("ran a command"),
        prompt_preview: Some("list the files"),
        cwd: Some("/tmp"),
        dedup_key,
        content_input_bytes: 120,
        content_output_bytes: 340,
        loc_added: 5,
        loc_removed: 1,
    }
}

#[tokio::test]
async fn insert_plugin_usage_event_records_a_new_event() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = user_id(&unique("u"));
    let session = SessionId::new(unique("sess"));
    let metadata = serde_json::json!({ "exit_code": 0 });
    let dedup = unique("dedup");

    let inserted =
        insert_plugin_usage_event(&db.pool, &usage_params(&user, &session, &metadata, &dedup))
            .await
            .expect("insert usage event");

    assert!(inserted, "a fresh event is written");
    let row = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT event_type, tool_name, cwd FROM plugin_usage_events WHERE dedup_key = $1",
    )
    .bind(&dedup)
    .fetch_one(&*db.pool)
    .await
    .expect("read the event back");
    assert_eq!(row.0, "PostToolUse");
    assert_eq!(row.1.as_deref(), Some("Bash"));
    assert_eq!(row.2.as_deref(), Some("/tmp"));

    db.cleanup().await;
}

#[tokio::test]
async fn insert_plugin_usage_event_reports_a_replay_as_not_written() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = user_id(&unique("u"));
    let session = SessionId::new(unique("sess"));
    let metadata = serde_json::json!({});
    let dedup = unique("dedup");
    let params = usage_params(&user, &session, &metadata, &dedup);

    let first = insert_plugin_usage_event(&db.pool, &params)
        .await
        .expect("first insert");
    let second = insert_plugin_usage_event(&db.pool, &params)
        .await
        .expect("replayed insert");

    assert!(first);
    assert!(
        !second,
        "the hook retries, so a duplicate must be reported rather than stored"
    );
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM plugin_usage_events WHERE dedup_key = $1",
    )
    .bind(&dedup)
    .fetch_one(&*db.pool)
    .await
    .expect("count events");
    assert_eq!(count, 1);

    db.cleanup().await;
}

#[tokio::test]
async fn insert_plugin_usage_event_keeps_distinct_dedup_keys_apart() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = user_id(&unique("u"));
    let session = SessionId::new(unique("sess"));
    let metadata = serde_json::json!({});
    let first_key = unique("dedup");
    let second_key = unique("dedup");

    insert_plugin_usage_event(
        &db.pool,
        &usage_params(&user, &session, &metadata, &first_key),
    )
    .await
    .expect("first insert");
    let second = insert_plugin_usage_event(
        &db.pool,
        &usage_params(&user, &session, &metadata, &second_key),
    )
    .await
    .expect("second insert");

    assert!(second);
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM plugin_usage_events WHERE session_id = $1",
    )
    .bind(session.as_str())
    .fetch_one(&*db.pool)
    .await
    .expect("count events");
    assert_eq!(count, 2);

    db.cleanup().await;
}

#[tokio::test]
async fn insert_plugin_usage_event_stores_the_metadata_verbatim() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = user_id(&unique("u"));
    let session = SessionId::new(unique("sess"));
    let metadata = serde_json::json!({ "nested": { "tool": "Bash", "ok": true } });
    let dedup = unique("dedup");

    insert_plugin_usage_event(&db.pool, &usage_params(&user, &session, &metadata, &dedup))
        .await
        .expect("insert usage event");

    let stored = sqlx::query_scalar::<_, serde_json::Value>(
        "SELECT metadata FROM plugin_usage_events WHERE dedup_key = $1",
    )
    .bind(&dedup)
    .fetch_one(&*db.pool)
    .await
    .expect("read metadata back");
    assert_eq!(stored, metadata);

    db.cleanup().await;
}

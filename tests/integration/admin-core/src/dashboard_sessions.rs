//! `repositories::dashboard::hooks_track` — the session reads and writes the
//! hook endpoints depend on, plus the row builders the dashboard suite shares.

use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::dashboard::hooks_track;

use crate::fixtures::{EventSpec, insert_event, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;


pub async fn insert_mcp_execution(pool: &sqlx::PgPool, user: &str, tool: &str, status: &str) {
    sqlx::query(
        "INSERT INTO mcp_tool_executions
             (mcp_execution_id, tool_name, server_name, started_at, input, status, user_id)
         VALUES ($1, $2, 'test-server', NOW(), '{}', $3, $4)",
    )
    .bind(unique("exec"))
    .bind(tool)
    .bind(status)
    .bind(user)
    .execute(pool)
    .await
    .expect("insert mcp execution");
}

async fn insert_session_summary(pool: &sqlx::PgPool, session_id: &str, user_id: &str) {
    sqlx::query(
        "INSERT INTO plugin_session_summaries
             (id, session_id, user_id, started_at, total_events, tool_uses, prompts, errors,
              client_source, permission_mode, model)
         VALUES ($1, $2, $3, NOW(), 7, 3, 2, 1, 'claude-code', 'acceptEdits', 'claude-test')",
    )
    .bind(unique("summary"))
    .bind(session_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert session summary");
}

#[tokio::test]
async fn find_session_metrics_returns_none_for_an_unknown_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = SessionId::new(unique("session"));

    let metrics = hooks_track::find_session_metrics(&db.pool, &session).await;

    assert!(metrics.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn find_session_metrics_reads_the_summary_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("summary")).await;
    let session_id = unique("session");
    insert_session_summary(&db.pool, &session_id, user.as_str()).await;
    let session = SessionId::new(session_id);

    let metrics = hooks_track::find_session_metrics(&db.pool, &session)
        .await
        .expect("the summary exists");

    assert_eq!(metrics.prompts, 2);
    assert_eq!(metrics.errors, 1);
    assert_eq!(metrics.client_source, "claude-code");
    assert_eq!(metrics.permission_mode, "acceptEdits");
    db.cleanup().await;
}

#[tokio::test]
async fn mark_session_ended_closes_an_open_session_once() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ending")).await;
    let session_id = unique("session");
    insert_session_summary(&db.pool, &session_id, user.as_str()).await;
    let session = SessionId::new(session_id.clone());

    hooks_track::mark_session_ended(&db.pool, &session)
        .await
        .expect("the first end succeeds");
    let first: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT ended_at FROM plugin_session_summaries WHERE session_id = $1")
            .bind(&session_id)
            .fetch_one(&*db.pool)
            .await
            .expect("the summary exists");
    hooks_track::mark_session_ended(&db.pool, &session)
        .await
        .expect("a repeat end is not an error");
    let second: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT ended_at FROM plugin_session_summaries WHERE session_id = $1")
            .bind(&session_id)
            .fetch_one(&*db.pool)
            .await
            .expect("the summary exists");

    assert!(first.is_some());
    assert_eq!(
        first, second,
        "the guard stops a second end from moving the timestamp"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn count_concurrent_sessions_excludes_the_asking_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("concurrent")).await;
    let asking = unique("session");
    let other = unique("session");
    insert_session_summary(&db.pool, &asking, user.as_str()).await;
    insert_session_summary(&db.pool, &other, user.as_str()).await;

    let count = hooks_track::count_concurrent_sessions(&db.pool, &user, &SessionId::new(asking))
        .await
        .expect("count succeeds");

    assert_eq!(count, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn list_session_events_returns_this_users_events_in_order() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("sessevents")).await;
    let session = unique("session");
    let first = unique("evt");
    let mut early = EventSpec::tool_use(&first, &user, &session);
    early.created_at = chrono::Utc::now() - chrono::Duration::minutes(5);
    insert_event(&db.pool, &early).await;
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;

    let events = hooks_track::list_session_events(&db.pool, &SessionId::new(session), &user)
        .await
        .expect("query succeeds");

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].tool_name.as_deref(), Some("Bash"));
    db.cleanup().await;
}

#[tokio::test]
async fn list_user_messages_returns_only_prompt_submissions() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("prompts")).await;
    let session = unique("session");
    let prompt_id = unique("evt");
    let mut prompt = EventSpec::tool_use(&prompt_id, &user, &session);
    prompt.event_type = "UserPromptSubmit";
    insert_event(&db.pool, &prompt).await;
    sqlx::query(
        "UPDATE plugin_usage_events SET prompt_preview = 'rename the widget' WHERE id = $1",
    )
    .bind(&prompt_id)
    .execute(&*db.pool)
    .await
    .expect("attach the preview");
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;

    let messages = hooks_track::list_user_messages(&db.pool, &SessionId::new(session), &user).await;

    assert_eq!(messages, vec!["rename the widget".to_owned()]);
    db.cleanup().await;
}

#[tokio::test]
async fn get_last_message_is_empty_when_the_session_never_stopped() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("nostop")).await;
    let session = unique("session");
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;

    let last = hooks_track::get_last_message(&db.pool, &SessionId::new(session), &user).await;

    assert_eq!(
        last, "",
        "no terminal event yields an empty string, not an error"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_session_timing_spans_the_users_events() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("timing")).await;
    let session = unique("session");
    let early_id = unique("evt");
    let mut early = EventSpec::tool_use(&early_id, &user, &session);
    early.created_at = chrono::Utc::now() - chrono::Duration::minutes(10);
    insert_event(&db.pool, &early).await;
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;

    let timing = hooks_track::find_session_timing(&db.pool, &SessionId::new(session), &user)
        .await
        .expect("the aggregate always returns a row");

    let started = timing.started.expect("events exist, so there is a start");
    let ended = timing.ended.expect("events exist, so there is an end");
    assert!(ended > started);
    db.cleanup().await;
}

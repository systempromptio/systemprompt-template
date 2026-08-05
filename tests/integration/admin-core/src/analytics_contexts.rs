//! `repositories::analytics::{contexts_list, conversations, agents, tools}` —
//! the contexts index and its KPI strip, transcript flattening, and the
//! per-agent / per-tool rollups.

use systemprompt::identifiers::{ContextId, SessionId, UserId};
use systemprompt_web_admin::repositories::analytics::contexts_list::{
    ContextListFilter, get_context_list_kpis, list_context_list, list_distinct_models,
};
use systemprompt_web_admin::repositories::analytics::{conversations, list_agents, list_tools};

use crate::fixtures::{
    EventSpec, RequestSpec, insert_event, insert_request, insert_user, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_context_list_reports_a_contexts_rollup() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctx")).await;
    let context = unique("ctx");
    for _ in 0..2 {
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.context_id = Some(&context);
        insert_request(&db.pool, &spec).await;
    }
    let filter = ContextListFilter {
        user_id: Some(user.clone()),
        limit: 50,
        ..ContextListFilter::default()
    };

    let rows = list_context_list(&db.pool, &filter)
        .await
        .expect("query succeeds");

    let row = rows
        .iter()
        .find(|r| r.context_id.as_str() == context)
        .expect("the context appears");
    assert_eq!(row.request_count, 2);
    assert_eq!(row.total_input_tokens, 200);
    assert_eq!(row.total_cost_microdollars, 10_000);
    assert_eq!(row.error_count, 0);
    assert_eq!(
        row.message_count, 0,
        "no ai_request_messages rows means no messages to count"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_context_list_filters_to_one_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxmine")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxtheirs")).await;
    for owner in [&mine, &theirs] {
        let context = unique("ctx");
        let mut spec = RequestSpec::completed(&unique("req"), owner);
        spec.context_id = Some(&context);
        insert_request(&db.pool, &spec).await;
    }
    let filter = ContextListFilter {
        user_id: Some(mine.clone()),
        limit: 50,
        ..ContextListFilter::default()
    };

    let rows = list_context_list(&db.pool, &filter)
        .await
        .expect("query succeeds");

    assert!(
        rows.iter()
            .all(|r| r.user_id.as_ref().is_none_or(|u| *u == mine)),
        "the filter must not leak another user's contexts"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_context_list_counts_failed_requests_as_errors() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxerr")).await;
    let context = unique("ctx");
    let mut failed = RequestSpec::completed(&unique("req"), &user);
    failed.context_id = Some(&context);
    failed.status = "failed";
    insert_request(&db.pool, &failed).await;
    let filter = ContextListFilter {
        user_id: Some(user.clone()),
        limit: 50,
        ..ContextListFilter::default()
    };

    let rows = list_context_list(&db.pool, &filter)
        .await
        .expect("query succeeds");

    let row = rows
        .iter()
        .find(|r| r.context_id.as_str() == context)
        .expect("the context appears");
    assert_eq!(row.error_count, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn get_context_list_kpis_sums_only_the_filtered_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("kpi")).await;
    let noise = insert_user(&db.pool, &unique("user"), &unclaimed_email("noise")).await;
    let context = unique("ctx");
    let mut mine = RequestSpec::completed(&unique("req"), &user);
    mine.context_id = Some(&context);
    insert_request(&db.pool, &mine).await;
    let other = unique("ctx");
    let mut theirs = RequestSpec::completed(&unique("req"), &noise);
    theirs.context_id = Some(&other);
    insert_request(&db.pool, &theirs).await;
    let filter = ContextListFilter {
        user_id: Some(user.clone()),
        limit: 50,
        ..ContextListFilter::default()
    };

    let kpis = get_context_list_kpis(&db.pool, &filter)
        .await
        .expect("query succeeds");

    assert_eq!(kpis.total_contexts, 1);
    assert_eq!(kpis.active_users, 1);
    assert_eq!(kpis.total_requests, 1);
    assert_eq!(kpis.total_cost_microdollars, 5_000);
    db.cleanup().await;
}

#[tokio::test]
async fn get_context_list_kpis_returns_a_row_even_when_nothing_matches() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let filter = ContextListFilter {
        user_id: Some(UserId::new(unique("absent"))),
        limit: 50,
        ..ContextListFilter::default()
    };

    let kpis = get_context_list_kpis(&db.pool, &filter)
        .await
        .expect("get_ over an aggregate always has a row to return");

    assert_eq!(kpis.total_contexts, 0);
    assert_eq!(kpis.total_cost_microdollars, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_distinct_models_only_reports_models_used_inside_a_context() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("models")).await;
    let context = unique("ctx");
    let mut inside = RequestSpec::completed(&unique("req"), &user);
    inside.context_id = Some(&context);
    inside.model = "context-model";
    insert_request(&db.pool, &inside).await;
    let mut outside = RequestSpec::completed(&unique("req"), &user);
    outside.model = "contextless-model";
    insert_request(&db.pool, &outside).await;

    let models = list_distinct_models(&db.pool)
        .await
        .expect("query succeeds");

    assert!(models.contains(&"context-model".to_owned()));
    assert!(!models.contains(&"contextless-model".to_owned()));
    db.cleanup().await;
}

#[tokio::test]
async fn find_raw_turns_returns_none_without_a_transcript() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = SessionId::new(unique("session"));

    let turns = conversations::find_raw_turns(&db.pool, &session)
        .await
        .expect("lookup succeeds");

    assert!(turns.is_none(), "no transcript is None, not an empty page");
    db.cleanup().await;
}

#[tokio::test]
async fn find_raw_turns_flattens_the_stored_transcript_array() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session_id = unique("session");
    let transcript = serde_json::json!([
        { "role": "user", "content": "plain string body" },
        { "role": "assistant", "content": [{ "text": "first" }, { "text": "second" }] },
        { "role": "assistant", "text": "bare text field" },
    ]);
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("transcript")).await;
    sqlx::query(
        "INSERT INTO session_transcripts (id, user_id, session_id, transcript)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(unique("transcript"))
    .bind(user.as_str())
    .bind(&session_id)
    .bind(&transcript)
    .execute(&*db.pool)
    .await
    .expect("insert transcript");
    let session = SessionId::new(session_id);

    let turns = conversations::find_raw_turns(&db.pool, &session)
        .await
        .expect("lookup succeeds")
        .expect("the transcript exists");

    assert_eq!(turns.len(), 3);
    assert_eq!(turns[0].ordinal, 0);
    assert_eq!(turns[0].content, "plain string body");
    assert_eq!(
        turns[1].content, "first\nsecond",
        "block arrays join on newlines"
    );
    assert_eq!(turns[2].content, "bare text field");
    db.cleanup().await;
}

#[tokio::test]
async fn list_tools_rolls_up_recent_tool_events() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("tools")).await;
    let session = unique("session");
    let tool = unique("tool");
    for _ in 0..2 {
        let event_id = unique("evt");
        let mut spec = EventSpec::tool_use(&event_id, &user, &session);
        spec.tool_name = Some(&tool);
        insert_event(&db.pool, &spec).await;
    }
    let error_id = unique("evt");
    let mut failure = EventSpec::tool_use(&error_id, &user, &session);
    failure.tool_name = Some(&tool);
    failure.event_type = "claude_code_ToolFailure";
    insert_event(&db.pool, &failure).await;

    let rows = list_tools(&db.pool).await.expect("query succeeds");

    let row = rows
        .iter()
        .find(|r| r.tool_name == tool)
        .expect("the tool appears");
    assert_eq!(row.calls, 3);
    assert_eq!(row.errors, 1);
    assert_eq!(row.sessions, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn list_agents_keys_on_the_metadata_agent_id_before_the_plugin() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("agents")).await;
    let session = unique("session");
    let event_id = unique("evt");
    insert_event(&db.pool, &EventSpec::tool_use(&event_id, &user, &session)).await;
    let agent = unique("agent");
    sqlx::query(
        "UPDATE plugin_usage_events SET metadata = $2, plugin_id = 'some-plugin' WHERE id = $1",
    )
    .bind(&event_id)
    .bind(serde_json::json!({ "agent_id": agent }))
    .execute(&*db.pool)
    .await
    .expect("attach the agent id");

    let rows = list_agents(&db.pool).await.expect("query succeeds");

    let row = rows
        .iter()
        .find(|r| r.agent_id.as_str() == agent)
        .expect("the agent id wins over the plugin id");
    assert_eq!(row.calls, 1);
    assert_eq!(row.sessions, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn context_ids_round_trip_through_the_typed_identifier() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("typed")).await;
    // `ContextId::new` validates UUID v4, so the round trip only holds for the
    // shape production writes — the other tests here store opaque strings that
    // survive the read path but would not survive re-validation.
    let context = ContextId::generate().to_string();
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.context_id = Some(&context);
    insert_request(&db.pool, &spec).await;
    let filter = ContextListFilter {
        user_id: Some(user.clone()),
        limit: 50,
        ..ContextListFilter::default()
    };

    let rows = list_context_list(&db.pool, &filter)
        .await
        .expect("query succeeds");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].context_id, ContextId::new(context));
    db.cleanup().await;
}

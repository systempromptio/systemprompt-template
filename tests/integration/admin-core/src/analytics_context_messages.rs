//! `repositories::analytics::context_detail` — the request, message, and
//! tool-call lists that make up a context transcript.
//!
//! All three join through `ai_requests`, and all three are ordered oldest
//! first so the page reads as a conversation. `tool_input` is stored as TEXT
//! holding a JSON document and is cast in SQL, so the tool-call test asserts on
//! a parsed field rather than merely on the row being present.

use chrono::{Duration, Utc};
use systemprompt::identifiers::ContextId;
use systemprompt_web_admin::repositories::analytics::context_detail as repo;

use crate::analytics_context_detail::new_context_id;
use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

async fn insert_message(pool: &sqlx::PgPool, request_id: &str, seq: i32, role: &str, body: &str) {
    sqlx::query(
        "INSERT INTO ai_request_messages (id, request_id, role, content, sequence_number)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(unique("msg"))
    .bind(request_id)
    .bind(role)
    .bind(body)
    .bind(seq)
    .execute(pool)
    .await
    .expect("insert ai request message");
}

async fn insert_tool_call(pool: &sqlx::PgPool, request_id: &str, seq: i32, tool: &str) {
    sqlx::query(
        "INSERT INTO ai_request_tool_calls
             (id, request_id, tool_name, tool_input, sequence_number, tool_result_payload)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(unique("call"))
    .bind(request_id)
    .bind(tool)
    .bind(r#"{"command":"ls"}"#)
    .bind(seq)
    .bind(serde_json::json!({"stdout": "ok"}))
    .execute(pool)
    .await
    .expect("insert ai request tool call");
}

#[tokio::test]
async fn list_context_requests_returns_the_oldest_request_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxreqs")).await;
    let context = new_context_id();
    let old_id = unique("req-old");
    let new_id = unique("req-new");
    let mut old = RequestSpec::completed(&old_id, &user);
    old.context_id = Some(&context);
    old.created_at = Utc::now() - Duration::minutes(5);
    insert_request(&db.pool, &old).await;
    let mut new = RequestSpec::completed(&new_id, &user);
    new.context_id = Some(&context);
    insert_request(&db.pool, &new).await;

    let requests = repo::list_context_requests(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("list requests");

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].id.as_str(), old_id);
    assert_eq!(requests[1].id.as_str(), new_id);
    assert_eq!(requests[0].input_tokens, Some(100));
    assert_eq!(requests[0].output_tokens, Some(20));
    db.cleanup().await;
}

#[tokio::test]
async fn list_context_messages_is_empty_when_no_request_carries_messages() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("nomsgs")).await;
    let context = new_context_id();
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.context_id = Some(&context);
    insert_request(&db.pool, &spec).await;

    let messages = repo::list_context_messages(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("list messages");

    assert!(messages.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_context_messages_orders_by_request_then_sequence() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("msgs")).await;
    let context = new_context_id();
    let first_req = unique("req-first");
    let second_req = unique("req-second");
    let mut first = RequestSpec::completed(&first_req, &user);
    first.context_id = Some(&context);
    first.created_at = Utc::now() - Duration::minutes(5);
    insert_request(&db.pool, &first).await;
    let mut second = RequestSpec::completed(&second_req, &user);
    second.context_id = Some(&context);
    insert_request(&db.pool, &second).await;

    insert_message(&db.pool, &first_req, 1, "assistant", "second line").await;
    insert_message(&db.pool, &first_req, 0, "user", "first line").await;
    insert_message(&db.pool, &second_req, 0, "user", "third line").await;

    let messages = repo::list_context_messages(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("list messages");

    let bodies: Vec<&str> = messages.iter().map(|m| m.content.as_str()).collect();
    assert_eq!(bodies, ["first line", "second line", "third line"]);
    assert_eq!(messages[0].role, "user");
    assert_eq!(messages[0].sequence_number, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_context_tool_calls_returns_the_calls_in_sequence_order() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("calls")).await;
    let context = new_context_id();
    let request = unique("req");
    let mut spec = RequestSpec::completed(&request, &user);
    spec.context_id = Some(&context);
    insert_request(&db.pool, &spec).await;
    insert_tool_call(&db.pool, &request, 1, "Read").await;
    insert_tool_call(&db.pool, &request, 0, "Bash").await;

    let calls = repo::list_context_tool_calls(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("list tool calls");

    let names: Vec<&str> = calls.iter().map(|c| c.tool_name.as_str()).collect();
    assert_eq!(names, ["Bash", "Read"]);
    assert_eq!(calls[0].tool_input["command"], "ls");
    assert_eq!(
        calls[0]
            .tool_result_payload
            .as_ref()
            .map(|p| p["stdout"].clone()),
        Some(serde_json::json!("ok"))
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_context_tool_calls_is_empty_for_a_context_with_no_calls() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let calls = repo::list_context_tool_calls(&db.pool, &ContextId::new_unchecked(new_context_id()))
        .await
        .expect("list tool calls");

    assert!(calls.is_empty());
    db.cleanup().await;
}

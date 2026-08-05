//! `repositories::traces::spans` — the per-session waterfall, unioned across
//! `governance_decisions`, `ai_requests`, and `plugin_usage_events`.
//!
//! `resolve_trace_session` accepts either a `session_id` or a `trace_id` and
//! probes four tables in a fixed order; each arm gets its own test because the
//! detail page 404s the moment one stops matching. `list_trace_spans` then
//! normalises three row shapes into one `Span`, where the status and duration
//! mappings are the parts a schema change would silently break.

use chrono::{Duration, Utc};
use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::traces::list_trace_spans;

use crate::fixtures::{
    DecisionSpec, EventSpec, RequestSpec, insert_decision, insert_event, insert_request,
    insert_session, insert_user, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_trace_spans_is_empty_for_a_session_with_no_activity() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let spans = list_trace_spans(&db.pool, &SessionId::new(unique("session")))
        .await
        .expect("list spans");

    assert!(spans.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_unions_the_three_sources_in_start_order() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("union")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let now = Utc::now();

    let mut decision = DecisionSpec::allow(&unique("dec"), &user, &session);
    decision.created_at = now - Duration::seconds(30);
    insert_decision(&db.pool, &decision).await;

    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.created_at = now - Duration::seconds(20);
    insert_request(&db.pool, &request).await;

    let mut event = EventSpec::tool_use(&unique("event"), &user, &session);
    event.created_at = now - Duration::seconds(10);
    insert_event(&db.pool, &event).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    let kinds: Vec<&str> = spans.iter().map(|s| s.kind.as_str()).collect();
    assert_eq!(kinds, ["governance", "model", "tool"]);
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_names_a_governance_span_policy_slash_tool() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("govspan")).await;
    let session = unique("session");
    let mut decision = DecisionSpec::allow(&unique("dec"), &user, &session);
    decision.policy = "blocklist";
    decision.tool_name = "Write";
    decision.agent_id = Some("reviewer");
    decision.agent_scope = Some("repo");
    insert_decision(&db.pool, &decision).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].name, "blocklist / Write");
    assert_eq!(spans[0].status.as_str(), "ok");
    assert_eq!(spans[0].duration_ms, 0);
    let label = spans[0].identity_label.as_deref().expect("identity label");
    assert!(label.contains("reviewer (repo)"), "got {label}");
    assert_eq!(spans[0].raw["decision"], "allow");
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_marks_a_denied_decision() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("denyspan")).await;
    let session = unique("session");
    let mut decision = DecisionSpec::allow(&unique("dec"), &user, &session);
    decision.decision = "deny";
    insert_decision(&db.pool, &decision).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].status.as_str(), "deny");
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_derives_a_request_duration_from_the_latency_when_it_never_completed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("latency")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.latency_ms = 1_750;
    insert_request(&db.pool, &request).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].duration_ms, 1_750);
    assert_eq!(spans[0].name, "anthropic/claude-test-model");
    assert_eq!(spans[0].status.as_str(), "ok");
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_prefers_completed_at_over_the_recorded_latency() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("completed")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let request_id = unique("req");
    let mut request = RequestSpec::completed(&request_id, &user);
    request.session_id = Some(&session);
    request.latency_ms = 100;
    insert_request(&db.pool, &request).await;
    sqlx::query(
        "UPDATE ai_requests SET completed_at = created_at + INTERVAL '4 seconds' WHERE id = $1",
    )
    .bind(&request_id)
    .execute(&*db.pool)
    .await
    .expect("set completed_at");

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].duration_ms, 4_000);
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_maps_a_failed_request_to_an_error_span() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("errspan")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.status = "failed";
    insert_request(&db.pool, &request).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].status.as_str(), "error");
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_maps_a_pending_request_to_a_pending_span() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("pending")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.status = "pending";
    insert_request(&db.pool, &request).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].status.as_str(), "pending");
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_classifies_a_spawn_event_by_its_event_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("spawn")).await;
    let session = unique("session");
    let mut event = EventSpec::tool_use(&unique("event"), &user, &session);
    event.event_type = "claude_code_SubagentSpawn";
    event.tool_name = None;
    insert_event(&db.pool, &event).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].kind.as_str(), "spawn");
    assert_eq!(
        spans[0].name, "claude_code_SubagentSpawn",
        "with no tool name the event type is the span name"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_marks_a_failure_event_as_an_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("evterr")).await;
    let session = unique("session");
    let mut event = EventSpec::tool_use(&unique("event"), &user, &session);
    event.event_type = "claude_code_PostToolUseFailure";
    insert_event(&db.pool, &event).await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(spans[0].status.as_str(), "error");
    assert_eq!(spans[0].kind.as_str(), "tool");
    assert_eq!(spans[0].name, "Bash");
    db.cleanup().await;
}

#[tokio::test]
async fn list_trace_spans_picks_up_a_decision_recorded_against_the_trace_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("bytraceid")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let trace = unique("trace");
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.trace_id = Some(&trace);
    insert_request(&db.pool, &request).await;
    // The hook wrote its decision keyed on the trace, not the session.
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &trace),
    )
    .await;

    let spans = list_trace_spans(&db.pool, &SessionId::new(session))
        .await
        .expect("list spans");

    assert_eq!(
        spans
            .iter()
            .filter(|s| s.kind.as_str() == "governance")
            .count(),
        1
    );
    db.cleanup().await;
}

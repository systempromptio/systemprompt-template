//! `repositories::traces::list` — the session-keyed trace list, its filters,
//! and its sorts. The window starts after the newest seeded `ai_requests` row.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::traces::{
    TraceFilter, TracePage, TraceSort, TraceSortColumn, TraceSortDir, list_traces,
};

use crate::fixtures::{
    DecisionSpec, EventSpec, RequestSpec, insert_decision, insert_event, insert_request,
    insert_session, insert_user, narrow_window, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

fn page() -> TracePage {
    TracePage {
        sort: TraceSort::default(),
        limit: 50,
        offset: 0,
    }
}

#[tokio::test]
async fn list_traces_finds_nothing_in_an_empty_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (traces, total) = list_traces(&db.pool, TraceFilter::default(), narrow_window(), page())
        .await
        .expect("query succeeds");

    assert!(traces.is_empty());
    assert_eq!(total, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_summarises_a_session_across_all_three_sources() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("trace")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.latency_ms = 800;
    insert_request(&db.pool, &request).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;

    let (traces, total) = list_traces(&db.pool, TraceFilter::default(), narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(total, 1);
    let trace = &traces[0];
    assert_eq!(trace.session_id.as_str(), session);
    assert_eq!(trace.span_count, 3, "one row per source contributes a span");
    assert_eq!(trace.request_count, 1);
    assert_eq!(trace.governance_count, 1);
    assert_eq!(trace.tool_call_count, 1);
    assert_eq!(trace.active_ms, 800, "active_ms is summed request latency");
    assert_eq!(trace.total_cost_microdollars, 5_000);
    assert_eq!(trace.total_tokens, 120);
    assert!(!trace.has_error);
    assert!(!trace.has_deny);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_keeps_active_and_window_clocks_apart() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("clocks")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.latency_ms = 100;
    request.created_at = Utc::now() - Duration::seconds(20);
    insert_request(&db.pool, &request).await;
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;

    let (traces, _) = list_traces(&db.pool, TraceFilter::default(), narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(traces[0].active_ms, 100);
    assert!(
        traces[0].window_ms > 1_000,
        "a session held open for 20s must not report 100ms of wall clock"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_flags_a_failed_request_as_an_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("failing")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.status = "failed";
    insert_request(&db.pool, &request).await;

    let (traces, _) = list_traces(&db.pool, TraceFilter::default(), narrow_window(), page())
        .await
        .expect("query succeeds");

    assert!(traces[0].has_error);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_flags_a_denial() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("denied")).await;
    let session = unique("session");
    let id = unique("dec");
    let mut denial = DecisionSpec::allow(&id, &user, &session);
    denial.decision = "deny";
    insert_decision(&db.pool, &denial).await;

    let (traces, _) = list_traces(&db.pool, TraceFilter::default(), narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(traces.len(), 1, "a governance-only trace is still a trace");
    assert!(traces[0].has_deny);
    assert_eq!(traces[0].deny_count, 1);
    assert_eq!(traces[0].request_count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_filters_by_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = insert_user(&db.pool, &unique("user"), &unclaimed_email("mine")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("theirs")).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &mine, &unique("session")),
    )
    .await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &theirs, &unique("session")),
    )
    .await;
    let filter = TraceFilter {
        user_id: Some(mine.as_str()),
        ..TraceFilter::default()
    };

    let (traces, total) = list_traces(&db.pool, filter, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(total, 1);
    assert_eq!(
        traces[0].user_id.as_ref().map(|u| u.as_str()),
        Some(mine.as_str())
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_filters_by_agent_and_scope() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("agentfilter")).await;
    let agent = unique("agent");
    let id = unique("dec");
    let session = unique("session");
    let mut tagged = DecisionSpec::allow(&id, &user, &session);
    tagged.agent_id = Some(&agent);
    tagged.agent_scope = Some("subagent");
    insert_decision(&db.pool, &tagged).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &unique("session")),
    )
    .await;

    let by_agent = TraceFilter {
        agent_id: Some(&agent),
        ..TraceFilter::default()
    };
    let by_scope = TraceFilter {
        agent_scope: Some("subagent"),
        ..TraceFilter::default()
    };
    let (agent_traces, _) = list_traces(&db.pool, by_agent, narrow_window(), page())
        .await
        .expect("query succeeds");
    let (scope_traces, _) = list_traces(&db.pool, by_scope, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(agent_traces.len(), 1);
    assert_eq!(scope_traces.len(), 1);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_deny_only_and_error_only_narrow_the_set() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("onlys")).await;
    let denied_session = unique("session");
    let id = unique("dec");
    let mut denial = DecisionSpec::allow(&id, &user, &denied_session);
    denial.decision = "deny";
    insert_decision(&db.pool, &denial).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &unique("session")),
    )
    .await;

    let deny_only = TraceFilter {
        deny_only: true,
        ..TraceFilter::default()
    };
    let error_only = TraceFilter {
        error_only: true,
        ..TraceFilter::default()
    };
    let (denied, _) = list_traces(&db.pool, deny_only, narrow_window(), page())
        .await
        .expect("query succeeds");
    let (errored, _) = list_traces(&db.pool, error_only, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(denied.len(), 1);
    assert_eq!(denied[0].session_id.as_str(), denied_session);
    assert!(errored.is_empty(), "nothing failed, so error_only is empty");
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_filters_by_policy_and_decision() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("policyfilter")).await;
    let wanted_session = unique("session");
    let id = unique("dec");
    let mut spec = DecisionSpec::allow(&id, &user, &wanted_session);
    spec.policy = "secret_scan";
    spec.decision = "deny";
    insert_decision(&db.pool, &spec).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &unique("session")),
    )
    .await;

    let by_policy = TraceFilter {
        policy: Some("secret_scan"),
        ..TraceFilter::default()
    };
    let by_decision = TraceFilter {
        decision: Some("deny"),
        ..TraceFilter::default()
    };
    let (policy_traces, _) = list_traces(&db.pool, by_policy, narrow_window(), page())
        .await
        .expect("query succeeds");
    let (decision_traces, _) = list_traces(&db.pool, by_decision, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(policy_traces.len(), 1);
    assert_eq!(policy_traces[0].session_id.as_str(), wanted_session);
    assert_eq!(decision_traces.len(), 1);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_sorts_by_cost_in_both_directions() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("sorted")).await;
    let cheap_session = unique("session");
    let dear_session = unique("session");
    insert_session(&db.pool, &cheap_session, &user).await;
    insert_session(&db.pool, &dear_session, &user).await;
    let mut cheap = RequestSpec::completed(&unique("req"), &user);
    cheap.session_id = Some(&cheap_session);
    cheap.cost_microdollars = 10;
    insert_request(&db.pool, &cheap).await;
    let mut dear = RequestSpec::completed(&unique("req"), &user);
    dear.session_id = Some(&dear_session);
    dear.cost_microdollars = 90_000;
    insert_request(&db.pool, &dear).await;

    let descending = TracePage {
        sort: TraceSort {
            column: TraceSortColumn::Cost,
            dir: TraceSortDir::Desc,
        },
        limit: 50,
        offset: 0,
    };
    let ascending = TracePage {
        sort: TraceSort {
            column: TraceSortColumn::Cost,
            dir: TraceSortDir::Asc,
        },
        ..descending
    };
    let (desc, _) = list_traces(
        &db.pool,
        TraceFilter::default(),
        narrow_window(),
        descending,
    )
    .await
    .expect("query succeeds");
    let (asc, _) = list_traces(&db.pool, TraceFilter::default(), narrow_window(), ascending)
        .await
        .expect("query succeeds");

    assert_eq!(desc[0].session_id.as_str(), dear_session);
    assert_eq!(asc[0].session_id.as_str(), cheap_session);
    db.cleanup().await;
}

#[tokio::test]
async fn list_traces_reports_the_unpaged_total() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("paged")).await;
    for _ in 0..3 {
        insert_decision(
            &db.pool,
            &DecisionSpec::allow(&unique("dec"), &user, &unique("session")),
        )
        .await;
    }
    let one = TracePage {
        sort: TraceSort::default(),
        limit: 1,
        offset: 0,
    };

    let (traces, total) = list_traces(&db.pool, TraceFilter::default(), narrow_window(), one)
        .await
        .expect("query succeeds");

    assert_eq!(traces.len(), 1);
    assert_eq!(total, 3);
    db.cleanup().await;
}

//! `repositories::traces::{stats, spans}` — the percentile strip over the same
//! window as the list, and resolving an opaque id to a session.

use systemprompt_web_admin::repositories::traces::{get_trace_stats, resolve_trace_session};

use crate::fixtures::{
    DecisionSpec, RequestSpec, insert_decision, insert_request, insert_session, insert_user,
    narrow_window, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn get_trace_stats_is_zero_in_an_empty_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let stats = get_trace_stats(&db.pool, narrow_window())
        .await
        .expect("query succeeds");

    assert_eq!(stats.total_traces, 0);
    assert_eq!(stats.p50_active_ms, 0);
    assert_eq!(stats.total_cost_microdollars, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn get_trace_stats_takes_percentiles_from_request_bearing_traces_only() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("stats")).await;
    let busy = unique("session");
    insert_session(&db.pool, &busy, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&busy);
    request.latency_ms = 500;
    insert_request(&db.pool, &request).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &unique("session")),
    )
    .await;

    let stats = get_trace_stats(&db.pool, narrow_window())
        .await
        .expect("query succeeds");

    assert_eq!(stats.total_traces, 2, "both sessions are traces");
    assert_eq!(
        stats.p50_active_ms, 500,
        "the governance-only trace has no latency to drag the percentile to zero"
    );
    assert_eq!(stats.total_cost_microdollars, 5_000);
    db.cleanup().await;
}

#[tokio::test]
async fn get_trace_stats_counts_errors_and_denials_per_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("counts")).await;
    let failing = unique("session");
    insert_session(&db.pool, &failing, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&failing);
    request.status = "failed";
    insert_request(&db.pool, &request).await;
    let denied = unique("session");
    let id = unique("dec");
    let mut denial = DecisionSpec::allow(&id, &user, &denied);
    denial.decision = "deny";
    insert_decision(&db.pool, &denial).await;

    let stats = get_trace_stats(&db.pool, narrow_window())
        .await
        .expect("query succeeds");

    assert_eq!(stats.error_count, 1);
    assert_eq!(stats.deny_count, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_trace_session_returns_none_for_an_unknown_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let resolved = resolve_trace_session(&db.pool, &unique("mystery"))
        .await
        .expect("lookup succeeds");

    assert!(resolved.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_trace_session_maps_a_trace_id_onto_its_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("resolve")).await;
    let session = unique("session");
    let trace = unique("trace");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.trace_id = Some(&trace);
    insert_request(&db.pool, &request).await;

    let resolved = resolve_trace_session(&db.pool, &trace)
        .await
        .expect("lookup succeeds")
        .expect("the trace id maps to a session");

    assert_eq!(resolved.as_str(), session);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_trace_session_accepts_a_governance_only_session_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("govonly")).await;
    let session = unique("session");
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;

    let resolved = resolve_trace_session(&db.pool, &session)
        .await
        .expect("lookup succeeds")
        .expect("a session with only governance rows still resolves");

    assert_eq!(resolved.as_str(), session);
    db.cleanup().await;
}

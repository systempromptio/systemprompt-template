//! `repositories::traces::spans::resolve_trace_session` — the lookup that
//! turns whatever id the trace list linked into the `session_id` the detail
//! page is keyed on.
//!
//! It probes four tables in a fixed order — `ai_requests` by trace, then
//! `governance_decisions`, `ai_requests` and `plugin_usage_events` by session.
//! Each arm exists because some client writes only to that table, so each gets
//! its own test: losing one turns a working trace link into a 404.

use systemprompt_web_admin::repositories::traces::resolve_trace_session;

use crate::fixtures::{
    DecisionSpec, EventSpec, RequestSpec, insert_decision, insert_event, insert_request,
    insert_session, insert_user, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn resolve_trace_session_returns_none_for_an_unknown_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let resolved = resolve_trace_session(&db.pool, &unique("nothing"))
        .await
        .expect("resolve");

    assert!(resolved.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_trace_session_maps_a_trace_id_to_its_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("bytrace")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let trace = unique("trace");
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.session_id = Some(&session);
    spec.trace_id = Some(&trace);
    insert_request(&db.pool, &spec).await;

    let resolved = resolve_trace_session(&db.pool, &trace)
        .await
        .expect("resolve")
        .expect("session found");

    assert_eq!(resolved.as_str(), session);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_trace_session_accepts_a_session_known_only_to_governance() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("bygov")).await;
    let session = unique("session");
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;

    let resolved = resolve_trace_session(&db.pool, &session)
        .await
        .expect("resolve")
        .expect("session found");

    assert_eq!(resolved.as_str(), session);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_trace_session_accepts_a_session_known_only_to_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("byreq")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.session_id = Some(&session);
    insert_request(&db.pool, &spec).await;

    let resolved = resolve_trace_session(&db.pool, &session)
        .await
        .expect("resolve")
        .expect("session found");

    assert_eq!(resolved.as_str(), session);
    db.cleanup().await;
}

#[tokio::test]
async fn resolve_trace_session_accepts_a_hook_only_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("byhook")).await;
    let session = unique("session");
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("event"), &user, &session),
    )
    .await;

    let resolved = resolve_trace_session(&db.pool, &session)
        .await
        .expect("resolve")
        .expect("session found");

    assert_eq!(resolved.as_str(), session);
    db.cleanup().await;
}

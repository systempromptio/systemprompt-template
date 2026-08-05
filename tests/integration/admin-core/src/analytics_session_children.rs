//! `repositories::analytics::session_detail` — the three child lists a session
//! page renders below its header: contexts, traces, and raw requests.
//!
//! All three are grouped or ordered reads over `ai_requests` scoped to one
//! session. The contexts and traces lists silently drop requests whose
//! `context_id` / `trace_id` is null, which is deliberate but easy to lose in
//! a rewrite, so both exclusions are asserted directly.

use chrono::{Duration, Utc};
use systemprompt::identifiers::{ContextId, SessionId};
use systemprompt_web_admin::repositories::analytics::session_detail as repo;

use crate::fixtures::{
    RequestSpec, insert_context, insert_request, insert_session, insert_user, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_session_contexts_skips_requests_with_no_context() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxskip")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut bare = RequestSpec::completed(&unique("req"), &user);
    bare.session_id = Some(&session);
    insert_request(&db.pool, &bare).await;

    let contexts = repo::list_session_contexts(&db.pool, &SessionId::new(session))
        .await
        .expect("list contexts");

    assert!(contexts.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_session_contexts_rolls_up_per_context_and_joins_the_name() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxroll")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let context = unique("ctx");
    insert_context(&db.pool, &context, &user, Some(&session), "Planning").await;
    for status in ["completed", "failed"] {
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.session_id = Some(&session);
        spec.context_id = Some(&context);
        spec.status = status;
        insert_request(&db.pool, &spec).await;
    }

    let contexts = repo::list_session_contexts(&db.pool, &SessionId::new(session))
        .await
        .expect("list contexts");

    assert_eq!(contexts.len(), 1);
    let row = &contexts[0];
    assert_eq!(row.context_id.as_str(), context);
    assert_eq!(row.name.as_deref(), Some("Planning"));
    assert_eq!(row.request_count, 2);
    assert_eq!(row.error_count, 1);
    assert_eq!(row.cost_microdollars, 10_000);
    db.cleanup().await;
}

#[tokio::test]
async fn list_session_contexts_orders_the_most_recent_context_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxorder")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let older = unique("ctx-old");
    let newer = unique("ctx-new");
    let mut first = RequestSpec::completed(&unique("req"), &user);
    first.session_id = Some(&session);
    first.context_id = Some(&older);
    first.created_at = Utc::now() - Duration::minutes(10);
    insert_request(&db.pool, &first).await;
    let mut second = RequestSpec::completed(&unique("req"), &user);
    second.session_id = Some(&session);
    second.context_id = Some(&newer);
    insert_request(&db.pool, &second).await;

    let contexts = repo::list_session_contexts(&db.pool, &SessionId::new(session))
        .await
        .expect("list contexts");

    assert_eq!(contexts[0].context_id.as_str(), newer);
    assert_eq!(contexts[1].context_id.as_str(), older);
    db.cleanup().await;
}

#[tokio::test]
async fn list_session_traces_groups_by_trace_and_reports_the_error_count() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("traces")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let trace = unique("trace");
    for status in ["completed", "failed", "failed"] {
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.session_id = Some(&session);
        spec.trace_id = Some(&trace);
        spec.status = status;
        insert_request(&db.pool, &spec).await;
    }
    let mut untraced = RequestSpec::completed(&unique("req"), &user);
    untraced.session_id = Some(&session);
    insert_request(&db.pool, &untraced).await;

    let traces = repo::list_session_traces(&db.pool, &SessionId::new(session))
        .await
        .expect("list traces");

    assert_eq!(traces.len(), 1, "the request with no trace is excluded");
    assert_eq!(traces[0].trace_id.as_str(), trace);
    assert_eq!(traces[0].request_count, 3);
    assert_eq!(traces[0].error_count, 2);
    assert!(traces[0].started_at.is_some());
    db.cleanup().await;
}

#[tokio::test]
async fn list_session_requests_returns_the_newest_request_first() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("reqorder")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let context = unique("ctx");
    let old_id = unique("req-old");
    let new_id = unique("req-new");
    let mut old = RequestSpec::completed(&old_id, &user);
    old.session_id = Some(&session);
    old.created_at = Utc::now() - Duration::minutes(5);
    insert_request(&db.pool, &old).await;
    let mut new = RequestSpec::completed(&new_id, &user);
    new.session_id = Some(&session);
    new.context_id = Some(&context);
    insert_request(&db.pool, &new).await;

    let requests = repo::list_session_requests(&db.pool, &SessionId::new(session))
        .await
        .expect("list requests");

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].id.as_str(), new_id);
    assert_eq!(
        requests[0].context_id.as_ref().map(ContextId::as_str),
        Some(context.as_str())
    );
    assert_eq!(requests[1].id.as_str(), old_id);
    assert!(requests[1].context_id.is_none());
    assert_eq!(requests[0].latency_ms, Some(250));
    db.cleanup().await;
}

#[tokio::test]
async fn list_session_requests_is_empty_for_a_session_with_no_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let requests = repo::list_session_requests(&db.pool, &SessionId::new(unique("session")))
        .await
        .expect("list requests");

    assert!(requests.is_empty());
    db.cleanup().await;
}

//! `repositories::analytics::context_detail` — the
//! `/admin/entities/contexts/{id}` read model.
//!
//! Like the session header, `find_context_header` full-outer-joins the
//! `ai_requests` rollup against the stored `user_contexts` row, so a context
//! known only to the gateway and one known only to the agent runtime both
//! resolve. The message and tool-call lists join through `ai_requests`, which
//! is where the ordering contract lives.

use chrono::{Duration, Utc};
use systemprompt::identifiers::ContextId;
use systemprompt_web_admin::repositories::analytics::context_detail as repo;

use crate::fixtures::{
    RequestSpec, insert_context, insert_request, insert_session, insert_user, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

// `ContextId::new_unchecked` panics on anything that is not a UUID v4, so context ids
// here are minted as UUIDs rather than with the suite's readable `unique`.
pub fn new_context_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tokio::test]
async fn find_context_header_returns_none_for_an_unknown_context() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let header = repo::find_context_header(&db.pool, &ContextId::new_unchecked(new_context_id()))
        .await
        .expect("query header");

    assert!(header.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn find_context_header_resolves_a_context_known_only_to_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxhdr")).await;
    let context = new_context_id();
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.context_id = Some(&context);
    insert_request(&db.pool, &spec).await;

    let header = repo::find_context_header(&db.pool, &ContextId::new_unchecked(context.clone()))
        .await
        .expect("query header")
        .expect("header present");

    assert_eq!(header.context_id.as_str(), context);
    assert_eq!(
        header.user_id.map(|u| u.as_str().to_owned()),
        Some(user.as_str().to_owned())
    );
    assert!(header.name.is_none(), "no user_contexts row means no name");
    assert!(header.created_at.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn find_context_header_resolves_a_stored_context_with_no_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("stored")).await;
    let context = new_context_id();
    insert_context(&db.pool, &context, &user, None, "Design review").await;

    let header = repo::find_context_header(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("query header")
        .expect("header present");

    assert_eq!(header.name.as_deref(), Some("Design review"));
    assert!(header.created_at.is_some());
    assert!(header.session_id.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn find_context_header_carries_the_session_from_the_stored_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxsess")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let context = new_context_id();
    insert_context(&db.pool, &context, &user, Some(&session), "Linked").await;
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.context_id = Some(&context);
    spec.session_id = Some(&session);
    insert_request(&db.pool, &spec).await;

    let header = repo::find_context_header(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("query header")
        .expect("header present");

    assert_eq!(
        header.session_id.map(|s| s.as_str().to_owned()),
        Some(session)
    );
    assert!(header.display_name.is_some());
    db.cleanup().await;
}

#[tokio::test]
async fn get_context_kpis_returns_zeroes_for_a_context_with_no_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let kpis = repo::get_context_kpis(&db.pool, &ContextId::new_unchecked(new_context_id()))
        .await
        .expect("query kpis");

    assert_eq!(kpis.request_count, 0);
    assert_eq!(kpis.total_cost_microdollars, 0);
    assert!(kpis.first_request_at.is_none());
    assert!(kpis.model.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn get_context_kpis_sums_the_requests_and_counts_failures() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxkpi")).await;
    let context = new_context_id();
    let trace = unique("trace");
    for status in ["completed", "failed"] {
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.context_id = Some(&context);
        spec.trace_id = Some(&trace);
        spec.status = status;
        insert_request(&db.pool, &spec).await;
    }

    let kpis = repo::get_context_kpis(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("query kpis");

    assert_eq!(kpis.request_count, 2);
    assert_eq!(kpis.error_count, 1);
    assert_eq!(kpis.trace_count, 1);
    assert_eq!(kpis.total_input_tokens, 200);
    assert_eq!(kpis.total_output_tokens, 40);
    assert_eq!(kpis.total_cost_microdollars, 10_000);
    assert!(kpis.first_request_at.is_some());
    assert!(kpis.last_request_at.is_some());
    db.cleanup().await;
}

#[tokio::test]
async fn get_context_kpis_reports_the_model_of_the_newest_request() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ctxmodel")).await;
    let context = new_context_id();
    let mut old = RequestSpec::completed(&unique("req"), &user);
    old.context_id = Some(&context);
    old.model = "old-model";
    old.created_at = Utc::now() - Duration::minutes(30);
    insert_request(&db.pool, &old).await;
    let mut new = RequestSpec::completed(&unique("req"), &user);
    new.context_id = Some(&context);
    new.model = "new-model";
    insert_request(&db.pool, &new).await;

    let kpis = repo::get_context_kpis(&db.pool, &ContextId::new_unchecked(context))
        .await
        .expect("query kpis");

    assert_eq!(kpis.model.as_deref(), Some("new-model"));
    db.cleanup().await;
}

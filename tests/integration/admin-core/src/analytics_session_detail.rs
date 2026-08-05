//! `repositories::analytics::session_detail` — the
//! `/admin/entities/sessions/{id}` read model.
//!
//! The header is a FULL OUTER JOIN between the `ai_requests` rollup and the
//! hook-written `plugin_session_summaries` row, so each of the three shapes a
//! session can take (requests only, summary only, both) is exercised
//! separately. The KPI and list queries are scoped by `session_id`, and the
//! seeded demo `ai_requests` carry no session, so these tests see only their
//! own rows.

use chrono::{Duration, Utc};
use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::analytics::session_detail as repo;

use crate::fixtures::{
    RequestSpec, SummarySpec, insert_request, insert_session, insert_summary, insert_user,
    set_department, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn find_session_header_returns_none_for_an_unknown_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let header = repo::find_session_header(&db.pool, &SessionId::new(unique("session")))
        .await
        .expect("query header");

    assert!(header.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn find_session_header_builds_a_header_from_requests_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hdr")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.session_id = Some(&session);
    spec.model = "claude-header-model";
    insert_request(&db.pool, &spec).await;

    let header = repo::find_session_header(&db.pool, &SessionId::new(session.clone()))
        .await
        .expect("query header")
        .expect("header present");

    assert_eq!(header.session_id.as_str(), session);
    assert_eq!(
        header.user_id.map(|u| u.as_str().to_owned()),
        Some(user.as_str().to_owned())
    );
    assert_eq!(header.model.as_deref(), Some("claude-header-model"));
    assert!(
        header.status.is_none(),
        "no summary row means no hook status"
    );
    assert!(header.started_at.is_some());
    db.cleanup().await;
}

#[tokio::test]
async fn find_session_header_builds_a_header_from_a_summary_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hookonly")).await;
    let session = unique("session");
    let mut spec = SummarySpec::open(&session, &user);
    spec.status = Some("active");
    spec.model = Some("hook-model");
    spec.ai_title = Some("Refactor the parser");
    insert_summary(&db.pool, &spec).await;

    let header = repo::find_session_header(&db.pool, &SessionId::new(session.clone()))
        .await
        .expect("query header")
        .expect("header present");

    assert_eq!(header.session_id.as_str(), session);
    assert_eq!(header.status.as_deref(), Some("active"));
    assert_eq!(header.model.as_deref(), Some("hook-model"));
    assert_eq!(header.ai_title.as_deref(), Some("Refactor the parser"));
    db.cleanup().await;
}

#[tokio::test]
async fn find_session_header_falls_back_to_started_at_when_the_session_never_ended() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("neverended")).await;
    let session = unique("session");
    let started = Utc::now() - Duration::hours(2);
    let mut spec = SummarySpec::open(&session, &user);
    spec.started_at = Some(started);
    insert_summary(&db.pool, &spec).await;

    let header = repo::find_session_header(&db.pool, &SessionId::new(session))
        .await
        .expect("query header")
        .expect("header present");

    let last = header
        .last_activity_at
        .expect("last activity falls back to started_at");
    assert!((last - started).num_seconds().abs() < 2);
    db.cleanup().await;
}

#[tokio::test]
async fn find_session_header_prefers_the_summary_over_the_request_rollup() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("both")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut request = RequestSpec::completed(&unique("req"), &user);
    request.session_id = Some(&session);
    request.model = "request-model";
    insert_request(&db.pool, &request).await;
    let mut summary = SummarySpec::open(&session, &user);
    summary.model = Some("summary-model");
    summary.status = Some("completed");
    insert_summary(&db.pool, &summary).await;

    let header = repo::find_session_header(&db.pool, &SessionId::new(session))
        .await
        .expect("query header")
        .expect("header present");

    assert_eq!(header.model.as_deref(), Some("summary-model"));
    assert_eq!(header.status.as_deref(), Some("completed"));
    db.cleanup().await;
}

#[tokio::test]
async fn find_session_header_joins_the_display_name_and_department() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("profile")).await;
    set_department(&db.pool, &user, "Platform").await;
    let session = unique("session");
    insert_summary(&db.pool, &SummarySpec::open(&session, &user)).await;

    let header = repo::find_session_header(&db.pool, &SessionId::new(session))
        .await
        .expect("query header")
        .expect("header present");

    assert!(
        header.display_name.is_some(),
        "insert_user sets display_name"
    );
    assert_eq!(header.department.as_deref(), Some("Platform"));
    db.cleanup().await;
}

#[tokio::test]
async fn get_session_kpis_returns_zeroes_for_a_session_with_no_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let kpis = repo::get_session_kpis(&db.pool, &SessionId::new(unique("session")))
        .await
        .expect("query kpis");

    assert_eq!(kpis.request_count, 0);
    assert_eq!(kpis.total_cost_microdollars, 0);
    assert_eq!(kpis.total_input_tokens, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn get_session_kpis_sums_tokens_cost_and_counts_distinct_children() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("kpis")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let context = unique("ctx");
    let trace = unique("trace");
    for (id, status) in [("a", "completed"), ("b", "completed"), ("c", "failed")] {
        let mut spec = RequestSpec::completed(&unique(id), &user);
        spec.session_id = Some(&session);
        spec.context_id = Some(&context);
        spec.trace_id = Some(&trace);
        spec.status = status;
        insert_request(&db.pool, &spec).await;
    }

    let kpis = repo::get_session_kpis(&db.pool, &SessionId::new(session))
        .await
        .expect("query kpis");

    assert_eq!(kpis.request_count, 3);
    assert_eq!(kpis.error_count, 1);
    assert_eq!(
        kpis.context_count, 1,
        "one distinct context across three requests"
    );
    assert_eq!(kpis.trace_count, 1);
    assert_eq!(kpis.total_input_tokens, 300);
    assert_eq!(kpis.total_output_tokens, 60);
    assert_eq!(kpis.total_cost_microdollars, 15_000);
    db.cleanup().await;
}

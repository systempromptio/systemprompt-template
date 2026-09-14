//! `repositories::users::usage::get_ai_request_summary` — the gateway rollup
//! behind the admin user-detail page's "AI requests" card.
//!
//! Two properties matter and neither is obvious from the signature: the
//! summary is lifetime, not windowed, and the legacy context sentinel is
//! excluded from the *conversation* count only. Its requests are real traffic
//! and still belong in the request, token, and cost totals.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::users::usage::get_ai_request_summary;

use crate::fixtures::{
    LEGACY_CONTEXT_ID, RequestSpec, insert_request, insert_user, new_context_id, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn the_summary_counts_conversations_without_the_legacy_sentinel() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("aisum")).await;
    let context_a = new_context_id();
    let context_b = new_context_id();

    for context in [context_a.as_str(), context_b.as_str(), LEGACY_CONTEXT_ID] {
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.context_id = Some(context);
        insert_request(&db.pool, &spec).await;
    }

    let summary = get_ai_request_summary(&db.pool, &user)
        .await
        .expect("query succeeds");

    assert_eq!(summary.requests, 3, "every request counts");
    assert_eq!(
        summary.conversations, 2,
        "the legacy sentinel is not a conversation"
    );
    assert!(summary.cost_microdollars > 0, "cost includes every request");
    assert!(summary.first_request_at.is_some());
    assert!(summary.last_request_at.is_some());
}

#[tokio::test]
async fn the_summary_is_lifetime_and_does_not_drop_rows_older_than_the_profile_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("aiold")).await;
    let context = new_context_id();
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.context_id = Some(&context);
    spec.created_at = Utc::now() - Duration::days(400);
    insert_request(&db.pool, &spec).await;

    let summary = get_ai_request_summary(&db.pool, &user)
        .await
        .expect("query succeeds");

    assert_eq!(summary.requests, 1);
    assert_eq!(summary.conversations, 1);
}

#[tokio::test]
async fn failed_requests_are_counted_separately() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("aifail")).await;
    let context = new_context_id();
    let mut ok = RequestSpec::completed(&unique("req"), &user);
    ok.context_id = Some(&context);
    insert_request(&db.pool, &ok).await;
    let mut bad = RequestSpec::completed(&unique("req"), &user);
    bad.context_id = Some(&context);
    bad.status = "failed";
    insert_request(&db.pool, &bad).await;

    let summary = get_ai_request_summary(&db.pool, &user)
        .await
        .expect("query succeeds");

    assert_eq!(summary.requests, 2);
    assert_eq!(summary.failed, 1);
}

#[tokio::test]
async fn a_user_with_no_traffic_gets_zeroes_rather_than_an_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("aizero")).await;

    let summary = get_ai_request_summary(&db.pool, &user)
        .await
        .expect("query succeeds");

    assert_eq!(summary.requests, 0);
    assert_eq!(summary.conversations, 0);
    assert!(summary.last_request_at.is_none());
}

//! `repositories::users::usage::get_conversation_summary` — the profile pane's
//! conversation rollup.
//!
//! The profile used to label a 30-day figure "all time" and to drop every
//! legacy-context request from the request count as well as the conversation
//! count, so a user whose traffic carried no context saw zero requests on a
//! page reporting their spend. The window is now carried on the summary and
//! the sentinel is excluded from the conversation count alone.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::users::usage::{
    CONVERSATION_WINDOW_DAYS, get_conversation_summary, list_top_models,
};

use crate::fixtures::{
    LEGACY_CONTEXT_ID, RequestSpec, insert_request, insert_user, new_context_id, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn legacy_context_requests_count_as_requests_but_not_as_conversations() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("convsum")).await;
    let context = new_context_id();
    // A lone request that offered no tools is a side call, not a
    // conversation; two requests in the context make it a turn thread.
    for _ in 0..2 {
        let mut real = RequestSpec::completed(&unique("req"), &user);
        real.context_id = Some(&context);
        insert_request(&db.pool, &real).await;
    }
    let mut legacy = RequestSpec::completed(&unique("req"), &user);
    legacy.context_id = Some(LEGACY_CONTEXT_ID);
    insert_request(&db.pool, &legacy).await;

    let summary = get_conversation_summary(&db.pool, &user)
        .await
        .expect("query succeeds");

    assert_eq!(summary.total_ai_requests, 3, "every request counts");
    assert_eq!(summary.total_conversations, 1, "the sentinel is not one");
    assert_eq!(summary.side_call_count, 0);
    assert_eq!(summary.recent.len(), 1);
    assert_eq!(summary.recent[0].turn_count, 2);
    assert!(summary.latest.is_some());
    assert_eq!(summary.window_days, CONVERSATION_WINDOW_DAYS);
}

#[tokio::test]
async fn the_recent_list_never_links_to_the_legacy_sentinel() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("convleg")).await;
    let mut legacy = RequestSpec::completed(&unique("req"), &user);
    legacy.context_id = Some(LEGACY_CONTEXT_ID);
    insert_request(&db.pool, &legacy).await;

    let summary = get_conversation_summary(&db.pool, &user)
        .await
        .expect("query succeeds");

    assert!(
        summary.recent.is_empty(),
        "a row here would open a context page belonging to no conversation"
    );
}

#[tokio::test]
async fn the_summary_covers_only_its_declared_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("convwin")).await;
    let context = new_context_id();
    let mut old = RequestSpec::completed(&unique("req"), &user);
    old.context_id = Some(&context);
    old.created_at = Utc::now() - Duration::days(i64::from(CONVERSATION_WINDOW_DAYS) + 5);
    insert_request(&db.pool, &old).await;

    let summary = get_conversation_summary(&db.pool, &user)
        .await
        .expect("query succeeds");

    assert_eq!(summary.total_ai_requests, 0);
    assert_eq!(summary.total_conversations, 0);
}

#[tokio::test]
async fn top_models_with_no_window_covers_traffic_the_windowed_call_drops() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("convmod")).await;
    let context = new_context_id();
    let mut old = RequestSpec::completed(&unique("req"), &user);
    old.context_id = Some(&context);
    old.created_at = Utc::now() - Duration::days(i64::from(CONVERSATION_WINDOW_DAYS) + 5);
    insert_request(&db.pool, &old).await;

    let windowed = list_top_models(&db.pool, &user, Some(CONVERSATION_WINDOW_DAYS), 5)
        .await
        .expect("query succeeds");
    let lifetime = list_top_models(&db.pool, &user, None, 5)
        .await
        .expect("query succeeds");

    assert!(windowed.is_empty(), "the row is outside the window");
    assert_eq!(lifetime.len(), 1, "the lifetime view keeps it");
    assert!((lifetime[0].token_share - 1.0).abs() < f64::EPSILON);
}

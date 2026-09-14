//! `repositories::analytics::conversations::list_history_items` — the unified
//! `/admin/history` list.
//!
//! The bug this exists for: a user whose only AI traffic goes through the
//! gateway has no `session_transcripts` rows at all, so the transcript-only
//! query showed them an empty history while their conversations were plainly
//! visible to an admin on the contexts page. These tests pin that both sources
//! appear, that the user-id scope constrains both of them, and that paging and
//! the total count are computed over the union rather than per source.

use systemprompt_web_admin::repositories::analytics::conversations::{
    HistoryFilter, HistorySource, list_history_items,
};

use crate::fixtures::{
    LEGACY_CONTEXT_ID, RequestSpec, insert_message, insert_offered_tools, insert_request,
    insert_session, insert_transcript, insert_user, new_context_id, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn both_sources_appear_for_the_same_viewer() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hist")).await;
    let session = unique("hist-session");
    insert_session(&db.pool, &session, &user).await;
    insert_transcript(&db.pool, &session, &user, "claude-test-model", "hook side").await;

    let context = new_context_id();
    let request = unique("req");
    let mut spec = RequestSpec::completed(&request, &user);
    spec.context_id = Some(&context);
    insert_request(&db.pool, &spec).await;
    insert_offered_tools(&db.pool, &request).await;
    insert_message(
        &db.pool,
        &request,
        0,
        "user",
        "=== USER PROMPT ===\ngateway side",
    )
    .await;

    let scope = vec![user.as_str().to_owned()];
    let (items, total) = list_history_items(
        &db.pool,
        HistoryFilter {
            scope_user_ids: Some(&scope),
            search: None,
            include_side_calls: true,
        },
        25,
        0,
    )
    .await
    .expect("query succeeds");

    assert_eq!(total, 2, "one transcript and one gateway conversation");
    assert!(
        items
            .iter()
            .any(|i| i.source == HistorySource::Transcript && i.session_id.is_some()),
        "the transcript row carries its session id"
    );
    let gateway = items
        .iter()
        .find(|i| i.source == HistorySource::Gateway)
        .expect("the gateway conversation is listed");
    assert_eq!(
        gateway.context_id.as_ref().map(|c| c.as_str()),
        Some(context.as_str())
    );
    assert_eq!(
        gateway.preview.as_deref(),
        Some("gateway side"),
        "the preview has the marker framing cut"
    );
}

#[tokio::test]
async fn the_scope_hides_another_users_rows_from_both_sources() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let viewer = insert_user(&db.pool, &unique("user"), &unclaimed_email("hscope")).await;
    let other = insert_user(&db.pool, &unique("user"), &unclaimed_email("hother")).await;

    let session = unique("other-session");
    insert_session(&db.pool, &session, &other).await;
    insert_transcript(&db.pool, &session, &other, "claude-test-model", "not yours").await;
    let context = new_context_id();
    let mut spec = RequestSpec::completed(&unique("req"), &other);
    spec.context_id = Some(&context);
    insert_request(&db.pool, &spec).await;

    let scope = vec![viewer.as_str().to_owned()];
    let (items, total) = list_history_items(
        &db.pool,
        HistoryFilter {
            scope_user_ids: Some(&scope),
            search: None,
            include_side_calls: true,
        },
        25,
        0,
    )
    .await
    .expect("query succeeds");

    assert_eq!(total, 0, "the viewer owns none of it");
    assert!(items.is_empty());
}

#[tokio::test]
async fn the_legacy_context_sentinel_is_not_listed_as_a_conversation() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hleg")).await;
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.context_id = Some(LEGACY_CONTEXT_ID);
    insert_request(&db.pool, &spec).await;

    let scope = vec![user.as_str().to_owned()];
    let (items, total) = list_history_items(
        &db.pool,
        HistoryFilter {
            scope_user_ids: Some(&scope),
            search: None,
            include_side_calls: true,
        },
        25,
        0,
    )
    .await
    .expect("query succeeds");

    assert_eq!(total, 0, "context-less traffic is not a conversation");
    assert!(items.is_empty());
}

#[tokio::test]
async fn search_matches_a_gateway_conversation_by_its_opening_prompt() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hsearch")).await;

    let wanted = new_context_id();
    let wanted_req = unique("req");
    let mut spec = RequestSpec::completed(&wanted_req, &user);
    spec.context_id = Some(&wanted);
    insert_request(&db.pool, &spec).await;
    insert_offered_tools(&db.pool, &wanted_req).await;
    insert_message(
        &db.pool,
        &wanted_req,
        0,
        "user",
        "how do I rotate the zygote key",
    )
    .await;

    let other = new_context_id();
    let other_req = unique("req");
    let mut spec = RequestSpec::completed(&other_req, &user);
    spec.context_id = Some(&other);
    insert_request(&db.pool, &spec).await;
    insert_offered_tools(&db.pool, &other_req).await;
    insert_message(&db.pool, &other_req, 0, "user", "unrelated question").await;

    let scope = vec![user.as_str().to_owned()];
    let (items, total) = list_history_items(
        &db.pool,
        HistoryFilter {
            scope_user_ids: Some(&scope),
            search: Some("zygote"),
            include_side_calls: true,
        },
        25,
        0,
    )
    .await
    .expect("query succeeds");

    assert_eq!(total, 1, "only the matching conversation");
    assert_eq!(
        items[0].context_id.as_ref().map(|c| c.as_str()),
        Some(wanted.as_str())
    );
}

#[tokio::test]
async fn the_total_count_spans_the_union_and_is_not_capped_by_the_page_size() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hpage")).await;
    for _ in 0..3 {
        let context = new_context_id();
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.context_id = Some(&context);
        insert_request(&db.pool, &spec).await;
    }

    let scope = vec![user.as_str().to_owned()];
    let (first, total) = list_history_items(
        &db.pool,
        HistoryFilter {
            scope_user_ids: Some(&scope),
            search: None,
            include_side_calls: true,
        },
        2,
        0,
    )
    .await
    .expect("query succeeds");
    let (second, second_total) = list_history_items(
        &db.pool,
        HistoryFilter {
            scope_user_ids: Some(&scope),
            search: None,
            include_side_calls: true,
        },
        2,
        2,
    )
    .await
    .expect("query succeeds");

    assert_eq!(total, 3);
    assert_eq!(second_total, 3, "the total is stable across pages");
    assert_eq!(first.len(), 2);
    assert_eq!(second.len(), 1);
}

#[tokio::test]
async fn an_unrestricted_scope_sees_every_users_rows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let a = insert_user(&db.pool, &unique("user"), &unclaimed_email("hall1")).await;
    let b = insert_user(&db.pool, &unique("user"), &unclaimed_email("hall2")).await;
    for owner in [&a, &b] {
        let context = new_context_id();
        let mut spec = RequestSpec::completed(&unique("req"), owner);
        spec.context_id = Some(&context);
        insert_request(&db.pool, &spec).await;
    }

    let (items, _) = list_history_items(
        &db.pool,
        HistoryFilter {
            scope_user_ids: None,
            search: None,
            include_side_calls: true,
        },
        100,
        0,
    )
    .await
    .expect("query succeeds");

    assert!(items.iter().any(|i| i.user_id == a));
    assert!(items.iter().any(|i| i.user_id == b));
}

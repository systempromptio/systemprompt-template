//! Pagination and scope invariants while sharing conversation aggregation.
use crate::fixtures::{
    RequestSpec, insert_request, insert_user, new_context_id, unclaimed_email, unique,
};
use crate::tempdb::TempDb;
use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::analytics::conversation_rows::{
    ConversationFilter, ConversationPage, ConversationPageMode, ConversationSort,
    load_conversation_page,
};
use systemprompt_web_admin::repositories::scope::SubjectScope;
use systemprompt_web_admin::repositories::users::roster::{
    RosterFilter, RosterQuery, RosterSort, get_roster_stats, list_users_paged,
};

#[tokio::test]
async fn user_pages_have_three_previews_and_stable_complete_totals() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mut ids = Vec::new();
    for index in 0..51 {
        let user = insert_user(
            &db.pool,
            &format!("page-user-{index:03}"),
            &unclaimed_email("pages"),
        )
        .await;
        ids.push(user.as_str().to_owned());
        for conversation in 0..4 {
            let context = new_context_id();
            for _ in 0..2 {
                let mut request = RequestSpec::completed(&unique("request"), &user);
                request.context_id = Some(&context);
                request.cost_microdollars = (index + 1) * 100;
                request.created_at = Utc::now() - Duration::days(conversation);
                insert_request(&db.pool, &request).await;
            }
        }
    }
    let filter = ConversationFilter {
        subject_ids: Some(ids),
        ..Default::default()
    };
    let page = ConversationPage {
        sort: ConversationSort::Cost,
        ..Default::default()
    };
    let first = load_conversation_page(&db.pool, &filter, page, ConversationPageMode::Users)
        .await
        .unwrap();
    assert_eq!(first.user_summaries.len(), 50);
    assert_eq!(first.conversations.len(), 150);
    assert_eq!(first.totals.users, 51);
    assert_eq!(first.totals.conversations, 204);
    assert_eq!(first.user_summaries[0].user_id.as_str(), "page-user-050");
    for summary in &first.user_summaries {
        assert_eq!(summary.conversation_count, 4);
        assert_eq!(summary.turn_count, 8);
        assert_eq!(summary.total_tokens, 960);
        assert_eq!(
            first
                .conversations
                .iter()
                .filter(|r| r.user_id.as_ref() == Some(&summary.user_id))
                .count(),
            3
        );
    }
    for sort in [ConversationSort::Turns, ConversationSort::Tokens] {
        let tied = load_conversation_page(
            &db.pool,
            &filter,
            ConversationPage { sort, ..page },
            ConversationPageMode::Users,
        )
        .await
        .unwrap();
        assert_eq!(
            tied.user_summaries[0].user_id.as_str(),
            "page-user-000",
            "equal aggregate values use stable user ID ordering"
        );
    }
    let last = load_conversation_page(
        &db.pool,
        &filter,
        ConversationPage { offset: 50, ..page },
        ConversationPageMode::Users,
    )
    .await
    .unwrap();
    assert_eq!(last.user_summaries.len(), 1);
    assert_eq!(last.user_summaries[0].user_id.as_str(), "page-user-000");
    assert_eq!(last.totals.users, 51);
    let empty = load_conversation_page(
        &db.pool,
        &filter,
        ConversationPage {
            offset: 1000,
            ..page
        },
        ConversationPageMode::All,
    )
    .await
    .unwrap();
    assert!(empty.conversations.is_empty());
    assert!(empty.user_summaries.is_empty());
    assert_eq!(empty.totals.conversations, 204);
    let ascending = load_conversation_page(
        &db.pool,
        &filter,
        ConversationPage {
            descending: false,
            ..page
        },
        ConversationPageMode::Users,
    )
    .await
    .unwrap();
    assert_eq!(
        ascending.user_summaries[0].user_id.as_str(),
        "page-user-000"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn recent_contexts_keep_old_turns_and_latest_owner_scope() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let old_owner = insert_user(&db.pool, &unique("old-owner"), &unclaimed_email("old")).await;
    let new_owner = insert_user(&db.pool, &unique("new-owner"), &unclaimed_email("new")).await;
    let context = new_context_id();
    let mut old = RequestSpec::completed(&unique("old-request"), &old_owner);
    old.context_id = Some(&context);
    old.created_at = Utc::now() - Duration::days(60);
    insert_request(&db.pool, &old).await;
    let mut recent = RequestSpec::completed(&unique("new-request"), &new_owner);
    recent.context_id = Some(&context);
    insert_request(&db.pool, &recent).await;
    let filter = ConversationFilter {
        user_id: Some(new_owner.clone()),
        since: Some(Utc::now() - Duration::days(30)),
        ..Default::default()
    };
    let result = load_conversation_page(
        &db.pool,
        &filter,
        ConversationPage::default(),
        ConversationPageMode::All,
    )
    .await
    .unwrap();
    assert_eq!(
        result.totals.turns, 2,
        "old request is retained and makes this a two-turn thread"
    );
    assert_eq!(result.totals.total_cost_microdollars, 10_000);
    assert_eq!(result.conversations[0].user_id.as_ref(), Some(&new_owner));
    assert!(result.user_summaries.is_empty());
    let hidden = load_conversation_page(
        &db.pool,
        &ConversationFilter {
            user_id: Some(old_owner),
            ..filter.clone()
        },
        ConversationPage::default(),
        ConversationPageMode::All,
    )
    .await
    .unwrap();
    assert_eq!(hidden.totals.conversations, 0);
    let none = load_conversation_page(
        &db.pool,
        &ConversationFilter {
            subject_ids: Some(Vec::new()),
            ..filter
        },
        ConversationPage::default(),
        ConversationPageMode::All,
    )
    .await
    .unwrap();
    assert_eq!(none.totals.conversations, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn roster_windows_and_empty_pages_keep_counts_and_ranking() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("roster"), &unclaimed_email("roster")).await;
    for days in [1, 31, 90] {
        let mut request = RequestSpec::completed(&unique("spend"), &user);
        request.created_at = Utc::now() - Duration::days(days);
        insert_request(&db.pool, &request).await;
    }
    let scope = SubjectScope::All;
    let stats = get_roster_stats(&db.pool, &scope).await.unwrap();
    assert_eq!(stats.cost_microdollars, 5000);
    assert_eq!(stats.prior_cost_microdollars, 5000);
    let query = RosterQuery {
        filter: RosterFilter::None,
        role: None,
        search: Some(user.as_str().to_owned()),
        sort: RosterSort::default(),
        limit: 50,
        offset: 0,
    };
    let (rows, total) = list_users_paged(&db.pool, &scope, &query).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(rows[0].cost_microdollars, 5000);
    let (rows, total) = list_users_paged(
        &db.pool,
        &scope,
        &RosterQuery {
            offset: 50,
            ..query
        },
    )
    .await
    .unwrap();
    assert!(rows.is_empty());
    assert_eq!(total, 1);
    db.cleanup().await;
}

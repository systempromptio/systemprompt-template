//! `repositories::analytics::{requests, request_stats}` — the paged gateway
//! request listing, its per-dimension breakdowns, and the KPI strip.
//!
//! Every windowed query runs over `fixtures::narrow_window`, which starts 30
//! seconds ago. Migration 025 seeds ~1080 demo `ai_requests`, but none newer
//! than a minute old, so the window isolates the rows each test inserts.

use systemprompt_web_admin::repositories::analytics::requests::{
    RequestFilter, RequestPage, RequestSortColumn, RequestSortSpec, SortDir,
    list_requests_by_model, list_requests_by_provider, list_requests_by_status,
    list_requests_paged,
};
use systemprompt_web_admin::util::org_scope::OrgScope;

use crate::fixtures::{
    DecisionSpec, EventSpec, RequestSpec, insert_decision, insert_event, insert_request,
    insert_session, insert_user, narrow_window, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

fn page() -> RequestPage {
    RequestPage {
        sort: RequestSortSpec::default(),
        limit: 50,
        offset: 0,
    }
}

#[tokio::test]
async fn list_requests_paged_finds_nothing_in_an_empty_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (rows, total) =
        list_requests_paged(&db.pool, &RequestFilter::default(), narrow_window(), page())
            .await
            .expect("query succeeds");

    assert!(rows.is_empty());
    assert_eq!(
        total, 0,
        "the total comes from the first row, so it is 0 here"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_returns_the_row_it_was_given() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("req")).await;
    let id = unique("req");
    let mut spec = RequestSpec::completed(&id, &user);
    spec.model = "claude-under-test";
    spec.latency_ms = 640;
    insert_request(&db.pool, &spec).await;

    let (rows, total) =
        list_requests_paged(&db.pool, &RequestFilter::default(), narrow_window(), page())
            .await
            .expect("query succeeds");

    assert_eq!(total, 1);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, id);
    assert_eq!(rows[0].model, "claude-under-test");
    assert_eq!(rows[0].latency_ms, Some(640));
    assert_eq!(rows[0].cost_microdollars, 5_000);
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_labels_the_row_with_the_users_display_name() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("labelled");
    let user = insert_user(&db.pool, &unique("user"), &email).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &user)).await;

    let (rows, _) =
        list_requests_paged(&db.pool, &RequestFilter::default(), narrow_window(), page())
            .await
            .expect("query succeeds");

    assert_eq!(
        rows[0].user_label.as_deref(),
        Some(email.as_str()),
        "the label is the joined display name, not the raw user id"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_filters_by_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = insert_user(&db.pool, &unique("user"), &unclaimed_email("mine")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("theirs")).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &mine)).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &theirs)).await;
    let filter = RequestFilter {
        user_id: Some(mine.clone()),
        ..RequestFilter::default()
    };

    let (rows, total) = list_requests_paged(&db.pool, &filter, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(total, 1);
    assert_eq!(rows[0].user_id, mine);
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_filters_by_model_and_provider() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("filters")).await;
    let wanted = unique("req");
    let mut hit = RequestSpec::completed(&wanted, &user);
    hit.model = "wanted-model";
    hit.provider = "wanted-provider";
    insert_request(&db.pool, &hit).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &user)).await;

    let by_model = RequestFilter {
        model: Some("wanted-model".to_owned()),
        ..RequestFilter::default()
    };
    let by_provider = RequestFilter {
        provider: Some("wanted-provider".to_owned()),
        ..RequestFilter::default()
    };
    let (model_rows, _) = list_requests_paged(&db.pool, &by_model, narrow_window(), page())
        .await
        .expect("query succeeds");
    let (provider_rows, _) = list_requests_paged(&db.pool, &by_provider, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(model_rows.len(), 1);
    assert_eq!(model_rows[0].id, wanted);
    assert_eq!(provider_rows.len(), 1);
    assert_eq!(provider_rows[0].id, wanted);
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_search_matches_the_error_message() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("search")).await;
    let failing = unique("req");
    let mut spec = RequestSpec::completed(&failing, &user);
    spec.status = "failed";
    insert_request(&db.pool, &spec).await;
    sqlx::query(
        "UPDATE ai_requests SET error_message = 'upstream refused the handshake' WHERE id = $1",
    )
    .bind(&failing)
    .execute(&*db.pool)
    .await
    .expect("set the error message");
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &user)).await;
    let filter = RequestFilter {
        search: Some("handshake".to_owned()),
        ..RequestFilter::default()
    };

    let (rows, _) = list_requests_paged(&db.pool, &filter, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, failing);
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_treats_an_empty_search_as_no_filter() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("blank")).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &user)).await;
    let filter = RequestFilter {
        search: Some(String::new()),
        ..RequestFilter::default()
    };

    let (rows, _) = list_requests_paged(&db.pool, &filter, narrow_window(), page())
        .await
        .expect("query succeeds");

    assert_eq!(rows.len(), 1);
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_sorts_by_cost_in_both_directions() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("sorted")).await;
    let cheap = unique("req");
    let dear = unique("req");
    let mut low = RequestSpec::completed(&cheap, &user);
    low.cost_microdollars = 10;
    insert_request(&db.pool, &low).await;
    let mut high = RequestSpec::completed(&dear, &user);
    high.cost_microdollars = 99_000;
    insert_request(&db.pool, &high).await;

    let descending = RequestPage {
        sort: RequestSortSpec {
            column: RequestSortColumn::Cost,
            dir: SortDir::Desc,
        },
        limit: 50,
        offset: 0,
    };
    let ascending = RequestPage {
        sort: RequestSortSpec {
            column: RequestSortColumn::Cost,
            dir: SortDir::Asc,
        },
        ..descending
    };
    let (desc_rows, _) = list_requests_paged(
        &db.pool,
        &RequestFilter::default(),
        narrow_window(),
        descending,
    )
    .await
    .expect("query succeeds");
    let (asc_rows, _) = list_requests_paged(
        &db.pool,
        &RequestFilter::default(),
        narrow_window(),
        ascending,
    )
    .await
    .expect("query succeeds");

    assert_eq!(desc_rows[0].id, dear);
    assert_eq!(asc_rows[0].id, cheap);
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_reports_the_unpaged_total_alongside_the_page() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("paged")).await;
    for _ in 0..3 {
        insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &user)).await;
    }
    let one_at_a_time = RequestPage {
        sort: RequestSortSpec::default(),
        limit: 1,
        offset: 1,
    };

    let (rows, total) = list_requests_paged(
        &db.pool,
        &RequestFilter::default(),
        narrow_window(),
        one_at_a_time,
    )
    .await
    .expect("query succeeds");

    assert_eq!(rows.len(), 1, "the page is the limit");
    assert_eq!(total, 3, "the total is the whole filtered set");
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_paged_counts_governance_and_tool_activity_per_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("counted")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.session_id = Some(&session);
    insert_request(&db.pool, &spec).await;
    insert_decision(
        &db.pool,
        &DecisionSpec::allow(&unique("dec"), &user, &session),
    )
    .await;
    let mut denied = DecisionSpec::allow(&unique("dec"), &user, &session);
    denied.decision = "deny";
    insert_decision(&db.pool, &denied).await;
    insert_event(
        &db.pool,
        &EventSpec::tool_use(&unique("evt"), &user, &session),
    )
    .await;

    let (rows, _) =
        list_requests_paged(&db.pool, &RequestFilter::default(), narrow_window(), page())
            .await
            .expect("query succeeds");

    assert_eq!(rows[0].decision_count, 2);
    assert_eq!(rows[0].deny_count, 1);
    assert_eq!(rows[0].tool_call_count, 1, "PostToolUse matches the ILIKE");
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_by_model_rolls_up_the_windows_traffic() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("bymodel")).await;
    for _ in 0..2 {
        let mut spec = RequestSpec::completed(&unique("req"), &user);
        spec.model = "rollup-model";
        insert_request(&db.pool, &spec).await;
    }

    let rows = list_requests_by_model(&db.pool, narrow_window(), &OrgScope::AllOrganizations)
        .await
        .expect("query succeeds");

    let row = rows
        .iter()
        .find(|r| r.key == "rollup-model")
        .expect("the model appears in the breakdown");
    assert_eq!(row.requests, 2);
    assert_eq!(row.input_tokens, 200);
    assert_eq!(row.output_tokens, 40);
    assert_eq!(row.cost_microdollars, 10_000);
    assert_eq!(row.error_count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_requests_by_provider_and_status_agree_on_what_an_error_is() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("errors")).await;
    let mut failed = RequestSpec::completed(&unique("req"), &user);
    failed.provider = "flaky-provider";
    failed.status = "failed";
    insert_request(&db.pool, &failed).await;
    let mut fine = RequestSpec::completed(&unique("req"), &user);
    fine.provider = "flaky-provider";
    insert_request(&db.pool, &fine).await;

    let by_provider =
        list_requests_by_provider(&db.pool, narrow_window(), &OrgScope::AllOrganizations)
            .await
            .expect("query succeeds");
    let by_status = list_requests_by_status(&db.pool, narrow_window(), &OrgScope::AllOrganizations)
        .await
        .expect("query succeeds");

    let provider_row = by_provider
        .iter()
        .find(|r| r.key == "flaky-provider")
        .expect("the provider appears");
    assert_eq!(provider_row.requests, 2);
    assert_eq!(provider_row.error_count, 1);
    let failed_row = by_status
        .iter()
        .find(|r| r.key == "failed")
        .expect("the status appears");
    assert_eq!(
        failed_row.error_count, 1,
        "'failed' is an error on both tabs"
    );
    let completed_row = by_status
        .iter()
        .find(|r| r.key == "completed")
        .expect("the status appears");
    assert_eq!(completed_row.error_count, 0);
    db.cleanup().await;
}

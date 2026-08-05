//! `repositories::analytics::content_rollup::upsert_metrics` — the write half
//! of the content rollup.
//!
//! The row's primary key is `id` but its conflict target is `content_id`, so a
//! second run with a freshly minted `id` must update rather than insert. That
//! asymmetry is the whole reason this function has its own tests.

use systemprompt_web_admin::repositories::analytics::content_rollup::{
    UpsertMetricsParams, upsert_metrics,
};

use crate::analytics_content_rollup::insert_content;
use crate::fixtures::unique;
use crate::tempdb::TempDb;

#[tokio::test]
async fn upsert_metrics_inserts_then_overwrites_the_row_for_a_content_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_id = unique("content");
    insert_content(&db.pool, &content_id, &unique("slug"), "blog").await;

    upsert_metrics(
        &db.pool,
        &UpsertMetricsParams {
            id: &unique("metric"),
            content_id: &content_id,
            total_views: 10,
            unique_visitors: 4,
            avg_time_seconds: 12.5,
            views_7d: 6,
            views_30d: 9,
            trend_direction: "up",
        },
    )
    .await
    .expect("insert metrics");

    upsert_metrics(
        &db.pool,
        &UpsertMetricsParams {
            id: &unique("metric"),
            content_id: &content_id,
            total_views: 20,
            unique_visitors: 7,
            avg_time_seconds: 30.0,
            views_7d: 11,
            views_30d: 18,
            trend_direction: "down",
        },
    )
    .await
    .expect("update metrics");

    let row: (i64, i32, String) = sqlx::query_as(
        "SELECT COUNT(*) OVER ()::bigint, total_views, trend_direction
         FROM content_performance_metrics WHERE content_id = $1",
    )
    .bind(&content_id)
    .fetch_one(&*db.pool)
    .await
    .expect("read metrics");

    assert_eq!(row.0, 1, "the conflict target is content_id, not id");
    assert_eq!(row.1, 20);
    assert_eq!(row.2, "down");
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_metrics_rejects_a_content_id_with_no_markdown_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = upsert_metrics(
        &db.pool,
        &UpsertMetricsParams {
            id: &unique("metric"),
            content_id: &unique("missing"),
            total_views: 1,
            unique_visitors: 1,
            avg_time_seconds: 1.0,
            views_7d: 1,
            views_30d: 1,
            trend_direction: "flat",
        },
    )
    .await;

    assert!(result.is_err(), "content_id carries a foreign key");
    db.cleanup().await;
}

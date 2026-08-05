//! `repositories::dashboard::traffic::queries::list_content_performance` — the
//! per-page engagement table on the traffic dashboard.
//!
//! The range argument picks the data source, not just a filter: `7d` and `30d`
//! read the rollup the aggregation job materialises, and every other range is
//! computed live from `engagement_events`. The two paths return the same row
//! shape but differ in what they can populate, so both are covered here.

use systemprompt_web_admin::repositories::dashboard::traffic::list_content_performance;

use crate::dashboard_traffic_queries::{
    insert_content_metrics, insert_view, insert_view_yesterday,
};
use crate::fixtures::unique;
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_content_performance_reads_the_precomputed_rollup_for_seven_days() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_content_metrics(&db.pool, "Weekly winner", 40, 5).await;
    insert_content_metrics(&db.pool, "Monthly winner", 1, 900).await;

    let rows = list_content_performance(&db.pool, "7d")
        .await
        .expect("content performance");

    assert_eq!(rows[0].title, "Weekly winner");
    assert_eq!(rows[0].views, 40);
    assert_eq!(rows[0].trend.as_deref(), Some("up"));
    db.cleanup().await;
}

#[tokio::test]
async fn list_content_performance_switches_to_the_thirty_day_column() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_content_metrics(&db.pool, "Weekly winner", 40, 5).await;
    insert_content_metrics(&db.pool, "Monthly winner", 1, 900).await;

    let rows = list_content_performance(&db.pool, "30d")
        .await
        .expect("content performance");

    assert_eq!(rows[0].title, "Monthly winner");
    assert_eq!(rows[0].views, 900);
    db.cleanup().await;
}

#[tokio::test]
async fn list_content_performance_drops_rollup_rows_with_no_views_in_the_range() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_content_metrics(&db.pool, "Never read this week", 0, 12).await;

    let rows = list_content_performance(&db.pool, "7d")
        .await
        .expect("content performance");

    assert!(rows.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_content_performance_computes_short_ranges_live_from_engagement_events() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_view(&db.pool, "/blog/live", &unique("sess"), 8_000).await;
    insert_view(&db.pool, "/blog/live", &unique("sess"), 4_000).await;

    let rows = list_content_performance(&db.pool, "1h")
        .await
        .expect("content performance");

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].title, "/blog/live",
        "no markdown row, so the URL is the title"
    );
    assert_eq!(rows[0].views, 2);
    assert!(
        rows[0].trend.is_none(),
        "the live path has no trend to report"
    );
    assert!((rows[0].avg_time_seconds - 6.0).abs() < 0.01);
    db.cleanup().await;
}

#[tokio::test]
async fn list_content_performance_for_yesterday_excludes_todays_views() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_view(&db.pool, "/blog/today", &unique("sess"), 3_000).await;
    insert_view_yesterday(&db.pool, "/blog/yesterday").await;

    let rows = list_content_performance(&db.pool, "yesterday")
        .await
        .expect("content performance");

    let titles: Vec<&str> = rows.iter().map(|r| r.title.as_str()).collect();
    assert_eq!(titles, ["/blog/yesterday"]);
    db.cleanup().await;
}

#[tokio::test]
async fn list_content_performance_falls_back_to_a_twenty_four_hour_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_view(&db.pool, "/blog/today", &unique("sess"), 3_000).await;
    insert_view_yesterday(&db.pool, "/blog/yesterday").await;

    let rows = list_content_performance(&db.pool, "today")
        .await
        .expect("content performance");

    assert!(
        rows.iter().any(|r| r.title == "/blog/today"),
        "an unrecognised range means the default 24-hour window"
    );
    db.cleanup().await;
}

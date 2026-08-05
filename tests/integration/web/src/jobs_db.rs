//! The database-backed half of `systemprompt-web-jobs`.
//!
//! Only `ContentAnalyticsAggregationJob` exposes a pool-only entry point
//! (`execute_with_pool`); the rest of the jobs reach for `JobContext`,
//! `AppPaths`, or the global `Config`, none of which exist in a test process.
//! See the module note at the bottom of this file for what that leaves
//! uncovered.

use std::sync::Arc;

use systemprompt::identifiers::{ContentId, SourceId};
use systemprompt_web_admin::repositories::analytics::content_rollup::{self, UpsertMetricsParams};
use systemprompt_web_content::repository::ContentRepository;
use systemprompt_web_jobs::ContentAnalyticsAggregationJob;

use crate::fixtures::content_params;
use crate::tempdb::TempDb;

fn blog_source() -> SourceId {
    SourceId::new("blog".to_string())
}

async fn seed_article(db: &TempDb, slug: &str) -> ContentId {
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    repo.create(&content_params(slug, &blog_source()))
        .await
        .expect("create the article the metrics hang off")
        .id
}

async fn record_view(db: &TempDb, page_url: &str, session: &str, time_on_page_ms: i32) {
    sqlx::query(
        "INSERT INTO engagement_events (session_id, user_id, page_url, time_on_page_ms) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(session)
    .bind("user-analytics")
    .bind(page_url)
    .bind(time_on_page_ms)
    .execute(&*db.pool)
    .await
    .expect("record an engagement event");
}

async fn metrics_row(db: &TempDb, content_id: &ContentId) -> Option<(i32, i32, String)> {
    sqlx::query_as::<_, (i32, i32, Option<String>)>(
        "SELECT total_views, views_last_7_days, trend_direction FROM \
         content_performance_metrics WHERE content_id = $1",
    )
    .bind(content_id.as_str())
    .fetch_optional(&*db.pool)
    .await
    .expect("read the metrics row")
    .map(|(total, week, trend)| (total, week, trend.unwrap_or_default()))
}

async fn metrics_row_count(db: &TempDb, content_id: &ContentId) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM content_performance_metrics WHERE content_id = $1",
    )
    .bind(content_id.as_str())
    .fetch_one(&*db.pool)
    .await
    .expect("count the metrics rows")
}

#[tokio::test]
async fn upsert_metrics_inserts_the_first_rollup() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_id = seed_article(&db, "rollup-insert").await;

    content_rollup::upsert_metrics(
        &db.pool,
        &UpsertMetricsParams {
            id: "cpm_first",
            content_id: content_id.as_str(),
            total_views: 12,
            unique_visitors: 7,
            avg_time_seconds: 42.5,
            views_7d: 5,
            views_30d: 11,
            trend_direction: "up",
        },
    )
    .await
    .expect("insert the first rollup");

    let row = metrics_row(&db, &content_id)
        .await
        .expect("the rollup row exists");
    assert_eq!(row, (12, 5, "up".to_string()));

    db.cleanup().await;
}

#[tokio::test]
async fn upsert_metrics_updates_in_place_on_the_second_run() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_id = seed_article(&db, "rollup-update").await;

    for (id, total, week, trend) in [("cpm_run_one", 12, 5, "up"), ("cpm_run_two", 30, 2, "down")] {
        content_rollup::upsert_metrics(
            &db.pool,
            &UpsertMetricsParams {
                id,
                content_id: content_id.as_str(),
                total_views: total,
                unique_visitors: 7,
                avg_time_seconds: 42.5,
                views_7d: week,
                views_30d: 11,
                trend_direction: trend,
            },
        )
        .await
        .expect("upsert the rollup");
    }

    assert_eq!(
        metrics_row_count(&db, &content_id).await,
        1,
        "content_id is unique, so the second run updates rather than inserting"
    );
    let row = metrics_row(&db, &content_id)
        .await
        .expect("the rollup row exists");
    assert_eq!(
        row,
        (30, 2, "down".to_string()),
        "the second run's values win"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_is_empty_with_no_events() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_article(&db, "unvisited").await;

    let stats = content_rollup::aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate with no engagement events");

    assert!(
        stats.is_empty(),
        "content with no engagement events produces no rollup row"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_joins_blog_page_urls_to_their_slug() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_id = seed_article(&db, "visited").await;
    record_view(&db, "/blog/visited", "sess-a", 4_000).await;
    record_view(&db, "/blog/visited", "sess-b", 6_000).await;

    let stats = content_rollup::aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate engagement events");

    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].content_id, content_id);
    assert_eq!(stats[0].total_views, 2);
    assert_eq!(
        stats[0].unique_visitors, 2,
        "unique visitors counts distinct sessions"
    );
    assert!(
        (stats[0].avg_time_seconds - 5.0).abs() < f64::EPSILON,
        "time on page is reported in seconds, got {}",
        stats[0].avg_time_seconds
    );

    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_ignores_urls_that_match_no_content() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_article(&db, "known").await;
    record_view(&db, "/blog/does-not-exist", "sess-a", 1_000).await;
    record_view(&db, "/some/other/page", "sess-b", 1_000).await;

    let stats = content_rollup::aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate engagement events");

    assert!(
        stats.is_empty(),
        "events whose page_url resolves to no content row are dropped by the join"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_job_writes_a_rollup_row_for_every_visited_article() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let visited = seed_article(&db, "job-visited").await;
    let unvisited = seed_article(&db, "job-unvisited").await;
    record_view(&db, "/blog/job-visited", "sess-a", 3_000).await;
    record_view(&db, "/blog/job-visited", "sess-b", 3_000).await;

    let result = ContentAnalyticsAggregationJob::execute_with_pool(&db.pool)
        .await
        .expect("run the aggregation job");

    assert!(result.success);
    assert_eq!(result.items_processed, Some(1));
    assert_eq!(result.items_failed, Some(0));

    let row = metrics_row(&db, &visited)
        .await
        .expect("the visited article got a rollup row");
    assert_eq!(row.0, 2, "both views are counted");
    assert_eq!(
        row.2, "up",
        "views entirely inside the last 7 days trend upwards"
    );
    assert!(
        metrics_row(&db, &unvisited).await.is_none(),
        "an article with no events gets no rollup row"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_job_succeeds_on_an_empty_database() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = ContentAnalyticsAggregationJob::execute_with_pool(&db.pool)
        .await
        .expect("run the aggregation job against an empty database");

    assert!(result.success);
    assert_eq!(result.items_processed, Some(0));
    assert_eq!(result.items_failed, Some(0));

    db.cleanup().await;
}

#[tokio::test]
async fn the_job_is_idempotent_across_runs() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content_id = seed_article(&db, "job-idempotent").await;
    record_view(&db, "/blog/job-idempotent", "sess-a", 2_000).await;

    for _ in 0..2 {
        ContentAnalyticsAggregationJob::execute_with_pool(&db.pool)
            .await
            .expect("run the aggregation job");
    }

    assert_eq!(
        metrics_row_count(&db, &content_id).await,
        1,
        "a second run of the job updates the existing rollup rather than adding one"
    );

    db.cleanup().await;
}

// Deliberately uncovered here:
//
// - `llms_txt`: `generate_llms_txt` needs a global `Config` and an `AppPaths`,
//   and its two pool-touching helpers (`build_llms_txt_content`,
//   `write_documentation_section`) are private. Only the pure formatting
//   helpers are re-exported (`jobs::internals`), and they take no pool.
// - `daily_summary::generate_user_daily_summary`: the `daily_summary` module is
//   not declared in `extensions/web/jobs/src/lib.rs`, so it is not compiled
//   into the crate and cannot be called from outside it.

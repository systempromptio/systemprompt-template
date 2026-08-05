//! `repositories::analytics::content_rollup` — the per-content engagement
//! rollup the analytics job computes and persists.
//!
//! `aggregate_engagement_stats` joins raw `engagement_events` to
//! `markdown_content` by reconstructing the slug out of `page_url`, one
//! `SUBSTRING` offset per public route prefix. Those offsets are the fragile
//! part, so each prefix gets its own test. Neither table is seeded by any
//! migration, so these tests own the whole result set.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::analytics::content_rollup::aggregate_engagement_stats;

use crate::fixtures::unique;
use crate::tempdb::TempDb;

pub async fn insert_content(pool: &sqlx::PgPool, id: &str, slug: &str, source_id: &str) {
    sqlx::query(
        "INSERT INTO markdown_content
             (id, slug, title, description, body, author, published_at, keywords,
              source_id, version_hash)
         VALUES ($1, $2, $3, 'fixture', 'body', 'fixture', NOW(), '', $4, 'v1')",
    )
    .bind(id)
    .bind(slug)
    .bind(format!("Title {slug}"))
    .bind(source_id)
    .execute(pool)
    .await
    .expect("insert markdown content");
}

struct Event<'a> {
    page_url: &'a str,
    session_id: &'a str,
    time_on_page_ms: i32,
    age: Duration,
}

async fn insert_engagement(pool: &sqlx::PgPool, event: &Event<'_>) {
    sqlx::query(
        "INSERT INTO engagement_events
             (id, session_id, user_id, page_url, time_on_page_ms, created_at)
         VALUES ($1, $2, 'anon', $3, $4, $5)",
    )
    .bind(unique("ee"))
    .bind(event.session_id)
    .bind(event.page_url)
    .bind(event.time_on_page_ms)
    .bind(Utc::now() - event.age)
    .execute(pool)
    .await
    .expect("insert engagement event");
}

async fn seed_one(pool: &sqlx::PgPool, source_id: &str, page_url: &str) -> String {
    let id = unique("content");
    let slug = unique("slug");
    insert_content(pool, &id, &slug, source_id).await;
    insert_engagement(
        pool,
        &Event {
            page_url: &format!("{page_url}{slug}"),
            session_id: &unique("sess"),
            time_on_page_ms: 4_000,
            age: Duration::hours(1),
        },
    )
    .await;
    id
}

#[tokio::test]
async fn aggregate_engagement_stats_is_empty_with_no_events() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert!(rows.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_matches_blog_urls() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = seed_one(&db.pool, "blog", "/blog/").await;

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].content_id.as_str(), id);
    assert_eq!(rows[0].total_views, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_matches_documentation_urls() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = seed_one(&db.pool, "documentation", "/documentation/").await;

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].content_id.as_str(), id);
    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_matches_playbook_urls() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = seed_one(&db.pool, "playbooks", "/playbooks/").await;

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].content_id.as_str(), id);
    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_matches_legal_urls() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = seed_one(&db.pool, "legal", "/legal/").await;

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].content_id.as_str(), id);
    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_ignores_a_url_whose_prefix_is_not_a_content_route() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let slug = unique("slug");
    insert_content(&db.pool, &unique("content"), &slug, "blog").await;
    insert_engagement(
        &db.pool,
        &Event {
            page_url: &format!("/features/{slug}"),
            session_id: &unique("sess"),
            time_on_page_ms: 4_000,
            age: Duration::hours(1),
        },
    )
    .await;

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert!(rows.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_counts_unique_visitors_and_windows_separately() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("content");
    let slug = unique("slug");
    insert_content(&db.pool, &id, &slug, "blog").await;
    let url = format!("/blog/{slug}");
    let session = unique("sess");
    // Two hits from one session today, one from another session 20 days ago.
    for age in [Duration::hours(1), Duration::hours(2)] {
        insert_engagement(
            &db.pool,
            &Event {
                page_url: &url,
                session_id: &session,
                time_on_page_ms: 2_000,
                age,
            },
        )
        .await;
    }
    insert_engagement(
        &db.pool,
        &Event {
            page_url: &url,
            session_id: &unique("sess"),
            time_on_page_ms: 6_000,
            age: Duration::days(20),
        },
    )
    .await;

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.total_views, 3);
    assert_eq!(row.unique_visitors, 2);
    assert_eq!(row.views_7d, 2);
    assert_eq!(row.views_30d, 3);
    assert!(
        (row.avg_time_seconds - 10.0 / 3.0).abs() < 0.01,
        "avg of 2s, 2s, 6s: {}",
        row.avg_time_seconds
    );
    db.cleanup().await;
}

#[tokio::test]
async fn aggregate_engagement_stats_excludes_zero_duration_hits_from_the_view_counts() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("content");
    let slug = unique("slug");
    insert_content(&db.pool, &id, &slug, "blog").await;
    insert_engagement(
        &db.pool,
        &Event {
            page_url: &format!("/blog/{slug}"),
            session_id: &unique("sess"),
            time_on_page_ms: 0,
            age: Duration::hours(1),
        },
    )
    .await;

    let rows = aggregate_engagement_stats(&db.pool)
        .await
        .expect("aggregate");

    assert_eq!(
        rows.len(),
        1,
        "the row still appears — HAVING counts all events"
    );
    assert_eq!(rows[0].total_views, 0);
    assert_eq!(rows[0].unique_visitors, 1);
    db.cleanup().await;
}

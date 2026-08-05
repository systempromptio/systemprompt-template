//! `repositories::dashboard::traffic::queries` — the public-site traffic reads.
//!
//! `user_sessions`, `engagement_events`, `markdown_content` and
//! `content_performance_metrics` are all unseeded by the migrations, so every
//! assertion here is over rows this suite inserted.
//!
//! The bot predicate (`is_bot`, `is_scanner`, `is_behavioral_bot`,
//! `request_count > 0`) is restated in every one of these queries rather than
//! being read from `v_clean_traffic`, so it is pinned per query.

use systemprompt_web_admin::repositories::dashboard::traffic::{
    get_realtime_pulse, list_top_pages_today,
};

use crate::fixtures::unique;
use crate::tempdb::TempDb;

pub struct SessionSpec<'a> {
    pub country: &'a str,
    pub is_bot: bool,
    pub is_scanner: bool,
    pub is_behavioral_bot: bool,
    pub request_count: i32,
}

impl Default for SessionSpec<'_> {
    fn default() -> Self {
        Self {
            country: "ES",
            is_bot: false,
            is_scanner: false,
            is_behavioral_bot: false,
            request_count: 3,
        }
    }
}

pub async fn insert_traffic_session(pool: &sqlx::PgPool, spec: &SessionSpec<'_>) {
    sqlx::query(
        "INSERT INTO user_sessions
             (session_id, started_at, country, is_bot, is_scanner, is_behavioral_bot,
              request_count)
         VALUES ($1, NOW(), $2, $3, $4, $5, $6)",
    )
    .bind(unique("sess"))
    .bind(spec.country)
    .bind(spec.is_bot)
    .bind(spec.is_scanner)
    .bind(spec.is_behavioral_bot)
    .bind(spec.request_count)
    .execute(pool)
    .await
    .expect("insert traffic session");
}

pub async fn insert_view(
    pool: &sqlx::PgPool,
    page_url: &str,
    session_id: &str,
    time_on_page_ms: i32,
) {
    sqlx::query(
        "INSERT INTO engagement_events
             (id, session_id, user_id, page_url, time_on_page_ms, created_at)
         VALUES ($1, $2, 'anon', $3, $4, NOW())",
    )
    .bind(unique("ee"))
    .bind(session_id)
    .bind(page_url)
    .bind(time_on_page_ms)
    .execute(pool)
    .await
    .expect("insert engagement event");
}

pub async fn insert_view_yesterday(pool: &sqlx::PgPool, page_url: &str) {
    sqlx::query(
        "INSERT INTO engagement_events
             (id, session_id, user_id, page_url, time_on_page_ms, created_at)
         VALUES ($1, $2, 'anon', $3, 5000, CURRENT_DATE - INTERVAL '12 hours')",
    )
    .bind(unique("ee"))
    .bind(unique("sess"))
    .bind(page_url)
    .execute(pool)
    .await
    .expect("insert yesterday engagement event");
}

pub async fn insert_content_metrics(
    pool: &sqlx::PgPool,
    title: &str,
    views_7d: i32,
    views_30d: i32,
) {
    let content_id = unique("content");
    sqlx::query(
        "INSERT INTO markdown_content
             (id, slug, title, description, body, author, published_at, keywords,
              source_id, version_hash)
         VALUES ($1, $2, $3, 'fixture', 'body', 'fixture', NOW(), '', 'blog', 'v1')",
    )
    .bind(&content_id)
    .bind(unique("slug"))
    .bind(title)
    .execute(pool)
    .await
    .expect("insert markdown content");

    sqlx::query(
        "INSERT INTO content_performance_metrics
             (id, content_id, views_last_7_days, views_last_30_days,
              avg_time_on_page_seconds, trend_direction)
         VALUES ($1, $2, $3, $4, 42.0, 'up')",
    )
    .bind(unique("metric"))
    .bind(&content_id)
    .bind(views_7d)
    .bind(views_30d)
    .execute(pool)
    .await
    .expect("insert content metrics");
}

#[tokio::test]
async fn get_realtime_pulse_is_all_zero_on_a_fresh_database() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let pulse = get_realtime_pulse(&db.pool).await.expect("pulse");

    assert_eq!(pulse.sessions_this_hour, 0);
    assert_eq!(pulse.page_views_this_hour, 0);
    assert_eq!(pulse.unique_visitors_today, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn get_realtime_pulse_counts_only_human_sessions_with_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_traffic_session(&db.pool, &SessionSpec::default()).await;
    insert_traffic_session(
        &db.pool,
        &SessionSpec {
            is_bot: true,
            ..SessionSpec::default()
        },
    )
    .await;
    insert_traffic_session(
        &db.pool,
        &SessionSpec {
            is_scanner: true,
            ..SessionSpec::default()
        },
    )
    .await;
    insert_traffic_session(
        &db.pool,
        &SessionSpec {
            is_behavioral_bot: true,
            ..SessionSpec::default()
        },
    )
    .await;
    insert_traffic_session(
        &db.pool,
        &SessionSpec {
            request_count: 0,
            ..SessionSpec::default()
        },
    )
    .await;

    let pulse = get_realtime_pulse(&db.pool).await.expect("pulse");

    assert_eq!(pulse.sessions_this_hour, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn get_realtime_pulse_counts_page_views_and_distinct_visitors() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let session = unique("sess");
    insert_view(&db.pool, "/blog/a", &session, 3_000).await;
    insert_view(&db.pool, "/blog/b", &session, 4_000).await;
    insert_view(&db.pool, "/blog/a", &unique("sess"), 5_000).await;
    insert_view(&db.pool, "/blog/c", &unique("sess"), 0).await;

    let pulse = get_realtime_pulse(&db.pool).await.expect("pulse");

    assert_eq!(
        pulse.page_views_this_hour, 3,
        "the zero-duration hit is not a view"
    );
    assert_eq!(
        pulse.unique_visitors_today, 3,
        "distinct visitors counts every event, zero-duration included"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_top_pages_today_is_empty_with_no_events() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let pages = list_top_pages_today(&db.pool).await.expect("top pages");

    assert!(pages.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_top_pages_today_returns_the_three_busiest_pages() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    for (page, hits) in [("/a", 4), ("/b", 3), ("/c", 2), ("/d", 1)] {
        for _ in 0..hits {
            insert_view(&db.pool, page, &unique("sess"), 2_000).await;
        }
    }

    let pages = list_top_pages_today(&db.pool).await.expect("top pages");

    let urls: Vec<&str> = pages.iter().map(|p| p.page_url.as_str()).collect();
    assert_eq!(urls, ["/a", "/b", "/c"]);
    assert_eq!(pages[0].events, 4);
    assert_eq!(pages[0].sessions, 4);
    db.cleanup().await;
}

#[tokio::test]
async fn list_top_pages_today_caps_the_average_dwell_time_at_ten_minutes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_view(&db.pool, "/slow", &unique("sess"), 3_600_000).await;

    let pages = list_top_pages_today(&db.pool).await.expect("top pages");

    assert!(
        (pages[0].avg_time_ms - 600_000.0).abs() < 1.0,
        "an hour-long dwell is clamped to 600000ms, got {}",
        pages[0].avg_time_ms
    );
    db.cleanup().await;
}

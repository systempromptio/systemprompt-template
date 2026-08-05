//! `repositories::dashboard::usage_aggregations::daily` — the two counters the
//! hook endpoint bumps on every event.
//!
//! Both writers are fire-and-forget: they log and return on error, so the only
//! way to tell success from silent failure is to read the row back. That also
//! makes the conflict keys worth pinning — `plugin_usage_daily` conflicts on
//! `(date, user_id, event_type, COALESCE(tool_name, ''))` and
//! `plugin_session_summaries` on `session_id`, and a mismatch there would
//! duplicate rows instead of accumulating them.

use chrono::Utc;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::dashboard::usage_aggregations::{
    DailyAggregationParams, upsert_daily_aggregation,
};
use systemprompt_web_admin::types::{EVENT_POST_TOOL_USE, EVENT_POST_TOOL_USE_FAILURE};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

struct DailyRow {
    event_count: i64,
    error_count: i64,
    content_input_bytes: i64,
    content_output_bytes: i64,
}

async fn read_daily(pool: &sqlx::PgPool, user_id: &UserId) -> Vec<DailyRow> {
    sqlx::query_as::<_, (i64, i64, Option<i64>, Option<i64>)>(
        "SELECT event_count, error_count, content_input_bytes, content_output_bytes
         FROM plugin_usage_daily WHERE user_id = $1 ORDER BY event_type, tool_name",
    )
    .bind(user_id.as_str())
    .fetch_all(pool)
    .await
    .expect("read daily aggregations")
    .into_iter()
    .map(|(event_count, error_count, input, output)| DailyRow {
        event_count,
        error_count,
        content_input_bytes: input.unwrap_or_default(),
        content_output_bytes: output.unwrap_or_default(),
    })
    .collect()
}

fn daily_params<'a>(
    pool: &'a sqlx::PgPool,
    user_id: &'a UserId,
    date: &'a chrono::NaiveDate,
    event_type: &'a str,
) -> DailyAggregationParams<'a> {
    DailyAggregationParams {
        pool,
        user_id,
        date,
        event_type,
        tool_name: Some("Bash"),
        content_input_bytes: 100,
        content_output_bytes: 40,
        is_error: false,
    }
}

#[tokio::test]
async fn upsert_daily_aggregation_inserts_the_first_event_of_the_day() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("daily")).await;
    let date = Utc::now().date_naive();

    upsert_daily_aggregation(&daily_params(&db.pool, &user, &date, EVENT_POST_TOOL_USE)).await;

    let rows = read_daily(&db.pool, &user).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].event_count, 1);
    assert_eq!(rows[0].error_count, 0);
    assert_eq!(rows[0].content_input_bytes, 100);
    assert_eq!(rows[0].content_output_bytes, 40);
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_daily_aggregation_accumulates_into_one_row_per_tool_and_day() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("dailyacc")).await;
    let date = Utc::now().date_naive();

    for _ in 0..3 {
        upsert_daily_aggregation(&daily_params(&db.pool, &user, &date, EVENT_POST_TOOL_USE)).await;
    }

    let rows = read_daily(&db.pool, &user).await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].event_count, 3);
    assert_eq!(rows[0].content_input_bytes, 300);
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_daily_aggregation_counts_errors_separately_from_events() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("dailyerr")).await;
    let date = Utc::now().date_naive();
    let mut params = daily_params(&db.pool, &user, &date, EVENT_POST_TOOL_USE);
    params.is_error = true;

    upsert_daily_aggregation(&params).await;
    upsert_daily_aggregation(&params).await;

    let rows = read_daily(&db.pool, &user).await;
    assert_eq!(rows[0].event_count, 2);
    assert_eq!(rows[0].error_count, 2);
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_daily_aggregation_keeps_a_null_tool_name_on_its_own_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("dailynull")).await;
    let date = Utc::now().date_naive();
    let with_tool = daily_params(&db.pool, &user, &date, EVENT_POST_TOOL_USE);
    let mut without_tool = with_tool;
    without_tool.tool_name = None;

    upsert_daily_aggregation(&with_tool).await;
    upsert_daily_aggregation(&without_tool).await;
    upsert_daily_aggregation(&without_tool).await;

    let rows = read_daily(&db.pool, &user).await;
    assert_eq!(rows.len(), 2, "COALESCE(tool_name, '') keys the two apart");
    let counts: Vec<i64> = rows.iter().map(|r| r.event_count).collect();
    assert!(counts.contains(&1) && counts.contains(&2));
    db.cleanup().await;
}

#[tokio::test]
async fn upsert_daily_aggregation_keeps_separate_rows_per_event_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("dailytypes")).await;
    let date = Utc::now().date_naive();

    upsert_daily_aggregation(&daily_params(&db.pool, &user, &date, EVENT_POST_TOOL_USE)).await;
    upsert_daily_aggregation(&daily_params(
        &db.pool,
        &user,
        &date,
        EVENT_POST_TOOL_USE_FAILURE,
    ))
    .await;

    assert_eq!(read_daily(&db.pool, &user).await.len(), 2);
    db.cleanup().await;
}

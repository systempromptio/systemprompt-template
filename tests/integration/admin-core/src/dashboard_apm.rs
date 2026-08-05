//! `repositories::dashboard::apm_metrics` — actions-per-minute derived from a
//! session summary's counters and its start/end timestamps.
//!
//! Both entry points swallow their errors (they are fire-and-forget rollup
//! writers), so the tests read the row back rather than trusting a return
//! value, and the "unknown session" cases assert that nothing blows up and
//! nothing is written.

use chrono::{Duration, Utc};
use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::dashboard::apm_metrics::{
    calculate_session_apm, update_session_apm,
};

use crate::fixtures::{SummarySpec, insert_summary, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

async fn read_apm(
    pool: &sqlx::PgPool,
    session_id: &str,
) -> (Option<f32>, Option<f32>, Option<i32>) {
    sqlx::query_as(
        "SELECT apm, eapm, peak_concurrent FROM plugin_session_summaries WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await
    .expect("read apm columns")
}

#[tokio::test]
async fn calculate_session_apm_is_zero_for_an_unknown_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (apm, eapm) = calculate_session_apm(&db.pool, &SessionId::new(unique("session"))).await;

    assert_eq!(apm, 0.0);
    assert_eq!(eapm, 0.0);
    db.cleanup().await;
}

#[tokio::test]
async fn calculate_session_apm_treats_an_unfinished_session_as_one_minute() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("apm")).await;
    let session = unique("session");
    let mut spec = SummarySpec::open(&session, &user);
    spec.tool_uses = 8;
    spec.prompts = 4;
    insert_summary(&db.pool, &spec).await;

    let (apm, eapm) = calculate_session_apm(&db.pool, &SessionId::new(session)).await;

    assert_eq!(apm, 12.0, "no ended_at means the divisor is one minute");
    assert_eq!(eapm, 12.0);
    db.cleanup().await;
}

#[tokio::test]
async fn calculate_session_apm_divides_by_the_elapsed_minutes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("apmspan")).await;
    let session = unique("session");
    let started = Utc::now() - Duration::minutes(10);
    let mut spec = SummarySpec::open(&session, &user);
    spec.started_at = Some(started);
    spec.ended_at = Some(started + Duration::minutes(10));
    spec.tool_uses = 90;
    spec.prompts = 10;
    insert_summary(&db.pool, &spec).await;

    let (apm, _) = calculate_session_apm(&db.pool, &SessionId::new(session)).await;

    assert!(
        (apm - 10.0).abs() < 0.001,
        "100 actions over 10 minutes: {apm}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn calculate_session_apm_floors_a_short_session_at_one_minute() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("apmshort")).await;
    let session = unique("session");
    let started = Utc::now() - Duration::seconds(6);
    let mut spec = SummarySpec::open(&session, &user);
    spec.started_at = Some(started);
    spec.ended_at = Some(started + Duration::seconds(6));
    spec.tool_uses = 5;
    insert_summary(&db.pool, &spec).await;

    let (apm, _) = calculate_session_apm(&db.pool, &SessionId::new(session)).await;

    assert_eq!(apm, 5.0, "a six-second session is not reported as 50 apm");
    db.cleanup().await;
}

#[tokio::test]
async fn calculate_session_apm_subtracts_errors_from_the_effective_rate() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("apmerr")).await;
    let session = unique("session");
    let mut spec = SummarySpec::open(&session, &user);
    spec.tool_uses = 10;
    spec.prompts = 2;
    spec.errors = 4;
    insert_summary(&db.pool, &spec).await;

    let (apm, eapm) = calculate_session_apm(&db.pool, &SessionId::new(session)).await;

    assert_eq!(apm, 12.0);
    assert_eq!(eapm, 8.0);
    db.cleanup().await;
}

#[tokio::test]
async fn calculate_session_apm_never_reports_a_negative_effective_rate() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("apmneg")).await;
    let session = unique("session");
    let mut spec = SummarySpec::open(&session, &user);
    spec.tool_uses = 1;
    spec.errors = 9;
    insert_summary(&db.pool, &spec).await;

    let (_, eapm) = calculate_session_apm(&db.pool, &SessionId::new(session)).await;

    assert_eq!(eapm, 0.0);
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_apm_writes_the_three_columns_back() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("apmwrite")).await;
    let session = unique("session");
    insert_summary(&db.pool, &SummarySpec::open(&session, &user)).await;

    update_session_apm(&db.pool, &SessionId::new(session.clone()), 4.5, 3.25, 7).await;

    let (apm, eapm, peak) = read_apm(&db.pool, &session).await;
    assert_eq!(apm, Some(4.5));
    assert_eq!(eapm, Some(3.25));
    assert_eq!(peak, Some(7));
    db.cleanup().await;
}

#[tokio::test]
async fn update_session_apm_is_a_no_op_for_an_unknown_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    update_session_apm(&db.pool, &SessionId::new(unique("session")), 1.0, 1.0, 1).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM plugin_session_summaries")
        .fetch_one(&*db.pool)
        .await
        .expect("count summaries");
    assert_eq!(count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn calculate_and_update_round_trip_through_the_summary_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("apmround")).await;
    let session_id = unique("session");
    let mut spec = SummarySpec::open(&session_id, &user);
    spec.tool_uses = 6;
    spec.prompts = 3;
    spec.errors = 1;
    insert_summary(&db.pool, &spec).await;
    let session = SessionId::new(session_id.clone());

    let (apm, eapm) = calculate_session_apm(&db.pool, &session).await;
    update_session_apm(&db.pool, &session, apm, eapm, 2).await;

    let (stored_apm, stored_eapm, _) = read_apm(&db.pool, &session_id).await;
    assert_eq!(stored_apm, Some(9.0));
    assert_eq!(stored_eapm, Some(8.0));
    db.cleanup().await;
}

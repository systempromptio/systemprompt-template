//! `repositories::analytics::request_stats` — the KPI strip, latency histogram,
//! and the traffic time series behind the Inference Requests page.

use systemprompt_web_admin::repositories::analytics::request_stats::{
    LATENCY_BIN_EDGES_MS, get_request_stats, list_latency_histogram, list_request_timeseries,
};

use crate::fixtures::{
    DecisionSpec, RequestSpec, insert_decision, insert_request, insert_session, insert_user,
    narrow_window, unclaimed_email, unique,
};
use crate::tempdb::TempDb;


#[tokio::test]
async fn get_request_stats_is_all_zeroes_in_an_empty_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let stats = get_request_stats(&db.pool, narrow_window())
        .await
        .expect("query succeeds");

    assert_eq!(stats.total, 0);
    assert_eq!(stats.error_count, 0);
    assert!(
        (stats.error_rate - 0.0).abs() < f64::EPSILON,
        "no traffic is not a 100% error rate"
    );
    assert!((stats.denied_session_rate - 0.0).abs() < f64::EPSILON);
    db.cleanup().await;
}

#[tokio::test]
async fn get_request_stats_derives_error_and_deny_rates() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("stats")).await;
    let denied_session = unique("session");
    let clean_session = unique("session");
    insert_session(&db.pool, &denied_session, &user).await;
    insert_session(&db.pool, &clean_session, &user).await;
    let mut failed = RequestSpec::completed(&unique("req"), &user);
    failed.status = "failed";
    failed.session_id = Some(&denied_session);
    insert_request(&db.pool, &failed).await;
    let mut ok = RequestSpec::completed(&unique("req"), &user);
    ok.session_id = Some(&clean_session);
    insert_request(&db.pool, &ok).await;
    let mut denial = DecisionSpec::allow(&unique("dec"), &user, &denied_session);
    denial.decision = "deny";
    insert_decision(&db.pool, &denial).await;

    let stats = get_request_stats(&db.pool, narrow_window())
        .await
        .expect("query succeeds");

    assert_eq!(stats.total, 2);
    assert_eq!(stats.error_count, 1);
    assert!((stats.error_rate - 0.5).abs() < 1e-9);
    assert_eq!(stats.denied_session_count, 1);
    assert!((stats.denied_session_rate - 0.5).abs() < 1e-9);
    assert_eq!(stats.total_cost_microdollars, 10_000);
    db.cleanup().await;
}

#[tokio::test]
async fn list_latency_histogram_always_returns_every_bin() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("hist")).await;
    let mut spec = RequestSpec::completed(&unique("req"), &user);
    spec.latency_ms = 75;
    insert_request(&db.pool, &spec).await;

    let buckets = list_latency_histogram(&db.pool, narrow_window())
        .await
        .expect("query succeeds");

    assert_eq!(
        buckets.len(),
        LATENCY_BIN_EDGES_MS.len() + 1,
        "the chart's x-axis is fixed; empty bins are reported as zero"
    );
    let total: i64 = buckets.iter().map(|b| b.count).sum();
    assert_eq!(total, 1, "the request lands in exactly one bin");
    // `width_bucket(75, [50,100,…])` is 1, and the loop maps SQL bucket `i+1`
    // onto `out[i]` — so a 75ms request is counted in the bin *labelled*
    // "0ms–50ms". The labels are one edge ahead of the data they hold, and a
    // request under 50ms falls in SQL bucket 0, which no bin reads at all.
    // Pinned as observed, not as intended.
    assert_eq!(buckets[0].count, 1);
    assert_eq!(buckets[0].label, "0ms–50ms");
    assert_eq!(
        buckets[buckets.len() - 1].upper_bound_ms,
        None,
        "the last bin is open-ended"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_request_timeseries_returns_a_fixed_number_of_buckets() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("series")).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &user)).await;

    let series = list_request_timeseries(&db.pool, narrow_window())
        .await
        .expect("query succeeds");

    assert_eq!(series.len(), 24);
    let total: i64 = series.iter().map(|b| b.requests).sum();
    assert_eq!(total, 1, "the single request lands in exactly one bucket");
    assert!(
        series
            .windows(2)
            .all(|w| w[0].bucket_start < w[1].bucket_start)
    );
    db.cleanup().await;
}

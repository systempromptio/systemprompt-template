//! The analytics and housekeeping jobs, driven through the registry that owns
//! them.
//!
//! `UsageAnomalyJob`, `UsageDailyRollupJob` and `PluginUsageRetentionJob` are
//! private to their crate — the scheduler reaches them through the inventory,
//! and so does this suite, by name out of `extension_jobs()`. That is the same
//! entry point production uses, so the SQL each job runs, the thresholds it
//! applies, and its re-run behaviour are all exercised rather than only the
//! pure helpers beside them.
//!
//! Anomaly detection is the interesting one: a metric alerts only when it
//! passes both its multiplier and its absolute floor, and only on the first
//! detection for an hour, so a job that ran twice cannot alert twice.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::{Actor, UserId};
use systemprompt::traits::{Job, JobContext};
use systemprompt_web_jobs::extension_jobs;

use crate::tempdb::TempDb;

// The scheduler finds jobs by name out of the inventory; so does this.
fn job(name: &str) -> Arc<dyn Job> {
    extension_jobs()
        .into_iter()
        .find(|j| j.name() == name)
        .unwrap_or_else(|| panic!("`{name}` is registered with the web extension"))
}

fn context(pool: &Arc<PgPool>) -> JobContext {
    let database = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    JobContext::new(
        Actor::user(UserId::new("jobs-usage-test")),
        Arc::new(database),
        Arc::new(()),
        Arc::new(()),
    )
}

// One request row. The job judges the last *complete* hour, so a row is placed
// relative to the hour boundary rather than to now — "70 minutes ago" would
// fall inside that window or two hours before it depending on what minute the
// suite happened to run at.
struct Request {
    minutes_before_the_hour: i64,
    cost_microdollars: i64,
    status: &'static str,
}

async fn seed(pool: &PgPool, user: &str, requests: &[Request]) {
    sqlx::query(
        "INSERT INTO users (id, name, email) VALUES ($1, $1, $2) ON CONFLICT (id) DO NOTHING",
    )
    .bind(user)
    .bind(format!("{user}@example.com"))
    .execute(pool)
    .await
    .expect("seed the requesting user");

    // `ai_requests.session_id` is a foreign key, so the session has to exist
    // before any traffic can be attributed to it.
    let session = format!("sess-{user}");
    sqlx::query(
        "INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)
         ON CONFLICT (session_id) DO NOTHING",
    )
    .bind(&session)
    .bind(user)
    .execute(pool)
    .await
    .expect("seed the session the traffic belongs to");

    for (index, request) in requests.iter().enumerate() {
        let id = format!("req-{user}-{index}");

        sqlx::query(
            "INSERT INTO ai_requests (
                 id, request_id, user_id, session_id, trace_id, context_id,
                 provider, model, input_tokens, output_tokens, tokens_used,
                 cost_microdollars, latency_ms, status, actor_kind, actor_id,
                 created_at, updated_at)
             VALUES ($1, $1, $2, $6, 'trace',
                     '00000000-0000-4000-8000-000000000001',
                     'anthropic', 'claude-opus-5', 10, 20, 30, $3, 100, $4,
                     'user', $2,
                     DATE_TRUNC('hour', NOW()) - ($5 || ' minutes')::interval,
                     DATE_TRUNC('hour', NOW()) - ($5 || ' minutes')::interval)",
        )
        .bind(&id)
        .bind(user)
        .bind(request.cost_microdollars)
        .bind(request.status)
        .bind(request.minutes_before_the_hour.to_string())
        .bind(&session)
        .execute(pool)
        .await
        .expect("seed an ai_request");
    }
}

// Requests inside the hour under judgement: half an hour before the boundary
// is inside the previous complete hour whatever the clock reads.
fn spike(count: usize, status: &'static str) -> Vec<Request> {
    (0..count)
        .map(|_| Request {
            minutes_before_the_hour: 30,
            cost_microdollars: 0,
            status,
        })
        .collect()
}

async fn anomalies(pool: &PgPool) -> Vec<(String, i64, i64)> {
    sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT metric, observed, baseline FROM usage_anomalies ORDER BY metric",
    )
    .fetch_all(pool)
    .await
    .expect("read the recorded anomalies")
}

#[tokio::test]
async fn a_quiet_hour_records_no_anomaly() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // Well under the 50-request floor: a small instance doubling its traffic
    // is not an incident, which is what the floor exists to say.
    seed(&db.pool, "quiet", &spike(6, "completed")).await;

    let result = job("usage_anomaly")
        .execute(&context(&db.pool))
        .await
        .expect("the sweep runs");

    assert_eq!(
        result.items_processed,
        Some(0),
        "nothing crossed a threshold"
    );
    assert!(anomalies(&db.pool).await.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn a_request_spike_past_the_floor_is_recorded_with_its_baseline() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool, "busy", &spike(60, "completed")).await;

    let result = job("usage_anomaly")
        .execute(&context(&db.pool))
        .await
        .expect("the sweep runs");

    assert_eq!(result.items_processed, Some(1));
    let recorded = anomalies(&db.pool).await;
    assert_eq!(recorded.len(), 1, "only the request count spiked");
    let (metric, observed, baseline) = &recorded[0];
    assert_eq!(metric, "requests");
    assert_eq!(*observed, 60);
    // Why: the trailing week is empty here, so the baseline is zero and the
    // absolute floor is what the observation had to clear.
    assert_eq!(*baseline, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn an_error_spike_alerts_at_a_far_lower_volume_than_a_traffic_spike() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // 12 failures: past the errors floor of 10, but well under the requests
    // floor of 50 — so the errors metric fires alone.
    seed(&db.pool, "failing", &spike(12, "failed")).await;

    job("usage_anomaly")
        .execute(&context(&db.pool))
        .await
        .expect("the sweep runs");

    let recorded = anomalies(&db.pool).await;
    assert_eq!(
        recorded.len(),
        1,
        "only the error count spiked: {recorded:?}"
    );
    assert_eq!(recorded[0].0, "errors");
    assert_eq!(recorded[0].1, 12);

    db.cleanup().await;
}

#[tokio::test]
async fn a_pending_or_streaming_request_is_not_counted_as_an_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mut requests = spike(11, "streaming");
    requests.extend(spike(11, "pending"));
    seed(&db.pool, "inflight", &requests).await;

    job("usage_anomaly")
        .execute(&context(&db.pool))
        .await
        .expect("the sweep runs");

    // 22 in-flight requests would be an error spike if in-flight counted as
    // failure; neither metric reaches its floor, so nothing is recorded.
    assert!(
        anomalies(&db.pool).await.is_empty(),
        "in-flight requests are not failures"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_second_sweep_over_the_same_hour_records_but_does_not_re_alert() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool, "repeat", &spike(60, "completed")).await;
    let anomaly = job("usage_anomaly");

    let first = anomaly
        .execute(&context(&db.pool))
        .await
        .expect("first sweep");
    let second = anomaly
        .execute(&context(&db.pool))
        .await
        .expect("second sweep");

    assert_eq!(first.items_processed, Some(1), "the first detection counts");
    assert_eq!(
        second.items_processed,
        Some(0),
        "a recurring condition alerts on its transition, not on every sweep"
    );
    assert_eq!(
        anomalies(&db.pool).await.len(),
        1,
        "and the incident is still recorded exactly once"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_sweep_refuses_a_context_with_no_database() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let empty = JobContext::new(
        Actor::user(UserId::new("jobs-usage-test")),
        Arc::new(()),
        Arc::new(()),
        Arc::new(()),
    );

    let error = job("usage_anomaly")
        .execute(&empty)
        .await
        .expect_err("a job wired without a pool fails rather than doing nothing");

    assert!(
        error.to_string().contains("DbPool"),
        "the failure names the missing slot: {error}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_daily_rollup_and_the_retention_sweep_run_over_the_same_traffic() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(
        &db.pool,
        "rollup",
        &[
            Request {
                minutes_before_the_hour: 30,
                cost_microdollars: 2_500,
                status: "completed",
            },
            Request {
                minutes_before_the_hour: 2_000,
                cost_microdollars: 7_500,
                status: "completed",
            },
        ],
    )
    .await;

    for name in ["usage_daily_rollup", "plugin_usage_retention"] {
        job(name)
            .execute(&context(&db.pool))
            .await
            .unwrap_or_else(|e| panic!("`{name}` runs against seeded traffic: {e}"));
    }

    db.cleanup().await;
}

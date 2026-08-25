//! `usage_anomaly` job: hourly spike detection over gateway traffic, persisted
//! and alerted.
//!
//! Compares the last complete hour's requests, cost, and errors against the
//! trailing week's hourly average. A metric past its multiplier *and* its
//! absolute floor is an anomaly: the floor keeps a quiet instance from
//! alerting on 3 requests where there were 1, the multiplier keeps a busy one
//! from alerting on ordinary growth. Findings persist to `usage_anomalies` —
//! unlike core's in-memory anomaly service, a restart cannot forget an
//! incident — and the primary key makes the first detection per window the
//! only one that alerts.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::slack_alerts;

use crate::error::JobError;

// Why: observed must exceed baseline * multiplier AND the absolute floor.
// Errors get the tighter multiplier because an error spike is an incident at
// far lower volume than a traffic spike is.
const REQUESTS_MULTIPLIER: i64 = 3;
const COST_MULTIPLIER: i64 = 3;
const ERRORS_MULTIPLIER: i64 = 5;
const REQUESTS_FLOOR: i64 = 50;
const COST_FLOOR_MICRODOLLARS: i64 = 1_000_000;
const ERRORS_FLOOR: i64 = 10;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct UsageAnomalyJob;

struct HourlyObservation {
    window_start: chrono::DateTime<chrono::Utc>,
    requests: i64,
    cost_microdollars: i64,
    errors: i64,
    baseline_requests: i64,
    baseline_cost: i64,
    baseline_errors: i64,
}

impl UsageAnomalyJob {
    pub(crate) async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let obs = load_hourly_observation(pool).await?;
        let findings = [
            evaluate(
                "requests",
                obs.requests,
                obs.baseline_requests,
                REQUESTS_MULTIPLIER,
                REQUESTS_FLOOR,
            ),
            evaluate(
                "cost",
                obs.cost_microdollars,
                obs.baseline_cost,
                COST_MULTIPLIER,
                COST_FLOOR_MICRODOLLARS,
            ),
            evaluate(
                "errors",
                obs.errors,
                obs.baseline_errors,
                ERRORS_MULTIPLIER,
                ERRORS_FLOOR,
            ),
        ];
        let mut recorded = 0u64;
        for finding in findings.into_iter().flatten() {
            if record_anomaly(pool, &obs, finding).await? {
                recorded += 1;
                alert(finding, &obs);
            }
        }
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            anomalies = recorded,
            duration_ms,
            "Usage anomaly sweep completed"
        );
        Ok(JobResult::success()
            .with_stats(recorded, 0)
            .with_duration(duration_ms))
    }
}

#[derive(Debug, Clone, Copy)]
struct Finding {
    metric: &'static str,
    observed: i64,
    baseline: i64,
}

fn evaluate(
    metric: &'static str,
    observed: i64,
    baseline: i64,
    multiplier: i64,
    floor: i64,
) -> Option<Finding> {
    let threshold = baseline.saturating_mul(multiplier).max(floor);
    (observed >= threshold && observed >= floor).then_some(Finding {
        metric,
        observed,
        baseline,
    })
}

// Why: the baseline divides the trailing week by its 168 hours rather than
// averaging only same-hour windows — simpler, and a spike big enough to matter
// clears a whole-week average with the multipliers above.
async fn load_hourly_observation(pool: &PgPool) -> Result<HourlyObservation, JobError> {
    let row = sqlx::query!(
        r#"
        SELECT
            DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour' AS "window_start!",
            COUNT(*) FILTER (WHERE r.created_at >= DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour'
                AND r.created_at < DATE_TRUNC('hour', NOW()))::BIGINT AS "requests!",
            COALESCE(SUM(r.cost_microdollars) FILTER (
                WHERE r.created_at >= DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour'
                AND r.created_at < DATE_TRUNC('hour', NOW())), 0)::BIGINT AS "cost!",
            COUNT(*) FILTER (WHERE r.created_at >= DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour'
                AND r.created_at < DATE_TRUNC('hour', NOW())
                AND r.status NOT IN ('completed', 'pending', 'streaming'))::BIGINT AS "errors!",
            (COUNT(*) FILTER (WHERE r.created_at < DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour')
                / 168)::BIGINT AS "baseline_requests!",
            (COALESCE(SUM(r.cost_microdollars) FILTER (
                WHERE r.created_at < DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour'), 0)
                / 168)::BIGINT AS "baseline_cost!",
            (COUNT(*) FILTER (WHERE r.created_at < DATE_TRUNC('hour', NOW()) - INTERVAL '1 hour'
                AND r.status NOT IN ('completed', 'pending', 'streaming'))
                / 168)::BIGINT AS "baseline_errors!"
        FROM ai_requests r
        WHERE r.created_at >= DATE_TRUNC('hour', NOW()) - INTERVAL '169 hours'
          AND r.created_at < DATE_TRUNC('hour', NOW())
          AND NOT r.synthetic
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(HourlyObservation {
        window_start: row.window_start,
        requests: row.requests,
        cost_microdollars: row.cost,
        errors: row.errors,
        baseline_requests: row.baseline_requests,
        baseline_cost: row.baseline_cost,
        baseline_errors: row.baseline_errors,
    })
}

// Why: `xmax = 0` reports the INSERT arm, so a re-run over the same hour never
// re-alerts — same transition-not-state contract as the budget warnings.
async fn record_anomaly(
    pool: &PgPool,
    obs: &HourlyObservation,
    finding: Finding,
) -> Result<bool, JobError> {
    let row = sqlx::query!(
        r#"INSERT INTO usage_anomalies (metric, window_start, observed, baseline)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (metric, window_start) DO UPDATE SET
            observed = EXCLUDED.observed,
            baseline = EXCLUDED.baseline
         RETURNING (xmax = 0) AS "first_detection!""#,
        finding.metric,
        obs.window_start,
        finding.observed,
        finding.baseline,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.first_detection)
}

fn alert(finding: Finding, obs: &HourlyObservation) {
    let (observed, baseline) = if finding.metric == "cost" {
        (usd(finding.observed), usd(finding.baseline))
    } else {
        (finding.observed.to_string(), finding.baseline.to_string())
    };
    slack_alerts::send_alert(format!(
        "*Usage anomaly* — {} spiked in the hour starting {}: {observed} against a trailing-week \
         hourly baseline of {baseline}. Check `/admin/analytics?tab=spend` and the request log.",
        finding.metric,
        obs.window_start.format("%Y-%m-%d %H:%M UTC"),
    ));
}

#[expect(
    clippy::cast_precision_loss,
    reason = "display only: hourly spend in dollars is far below f64's exact-integer range"
)]
fn usd(microdollars: i64) -> String {
    format!("${:.2}", microdollars as f64 / 1_000_000.0)
}

#[async_trait::async_trait]
impl Job for UsageAnomalyJob {
    fn name(&self) -> &'static str {
        "usage_anomaly"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Hourly spike detection over gateway requests/cost/errors, persisted and Slack-alerted"
    }

    // Why: five past the hour, so the hour being judged is complete and the
    // rollup-heavy top-of-hour jobs have the lock to themselves.
    fn schedule(&self) -> &'static str {
        "0 5 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Usage anomaly sweep invoked");

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&UsageAnomalyJob);

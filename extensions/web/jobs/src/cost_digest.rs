//! `cost_digest` job: a weekly spend and budget-utilization summary per
//! organization, delivered through the Slack alerting transport.
//!
//! The dashboards already show all of this; the digest exists so spend
//! visibility does not depend on someone remembering to open them. One
//! message per run, one line per active organization: month-to-date spend
//! against the plan's caps, the trailing week's spend and request count, and
//! a linear month-end projection. Delivery is best-effort by the same
//! contract as every Slack alert — a Slack outage fails the message, never
//! the job.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::gateway_org_budget::projected_month_end_spend;
use systemprompt_web_admin::slack_alerts;

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CostDigestJob;

/// One organization's digest line inputs. Public for the unit tests behind
/// `internals`; production construction stays in this module's query.
#[derive(Debug, Clone)]
pub struct OrgDigestRow {
    pub name: String,
    pub cap_microdollars: Option<i64>,
    pub mtd_microdollars: i64,
    pub week_microdollars: i64,
    pub week_requests: i64,
}

impl CostDigestJob {
    pub(crate) async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let rows = load_org_digest_rows(pool).await?;
        let count = rows.len() as u64;
        if rows.is_empty() {
            tracing::info!("Cost digest: no active organizations, nothing to send");
        } else {
            slack_alerts::send_alert(compose_digest(&rows));
        }
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(organizations = count, duration_ms, "Cost digest composed");
        Ok(JobResult::success()
            .with_stats(count, 0)
            .with_duration(duration_ms))
    }
}

// Why: the join window is the earlier of month start and seven days ago, so
// early in a month the weekly numbers still cover a full week instead of
// being truncated at the month boundary the MTD filter needs.
async fn load_org_digest_rows(pool: &PgPool) -> Result<Vec<OrgDigestRow>, JobError> {
    let rows = sqlx::query!(
        r#"
        SELECT o.name AS "name!",
               p.monthly_cost_cap_microdollars AS "cap?",
               COALESCE(SUM(r.cost_microdollars)
                   FILTER (WHERE r.created_at >= DATE_TRUNC('month', NOW())), 0)::BIGINT
                   AS "mtd!",
               COALESCE(SUM(r.cost_microdollars)
                   FILTER (WHERE r.created_at >= NOW() - INTERVAL '7 days'), 0)::BIGINT
                   AS "week!",
               COUNT(r.created_at)
                   FILTER (WHERE r.created_at >= NOW() - INTERVAL '7 days')
                   AS "week_requests!"
        FROM organizations o
        JOIN plans p ON p.id = o.plan_id
        LEFT JOIN organization_members m ON m.org_id = o.id
        LEFT JOIN ai_requests r ON r.user_id = m.user_id
            AND r.created_at >= LEAST(DATE_TRUNC('month', NOW()), NOW() - INTERVAL '7 days')
        WHERE o.status = 'active'
        GROUP BY o.id, o.name, p.monthly_cost_cap_microdollars
        ORDER BY "mtd!" DESC, o.name
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| OrgDigestRow {
            name: r.name,
            cap_microdollars: r.cap,
            mtd_microdollars: r.mtd,
            week_microdollars: r.week,
            week_requests: r.week_requests,
        })
        .collect())
}

pub fn compose_digest(rows: &[OrgDigestRow]) -> String {
    let now = chrono::Utc::now();
    let mut lines = vec![format!(
        "*Weekly AI cost digest* — {} · spend by organization",
        now.format("%Y-%m-%d")
    )];
    for row in rows {
        lines.push(compose_org_line(row, now));
    }
    lines.join("\n")
}

pub fn compose_org_line(row: &OrgDigestRow, now: chrono::DateTime<chrono::Utc>) -> String {
    let cap = row.cap_microdollars.map_or_else(
        || "uncapped".to_owned(),
        |cap| {
            let pct = row.mtd_microdollars.saturating_mul(100) / cap.max(1);
            format!("{pct}% of {} cap", usd(cap))
        },
    );
    let pace = projected_month_end_spend(row.mtd_microdollars, now)
        .map_or(String::new(), |p| format!(" · on pace for ~{}", usd(p)));
    format!(
        "• *{}* — {} MTD ({cap}){pace} · last 7d: {} over {} requests",
        row.name,
        usd(row.mtd_microdollars),
        usd(row.week_microdollars),
        row.week_requests,
    )
}

#[expect(
    clippy::cast_precision_loss,
    reason = "display only: monthly spend in dollars is far below f64's exact-integer range"
)]
fn usd(microdollars: i64) -> String {
    format!("${:.2}", microdollars as f64 / 1_000_000.0)
}

#[async_trait::async_trait]
impl Job for CostDigestJob {
    fn name(&self) -> &'static str {
        "cost_digest"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Weekly per-organization spend and budget-utilization digest to Slack"
    }

    // Why: Monday 08:00 UTC — early in the European workday, so a budget
    // owner reads it with the week ahead of them rather than behind.
    fn schedule(&self) -> &'static str {
        "0 0 8 * * 1"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Cost digest invoked");

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&CostDigestJob);

//! The monthly archive, health check and vacuum.
//!
//! The rollups (`admin_usage_daily_rollups`, `plugin_usage_daily`,
//! `conversation_facts`, …) are the instance's durable record and are never
//! expired, so their monthly archive under
//! `storage/exports/monthly/<yyyy>-<mm>/` is the off-database copy of
//! everything the console can still show. The health check runs after the
//! archive and its report is stored for the Data lifecycle tab; the vacuum runs
//! last, one plain statement per table on a pool connection, because `VACUUM`
//! cannot run inside a transaction.

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use super::archive::{ArchiveTarget, Window, archive_tables, prune_periods};
use super::ledger::{archive_exists, record_health_report};
use super::{ROLLUP_TABLES, all_managed, health};
use crate::error::JobError;

const TIER: &str = "monthly";

#[derive(Debug, Clone, Copy, Default)]
pub struct RetentionExportMonthlyJob;

#[derive(Debug, Clone, Copy)]
pub(crate) struct MonthlyParams {
    pub months_back: u32,
    // Why: `0` keeps every monthly archive; the rollups are the durable record.
    pub keep_months: u32,
}

// Why: the calendar month `months_back` before the one containing `day`,
// as a window and its `yyyy-mm` label.
pub(crate) fn month_window(day: NaiveDate, months_back: u32) -> (Window, String) {
    let mut year = day.year();
    let mut month = day.month();
    for _ in 0..months_back {
        if month == 1 {
            year -= 1;
            month = 12;
        } else {
            month -= 1;
        }
    }
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(day);
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or(first + Duration::days(31));
    (
        Window {
            from: first.and_time(NaiveTime::MIN).and_utc(),
            to: next.and_time(NaiveTime::MIN).and_utc(),
        },
        format!("{year}-{month:02}"),
    )
}

impl RetentionExportMonthlyJob {
    pub(crate) async fn execute_with_pool(
        pool: &PgPool,
        exports_root: &Path,
        params: MonthlyParams,
        now: DateTime<Utc>,
    ) -> Result<JobResult, JobError> {
        let started = std::time::Instant::now();
        let mut written: u64 = 0;
        for back in 1..=params.months_back.max(1) {
            let (window, period) = month_window(now.date_naive(), back);
            if archive_exists(pool, TIER, &period).await? {
                continue;
            }
            let target = ArchiveTarget {
                pool,
                exports_root,
                tier: TIER,
                period: &period,
                window,
            };
            let manifest = archive_tables(&target, ROLLUP_TABLES).await?;
            tracing::info!(
                period,
                files = manifest.files.len(),
                "monthly archive written"
            );
            written += 1;
        }
        let pruned = if params.keep_months == 0 {
            0
        } else {
            let (_, keep_from) = month_window(now.date_naive(), params.keep_months);
            prune_periods(exports_root, TIER, &keep_from)?
        };
        // Why: vacuum before the health check, not after. The check reports
        // dead tuples as bloat, and running it first reported every tuple the
        // vacuum was about to reclaim — a finding that reappeared every month
        // and told nobody anything. Measured after, dead tuples mean rows
        // vacuum could not reclaim, which is worth a finding.
        let vacuumed = vacuum_managed(pool).await?;
        let report = health::run(pool).await?;
        let counts = report.counts();
        record_health_report(pool, &report, counts).await?;
        for finding in &report.findings {
            match finding.rank {
                health::Rank::P1 => tracing::warn!(check = finding.check, "{}", finding.detail),
                health::Rank::P2 | health::Rank::P3 => {
                    tracing::info!(check = finding.check, "{}", finding.detail);
                },
            }
        }
        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(JobResult::success()
            .with_stats(written, 0)
            .with_duration(duration_ms)
            .with_message(format!(
                "{written} month(s) archived, {pruned} pruned; health P1={} P2={} P3={}; {vacuumed} tables vacuumed",
                counts.0, counts.1, counts.2
            )))
    }
}

async fn vacuum_managed(pool: &PgPool) -> Result<u64, JobError> {
    let mut vacuumed = 0;
    for table in all_managed() {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "VACUUM (ANALYZE) {}",
            table.name
        )))
        .execute(pool)
        .await?;
        vacuumed += 1;
    }
    Ok(vacuumed)
}

#[async_trait::async_trait]
impl Job for RetentionExportMonthlyJob {
    fn name(&self) -> &'static str {
        "retention_export_monthly"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn description(&self) -> &'static str {
        "Archives last month's rollups to storage/exports/monthly, runs the database health check and vacuums managed tables"
    }
    fn schedule(&self) -> &'static str {
        "0 30 2 1 * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx.get::<DbPool>()?;
        let paths = ctx.get::<Arc<AppPaths>>()?;
        let params = MonthlyParams {
            months_back: ctx.get_parameter_parsed::<u32>("months_back")?.unwrap_or(1),
            keep_months: ctx.get_parameter_parsed::<u32>("keep_months")?.unwrap_or(0),
        };
        Ok(Self::execute_with_pool(
            &db.write_pool(),
            paths.storage().exports(),
            params,
            Utc::now(),
        )
        .await?)
    }
}
systemprompt::traits::submit_job!(&RetentionExportMonthlyJob);

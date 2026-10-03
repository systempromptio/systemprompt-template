//! The weekly archive: the raw request tables for the previous ISO week,
//! one gzipped JSON Lines file per table under
//! `storage/exports/weekly/<yyyy>-W<ww>/` plus a manifest.
//!
//! Raw rows expire between 7 and 180 days after they are written, so this is
//! the only copy of a request older than its window. A week whose manifest
//! already exists is skipped; `weeks_back` (default 1) reaches further back
//! for a backfill, and `keep_weeks` (default 26) bounds the directory —
//! but a week is only pruned once the monthly archive covering it exists.

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc};
use sqlx::PgPool;
use systemprompt::config::AppPaths;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use super::RAW_TABLES;
use super::archive::{ArchiveTarget, Window, archive_tables, prune_periods};
use super::ledger::archive_exists;
use crate::error::JobError;

const TIER: &str = "weekly";
const DEFAULT_KEEP_WEEKS: i64 = 26;

#[derive(Debug, Clone, Copy, Default)]
pub struct RetentionExportWeeklyJob;

#[derive(Debug, Clone, Copy)]
pub(crate) struct WeeklyParams {
    pub weeks_back: i64,
    pub keep_weeks: i64,
}

// Why: the ISO week containing `day`, as a `[monday, next monday)` window
// and its `yyyy-Www` label.
pub(crate) fn iso_week(day: NaiveDate) -> (Window, String) {
    let monday = day - Duration::days(i64::from(day.weekday().num_days_from_monday()));
    let from = monday.and_time(NaiveTime::MIN).and_utc();
    let iso = monday.iso_week();
    (
        Window {
            from,
            to: from + Duration::weeks(1),
        },
        format!("{}-W{:02}", iso.year(), iso.week()),
    )
}

impl RetentionExportWeeklyJob {
    pub(crate) async fn execute_with_pool(
        pool: &PgPool,
        exports_root: &Path,
        params: WeeklyParams,
        now: DateTime<Utc>,
    ) -> Result<JobResult, JobError> {
        let started = std::time::Instant::now();
        let mut written: u64 = 0;
        let mut skipped: u64 = 0;
        for back in 1..=params.weeks_back.max(1) {
            let (window, period) = iso_week(now.date_naive() - Duration::weeks(back));
            if archive_exists(pool, TIER, &period).await? {
                skipped += 1;
                continue;
            }
            let target = ArchiveTarget {
                pool,
                exports_root,
                tier: TIER,
                period: &period,
                window,
            };
            let manifest = archive_tables(&target, RAW_TABLES).await?;
            let rows: i64 = manifest.files.iter().map(|f| f.rows).sum();
            tracing::info!(
                period,
                rows,
                files = manifest.files.len(),
                "weekly archive written"
            );
            written += 1;
        }
        let pruned = Self::prune(pool, exports_root, params.keep_weeks, now).await?;
        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(JobResult::success()
            .with_stats(written, 0)
            .with_duration(duration_ms)
            .with_message(format!(
                "{written} week(s) archived, {skipped} already present, {pruned} pruned"
            )))
    }

    // Why: the cut-off week is only pruned once the monthly archive for the
    // month it starts in exists, so a raw week never disappears before its
    // rollups are on disk.
    async fn prune(
        pool: &PgPool,
        exports_root: &Path,
        keep_weeks: i64,
        now: DateTime<Utc>,
    ) -> Result<u64, JobError> {
        let cutoff_day = now.date_naive() - Duration::weeks(keep_weeks.max(1));
        let (window, keep_from) = iso_week(cutoff_day);
        let month = window.from.format("%Y-%m").to_string();
        if !archive_exists(pool, "monthly", &month).await? {
            return Ok(0);
        }
        prune_periods(exports_root, TIER, &keep_from)
    }
}

#[async_trait::async_trait]
impl Job for RetentionExportWeeklyJob {
    fn name(&self) -> &'static str {
        "retention_export_weekly"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn description(&self) -> &'static str {
        "Archives the previous ISO week of raw request, session and governance rows to storage/exports/weekly"
    }
    fn schedule(&self) -> &'static str {
        "0 0 2 * * 0"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx.get::<DbPool>()?;
        let paths = ctx.get::<Arc<AppPaths>>()?;
        let params = WeeklyParams {
            weeks_back: ctx.get_parameter_parsed::<i64>("weeks_back")?.unwrap_or(1),
            keep_weeks: ctx
                .get_parameter_parsed::<i64>("keep_weeks")?
                .unwrap_or(DEFAULT_KEEP_WEEKS),
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
systemprompt::traits::submit_job!(&RetentionExportWeeklyJob);

//! Scheduled job records surfaced on the governance pages.
use systemprompt::identifiers::{JobName, ScheduledJobId};

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::types::JobSummary;

pub async fn list_jobs(pool: &PgPool) -> Result<Vec<JobSummary>, sqlx::Error> {
    sqlx::query_as!(
        JobSummary,
        r#"
        SELECT
            id AS "id!: ScheduledJobId",
            job_name AS "job_name!: JobName",
            schedule,
            enabled,
            last_run,
            next_run,
            last_status,
            last_error,
            run_count,
            created_at,
            updated_at
        FROM scheduled_jobs
        ORDER BY job_name
        "#,
    )
    .fetch_all(pool)
    .await
}

/// One scheduled job's last run as `scheduled_jobs` records it, with the
/// message a successful run left behind (the retention job reports
/// `table=deleted` pairs there).
#[derive(Debug, Clone)]
pub struct JobRunSummary {
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub last_message: Option<String>,
}

pub async fn find_job_run(
    pool: &PgPool,
    job_name: &str,
) -> Result<Option<JobRunSummary>, sqlx::Error> {
    sqlx::query_as!(
        JobRunSummary,
        r"
        SELECT last_run, next_run, last_status, last_error, last_message
        FROM scheduled_jobs
        WHERE job_name = $1
        ",
        job_name
    )
    .fetch_optional(pool)
    .await
}

use sqlx::PgPool;

use crate::admin::types::JobSummary;

pub async fn list_jobs(pool: &PgPool) -> Result<Vec<JobSummary>, sqlx::Error> {
    sqlx::query_as!(
        JobSummary,
        r"
        SELECT
            id,
            job_name,
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
        ",
    )
    .fetch_all(pool)
    .await
}

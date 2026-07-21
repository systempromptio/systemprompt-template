use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories;
use crate::types::JobSummary;

pub(crate) async fn list_jobs(pool: &PgPool) -> AdminResult<Vec<JobSummary>> {
    let jobs = repositories::governance::jobs::list_jobs(pool).await?;
    Ok(jobs)
}

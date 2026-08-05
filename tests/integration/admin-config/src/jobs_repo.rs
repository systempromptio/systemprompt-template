//! `repositories::jobs` — the scheduled-job rows the governance pages show.

use systemprompt_web_admin::repositories::jobs::list_jobs;

use crate::fixtures::unique;
use crate::tempdb::TempDb;

async fn insert_job(pool: &sqlx::PgPool, name: &str, enabled: bool) -> String {
    let id = unique("job");
    sqlx::query(
        "INSERT INTO scheduled_jobs (id, job_name, schedule, enabled, last_status, run_count)
         VALUES ($1, $2, '0 * * * *', $3, 'success', 4)",
    )
    .bind(&id)
    .bind(name)
    .bind(enabled)
    .execute(pool)
    .await
    .expect("insert scheduled job");
    id
}

#[tokio::test]
async fn list_jobs_returns_the_registered_jobs_by_name() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_job(&db.pool, "zzz_last_job", true).await;
    insert_job(&db.pool, "aaa_first_job", true).await;

    let jobs = list_jobs(&db.pool).await.expect("list jobs");

    let names: Vec<&str> = jobs.iter().map(|j| j.job_name.as_str()).collect();
    let first = names
        .iter()
        .position(|n| *n == "aaa_first_job")
        .expect("the first job is listed");
    let last = names
        .iter()
        .position(|n| *n == "zzz_last_job")
        .expect("the last job is listed");
    assert!(first < last, "jobs are ordered by name");

    db.cleanup().await;
}

#[tokio::test]
async fn list_jobs_carries_the_last_run_outcome() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("job_name");
    insert_job(&db.pool, &name, true).await;

    let jobs = list_jobs(&db.pool).await.expect("list jobs");

    let job = jobs
        .iter()
        .find(|j| j.job_name.as_str() == name)
        .expect("the seeded job is listed");
    assert_eq!(job.last_status.as_deref(), Some("success"));
    assert_eq!(job.run_count, 4);
    assert_eq!(job.schedule, "0 * * * *");
    assert!(job.enabled);
    assert!(job.last_error.is_none());
    assert!(job.last_run.is_none(), "the seed has never actually run");

    db.cleanup().await;
}

#[tokio::test]
async fn list_jobs_includes_disabled_jobs() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("job_name");
    insert_job(&db.pool, &name, false).await;

    let jobs = list_jobs(&db.pool).await.expect("list jobs");

    let job = jobs
        .iter()
        .find(|j| j.job_name.as_str() == name)
        .expect("a disabled job is still a configured job");
    assert!(!job.enabled);

    db.cleanup().await;
}

#[tokio::test]
async fn list_jobs_reports_a_failure_message() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("job_name");
    insert_job(&db.pool, &name, true).await;
    sqlx::query(
        "UPDATE scheduled_jobs SET last_status = 'failed', last_error = 'boom', last_run = NOW()
         WHERE job_name = $1",
    )
    .bind(&name)
    .execute(&*db.pool)
    .await
    .expect("record a failure");

    let jobs = list_jobs(&db.pool).await.expect("list jobs");

    let job = jobs
        .iter()
        .find(|j| j.job_name.as_str() == name)
        .expect("the seeded job is listed");
    assert_eq!(job.last_status.as_deref(), Some("failed"));
    assert_eq!(job.last_error.as_deref(), Some("boom"));
    assert!(job.last_run.is_some());

    db.cleanup().await;
}

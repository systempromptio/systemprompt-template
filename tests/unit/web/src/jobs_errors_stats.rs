//! Job plumbing: the error type every job normalises on, the pipeline's
//! success/failure tally, and the schedules the registry hands the scheduler.
//!
//! `JobError` is what `infra logs trace` shows an operator, so each variant
//! has to stay distinguishable in its rendered form — an io error must not
//! read like a config error. Every job's failure also converts to
//! `ProviderError::Internal`, which is the boundary the scheduler sees.
//!
//! `PipelineStats` is the reason a broken stage does not stop the publish
//! pipeline: failures are recorded and the run continues.

use systemprompt::traits::ProviderError;
use systemprompt_web_extension::jobs::internals::PipelineStats;
use systemprompt_web_extension::jobs::{JobError, extension_jobs};

#[test]
fn config_and_other_render_differently() {
    assert_eq!(
        JobError::config("bad profile").to_string(),
        "Configuration error: bad profile"
    );
    assert_eq!(JobError::other("bad profile").to_string(), "bad profile");
}

#[test]
fn missing_context_names_the_value_the_job_wanted() {
    assert_eq!(
        JobError::MissingContext("AppPaths").to_string(),
        "Job context missing required value: AppPaths"
    );
}

#[test]
fn an_io_error_converts_into_the_infra_variant() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
    let error = JobError::from(io);
    assert!(matches!(error, JobError::Infra(_)));
    assert!(error.to_string().contains("no such file"));
}

#[test]
fn a_json_error_converts_into_the_infra_variant() {
    let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    assert!(matches!(JobError::from(json), JobError::Infra(_)));
}

#[test]
fn a_format_error_is_a_distinct_variant() {
    assert!(matches!(
        JobError::from(std::fmt::Error),
        JobError::Format(_)
    ));
}

#[test]
fn the_pipeline_variant_reports_how_many_sub_jobs_failed() {
    assert_eq!(
        JobError::Pipeline { failed: 3 }.to_string(),
        "Pipeline failed: 3 sub-job(s) reported errors"
    );
}

#[test]
fn every_job_error_reaches_the_scheduler_as_internal() {
    let provider = ProviderError::from(JobError::config("boom"));
    assert!(matches!(provider, ProviderError::Internal(_)));
    assert!(provider.to_string().contains("boom"));
}

#[test]
fn a_fresh_tally_is_empty() {
    let stats = PipelineStats::default();
    assert_eq!(stats.succeeded, 0);
    assert_eq!(stats.failed, 0);
}

#[test]
fn successes_and_failures_are_tallied_independently() {
    let mut stats = PipelineStats::default();
    stats.record_success();
    stats.record_success();
    stats.record_failure();
    assert_eq!(stats.succeeded, 2);
    assert_eq!(stats.failed, 1);
}

#[test]
fn job_names_are_unique_and_sorted() {
    let names: Vec<&str> = extension_jobs().iter().map(|j| j.name()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(names, sorted);
}

#[test]
fn every_cron_scheduled_job_has_six_fields() {
    for job in extension_jobs() {
        let schedule = job.schedule();
        if schedule.is_empty() {
            assert_eq!(
                job.name(),
                "governance_bootstrap",
                "only the bootstrap job may opt out of a cron schedule"
            );
            continue;
        }
        assert_eq!(
            schedule.split_whitespace().count(),
            6,
            "job {} has a malformed schedule {schedule}",
            job.name()
        );
    }
}

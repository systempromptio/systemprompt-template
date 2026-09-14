//! What every registered job advertises to the scheduler and to
//! `infra jobs list`.
//!
//! The tag is not decorative: `crate::registry::JOB_TAG` is how the deployment
//! selects this crate's jobs out of the shared inventory, so a job registered
//! without it is invisible to the runner that is supposed to own it. The
//! description is what an operator reads in the CLI listing, so an empty one
//! is a job nobody can identify.

use systemprompt_web_extension::jobs::extension_jobs;

#[test]
fn every_job_carries_the_crates_selection_tag() {
    for job in extension_jobs() {
        assert!(
            job.tags().contains(&"web-extension"),
            "job {} is not tagged into this crate's job set: {:?}",
            job.name(),
            job.tags()
        );
    }
}

#[test]
fn every_job_describes_itself_for_the_cli_listing() {
    for job in extension_jobs() {
        let description = job.description();
        assert!(
            !description.trim().is_empty(),
            "job {} has no description",
            job.name()
        );
        assert!(
            description.len() > 15,
            "job {}'s description is too terse to identify it: {description}",
            job.name()
        );
    }
}

#[test]
fn no_two_jobs_share_a_description() {
    let mut seen: Vec<(&str, &str)> = extension_jobs()
        .iter()
        .map(|j| (j.description(), j.name()))
        .collect();
    seen.sort_unstable();

    for pair in seen.windows(2) {
        assert_ne!(
            pair[0].0, pair[1].0,
            "jobs {} and {} are indistinguishable in the CLI listing",
            pair[0].1, pair[1].1
        );
    }
}

#[test]
fn the_publish_pipeline_names_the_stages_it_runs() {
    let jobs = extension_jobs();
    let pipeline = jobs
        .iter()
        .find(|j| j.name() == "publish_pipeline")
        .expect("the pipeline job is registered");

    let description = pipeline.description();
    for stage in ["ingestion", "prerender", "sitemap", "robots.txt"] {
        assert!(
            description.contains(stage),
            "the pipeline description omits the {stage} stage: {description}"
        );
    }
}

//! What the web member contributes that the runtime discovers by name: its
//! jobs and its boot seeds. A duplicate job name means one registration
//! shadows another, which is silent at runtime.

use std::collections::HashSet;

use systemprompt::extension::Admitted;
use systemprompt::extension::contribute::ContributionSink;
use systemprompt_web_extension::WebExtension;

fn contributed() -> Admitted {
    template_test_common::composition::admitted()
}

fn web(admitted: &Admitted) -> &ContributionSink {
    &template_test_common::composition::member(admitted, WebExtension::ID).sink
}

#[test]
fn jobs_are_contributed_with_unique_names() {
    let admitted = contributed();
    let sink = web(&admitted);
    let jobs = sink.jobs();
    assert!(!jobs.is_empty(), "no job contributed");

    let names: HashSet<&str> = jobs.iter().map(|j| j.job().name()).collect();
    assert_eq!(names.len(), jobs.len(), "two jobs share a name");
    for job in jobs {
        assert!(!job.job().name().is_empty());
    }
    assert!(
        names.contains("publish_pipeline"),
        "the publish pipeline must be reachable from the extension: {names:?}"
    );
}

#[test]
fn the_boot_seeds_are_contributed_in_order() {
    let admitted = contributed();
    let sink = web(&admitted);
    let seeds = sink.seeds();
    let seed_ids: Vec<&str> = seeds.iter().map(|s| s.id).collect();
    assert_eq!(
        seed_ids,
        ["admin_oauth_client", "marketplace_plans", "groups_projects"],
        "the boot-seed manifest is a contract: add or remove seeds deliberately"
    );
    for seed in seeds {
        assert!(!seed.sql.trim().is_empty(), "seed {} is empty", seed.id);
    }
}

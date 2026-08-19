//! The provider getters are how the runtime discovers what the web extension
//! contributes to a render. They are inventory-backed, and inventory failures
//! are silent — a dropped `submit_*!` or an LTO-stripped registration shows up
//! as an empty list, not a compile error. Each getter is therefore asserted to
//! be non-empty with unique ids: a duplicate id means one registration shadows
//! another, which is equally silent.
//!
//! The config-derived providers (navigation, homepage) are absent here
//! because no profile is bootstrapped in a unit test; that absence is itself
//! the contract — a missing config must not panic the getter.

use std::collections::HashSet;

use systemprompt::extension::prelude::Extension;
use systemprompt_web_extension::WebExtension;

#[test]
fn page_data_providers_are_registered_with_unique_ids() {
    let providers = WebExtension::new().page_data_providers();
    assert!(!providers.is_empty(), "no page data provider registered");

    let ids: HashSet<&str> = providers.iter().map(|p| p.provider_id()).collect();
    assert_eq!(ids.len(), providers.len(), "two providers share an id");
    for provider in &providers {
        assert!(!provider.provider_id().is_empty());
    }
    assert!(
        ids.contains("docs-metadata"),
        "the docs provider registration was dropped: {ids:?}"
    );
}

#[test]
fn content_data_providers_are_registered_with_unique_ids() {
    let providers = WebExtension::new().content_data_providers();
    assert!(!providers.is_empty(), "no content data provider registered");

    let ids: HashSet<&str> = providers.iter().map(|p| p.provider_id()).collect();
    assert_eq!(ids.len(), providers.len(), "two providers share an id");
}

#[test]
fn component_renderers_are_registered_with_unique_ids_and_variables() {
    let renderers = WebExtension::new().component_renderers();
    assert!(!renderers.is_empty(), "no component renderer registered");

    let ids: HashSet<&str> = renderers.iter().map(|r| r.component_id()).collect();
    assert_eq!(ids.len(), renderers.len(), "two renderers share an id");

    let variables: HashSet<&str> = renderers.iter().map(|r| r.variable_name()).collect();
    assert_eq!(
        variables.len(),
        renderers.len(),
        "two renderers write to the same template variable"
    );
}

#[test]
fn template_data_extenders_are_registered() {
    let extenders = WebExtension::new().template_data_extenders();
    assert!(
        !extenders.is_empty(),
        "no template data extender registered"
    );

    let ids: HashSet<&str> = extenders.iter().map(|e| e.extender_id()).collect();
    assert_eq!(ids.len(), extenders.len(), "two extenders share an id");
}

#[test]
fn jobs_are_registered_with_unique_names() {
    let jobs = WebExtension::new().jobs();
    assert!(!jobs.is_empty(), "no job registered");

    let names: HashSet<&str> = jobs.iter().map(|j| j.name()).collect();
    assert_eq!(names.len(), jobs.len(), "two jobs share a name");
    for job in &jobs {
        assert!(!job.name().is_empty());
    }
    assert!(
        names.contains("publish_pipeline"),
        "the publish pipeline must be reachable from the extension: {names:?}"
    );
}

#[test]
fn prerenderers_and_seeds_survive_an_unconfigured_profile() {
    let extension = WebExtension::new();

    let prerenderers = extension.page_prerenderers();
    let page_types: HashSet<&str> = prerenderers.iter().map(|p| p.page_type()).collect();
    assert_eq!(
        page_types.len(),
        prerenderers.len(),
        "two prerenderers claim the same page type"
    );

    let seeds = extension.seeds();
    let seed_ids: Vec<&str> = seeds.iter().map(|s| s.id).collect();
    assert_eq!(
        seed_ids,
        ["admin_oauth_client", "marketplace_plans", "default_department"],
        "the boot-seed manifest is a contract: add or remove seeds deliberately"
    );
    for seed in &seeds {
        assert!(!seed.sql.trim().is_empty(), "seed {} is empty", seed.id);
    }
}

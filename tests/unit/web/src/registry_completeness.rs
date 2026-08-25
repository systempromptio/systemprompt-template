//! Inventory registries must advertise every implementation the crates
//! define; a `submit_*!` forgotten next to a new job or renderer shows up
//! here as a count mismatch (the drift that once hid `SecretMigrationJob`).

use std::collections::BTreeSet;

use systemprompt_web_extension::jobs::extension_jobs;
use systemprompt_web_extension::shared::registry;

#[test]
fn all_jobs_registered() {
    let names: BTreeSet<&'static str> = extension_jobs().iter().map(|j| j.name()).collect();
    let expected: BTreeSet<&'static str> = [
        "blog_content_ingestion",
        "bundle_admin_css",
        "content_analytics_aggregation",
        "content_prerender",
        "copy_extension_assets",
        "cost_digest",
        "governance_bootstrap",
        "llms_txt_generation",
        "plugin_usage_retention",
        "publish_pipeline",
        "robots_txt_generation",
        "salesforce_deprovision",
        "secret_migration",
        "sitemap_generation",
        "usage_anomaly",
        "usage_daily_rollup",
    ]
    .into();
    assert_eq!(names, expected);
}

#[test]
fn stateless_provider_registries_are_complete() {
    assert_eq!(registry::component_renderers().len(), 9);
    // Docs only: the template also registers the blog list and post providers,
    // and Astound ships no blog.
    assert_eq!(registry::page_data_providers().len(), 1);
    assert_eq!(registry::content_data_providers().len(), 1);
    assert_eq!(registry::template_data_extenders().len(), 1);
}

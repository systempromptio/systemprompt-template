//! The jobs crate must contribute every job it defines; a `.job(..)` forgotten
//! next to a new job shows up here as a set mismatch (the drift that once hid
//! `SecretMigrationJob`).

use std::collections::BTreeSet;
use std::sync::Arc;

use systemprompt::traits::Job;

fn extension_jobs() -> Vec<Arc<dyn Job>> {
    template_test_common::composition::member_jobs("web")
}

#[test]
fn all_jobs_registered() {
    let names: BTreeSet<&'static str> = extension_jobs().iter().map(|j| j.name()).collect();
    let expected: BTreeSet<&'static str> = [
        "access_expiry",
        "blog_content_ingestion",
        "bundle_admin_css",
        "content_analytics_aggregation",
        "conversation_judge",
        "conversation_rollup",
        "copy_extension_assets",
        "governance_bootstrap",
        "llms_txt_generation",
        "publish_pipeline",
        "plugin_usage_retention",
        "quota_month_window",
        "retention_daily_report",
        "retention_export_monthly",
        "retention_export_weekly",
        "robots_txt_generation",
        "scope_defaults_recompute",
        "secret_migration",
        "usage_anomaly",
        "usage_daily_rollup",
    ]
    .into();
    assert_eq!(names, expected);
}

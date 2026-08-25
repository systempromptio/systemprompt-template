//! Background jobs for the web extension.
//!
//! Every job in this crate implements the core `Job` trait and is registered
//! with the scheduler at extension boot. They split into three families:
//!
//! - **Publish pipeline** ([`PublishPipelineJob`]) — runs at server startup and
//!   orchestrates ACL/profile/config bootstrap, asset copy, content ingestion,
//!   prerender, sitemap/robots/llms.txt generation, and secret migration.
//!   Sub-jobs are individually addressable via the CLI for targeted re-runs.
//! - **Build helpers** ([`BundleAdminCssJob`], [`CopyExtensionAssetsJob`],
//!   [`ContentPrerenderJob`]) — emit the static surface under `web/dist/`
//!   consumed by the SSR layer.
//! - **Analytics / housekeeping** ([`ContentAnalyticsAggregationJob`],
//!   [`SecretMigrationJob`]) — periodic rollups and one-shot migrations.
//!
//! Errors normalise on [`JobError`]; the scheduler logs and surfaces them
//! through `infra logs trace`.

mod error;
mod registry;

mod bundle_admin_css;
mod content_analytics;
mod copy_assets;
mod cost_digest;
mod governance_bootstrap;
mod ingestion;
mod llms_txt;
mod prerender;
mod publish;
mod robots;
mod salesforce_deprovision;
mod secret_migration;
mod sitemap;
mod usage_anomaly;
mod usage_retention;
mod usage_rollup;

pub use error::JobError;
pub use registry::{JOB_TAG, extension_jobs};

pub use bundle_admin_css::BundleAdminCssJob;
pub use content_analytics::ContentAnalyticsAggregationJob;
pub use copy_assets::CopyExtensionAssetsJob;
pub use governance_bootstrap::GovernanceBootstrapJob;
pub use ingestion::ContentIngestionJob;
pub use llms_txt::LlmsTxtGenerationJob;
pub use prerender::ContentPrerenderJob;
pub use publish::PublishPipelineJob;
pub use robots::RobotsTxtGenerationJob;
pub use secret_migration::SecretMigrationJob;
pub use sitemap::SitemapGenerationJob;
pub use usage_retention::PluginUsageRetentionJob;
pub use usage_rollup::UsageDailyRollupJob;

/// The pure helpers behind the jobs above, re-exported for the external test
/// workspace so their file-format and accounting behaviour can be asserted
/// without a scheduler, a database, or an `AppPaths`. Not part of the public
/// API — the job structs are.
#[doc(hidden)]
pub mod internals {
    pub use crate::bundle_admin_css::{collect_css_files, concatenate_css_files};
    pub use crate::copy_assets::{copy_all_assets, copy_asset};
    pub use crate::cost_digest::{OrgDigestRow, compose_digest, compose_org_line};
    pub use crate::governance_bootstrap::{GovernanceStatus, check_governance_config};
    pub use crate::llms_txt::{sort_entries_in_place, write_header, write_section};
    pub use crate::publish::PipelineStats;
    pub use crate::robots::build_robots_txt_content;
    pub use crate::salesforce_deprovision::build_active_users_soql;
    pub use crate::usage_anomaly::{Finding, evaluate};
}

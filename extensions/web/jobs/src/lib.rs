//! Background jobs for the web extension.
//!
//! Every job in this crate implements the core `Job` trait and is contributed
//! by type in `WebExtension::contribute` through [`contribute_jobs`]. They
//! split into three families:
//!
//! - **Publish pipeline** ([`PublishPipelineJob`]) — runs at server startup and
//!   orchestrates asset copy, content ingestion, prerender, and robots/llms.txt
//!   generation. Sub-jobs are individually addressable via the CLI for targeted
//!   re-runs.
//! - **Build helpers** ([`BundleAdminCssJob`], [`CopyExtensionAssetsJob`]) —
//!   emit the static surface under `web/dist/` consumed by the SSR layer.
//!   Content prerendering is core's `content_prerender` job; the pipeline runs
//!   the same prerender out of turn.
//! - **Analytics / housekeeping** ([`ContentAnalyticsAggregationJob`],
//!   [`SecretMigrationJob`]) — periodic rollups and one-shot migrations.
//! - **Data lifecycle** ([`RetentionDailyReportJob`],
//!   [`RetentionExportWeeklyJob`], [`RetentionExportMonthlyJob`]) — measures,
//!   archives and health-checks the retention-managed tables; see
//!   `retention/mod.rs`.
//!
//! Errors normalise on [`JobError`]; the scheduler logs and surfaces them
//! through `infra logs trace`.

mod error;
mod registry;

mod access_expiry;
mod bundle_admin_css;
mod content_analytics;
mod conversation_judge;
mod conversation_rollup;
mod copy_assets;
mod governance_bootstrap;
mod ingestion;
mod llms_txt;
mod publish;
mod quota_month_window;
mod retention;
mod robots;
mod scope_defaults;
mod secret_migration;
mod usage_anomaly;
mod usage_retention;
mod usage_rollup;

pub use error::JobError;
pub use registry::{JOB_TAG, contribute_jobs};

pub use access_expiry::{AccessExpiryJob, ExpirySweep, lost_manage_role};
pub use bundle_admin_css::BundleAdminCssJob;
pub use content_analytics::ContentAnalyticsAggregationJob;
pub use conversation_judge::ConversationJudgeJob;
pub use conversation_rollup::ConversationRollupJob;
pub use copy_assets::CopyExtensionAssetsJob;
pub use governance_bootstrap::GovernanceBootstrapJob;
pub use ingestion::ContentIngestionJob;
pub use llms_txt::LlmsTxtGenerationJob;
pub use publish::PublishPipelineJob;
pub use quota_month_window::QuotaMonthWindowJob;
pub use retention::{RetentionDailyReportJob, RetentionExportMonthlyJob, RetentionExportWeeklyJob};
pub use robots::RobotsTxtGenerationJob;
pub use scope_defaults::ScopeDefaultsJob;
pub use secret_migration::SecretMigrationJob;
pub use usage_retention::PluginUsageRetentionJob;
pub use usage_rollup::UsageDailyRollupJob;

/// The pure helpers behind the jobs above, re-exported for the external test
/// workspace so their file-format and accounting behaviour can be asserted
/// without a scheduler, a database, or an `AppPaths`. Not part of the public
/// API — the job structs are.
#[doc(hidden)]
pub mod internals {
    pub use crate::bundle_admin_css::{collect_css_files, concatenate_css_files};
    pub use crate::conversation_judge::{
        Category, Classification, ConversationClassifier, JudgeParams, JudgeVerdict, Outcome,
        TranscriptMeta, classification_schema, parse_classification, render_transcript,
        run_with_classifier,
    };
    pub use crate::copy_assets::{copy_all_assets, copy_asset};
    pub use crate::governance_bootstrap::{
        GovernanceStatus, check_governance_config, check_judge_config,
    };
    pub use crate::llms_txt::{sort_entries_in_place, write_header, write_section};
    pub use crate::publish::PipelineStats;
    pub use crate::robots::build_robots_txt_content;
    pub use crate::usage_anomaly::{Finding, evaluate};
}

//! The admin plane's scheduled jobs.
//!
//! Governance bootstrap, secret migration, access expiry, scope defaults,
//! usage rollups and anomaly checks, quota windows, content analytics, the
//! conversation judge, and the data-lifecycle retention jobs.
//!
//! Contributed by type through [`contribute_jobs`]; each reports
//! `systemprompt_web_shared::JOB_TAG`. Errors normalise on [`JobError`].

mod error;

mod access_expiry;
#[cfg(feature = "content")]
mod content_analytics;
#[cfg(feature = "agents")]
mod conversation_judge;
mod conversation_rollup;
mod governance_bootstrap;
mod quota_month_window;
mod retention;
mod scope_defaults;
mod secret_migration;
mod usage_anomaly;
mod usage_retention;
mod usage_rollup;

use systemprompt::extension::Contributions;

pub use error::JobError;

pub use access_expiry::{AccessExpiryJob, ExpirySweep, lost_manage_role};
#[cfg(feature = "content")]
pub use content_analytics::ContentAnalyticsAggregationJob;
#[cfg(feature = "agents")]
pub use conversation_judge::ConversationJudgeJob;
pub use conversation_rollup::ConversationRollupJob;
pub use governance_bootstrap::GovernanceBootstrapJob;
pub use quota_month_window::QuotaMonthWindowJob;
pub use retention::{RetentionDailyReportJob, RetentionExportMonthlyJob, RetentionExportWeeklyJob};
pub use scope_defaults::ScopeDefaultsJob;
pub use secret_migration::SecretMigrationJob;
pub use usage_retention::PluginUsageRetentionJob;
pub use usage_rollup::UsageDailyRollupJob;

use usage_anomaly::UsageAnomalyJob;

pub fn contribute_jobs(c: &mut Contributions<'_>) {
    #[cfg(feature = "agents")]
    c.job(ConversationJudgeJob);
    #[cfg(feature = "content")]
    c.job(ContentAnalyticsAggregationJob);
    c.job(AccessExpiryJob)
        .job(ConversationRollupJob)
        .job(GovernanceBootstrapJob)
        .job(PluginUsageRetentionJob)
        .job(QuotaMonthWindowJob)
        .job(RetentionDailyReportJob)
        .job(RetentionExportMonthlyJob)
        .job(RetentionExportWeeklyJob)
        .job(ScopeDefaultsJob)
        .job(SecretMigrationJob)
        .job(UsageAnomalyJob)
        .job(UsageDailyRollupJob);
}

/// The pure helpers behind the jobs above, re-exported for the external test
/// workspace so their accounting and classification behaviour can be asserted
/// without a scheduler or a database. Not part of the public API — the job
/// structs are.
#[doc(hidden)]
pub mod internals {
    #[cfg(feature = "agents")]
    pub use super::conversation_judge::{
        Category, Classification, ConversationClassifier, JudgeParams, JudgeVerdict, Outcome,
        TranscriptMeta, classification_schema, parse_classification, render_transcript,
        run_with_classifier,
    };
    pub use super::governance_bootstrap::{
        GovernanceStatus, check_governance_config, check_judge_config,
    };
    pub use super::usage_anomaly::{Finding, evaluate};
}

//! The web extension's jobs, contributed by `WebExtension::contribute`.
//!
//! Each job is contributed by type, so its owner is the contributing member
//! and a disabled web extension contributes none. [`JOB_TAG`] remains the
//! tag every job in this crate reports, for operators filtering job listings.

use systemprompt::extension::Contributions;

use crate::usage_anomaly::UsageAnomalyJob;
use crate::{
    AccessExpiryJob, BundleAdminCssJob, ContentAnalyticsAggregationJob, ContentIngestionJob,
    ConversationJudgeJob, ConversationRollupJob, CopyExtensionAssetsJob, GovernanceBootstrapJob,
    LlmsTxtGenerationJob, PluginUsageRetentionJob, PublishPipelineJob, QuotaMonthWindowJob,
    RetentionDailyReportJob, RetentionExportMonthlyJob, RetentionExportWeeklyJob,
    RobotsTxtGenerationJob, ScopeDefaultsJob, SecretMigrationJob, UsageDailyRollupJob,
};

pub const JOB_TAG: &str = "web-extension";

pub fn contribute_jobs(c: &mut Contributions<'_>) {
    c.job(AccessExpiryJob)
        .job(BundleAdminCssJob)
        .job(ContentAnalyticsAggregationJob)
        .job(ContentIngestionJob)
        .job(ConversationJudgeJob)
        .job(ConversationRollupJob)
        .job(CopyExtensionAssetsJob)
        .job(GovernanceBootstrapJob)
        .job(LlmsTxtGenerationJob)
        .job(PluginUsageRetentionJob)
        .job(PublishPipelineJob)
        .job(QuotaMonthWindowJob)
        .job(RetentionDailyReportJob)
        .job(RetentionExportMonthlyJob)
        .job(RetentionExportWeeklyJob)
        .job(RobotsTxtGenerationJob)
        .job(ScopeDefaultsJob)
        .job(SecretMigrationJob)
        .job(UsageAnomalyJob)
        .job(UsageDailyRollupJob);
}

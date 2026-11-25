//! The publish-pipeline jobs, contributed by `WebExtension::contribute`.
//!
//! Each job is contributed by type, so its owner is the contributing member
//! and a disabled web extension contributes none. Every job reports
//! `systemprompt_web_shared::JOB_TAG`, for operators filtering job listings.

use systemprompt::extension::Contributions;

use crate::{
    AssetDeclaration, BundleAdminCssJob, ContentIngestionJob, CopyExtensionAssetsJob,
    LlmsTxtGenerationJob, PublishPipelineJob, RobotsTxtGenerationJob,
};

pub fn contribute_jobs(c: &mut Contributions<'_>, assets: AssetDeclaration) {
    c.job(BundleAdminCssJob)
        .job(ContentIngestionJob)
        .job(CopyExtensionAssetsJob::new(assets))
        .job(LlmsTxtGenerationJob)
        .job(PublishPipelineJob::new(assets))
        .job(RobotsTxtGenerationJob);
}

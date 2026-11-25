//! The publish-pipeline jobs, contributed by `WebExtension::contribute`.
//!
//! Each job is contributed by type, so its owner is the contributing member
//! and a disabled web extension contributes none. Every job reports
//! `systemprompt_web_shared::JOB_TAG`, for operators filtering job listings.

use systemprompt::extension::Contributions;
use systemprompt_web_shared::asset_jobs::AssetDeclaration;

use crate::{
    ContentIngestionJob, LlmsTxtGenerationJob, PublishPipelineJob, RobotsTxtGenerationJob,
};

pub fn contribute_jobs(c: &mut Contributions<'_>, assets: AssetDeclaration) {
    c.job(ContentIngestionJob)
        .job(LlmsTxtGenerationJob)
        .job(PublishPipelineJob::new(assets))
        .job(RobotsTxtGenerationJob);
}

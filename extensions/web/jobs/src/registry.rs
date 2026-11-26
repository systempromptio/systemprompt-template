//! The publish-pipeline jobs, contributed by `WebExtension::contribute`.
//!
//! Each job is contributed by type, so its owner is the contributing member
//! and a disabled web extension contributes none. Every job reports
//! `systemprompt_web_shared::JOB_TAG`, for operators filtering job listings.

use std::sync::Arc;

use systemprompt::extension::Contributions;
use systemprompt::extension::prelude::Resolved;
use systemprompt::manifest::ConfigView;
use systemprompt_web_shared::asset_jobs::AssetDeclaration;

use crate::{
    ContentIngestionJob, LlmsTxtGenerationJob, PublishConfig, PublishPipelineJob,
    RobotsTxtGenerationJob,
};

pub fn contribute_jobs(c: &mut Contributions<'_>, assets: AssetDeclaration) {
    c.provide_sealed::<PublishConfig, _>(|r: &Resolved<'_>| {
        Ok(PublishConfig(r.get::<Arc<ConfigView>>()?))
    });
    c.job(ContentIngestionJob)
        .job(LlmsTxtGenerationJob)
        .job(PublishPipelineJob::new(assets))
        .job(RobotsTxtGenerationJob);
}

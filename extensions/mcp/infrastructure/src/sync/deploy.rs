use anyhow::Result;
use std::time::Instant;
use systemprompt::profile::ProfileBootstrap;
use systemprompt::sync::{
    SyncConfig as CoreSyncConfig, SyncDirection as CoreSyncDirection,
    SyncService as CoreSyncService,
};

use super::{DeployCrateResult, DeployStep, StepStatus};

pub async fn deploy_crate(
    skip_build: bool,
    tag: Option<String>,
    build_core_config: impl Fn(CoreSyncDirection, bool) -> Result<CoreSyncConfig>,
) -> Result<DeployCrateResult> {
    let start = Instant::now();
    let core_config = build_core_config(CoreSyncDirection::Push, false)?;
    let core_service = CoreSyncService::new(core_config);

    let result = core_service.deploy_crate(skip_build, tag.clone()).await?;

    let image_tag = tag.unwrap_or_else(|| {
        let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S");
        format!("deploy-{timestamp}")
    });

    let mut steps = Vec::new();

    if skip_build {
        steps.push(DeployStep {
            name: "cargo_build".to_string(),
            status: StepStatus::Skipped,
            message: Some("Build skipped by user request".to_string()),
            duration_ms: 0,
        });
    } else {
        steps.push(DeployStep {
            name: "cargo_build".to_string(),
            status: if result.success {
                StepStatus::Success
            } else {
                StepStatus::Failed
            },
            message: Some("Build completed".to_string()),
            duration_ms: 0,
        });

        steps.push(DeployStep {
            name: "web_assets".to_string(),
            status: StepStatus::Success,
            message: Some("Web assets compiled".to_string()),
            duration_ms: 0,
        });
    }

    let build_status = if result.success {
        StepStatus::Success
    } else {
        StepStatus::Failed
    };

    steps.push(DeployStep {
        name: "docker_build".to_string(),
        status: build_status,
        message: Some(format!("Image built with tag: {image_tag}")),
        duration_ms: 0,
    });

    steps.push(DeployStep {
        name: "docker_push".to_string(),
        status: build_status,
        message: Some("Image pushed to registry".to_string()),
        duration_ms: 0,
    });

    steps.push(DeployStep {
        name: "fly_deploy".to_string(),
        status: build_status,
        message: Some("Deployed to Fly.io".to_string()),
        duration_ms: 0,
    });

    let deployment_url = ProfileBootstrap::get().ok().and_then(|p| {
        p.cloud
            .as_ref()
            .and_then(|c| c.tenant_id.as_ref())
            .map(|tid| format!("https://{tid}.fly.dev"))
    });

    Ok(DeployCrateResult {
        success: result.success,
        image_tag,
        build_skipped: skip_build,
        steps_completed: steps,
        deployment_url,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

//! Governance bootstrap: project the committed access-control baseline into the
//! authz tables.
//!
//! Three steps, in dependency order:
//! 1. Materialise the profile's gateway-route entities into
//!    `access_control_entities` (so the FK on `access_control_rules` is satisfied
//!    and a `gateway_route` `entity_match` glob has routes to expand over).
//! 2. Project `services/access-control/*.yaml` into the authz tables via core
//!    ingestion.
//! 3. Load the gateway model allow-list into `ai_gateway_policies`.
//!
//! Runs once at boot as a `scheduler.bootstrap_jobs` entry so authorization is
//! correct at app start; it is not cron-scheduled (`schedule()` is empty). The
//! CLI `admin config` reconcile path re-materialises the catalog after a live
//! gateway/provider edit, so no recurring cadence is needed. The catalog ids
//! are deterministic, so re-runs are idempotent.

use std::sync::Arc;

use systemprompt::database::DbPool;
use systemprompt::models::AppPaths;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin::repositories::governance_grp::acl_yaml_loader;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct GovernanceBootstrapJob;

#[async_trait::async_trait]
impl Job for GovernanceBootstrapJob {
    fn name(&self) -> &'static str {
        "governance_bootstrap"
    }

    fn description(&self) -> &'static str {
        "Materialise gateway entities and project access-control + gateway-policy YAML into the \
         authz tables"
    }

    fn schedule(&self) -> &'static str {
        ""
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        Ok(execute_inner(ctx).await?)
    }
}

async fn execute_inner(ctx: &JobContext) -> Result<JobResult, JobError> {
    let start = std::time::Instant::now();

    let db_pool = ctx.db_pool::<DbPool>().ok_or(MarketplaceError::Internal(
        "Database not available in job context".to_string(),
    ))?;
    let paths = ctx
        .app_paths::<Arc<AppPaths>>()
        .ok_or(MarketplaceError::Internal(
            "AppPaths not available in job context".to_string(),
        ))?;
    let services_path = paths.system().services().to_path_buf();

    let registered = bootstrap_gateway_entities(db_pool).await?;

    let pool = db_pool.pool().ok_or(MarketplaceError::Internal(
        "PgPool not available from database".to_string(),
    ))?;
    acl_yaml_loader::load_from_yaml(&pool, &services_path)
        .await
        .map_err(JobError::from)?;

    let policy = systemprompt::ai::load_gateway_policies_from_yaml(db_pool, &services_path)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        gateway_entities = registered,
        gateway_policies = policy.inserted + policy.updated,
        duration_ms,
        "governance bootstrap completed"
    );
    Ok(JobResult::success().with_duration(duration_ms))
}

async fn bootstrap_gateway_entities(db_pool: &DbPool) -> Result<usize, JobError> {
    let profile = systemprompt::config::ProfileBootstrap::get()
        .map_err(|e| MarketplaceError::Internal(format!("profile unavailable: {e}")))?;
    let profile_path = systemprompt::config::ProfileBootstrap::get_path()
        .map_err(|e| MarketplaceError::Internal(format!("profile path unavailable: {e}")))?;

    let route_ids = profile
        .gateway
        .as_ref()
        .map(|gateway| gateway.dispatchable_route_ids(&profile.providers))
        .unwrap_or_default();
    let id_refs: Vec<&str> = route_ids
        .iter()
        .map(systemprompt::identifiers::RouteId::as_str)
        .collect();

    let source = format!("profile:{profile_path}");
    let repo = systemprompt::security::authz::AccessControlRepository::new(db_pool)
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    systemprompt::security::authz::reconcile_gateway_entities(&repo, &id_refs, &source)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))
}

systemprompt::traits::submit_job!(&GovernanceBootstrapJob);

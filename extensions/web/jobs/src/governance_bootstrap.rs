//! Governance bootstrap: project the committed access-control baseline into the
//! authz tables.
//!
//! Four steps, in dependency order:
//! 0. Validate `services/governance/config.yaml`, failing the boot rather than
//!    letting an unparseable file degrade to the built-in defaults unnoticed,
//!    and warn when the resulting chain enforces nothing.
//! 1. Reconcile the profile's gateway-route entities into
//!    `access_control_entities` (so the FK on `access_control_rules` is
//!    satisfied and a `gateway_route` `entity_match` glob has routes to expand
//!    over), deleting catalog rows no profile route claims.
//! 2. Project `services/access-control/*.yaml` into the authz tables via core
//!    ingestion, handing it step 1's route ids as the authoritative
//!    `gateway_route` catalog. For a kind it is not handed, core ingestion
//!    self-materialises a catalog row for any literal `entity_id`, which would
//!    turn a mistyped or invented route id into a silent grant on a route that
//!    can never dispatch; passing the real set makes ingestion reject it
//!    instead. Step 1 must therefore run first — it is what makes the set
//!    authoritative.
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

use systemprompt::security::authz::{EntityKind, RegisteredEntities};

use crate::error::JobError;
use systemprompt_web_admin::repositories::config::acl_yaml_loader;
use systemprompt_web_admin::repositories::config::gateway::{
    dispatchable_route_ids, registered_routes,
};
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct GovernanceBootstrapJob;

#[async_trait::async_trait]
impl Job for GovernanceBootstrapJob {
    fn name(&self) -> &'static str {
        "governance_bootstrap"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
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
        "Database not available in job context".to_owned(),
    ))?;
    let paths = ctx
        .app_paths::<Arc<AppPaths>>()
        .ok_or(MarketplaceError::Internal(
            "AppPaths not available in job context".to_owned(),
        ))?;
    let services_path = paths.system().services().to_path_buf();

    let governance = check_governance_config(&services_path)?;

    let catalog = bootstrap_gateway_entities(db_pool).await?;

    let pool = db_pool.write_pool().ok_or(MarketplaceError::Internal(
        "PgPool not available from database".to_owned(),
    ))?;
    acl_yaml_loader::load_from_yaml(&pool, &services_path, &catalog.registered)
        .await
        .map_err(JobError::from)?;

    let policy_repo = systemprompt::ai::repository::AiGatewayPolicyRepository::new(db_pool)
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;
    let policy = systemprompt::ai::load_gateway_policies_from_yaml(&policy_repo, &services_path)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        gateway_entities = catalog.registered.known_ids(EntityKind::GatewayRoute).len(),
        gateway_entities_pruned = catalog.pruned,
        gateway_policies = policy.inserted + policy.updated,
        governance_policies_active = governance.active,
        duration_ms,
        "governance bootstrap completed"
    );
    Ok(JobResult::success().with_duration(duration_ms))
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct GovernanceStatus {
    pub active: usize,
}

// Why: `GovernanceConfig::load` cannot fail, so without this the request path
// silently restores the built-in defaults when the file is unparseable — an
// operator who edited it to relax a policy gets stricter enforcement than
// before and no signal. Boot is the last point that can still refuse.
#[doc(hidden)]
pub fn check_governance_config(
    services_path: &std::path::Path,
) -> Result<GovernanceStatus, JobError> {
    use systemprompt::security::policy::GovernanceConfig;

    let path = services_path.join("governance/config.yaml");
    GovernanceConfig::validate(&path)
        .map_err(|e| MarketplaceError::config_file(path.display().to_string(), e))?;

    let config = GovernanceConfig::load(&path);
    let active = if config.enabled {
        config.policies.iter().filter(|p| p.enabled).count()
    } else {
        0
    };
    if active == 0 {
        tracing::warn!(
            path = %path.display(),
            master_switch = config.enabled,
            "governance is not enforcing: no policy will run on any request"
        );
    }
    Ok(GovernanceStatus { active })
}

struct GatewayCatalog {
    registered: RegisteredEntities,
    pruned: u64,
}

async fn bootstrap_gateway_entities(db_pool: &DbPool) -> Result<GatewayCatalog, JobError> {
    let profile = systemprompt::config::ProfileBootstrap::get()?;
    let services_path = &profile.paths.services;

    let services = systemprompt::loader::ServicesBootstrap::get()
        // Why: lint-ok: error-adapt — ConfigLoadError is core's variant-less loader error.
        .map_err(|e| MarketplaceError::Internal(format!("services tree is not loaded: {e}")))?;
    let route_ids = dispatchable_route_ids(services);
    let registered = registered_routes(&route_ids);
    let id_refs: Vec<&str> = route_ids.iter().map(String::as_str).collect();

    // Why: reconciling against an empty set would delete every gateway_route
    // entity and cascade away every route grant. A services tree with no
    // gateway is a legitimate configuration, not a signal to empty the
    // catalog, so leave it untouched and let step 2 run unenforced.
    if id_refs.is_empty() {
        tracing::warn!(
            services = %services_path,
            "services tree declares no dispatchable gateway routes — leaving the \
             gateway_route catalog untouched and not enforcing route ids in roles.yaml"
        );
        return Ok(GatewayCatalog {
            registered,
            pruned: 0,
        });
    }

    let source = format!("services:{services_path}");
    let repo = systemprompt::security::authz::AccessControlRepository::new(db_pool)
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    let report =
        systemprompt::security::authz::reconcile_gateway_entities_exact(&repo, &id_refs, &source)
            .await
            .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    Ok(GatewayCatalog {
        registered,
        pruned: report.pruned,
    })
}

systemprompt::traits::submit_job!(&GovernanceBootstrapJob);

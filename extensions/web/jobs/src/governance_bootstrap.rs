//! Governance bootstrap: project the committed access-control baseline into the
//! authz tables.
//!
//! Four steps, in dependency order:
//! 0. Validate `services/governance/config.yaml`, failing the boot rather than
//!    letting an unparseable file degrade to the built-in defaults unnoticed,
//!    and warn when the resulting chain enforces nothing. Then report whether a
//!    plugin owns `hooks.judge`, the conversation judge's switch.
//! 1. Reconcile the services gateway-route entities into
//!    `access_control_entities` (so the FK on `access_control_rules` is
//!    satisfied and a `gateway_route` `entity_match` glob has routes to expand
//!    over), deleting catalog rows no configured route claims.
//! 2. Run every sync plane through the one boot contract
//!    (`repositories::sync::boot`): groups, access control, gateway policies,
//!    gateway routes and the governance chain, in that order. A plane whose
//!    projection is empty is seeded from its file; every other plane is
//!    compared and the drift logged, nothing written. Reconciling code and
//!    database is an administrator's act on `/admin/sync`, never a side effect
//!    of a restart. Step 1 must run first because the `gateway_route/*` glob
//!    expands over its catalog.
//! 3. Project each inbound Slack app's `authz.allowed_roles` onto its
//!    `slack_workspace` entity. `rules.yaml` does not declare these; the gate
//!    stays with the app it gates, so it is written on every boot, after the
//!    seed has seen an empty table.
//!
//! Runs once at boot as a `scheduler.bootstrap_jobs` entry so authorization is
//! correct at app start; it is not cron-scheduled (`schedule()` is empty). The
//! CLI `admin config` reconcile path re-materialises the catalog after a live
//! gateway/provider edit, so no recurring cadence is needed. The catalog ids
//! are deterministic, so re-runs are idempotent.

use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use systemprompt::security::authz::{EntityKind, RegisteredEntities};

use crate::error::JobError;
use systemprompt_web_admin::repositories::config::gateway::{
    dispatchable_route_ids, registered_routes,
};
use systemprompt_web_admin::repositories::config::slack_acl::load_slack_apps;
use systemprompt_web_admin::repositories::sync::boot::reconcile_all;
use systemprompt_web_admin::repositories::sync::sources_db::record_service_sources;
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

    let db_pool = ctx.get::<DbPool>()?;
    // Why: the composed root — the same tree the sync page reads — so a kit's
    // marketplace validates once its bundle is active, and boot and page can
    // never disagree about what the declaration says.
    let profile = systemprompt::config::ProfileBootstrap::get()?;
    let services_path = systemprompt::loader::services_root::ServicesRootBootstrap::active_root_or(
        &profile.paths.services,
    );

    let governance = check_governance_config(&services_path)?;
    check_judge_config()?;

    let catalog = bootstrap_gateway_entities(db_pool).await?;

    let pool = db_pool.write_pool();
    let planes = reconcile_all(&pool)
        .await
        .map_err(|e| JobError::from(MarketplaceError::Internal(e.to_string())))?;

    let slack_workspaces = load_slack_apps(&pool).await.map_err(JobError::from)?;

    // Why: every skill invocation is credited to the marketplace that owns
    // its plugin and the version that marketplace was serving; ownership and
    // versions come from the composition this process booted on, so they are
    // recorded here.
    let sources = record_service_sources(&pool)
        .await
        .map_err(JobError::from)?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        gateway_entities = catalog.registered.known_ids(EntityKind::GatewayRoute).len(),
        gateway_entities_pruned = catalog.pruned,
        groups_declared = planes.groups.declared,
        groups_seeded = planes.groups.seeded,
        groups_drift = planes.groups.drift_rows,
        access_rules_declared = planes.access_control.declared,
        access_rules_seeded = planes.access_control.seeded,
        access_drift = planes.access_control.drift_rows,
        gateway_policies_declared = planes.gateway_policies.declared,
        gateway_policies_seeded = planes.gateway_policies.seeded,
        gateway_policies_drift = planes.gateway_policies.drift_rows,
        slack_workspaces,
        service_sources = sources.sources,
        service_owned_ids = sources.owned_ids,
        marketplace_versions = sources.versions.marketplaces,
        marketplace_versions_new = sources.versions.new_versions,
        governance_policies_active = governance.active,
        governance_policies_warning = governance.warning,
        duration_ms,
        "governance bootstrap completed"
    );
    Ok(JobResult::success().with_duration(duration_ms))
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct GovernanceStatus {
    pub active: usize,
    // Why: reported separately from `active` because the two answer different
    // questions. `active` says the chain is wired; `warning` says how much of
    // it will actually refuse anything. Four active and four warning policies
    // enforce nothing, and a boot line showing only `active = 4` would read as
    // fully protected.
    pub warning: usize,
}

// Why: an unparseable policy file must fail boot rather than degrade to the
// built-in defaults — an operator who edited it to relax a policy would get
// stricter enforcement than before and no signal.
#[doc(hidden)]
pub fn check_governance_config(
    services_path: &std::path::Path,
) -> Result<GovernanceStatus, JobError> {
    use systemprompt::security::policy::GovernanceConfig;

    let path = services_path.join("governance/config.yaml");
    let config = GovernanceConfig::load(&path)
        .map_err(|e| MarketplaceError::config_file(path.display().to_string(), e))?;
    let active = if config.enabled {
        config.policies.iter().filter(|p| p.enabled).count()
    } else {
        0
    };
    let warning = if config.enabled {
        config
            .policies
            .iter()
            .filter(|p| p.enabled && p.mode.is_warn())
            .count()
    } else {
        0
    };
    if warning > 0 {
        tracing::warn!(
            path = %path.display(),
            active,
            warning,
            "governance policies are in warn mode: they evaluate and audit but refuse              nothing. Read them back with `systemprompt infra logs governance report`."
        );
    }
    if active == 0 {
        tracing::warn!(
            path = %path.display(),
            master_switch = config.enabled,
            "governance is not enforcing: no policy will run on any request"
        );
    }
    Ok(GovernanceStatus { active, warning })
}

// Why: `hooks.judge` on the governance owner is the switch for the
// conversation judge; core already refuses two owners. Reported so the boot
// log says whether conversations will be labelled.
#[doc(hidden)]
pub fn check_judge_config() -> Result<(), JobError> {
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    let owner = services
        .plugins
        .values()
        .find(|p| p.enabled && p.hooks.judge)
        .map(|p| p.id.as_str());
    let automatic = systemprompt::config::ProfileBootstrap::get()?
        .judge
        .automatic;
    if let Some(owner) = owner {
        tracing::info!(owner, automatic, "conversation judge configured");
    } else {
        tracing::warn!(
            "no enabled plugin sets hooks.judge: true — conversations will not be judged"
        );
    }
    Ok(())
}

struct GatewayCatalog {
    registered: RegisteredEntities,
    pruned: u64,
}

async fn bootstrap_gateway_entities(db_pool: &DbPool) -> Result<GatewayCatalog, JobError> {
    let profile = systemprompt::config::ProfileBootstrap::get()?;
    let services = systemprompt::loader::ServicesBootstrap::get()?;
    let gateway_path = std::path::Path::new(&profile.paths.services)
        .join("ai")
        .join("gateway.yaml");

    let route_ids = dispatchable_route_ids(services);
    let registered = registered_routes(&route_ids);
    let id_refs: Vec<&str> = route_ids.iter().map(String::as_str).collect();

    // Why: reconciling against an empty set would delete every gateway_route
    // entity and cascade away every route grant. A services tree with no
    // gateway is a legitimate configuration, not a signal to empty the catalog,
    // so leave it untouched and let step 2 run unenforced.
    if id_refs.is_empty() {
        tracing::warn!(
            gateway = %gateway_path.display(),
            "services config declares no dispatchable gateway routes — leaving the \
             gateway_route catalog untouched and not expanding the route glob in rules.yaml"
        );
        return Ok(GatewayCatalog {
            registered,
            pruned: 0,
        });
    }

    let source = format!("services:{}", gateway_path.display());
    let repo = systemprompt::security::authz::AccessControlRepository::new(db_pool);
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

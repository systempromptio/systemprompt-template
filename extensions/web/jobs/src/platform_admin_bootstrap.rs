//! Platform-admin bootstrap: make the configured system admin a platform
//! administrator.
//!
//! "Platform admin" is derived — the `admin` role plus membership in the one
//! `is_platform` organization — and every ordinary provisioning path (SSO JIT,
//! passkey signup, invites) deliberately refuses to write that membership.
//! Core's `admin bootstrap` grants only the role, and the house-organization
//! seed ships the platform org with no members, so without this job a fresh
//! install has no user who can assign elevated roles from the Admin UI, and no
//! UI path can create one (granting membership itself requires a platform
//! admin).
//!
//! Runs once at boot as a `scheduler.bootstrap_jobs` entry (`schedule()` is
//! empty). It is idempotent and strictly non-destructive: a missing admin user
//! warns and succeeds (healing on the next boot after core bootstrap), and an
//! admin already settled in a different organization is reported loudly but
//! never moved — membership is the billing and authorization boundary, so
//! relocating a user is an operator decision, not a boot side effect.

use systemprompt::database::DbPool;
use systemprompt::models::Config;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin::repositories::organizations::crud as org_repo;
use systemprompt_web_admin::repositories::users::queries as user_repo;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct PlatformAdminBootstrapJob;

#[async_trait::async_trait]
impl Job for PlatformAdminBootstrapJob {
    fn name(&self) -> &'static str {
        "platform_admin_bootstrap"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Ensure the configured system admin is a member of the platform organization so at \
         least one platform administrator exists"
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
    let pool = db_pool.write_pool().ok_or(MarketplaceError::Internal(
        "PgPool not available from database".to_owned(),
    ))?;
    let admin_name = Config::get()
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?
        .system_admin_username
        .clone();

    let outcome = reconcile_platform_admin(&pool, &admin_name).await?;

    let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        admin = %admin_name,
        outcome = %outcome,
        duration_ms,
        "platform-admin bootstrap completed"
    );
    Ok(JobResult::success().with_duration(duration_ms))
}

/// What the reconciler found and did; one boot log line renders it.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileOutcome {
    Granted,
    AlreadyPlatformMember,
    AdminUserMissing,
    PlatformOrgMissing,
    SettledInAnotherOrg,
}

impl std::fmt::Display for ReconcileOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Granted => "granted",
            Self::AlreadyPlatformMember => "already_platform_member",
            Self::AdminUserMissing => "admin_user_missing",
            Self::PlatformOrgMissing => "platform_org_missing",
            Self::SettledInAnotherOrg => "settled_in_another_org",
        };
        f.write_str(s)
    }
}

#[doc(hidden)]
pub async fn reconcile_platform_admin(
    pool: &sqlx::PgPool,
    admin_name: &str,
) -> Result<ReconcileOutcome, JobError> {
    let Some((org_id, org_name)) = org_repo::find_platform_organization(pool).await? else {
        tracing::warn!(
            "platform-admin bootstrap: no is_platform organization exists; \
             the house-organization seed has not run"
        );
        return Ok(ReconcileOutcome::PlatformOrgMissing);
    };

    let Some((user_id, _roles)) = user_repo::find_user_id_and_roles_by_name(pool, admin_name)
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?
    else {
        tracing::warn!(
            admin = %admin_name,
            "platform-admin bootstrap: system admin user does not exist yet; \
             run `systemprompt admin bootstrap`, membership is granted on the next boot"
        );
        return Ok(ReconcileOutcome::AdminUserMissing);
    };

    match org_repo::find_membership_org(pool, &user_id).await? {
        Some(existing) if existing == org_id => Ok(ReconcileOutcome::AlreadyPlatformMember),
        Some(existing) => {
            tracing::warn!(
                admin = %admin_name,
                current_org = %existing,
                platform_org = %org_name,
                "platform-admin bootstrap: the system admin belongs to another \
                 organization and is NOT moved; no platform admin was created — \
                 resolve the membership by hand (`systemprompt platform grant`)"
            );
            Ok(ReconcileOutcome::SettledInAnotherOrg)
        },
        None => {
            org_repo::insert_membership_if_absent(pool, &user_id, &org_id, "admin").await?;
            tracing::info!(
                admin = %admin_name,
                platform_org = %org_name,
                "platform-admin bootstrap: system admin joined the platform organization"
            );
            Ok(ReconcileOutcome::Granted)
        },
    }
}

systemprompt::traits::submit_job!(&PlatformAdminBootstrapJob);

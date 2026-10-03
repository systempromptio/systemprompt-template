//! `access_expiry` job: the hourly sweep that makes a validity window bind
//! everywhere the read path cannot.
//!
//! Group and project membership hide an expired row the moment its window
//! passes; the sweep stamps `revoked_at` so the row is durable evidence and
//! recomputes the person's primary scope. Manual roles, access rules and
//! device certificates are read by core — `users.roles`, the resolver, the
//! device gate — so for those the sweep is the enforcement: it rewrites the
//! effective role set, deletes the rule row, stamps the certificate revoked.
//! A person whose expiry took away a manage role loses their live
//! credentials too, exactly as a demotion in the role editor does.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::repositories::access_control::validity::delete_expired_rules;
use systemprompt_web_admin::repositories::devices::certs::revoke_expired_device_certs;
use systemprompt_web_admin::repositories::scope::defaults::recompute_scope_defaults_for_user;
use systemprompt_web_admin::repositories::scope::expiry::revoke_expired_memberships;
use systemprompt_web_admin::repositories::users::revocation::revoke_user_access;
use systemprompt_web_admin::repositories::users::roles;
use systemprompt_web_admin::types::{ROLES_MANAGE, has_any};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct AccessExpiryJob;

/// What one sweep did, for the log line and the job stats.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExpirySweep {
    pub memberships: u64,
    pub roles_users: u64,
    pub access_revoked: u64,
    pub rules: u64,
    pub device_certs: u64,
}

impl ExpirySweep {
    #[must_use]
    pub const fn total(self) -> u64 {
        self.memberships + self.roles_users + self.rules + self.device_certs
    }
}

// Why: whether losing these roles took a manage role away — the same test the
// role editor applies before it revokes live credentials.
#[must_use]
pub fn lost_manage_role(before: &[String], after: &[String]) -> bool {
    has_any(before, ROLES_MANAGE) && !has_any(after, ROLES_MANAGE)
}

impl AccessExpiryJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let sweep = Self::sweep(pool).await?;
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            memberships = sweep.memberships,
            roles_users = sweep.roles_users,
            access_revoked = sweep.access_revoked,
            rules = sweep.rules,
            device_certs = sweep.device_certs,
            duration_ms,
            "Access expiry sweep complete"
        );
        Ok(JobResult::success()
            .with_stats(sweep.total(), 0)
            .with_duration(duration_ms))
    }

    async fn sweep(pool: &PgPool) -> Result<ExpirySweep, JobError> {
        let mut sweep = ExpirySweep::default();

        let expired = revoke_expired_memberships(pool).await?;
        sweep.memberships = expired.group_rows + expired.project_rows;
        for user in &expired.users {
            recompute_scope_defaults_for_user(pool, user).await?;
        }

        for user in roles::list_users_with_expired_manual_roles(pool).await? {
            let (before, after) = roles::expire_manual_roles(pool, &user).await?;
            sweep.roles_users += 1;
            if lost_manage_role(&before, &after) {
                Self::revoke(pool, &user).await?;
                sweep.access_revoked += 1;
            }
        }

        sweep.rules = delete_expired_rules(pool).await?;
        sweep.device_certs = revoke_expired_device_certs(pool).await?;
        Ok(sweep)
    }

    async fn revoke(pool: &PgPool, user: &UserId) -> Result<(), JobError> {
        let counts = revoke_user_access(pool, user).await?;
        tracing::warn!(
            user = %user,
            sessions = counts.sessions,
            api_keys = counts.api_keys,
            device_certs = counts.device_certs,
            "Manage role expired; live credentials revoked"
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl Job for AccessExpiryJob {
    fn name(&self) -> &'static str {
        "access_expiry"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Revokes expired memberships, manual roles, access rules and device certificates"
    }

    fn schedule(&self) -> &'static str {
        "0 5 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx.get::<DbPool>()?;
        let pool = db.write_pool();

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&AccessExpiryJob);

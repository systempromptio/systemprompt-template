//! `salesforce_deprovision` job: SSO offboarding reconciliation (REQ-023).
//!
//! Salesforce is this instance's identity provider, and Salesforce does not
//! push standards-based SCIM to third-party apps — so removal in the `IdP` has
//! to be *pulled*. Every run lists the active local accounts linked to a
//! Salesforce Username, asks the org which of those users are still active,
//! and for each one deactivated or deleted in Salesforce: disables the local
//! account, revokes its sign-in sessions, and revokes its PATs. Access removal
//! in the `IdP` thereby removes platform access without manual cleanup.
//!
//! Connection identity is the integration user from the `SF_TARGET_*` env vars
//! (the same RFC 7523 JWT-bearer grant `salesforce apply` uses). Without them
//! the job logs that it is unconfigured and succeeds — a missing integration
//! must be visible in the job history, not a nightly error.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::repositories::users::salesforce_identity;
use systemprompt_web_admin::repositories::users::sessions::revoke_all_signin_sessions;
use systemprompt_web_admin::salesforce_org::{Connection, TargetOrg};
use systemprompt_web_admin::slack_alerts;

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SalesforceDeprovisionJob;

impl SalesforceDeprovisionJob {
    pub(crate) async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let Ok(target) = TargetOrg::from_env() else {
            tracing::info!(
                "Salesforce deprovisioning skipped: SF_TARGET_* not configured on this host"
            );
            return Ok(JobResult::success());
        };
        let linked = salesforce_identity::list_linked_identities(pool)
            .await
            .map_err(JobError::from)?;
        if linked.is_empty() {
            return Ok(JobResult::success());
        }

        let active = query_active_usernames(&target, &linked).await?;
        let mut disabled = 0u64;
        for identity in &linked {
            if active.contains(&identity.sf_username.to_lowercase()) {
                continue;
            }
            disable_account(pool, identity).await?;
            disabled += 1;
        }
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            linked = linked.len(),
            disabled,
            duration_ms,
            "Salesforce deprovisioning sweep completed"
        );
        Ok(JobResult::success()
            .with_stats(disabled, 0)
            .with_duration(duration_ms))
    }
}

// Why: one SOQL over the linked usernames rather than a call per user. A
// username absent from the result is treated exactly like `IsActive = false`:
// either way the `IdP` has stopped vouching for that person.
async fn query_active_usernames(
    target: &TargetOrg,
    linked: &[salesforce_identity::LinkedSalesforceIdentity],
) -> Result<std::collections::HashSet<String>, JobError> {
    let soql = build_active_users_soql(linked);
    let conn = Connection::connect(target)
        .await
        // Why: SalesforceError is foreign to JobError and this job is its
        // only caller; a dedicated variant would outlive it.
        // Why: lint-ok: error-adapt
        .map_err(|e| JobError::Other(format!("Salesforce connect failed: {e}")))?;
    let rows = conn
        .soql(&soql)
        .await
        // Why: lint-ok: error-adapt
        .map_err(|e| JobError::Other(format!("Salesforce user query failed: {e}")))?;
    Ok(rows
        .iter()
        .filter_map(|r| r.get("Username").and_then(|v| v.as_str()))
        .map(str::to_lowercase)
        .collect())
}

// Why: quoting is the whole defense here — the usernames come from our own
// database, but they originated in Salesforce SSO userinfo, so backslashes
// and quotes are escaped rather than trusted. Public for the unit tests
// behind `internals`.
pub fn build_active_users_soql(linked: &[salesforce_identity::LinkedSalesforceIdentity]) -> String {
    let names: Vec<String> = linked
        .iter()
        .map(|l| {
            format!(
                "'{}'",
                l.sf_username.replace('\\', "\\\\").replace('\'', "\\'")
            )
        })
        .collect();
    format!(
        "SELECT Username FROM User WHERE IsActive = true AND Username IN ({})",
        names.join(",")
    )
}

// Why: disable is an UPDATE of `users.status`, never a delete — the account's
// requests, costs, and audit rows must survive its owner leaving. Sessions and
// PATs are revoked in the same pass so no live credential outlasts the IdP's
// decision, and each offboarding is announced where the admins already look.
async fn disable_account(
    pool: &PgPool,
    identity: &salesforce_identity::LinkedSalesforceIdentity,
) -> Result<(), JobError> {
    let user_id = &identity.user_id;
    sqlx::query!(
        "UPDATE users SET status = 'inactive' WHERE id = $1",
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    let sessions = revoke_all_signin_sessions(pool, user_id).await?;
    let pats = sqlx::query!(
        "UPDATE user_api_keys SET revoked_at = NOW()
         WHERE user_id = $1 AND revoked_at IS NULL",
        user_id.as_str(),
    )
    .execute(pool)
    .await?
    .rows_affected();
    tracing::warn!(
        user_id = %user_id,
        sf_username = %identity.sf_username,
        sessions,
        pats,
        "account disabled: its Salesforce user is deactivated or deleted",
    );
    slack_alerts::send_alert(format!(
        "*Deprovisioned* — {} was deactivated in Salesforce; the linked account is disabled, \
         {sessions} session(s) and {pats} PAT(s) revoked.",
        identity.sf_username,
    ));
    Ok(())
}

#[async_trait::async_trait]
impl Job for SalesforceDeprovisionJob {
    fn name(&self) -> &'static str {
        "salesforce_deprovision"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Disables local accounts whose Salesforce user is deactivated or deleted (SSO offboarding)"
    }

    // Why: half-hourly — offboarding latency is a security window, and the
    // sweep is one SOQL plus a handful of UPDATEs.
    fn schedule(&self) -> &'static str {
        "0 */30 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Salesforce deprovisioning invoked");

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&SalesforceDeprovisionJob);

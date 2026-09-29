//! Account-row state transitions for a connector: how a failure, a rejected
//! grant or a successful probe is written back to `mcp_connector_accounts`.

use super::{Grant, Provider, seal};
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::connector_accounts::{ProviderConnection, update_account};
use crate::repositories::users::connector_credentials::{self as repo, EncryptedGrant};
use chrono::Utc;
use systemprompt::identifiers::UserId;

// Why: an outage or permission problem must not destroy a refresh grant; only
// a rejected grant is deleted and the generation bumped.
pub(super) async fn record_grant_failure(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user: &UserId,
    account: &mut ProviderConnection,
    grant: &Grant,
    error: &AdminError,
) -> AdminResult<()> {
    let provider = &grant.provider;
    if matches!(error, AdminError::Unauthorized(_)) {
        account.status = "reconnect_required".into();
        account.error_code = Some("grant_rejected".into());
        account.generation += 1;
        repo::delete(tx, user, provider.slug()).await?;
    } else {
        account.status = "temporarily_unavailable".into();
        account.error_code = Some(
            if matches!(error, AdminError::Forbidden(_)) {
                "provider_permission_denied"
            } else {
                "provider_unavailable"
            }
            .into(),
        );
        // Why: A successful refresh followed by a failed probe still rotates the
        // grant. Persist it before returning the probe error.
        repo::store(tx, user, provider.slug(), &seal(grant)?).await?;
    }
    update_account(tx, user, account).await?;
    tracing::warn!(
        target: "connector_oauth",
        provider = provider.slug(),
        status = %account.status,
        error_code = account.error_code.as_deref().unwrap_or(""),
        generation = account.generation,
        error = %error,
        "connector grant failure recorded"
    );
    Ok(())
}

// Why: A grant that cannot be opened under the current configuration flags the
// account for reconnection before the error propagates.
pub(super) async fn open_or_flag(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user: &UserId,
    provider: &Provider,
    row: &EncryptedGrant,
    account: &mut ProviderConnection,
) -> AdminResult<Grant> {
    match super::open(row, user, provider) {
        Ok(grant) => Ok(grant),
        Err(error) => {
            if matches!(error, AdminError::Unauthorized(_)) {
                account.status = "reconnect_required".into();
                account.error_code = Some("configuration_changed".into());
                update_account(tx, user, account).await?;
            }
            Err(error)
        },
    }
}

// Why: every accessor call retries the provider, and during a provider
// outage that is one refresh per MCP request. The failure store bumps the
// credential row, so its timestamp is the last attempt; a fresh failure holds
// the account without a provider call. A probe (Test connection) always goes
// through — it is the user asking.
pub(super) const OUTAGE_HOLD_SECONDS: i64 = 30;

pub(super) fn in_outage_hold(
    account: &ProviderConnection,
    last_attempt: Option<chrono::DateTime<Utc>>,
) -> bool {
    account.status == "temporarily_unavailable"
        && last_attempt.is_some_and(|at| {
            Utc::now().signed_duration_since(at).num_seconds() < OUTAGE_HOLD_SECONDS
        })
}

pub(super) fn token_state(grant: &Grant) -> String {
    if grant.expires_at == i64::MAX {
        return "Personal token on file".to_owned();
    }
    let minutes = (grant.expires_at - Utc::now().timestamp()) / 60;
    format!("Access token valid for {minutes} min")
}

pub(super) fn require_connection(status: &str) -> AdminResult<()> {
    if matches!(status, "not_connected" | "reconnect_required") {
        return Err(AdminError::NotFound(
            "Connect your provider account in Systemprompt".into(),
        ));
    }
    Ok(())
}

pub(super) fn apply_verification(account: &mut ProviderConnection, grant: &Grant) {
    account.auth_method = Some(grant.auth_method.clone());
    account.account_id = Some(grant.account_id.clone());
    account.account_name = Some(grant.account_name.clone());
    account.resource_id = Some(grant.resource_id.clone());
    account.resource_name = Some(grant.resource_name.clone());
    account.status = "connected".into();
    account.error_code = None;
    account.verified_at = Some(Utc::now());
}

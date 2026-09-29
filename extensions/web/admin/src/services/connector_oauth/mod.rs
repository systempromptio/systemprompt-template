//! Hosted MCP authorization, with encrypted user-bound grants and locked
//! refresh.

mod account_state;
pub mod config;
pub mod discovery;
pub mod generic;
mod generic_discovery;
pub mod payload;
mod report;
mod response;
pub mod site;
mod tokens;
mod transport;
mod userinfo;
pub mod verify;

use crate::error::{AdminError, AdminResult};
use crate::repositories::secrets::secret_crypto;
use crate::repositories::users::connector_credentials::{self as repo, EncryptedGrant};
use account_state::{
    apply_verification, open_or_flag, record_grant_failure, require_connection, token_state,
};
use chrono::Utc;
pub use config::Provider;
pub use generic::Consent;
pub use report::{VerificationReport, VerificationStep};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
pub use transport::{authorize, exchange, exchange_with_client, refresh_with_client};

// Why: Debug output must never reveal credentials; serialization is only for
// the encrypted credential store, not HTTP responses.
#[derive(Deserialize, Serialize)]
pub struct Grant {
    pub user: String,
    #[serde(default)]
    pub configuration_binding: String,
    #[serde(default)]
    pub authorization_issuer: String,
    #[serde(default)]
    pub token_auth_method: String,
    pub provider: Provider,
    pub client: String,
    pub client_secret: String,
    pub verifier: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    #[serde(default)]
    pub token_endpoint: String,
    #[serde(default)]
    pub generation: i64,
    #[serde(default)]
    pub session: Option<String>,
    #[serde(default)]
    pub auth_method: String,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub account_name: String,
    #[serde(default)]
    pub resource_id: String,
    #[serde(default)]
    pub resource_name: String,
    #[serde(default = "bearer_scheme")]
    pub authorization_scheme: String,
}

impl std::fmt::Debug for Grant {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Grant { <redacted> }")
    }
}

pub fn seal(grant: &Grant) -> AdminResult<EncryptedGrant> {
    let key = secret_crypto::load_master_key()?;
    let nonce = secret_crypto::generate_nonce();
    let bytes = serde_json::to_vec(grant).map_err(AdminError::internal)?;
    Ok(EncryptedGrant {
        ciphertext: secret_crypto::encrypt(&key, &nonce, &bytes)?,
        nonce: nonce.to_vec(),
    })
}

pub fn open(row: &EncryptedGrant, user: &UserId, provider: &Provider) -> AdminResult<Grant> {
    let key = secret_crypto::load_master_key()?;
    let nonce = row
        .nonce
        .as_slice()
        .try_into()
        .map_err(AdminError::internal)?;
    let bytes = secret_crypto::decrypt(&key, &nonce, &row.ciphertext)?;
    let grant: Grant = serde_json::from_slice(&bytes).map_err(AdminError::internal)?;
    // Why: Bind the encrypted payload as well as the database key to its principal.
    if grant.user != user.as_str() || grant.provider.slug() != provider.slug() {
        return Err(AdminError::Forbidden(
            "Connector credential owner mismatch".into(),
        ));
    }
    generic::validate_grant(&grant)?;
    Ok(grant)
}

// Why: Resolve a server-only Authorization header, refreshing under the account
// lock. One account-locked transaction keeps refresh, generation checks and
// error-state commits together.
pub async fn verified_token(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
    probe: bool,
) -> AdminResult<String> {
    let mut report = VerificationReport::for_provider(provider.slug());
    resolve_token(pool, user, provider, probe, &mut report).await
}

// Why: a probe that fails at a stage is a result, not an error — the page
// shows which stage. Only a failure before the first stage (no grant, not
// configured) propagates as an error.
pub async fn probe_connection(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
) -> AdminResult<VerificationReport> {
    let mut report = VerificationReport::for_provider(provider.slug());
    match resolve_token(pool, user, provider, true, &mut report).await {
        Err(error) if report.steps.is_empty() => Err(error),
        Ok(_) | Err(_) => Ok(report.finish()),
    }
}

async fn resolve_token(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
    probe: bool,
    report: &mut VerificationReport,
) -> AdminResult<String> {
    use crate::repositories::users::connector_accounts as accounts;
    if !provider.configured() {
        return Err(AdminError::Unavailable("Connector not configured".into()));
    }
    let mut tx = pool.begin().await?;
    let mut account = accounts::get_locked_account(&mut tx, user, provider.slug()).await?;
    let row = repo::lock(&mut tx, user, provider.slug()).await?;
    require_connection(&account.status)?;
    if !probe && account_state::in_outage_hold(&account, row.as_ref().map(|r| r.updated_at)) {
        return Err(AdminError::Unavailable(
            "Connector provider recently unavailable; retry shortly".into(),
        ));
    }
    let row = row.map(|r| r.grant);
    // Why: a saved OAuth grant is not usable until the MCP verification has
    // succeeded.
    if !probe && account.verified_at.is_none() {
        return Err(AdminError::Forbidden(
            "Test the connection on the Connectors page before using this MCP".into(),
        ));
    }
    let mut grant = match row {
        Some(row) => match open_or_flag(&mut tx, user, &provider, &row, &mut account).await {
            Ok(grant) => grant,
            Err(error) => {
                tx.commit().await?;
                return Err(error);
            },
        },
        None => {
            return Err(AdminError::NotFound(
                "Connect your provider account in Systemprompt".into(),
            ));
        },
    };
    let refresh = grant.auth_method == "oauth" && grant.expires_at <= Utc::now().timestamp() + 120;
    let result = refresh_and_verify(&mut grant, refresh, probe, report).await;
    if let Err(error) = result {
        record_grant_failure(&mut tx, user, &mut account, &grant, &error).await?;
        tx.commit().await?;
        return Err(error);
    }
    if refresh || probe {
        repo::store(&mut tx, user, provider.slug(), &seal(&grant)?).await?;
    }
    if probe {
        apply_verification(&mut account, &grant);
        accounts::update_account(&mut tx, user, &account).await?;
    }
    tx.commit().await?;
    Ok(format!(
        "{} {}",
        grant.authorization_scheme, grant.access_token
    ))
}

async fn refresh_and_verify(
    grant: &mut Grant,
    refresh: bool,
    verify: bool,
    report: &mut VerificationReport,
) -> AdminResult<()> {
    let started = std::time::Instant::now();
    let outcome = if refresh {
        transport::refresh(grant)
            .await
            .map(|()| "Access token refreshed".to_owned())
    } else {
        Ok(token_state(grant))
    };
    report.record("token", started, &outcome);
    outcome?;
    if verify {
        verify::verify_reporting(grant, &transport::client()?, report).await?;
    }
    Ok(())
}

// Why: Token-refresh transport seam used by the external integration tests.
#[doc(hidden)]
pub async fn refresh_at(grant: &mut Grant, endpoint: &str) -> AdminResult<()> {
    let previous = std::mem::replace(&mut grant.token_endpoint, endpoint.to_owned());
    let result = transport::refresh(grant).await;
    grant.token_endpoint = previous;
    result
}

fn bearer_scheme() -> String {
    "Bearer".into()
}

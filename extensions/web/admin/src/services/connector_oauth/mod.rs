//! Hosted MCP authorization, with encrypted user-bound grants and locked
//! refresh.

pub mod config;
pub mod discovery;
pub mod payload;
mod salesforce;
pub mod site;
mod transport;
pub mod verify;

use crate::error::{AdminError, AdminResult};
use crate::repositories::secrets::secret_crypto;
use crate::repositories::users::connector_credentials::{self as repo, EncryptedGrant};
use chrono::Utc;
pub use config::Provider;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
pub use transport::{authorize, exchange};

// Why: Debug output must never reveal credentials; serialization is only for
// the encrypted credential store, not HTTP responses.
#[derive(Deserialize, Serialize)]
pub struct Grant {
    pub user: String,
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

pub fn open(row: &EncryptedGrant, user: &UserId, provider: Provider) -> AdminResult<Grant> {
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
    Ok(grant)
}

// Why: Resolve a server-only Authorization header, refreshing under the account
// lock.
pub async fn verified_token(
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
    probe: bool,
) -> AdminResult<String> {
    use crate::repositories::users::connector_accounts as accounts;
    if !provider.configured() {
        return Err(AdminError::Unavailable("Connector not configured".into()));
    }
    let mut tx = pool.begin().await?;
    let mut account = accounts::get_locked_account(&mut tx, user, provider.slug()).await?;
    let row = repo::lock(&mut tx, user, provider.slug()).await?;
    let preauthorize = row.is_none() && provider == Provider::Salesforce && account.generation == 0;
    require_connection(&account.status, preauthorize)?;
    // Why: a saved OAuth grant is not usable until the MCP verification has
    // succeeded.
    if !probe && !preauthorize && account.verified_at.is_none() {
        return Err(AdminError::Forbidden(
            "Test the connection in Account before using this MCP".into(),
        ));
    }
    let mut grant = match row {
        Some(row) => open(&row, user, provider)?,
        None if preauthorize => salesforce::mint(pool, user, account.generation).await?,
        None => {
            return Err(AdminError::NotFound(
                "Connect your provider account in Systemprompt".into(),
            ));
        },
    };
    let remint = grant.auth_method == "jwt_bearer" && grant.expires_at <= Utc::now().timestamp();
    if remint {
        grant = salesforce::mint(pool, user, account.generation).await?;
    }
    let refresh = grant.auth_method == "oauth" && grant.expires_at <= Utc::now().timestamp() + 120;
    let result = async {
        if refresh {
            transport::refresh(&mut grant).await?;
        }
        if probe || preauthorize || remint {
            verify::verify(&mut grant).await?;
        }
        Ok::<(), AdminError>(())
    }
    .await;
    if let Err(error) = result {
        if matches!(error, AdminError::Unauthorized(_)) {
            account.status = "reconnect_required".into();
            account.error_code = Some("grant_rejected".into());
            account.generation += 1;
            repo::delete(&mut tx, user, provider.slug()).await?;
        } else {
            // Why: An outage or permission problem must not destroy a refresh grant.
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
            repo::store(&mut tx, user, provider.slug(), &seal(&grant)?).await?;
        }
        accounts::update_account(&mut tx, user, &account).await?;
        tx.commit().await?;
        return Err(error);
    }
    if refresh || probe || preauthorize || remint {
        repo::store(&mut tx, user, provider.slug(), &seal(&grant)?).await?;
    }
    if probe || preauthorize || remint {
        apply_verification(&mut account, &grant);
        accounts::update_account(&mut tx, user, &account).await?;
    }
    tx.commit().await?;
    Ok(format!(
        "{} {}",
        grant.authorization_scheme, grant.access_token
    ))
}

fn require_connection(status: &str, preauthorize: bool) -> AdminResult<()> {
    if !preauthorize && matches!(status, "not_connected" | "reconnect_required") {
        return Err(AdminError::NotFound(
            "Connect your provider account in Systemprompt".into(),
        ));
    }
    Ok(())
}

fn apply_verification(
    account: &mut crate::repositories::users::connector_accounts::ProviderConnection,
    grant: &Grant,
) {
    account.auth_method = Some(grant.auth_method.clone());
    account.account_id = Some(grant.account_id.clone());
    account.account_name = Some(grant.account_name.clone());
    account.resource_id = Some(grant.resource_id.clone());
    account.resource_name = Some(grant.resource_name.clone());
    account.status = "connected".into();
    account.error_code = None;
    account.verified_at = Some(Utc::now());
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

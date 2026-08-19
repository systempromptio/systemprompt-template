//! Identity half of the Salesforce callback: exchange the code for tokens,
//! read verified userinfo claims, gate them (verified email + allow-listed
//! domain), and resolve the identity to a local user.

use serde::Deserialize;

use super::SalesforceDeps;
use super::config::SalesforceConfig;
use super::tokens::exchange_code;
use crate::repositories::users::{federated, salesforce_identity};
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Deserialize)]
struct SalesforceUserInfo {
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
    /// The Salesforce Username (e.g. `ed.aa…@agentforce.com`) — distinct from
    /// the login email, and the value the JWT-bearer grant matches on
    /// `sub`.
    preferred_username: Option<String>,
}

// Why: Exchange the code for tokens, read verified claims, gate them, and
// resolve the identity to a local user. The access token is used only to fetch
// userinfo here — it is not retained. Each step logs its own failure and
// collapses to a login *reason*.
pub(super) async fn resolve_identity(
    deps: &SalesforceDeps,
    code: &str,
    code_verifier: &str,
) -> Result<federated::ResolvedFederatedUser, &'static str> {
    let cfg = &deps.config;
    let (sub, email, display_name, sf_username) =
        fetch_gated_claims(deps, code, code_verifier).await?;

    let claims = federated::FederatedClaims {
        issuer: cfg.issuer(),
        external_sub: &sub,
        email: &email,
        display_name: &display_name,
    };
    let resolved = federated::resolve_federated_user(&deps.write_pool, &claims, cfg.auto_provision)
        .await
        .map_err(|e| {
            // Why: a full plan is the customer's problem to fix, not a fault,
            // so it gets its own redirect reason and the login page can say
            // "your organization has no seats left", not "something broke".
            if matches!(e, MarketplaceError::Conflict(_)) {
                tracing::warn!(error = %e, email, "Salesforce login rejected: seat limit reached");
                return "seat_limit";
            }
            tracing::error!(error = %e, "Failed to resolve federated Salesforce user");
            "error"
        })?
        .ok_or_else(|| {
            tracing::warn!(
                email,
                "Salesforce login rejected: auto-provisioning disabled and no existing account"
            );
            "not_provisioned"
        })?;

    // Why: Record the Salesforce Username so the Hosted-MCP token accessor can mint
    // a JWT-bearer token as this user. A failure here must not break login —
    // the accessor falls back to the email if no row exists.
    if let Err(e) =
        salesforce_identity::upsert(&deps.write_pool, &resolved.user_id, &sf_username).await
    {
        tracing::warn!(error = %e, user_id = %resolved.user_id, "Failed to persist Salesforce username");
    }
    crate::authz::salesforce::invalidate(&resolved.user_id).await;

    Ok(resolved)
}

// Why: Shared front half of both callback modes: exchange the code, read
// verified claims, and gate them. Each failure logs once and collapses to a
// redirect reason.
async fn fetch_gated_claims(
    deps: &SalesforceDeps,
    code: &str,
    code_verifier: &str,
) -> Result<(String, String, String, String), &'static str> {
    let cfg = &deps.config;

    let client_secret = super::client_secret().ok_or_else(|| {
        tracing::error!("SALESFORCE_CLIENT_SECRET is not set; cannot complete Salesforce login");
        "unavailable"
    })?;

    let tokens = exchange_code(cfg, code, &client_secret, code_verifier)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Salesforce token exchange failed");
            "error"
        })?;

    let info = get_userinfo(cfg, &tokens.access_token).await.map_err(|e| {
        tracing::error!(error = %e, "Salesforce userinfo fetch failed");
        "error"
    })?;

    gate_claims(cfg, info)
}

// Why: Attach the Salesforce identity behind `code` to the already-signed-in
// `user_id` (the profile "Connect Salesforce" flow). Claims are gated exactly
// as for login; no session is minted. Returns the `?sf=` outcome for the
// profile redirect.
pub(super) async fn link_identity(
    deps: &SalesforceDeps,
    code: &str,
    code_verifier: &str,
    user_id: &systemprompt::identifiers::UserId,
) -> Result<&'static str, &'static str> {
    let (sub, _email, _display_name, sf_username) =
        fetch_gated_claims(deps, code, code_verifier).await?;

    let outcome = federated::link_identity_to_user(
        &deps.write_pool,
        deps.config.issuer(),
        &sub,
        user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, user_id = %user_id, "Failed to link Salesforce identity");
        "error"
    })?;

    if outcome == federated::LinkOutcome::AlreadyLinkedElsewhere {
        tracing::warn!(user_id = %user_id, "Salesforce identity already linked to another user");
        return Ok("already_linked");
    }

    if let Err(e) = salesforce_identity::upsert(&deps.write_pool, user_id, &sf_username).await {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to persist Salesforce username");
    }
    crate::authz::salesforce::invalidate(user_id).await;

    tracing::info!(user_id = %user_id, "Salesforce identity linked from profile");
    Ok("linked")
}

fn gate_claims(
    cfg: &SalesforceConfig,
    info: SalesforceUserInfo,
) -> Result<(String, String, String, String), &'static str> {
    let email = info
        .email
        .map(|e| e.trim().to_lowercase())
        .ok_or("no_email")?;
    // Why: Linking an unverified address would let a hostile IdP claim arbitrary
    // accounts via the email-merge path in `federated`.
    if !info.email_verified {
        tracing::warn!(email, "Salesforce login rejected: email not verified");
        return Err("unverified");
    }
    if !cfg.email_allowed(&email) {
        tracing::warn!(email, "Salesforce login rejected: domain not allow-listed");
        return Err("forbidden");
    }
    let display_name = info.name.unwrap_or_else(|| email.clone());
    let sf_username = select_sf_username(info.preferred_username.as_deref(), &email);
    Ok((info.sub, email, display_name, sf_username))
}

/// The Salesforce Username to sign JWT-bearer assertions with.
///
/// Uses the userinfo `preferred_username` when present and non-blank, else the
/// login email — the latter is only correct for orgs where the email *is* the
/// Username.
pub fn select_sf_username(preferred_username: Option<&str>, email: &str) -> String {
    preferred_username
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .map_or_else(|| email.to_owned(), str::to_owned)
}

async fn get_userinfo(
    cfg: &SalesforceConfig,
    access_token: &str,
) -> Result<SalesforceUserInfo, super::SalesforceError> {
    let resp = reqwest::Client::new()
        .get(cfg.userinfo_url())
        .bearer_auth(access_token)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(super::SalesforceError::UserInfo(resp.status()));
    }
    Ok(resp.json().await?)
}

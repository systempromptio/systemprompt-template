//! Aggregator for the bridge-style profile pane.
//!
//! Produces the same payload shape consumed by the bridge GUI's profile tab
//! so the SSR profile page and (future) `/v1/bridge/profile/usage` endpoint
//! render the same data from the same source.

mod assemble;

use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{TenantId, UserId};

use crate::types::UserContext;

use assemble::{
    build_agents_block, build_bridge_profile_block, build_marketplaces, build_usage,
    load_usage_sections, read_tenant_id,
};
// Why: re-exported because the profile page's on-demand connect-code endpoint
// needs the gateway URL, which lives one module over.
pub(crate) use assemble::read_config_strings;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProfileIdentity {
    pub email: String,
    pub display_name: Option<String>,
    pub user_id: UserId,
    pub tenant_id: Option<TenantId>,
    pub provider: Option<String>,
    pub roles: Vec<String>,
    pub jwt_issuer: Option<String>,
    pub gateway: Option<String>,
    pub is_admin: bool,
}

pub(crate) use crate::repositories::users::usage::{ConversationSummary, ModelShare, UsageWindow};

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct ProfileUsage {
    pub d1: UsageWindow,
    pub d7: UsageWindow,
    pub d30: UsageWindow,
    pub top_models: Vec<ModelShare>,
    pub conversations: ConversationSummary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BridgeProfileBlock {
    pub inference_gateway_base_url: String,
    pub auth_scheme: String,
    pub models: Vec<String>,
    pub models_count: usize,
    pub organization_uuid: Option<String>,
}

// Why: One marketplace this account actually reaches, with the band that
// granted it — a person reading their own profile can see *why* they have it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProfileMarketplaceView {
    pub id: String,
    pub name: String,
    pub version: String,
    pub plugin_count: usize,
    pub layer: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AgentItem {
    pub id: String,
    pub display_name: String,
    pub enabled: bool,
    pub host_running: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct AgentsBlock {
    pub total: i64,
    pub enabled: i64,
    pub items: Vec<AgentItem>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BridgeConnectBlock {
    pub code: String,
    pub expires_in_seconds: i64,
    pub gateway: String,
    pub install_command: String,
    pub login_command: String,
    pub opencode_install_command: String,
    pub opencode_login_command: String,
    pub desktop_windows_login_command: String,
}

// Why: not derivable here — `brand()` lives in the bridge crate, which the
// admin extension does not depend on.
pub(crate) const BRIDGE_BINARY: &str = "systemprompt-bridge";

// Why: the host is named rather than left to `install.sh`'s PATH probe, which
// enrols `opencode` only when its binary already exists. A user who installs
// OpenCode later would get no provider block and every prompt fails with
// `403 no loopback credential presented`, so each tab on the profile page says
// exactly which client its command enrols. `--hosts all` is a bridge-binary
// flag and is rejected by `install.sh`.
fn install_command(gateway: &str, code: &str, host: &str) -> String {
    super::bridge_downloads::install_command(gateway, Some(code), host).unwrap_or_default()
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BridgeProfilePageData {
    pub download_base_url: Option<String>,
    pub connections: Option<super::connector_accounts::ConnectionSnapshot>,
    pub page: &'static str,
    pub title: &'static str,
    pub identity: ProfileIdentity,
    // Why: Whether a gateway is configured, and so whether the page may offer to
    // issue a connect code. The code itself is never part of page data.
    pub bridge_connect_available: bool,
    pub bridge_profile: Option<BridgeProfileBlock>,
    pub usage: ProfileUsage,
    pub agents: AgentsBlock,
    pub marketplaces: Vec<ProfileMarketplaceView>,
    pub marketplaces_count: usize,
}

// Why: not called while assembling the page. A connect code is a bearer
// credential — redeeming one yields a durable PAT that signs in as its owner.
// Minting on every render printed a fresh one into the HTML of a page people
// leave open, screen-share and reload, and burned a code per view for the
// majority of views that never connect anything. Issuing is an explicit act,
// so a code exists because someone asked for one.
pub(crate) async fn issue_bridge_connect(
    pool: &PgPool,
    user_ctx: &UserContext,
    gateway: Option<&str>,
) -> Option<BridgeConnectBlock> {
    let gateway = gateway?.to_owned();
    let issued = crate::repositories::bridge::issue_exchange_code(pool, &user_ctx.user_id)
        .await
        .map_err(|e| {
            tracing::warn!(
                error = %e,
                "could not mint a bridge exchange code"
            );
        })
        .ok()?;

    let expires_in_seconds = (issued.expires_at - chrono::Utc::now())
        .num_seconds()
        .max(0);

    Some(BridgeConnectBlock {
        install_command: install_command(&gateway, &issued.code, "claude-code"),
        opencode_install_command: install_command(&gateway, &issued.code, "opencode"),
        opencode_login_command: format!("{BRIDGE_BINARY} install --host opencode"),
        login_command: format!(
            "{BRIDGE_BINARY} login --code {code} --gateway {gateway}",
            code = issued.code
        ),
        desktop_windows_login_command: format!(
            ".\\{BRIDGE_BINARY}-windows.exe login --code {code} --gateway {gateway}",
            code = issued.code
        ),
        code: issued.code,
        expires_in_seconds,
        gateway,
    })
}

// Why: Build the full payload. Falls back gracefully when individual sections
// fail — the bridge does the same so missing data renders as empty cards rather
// than a page-level error.
pub(crate) async fn build_bridge_profile_data(
    pool: Arc<PgPool>,
    user_ctx: &UserContext,
) -> crate::error::AdminResult<BridgeProfilePageData> {
    let user_id = user_ctx.user_id.clone();

    let sections = load_usage_sections(&pool, &user_id).await;
    let display_name = sections
        .bridge_user
        .as_ref()
        .and_then(|u| u.display_name.clone());

    // Why: the field existed and was always `None`, so the profile page said
    // nothing about how the account authenticates. A federated mapping is the
    // answer: its presence *is* the provider.
    let provider = crate::repositories::users::queries::find_identity_envelope(&pool, &user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "profile: identity provider lookup failed"))
        .ok()
        .flatten()
        .and_then(|env| env.idp_issuer.map(|_| "adfs".to_owned()));

    let (jwt_issuer, gateway_url) = read_config_strings();
    let bridge_profile = build_bridge_profile_block();
    // Why: the page only offers the button; `issue_bridge_connect` answers it,
    // so no credential is assembled into page data.
    let bridge_connect_available = gateway_url.is_some();

    let download_base_url = gateway_url
        .as_deref()
        .and_then(super::bridge_downloads::download_base);
    let identity = ProfileIdentity {
        email: user_ctx.email.as_str().to_owned(),
        display_name,
        user_id: user_ctx.user_id.clone(),
        tenant_id: read_tenant_id(),
        provider,
        roles: user_ctx.roles.clone(),
        jwt_issuer,
        gateway: gateway_url,
        is_admin: user_ctx.is_admin,
    };

    let usage = build_usage(sections);
    let agents = build_agents_block();
    let marketplaces = build_marketplaces(&pool, &user_id, user_ctx.roles.clone()).await;

    let connections = Some(super::connector_accounts::get_connections(&pool, &user_id).await?);
    Ok(BridgeProfilePageData {
        download_base_url,
        connections,
        marketplaces_count: marketplaces.len(),
        marketplaces,
        page: "profile",
        title: "Profile",
        identity,
        bridge_connect_available,
        bridge_profile,
        usage,
        agents,
    })
}

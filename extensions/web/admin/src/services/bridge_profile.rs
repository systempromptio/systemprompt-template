//! Aggregator for the bridge-style profile pane.
//!
//! Produces the same payload shape consumed by the bridge GUI's profile tab
//! so the SSR profile page and (future) `/v1/bridge/profile/usage` endpoint
//! render the same data from the same source.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::TenantId;
use systemprompt::models::Config;
use uuid::Uuid;

use crate::repositories::cowork_grp::find_cowork_user;
use crate::repositories::profile_grp::usage as usage_repo;
use crate::types::UserContext;

#[derive(Debug, Clone, Serialize)]
pub struct ProfileIdentity {
    pub email: String,
    pub display_name: Option<String>,
    pub user_id: String,
    pub tenant_id: Option<String>,
    pub provider: Option<String>,
    pub roles: Vec<String>,
    pub jwt_issuer: Option<String>,
    pub gateway: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UsageWindow {
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub previous_cost_microdollars: Option<i64>,
}

impl From<usage_repo::UsageWindow> for UsageWindow {
    fn from(w: usage_repo::UsageWindow) -> Self {
        Self {
            requests: w.requests,
            tokens: w.tokens,
            cost_microdollars: w.cost_microdollars,
            previous_cost_microdollars: w.previous_cost_microdollars,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelShare {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub token_share: f64,
}

impl From<usage_repo::ModelShare> for ModelShare {
    fn from(m: usage_repo::ModelShare) -> Self {
        Self {
            model: m.model,
            requests: m.requests,
            tokens: m.tokens,
            cost_microdollars: m.cost_microdollars,
            token_share: m.token_share,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationGroup {
    pub name: String,
    pub conversations: i64,
    pub ai_requests: i64,
}

impl From<usage_repo::ConversationGroup> for ConversationGroup {
    fn from(g: usage_repo::ConversationGroup) -> Self {
        Self {
            name: g.name,
            conversations: g.conversations,
            ai_requests: g.ai_requests,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentConversation {
    pub context_id: String,
    pub context_name: Option<String>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub ai_requests: i64,
    pub model: Option<String>,
    pub agent_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ConversationSummary {
    pub total_conversations: i64,
    pub total_ai_requests: i64,
    pub by_model: Vec<ConversationGroup>,
    pub by_agent: Vec<ConversationGroup>,
    pub recent: Vec<RecentConversation>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProfileUsage {
    pub d1: UsageWindow,
    pub d7: UsageWindow,
    pub d30: UsageWindow,
    pub top_models: Vec<ModelShare>,
    pub conversations: ConversationSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeProfileBlock {
    pub inference_gateway_base_url: String,
    pub auth_scheme: String,
    pub models: Vec<String>,
    pub models_count: usize,
    pub organization_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentItem {
    pub id: String,
    pub display_name: String,
    pub enabled: bool,
    pub host_running: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AgentsBlock {
    pub total: i64,
    pub enabled: i64,
    pub items: Vec<AgentItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeProfilePageData {
    pub page: &'static str,
    pub title: &'static str,
    pub identity: ProfileIdentity,
    pub bridge_profile: Option<BridgeProfileBlock>,
    pub usage: ProfileUsage,
    pub agents: AgentsBlock,
}

/// Build the full payload. Falls back gracefully when individual sections fail
/// — the bridge does the same so missing data renders as empty cards rather
/// than a page-level error.
pub async fn build_bridge_profile_data(
    pool: Arc<PgPool>,
    user_ctx: &UserContext,
) -> BridgeProfilePageData {
    let user_id = user_ctx.user_id.as_str().to_string();

    let pool_for_d1 = Arc::clone(&pool);
    let pool_for_d7 = Arc::clone(&pool);
    let pool_for_d30 = Arc::clone(&pool);
    let pool_for_models = Arc::clone(&pool);
    let pool_for_conv = Arc::clone(&pool);
    let pool_for_user = Arc::clone(&pool);

    let user_id_d1 = user_id.clone();
    let user_id_d7 = user_id.clone();
    let user_id_d30 = user_id.clone();
    let user_id_models = user_id.clone();
    let user_id_conv = user_id.clone();
    let user_id_user = user_id.clone();

    let (d1, d7, d30, top_models, conversations, cowork_user) = tokio::join!(
        async move {
            usage_repo::fetch_usage_window(&pool_for_d1, &user_id_d1, 1)
                .await
                .unwrap_or_default()
        },
        async move {
            usage_repo::fetch_usage_window(&pool_for_d7, &user_id_d7, 7)
                .await
                .unwrap_or_default()
        },
        async move {
            usage_repo::fetch_usage_window(&pool_for_d30, &user_id_d30, 30)
                .await
                .unwrap_or_default()
        },
        async move {
            usage_repo::fetch_top_models(&pool_for_models, &user_id_models, 5)
                .await
                .unwrap_or_default()
        },
        async move {
            usage_repo::fetch_conversation_summary(&pool_for_conv, &user_id_conv)
                .await
                .unwrap_or_default()
        },
        async move {
            find_cowork_user(&pool_for_user, &user_id_user)
                .await
                .inspect_err(|e| {
                    tracing::warn!(error = %e, user_id = %user_id_user, "bridge_profile: find_cowork_user failed");
                })
                .ok()
                .flatten()
        }
    );

    let display_name = cowork_user.as_ref().and_then(|u| u.display_name.clone());

    let (jwt_issuer, gateway_url) = read_config_strings();
    let bridge_profile = build_bridge_profile_block();

    let identity = ProfileIdentity {
        email: user_ctx.email.as_str().to_string(),
        display_name,
        user_id: user_ctx.user_id.as_str().to_string(),
        tenant_id: read_tenant_id(),
        provider: None,
        roles: user_ctx.roles.clone(),
        jwt_issuer,
        gateway: gateway_url,
        is_admin: user_ctx.is_admin,
    };

    let usage = ProfileUsage {
        d1: d1.into(),
        d7: d7.into(),
        d30: d30.into(),
        top_models: top_models.into_iter().map(Into::into).collect(),
        conversations: ConversationSummary {
            total_conversations: conversations.total_conversations,
            total_ai_requests: conversations.total_ai_requests,
            by_model: conversations.by_model.into_iter().map(Into::into).collect(),
            by_agent: conversations.by_agent.into_iter().map(Into::into).collect(),
            recent: conversations
                .recent
                .into_iter()
                .map(|r| RecentConversation {
                    context_id: r.context_id,
                    context_name: r.context_name,
                    last_activity: r.last_activity,
                    ai_requests: r.ai_requests,
                    model: r.model,
                    agent_name: r.agent_name,
                })
                .collect(),
        },
    };

    let agents = build_agents_block();

    BridgeProfilePageData {
        page: "profile",
        title: "Profile",
        identity,
        bridge_profile,
        usage,
        agents,
    }
}

fn read_config_strings() -> (Option<String>, Option<String>) {
    Config::get().map_or((None, None), |c| {
        (
            Some(c.jwt_issuer.clone()),
            Some(c.api_external_url.trim_end_matches('/').to_string()),
        )
    })
}

fn read_tenant_id() -> Option<String> {
    let bootstrap = ProfileBootstrap::get().ok()?;
    bootstrap
        .cloud
        .as_ref()
        .and_then(|cloud| cloud.tenant_id.as_ref().map(|id| id.as_str().to_string()))
}

fn build_bridge_profile_block() -> Option<BridgeProfileBlock> {
    let profile = ProfileBootstrap::get().ok()?;
    let gateway = profile.gateway.as_ref().filter(|g| g.enabled)?;

    let base = profile.server.api_external_url.trim_end_matches('/');
    let prefix = gateway.inference_path_prefix.trim_end_matches('/');
    let inference_gateway_base_url = format!("{base}{prefix}");

    let models: Vec<String> = gateway.catalog.as_ref().map_or_else(Vec::new, |catalog| {
        catalog
            .models
            .iter()
            .map(|m| m.id.as_str().to_owned())
            .collect()
    });

    let organization_uuid = profile
        .cloud
        .as_ref()
        .and_then(|cloud| cloud.tenant_id.as_ref().map(TenantId::as_str))
        .map(canonicalize_org_uuid);

    let models_count = models.len();
    Some(BridgeProfileBlock {
        inference_gateway_base_url,
        auth_scheme: gateway.auth_scheme.clone(),
        models,
        models_count,
        organization_uuid,
    })
}

fn canonicalize_org_uuid(tenant_id: &str) -> String {
    let suffix = tenant_id.strip_prefix("local_").unwrap_or(tenant_id);
    if let Ok(parsed) = Uuid::parse_str(suffix) {
        return parsed.to_string();
    }
    Uuid::new_v5(&Uuid::NAMESPACE_OID, tenant_id.as_bytes()).to_string()
}

fn build_agents_block() -> AgentsBlock {
    let services_path = match ProfileBootstrap::get() {
        Ok(p) => PathBuf::from(&p.paths.services),
        Err(_) => return AgentsBlock::default(),
    };

    let agents = match crate::repositories::governance_grp::agents::list_agents(&services_path) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(error = %e, "list_agents failed for profile pane");
            return AgentsBlock::default();
        }
    };

    let visible: Vec<_> = agents.into_iter().filter(|a| a.show_in_ui).collect();
    let total = visible.len() as i64;
    let enabled = visible.iter().filter(|a| a.enabled).count() as i64;

    let items = visible
        .into_iter()
        .map(|a| AgentItem {
            id: a.id.as_str().to_string(),
            display_name: if a.name.is_empty() {
                a.id.as_str().to_string()
            } else {
                a.name
            },
            enabled: a.enabled,
            host_running: false,
        })
        .collect();

    AgentsBlock {
        total,
        enabled,
        items,
    }
}

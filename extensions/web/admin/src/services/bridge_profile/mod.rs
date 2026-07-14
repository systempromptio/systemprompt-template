//! Aggregator for the bridge-style profile pane.
//!
//! Produces the same payload shape consumed by the bridge GUI's profile tab
//! so the SSR profile page and (future) `/v1/bridge/profile/usage` endpoint
//! render the same data from the same source.

mod assemble;

use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::profile_grp::usage as usage_repo;
use crate::types::UserContext;

use assemble::{
    build_agents_block, build_bridge_profile_block, build_usage, fetch_usage_sections,
    read_config_strings, read_tenant_id,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProfileIdentity {
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
pub(crate) struct UsageWindow {
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
pub(crate) struct ModelShare {
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
pub(crate) struct ConversationGroup {
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
pub(crate) struct RecentConversation {
    pub context_id: String,
    pub context_name: Option<String>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub ai_requests: i64,
    pub model: Option<String>,
    pub agent_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct ConversationSummary {
    pub total_conversations: i64,
    pub total_ai_requests: i64,
    pub by_model: Vec<ConversationGroup>,
    pub by_agent: Vec<ConversationGroup>,
    pub recent: Vec<RecentConversation>,
}

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
pub(crate) struct BridgeProfilePageData {
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
pub(crate) async fn build_bridge_profile_data(
    pool: Arc<PgPool>,
    user_ctx: &UserContext,
) -> BridgeProfilePageData {
    let user_id = user_ctx.user_id.as_str().to_owned();

    let sections = fetch_usage_sections(&pool, &user_id).await;
    let display_name = sections
        .bridge_user
        .as_ref()
        .and_then(|u| u.display_name.clone());

    let (jwt_issuer, gateway_url) = read_config_strings();
    let bridge_profile = build_bridge_profile_block();

    let identity = ProfileIdentity {
        email: user_ctx.email.as_str().to_owned(),
        display_name,
        user_id: user_ctx.user_id.as_str().to_owned(),
        tenant_id: read_tenant_id(),
        provider: None,
        roles: user_ctx.roles.clone(),
        jwt_issuer,
        gateway: gateway_url,
        is_admin: user_ctx.is_admin,
    };

    let usage = build_usage(sections);
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

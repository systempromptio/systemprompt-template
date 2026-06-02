//! Per-section assembly for the bridge profile payload.
//!
//! Each function owns one card on the profile pane: the concurrent usage
//! fan-out, the usage view-model, config/identity strings, the bridge gateway
//! block, and the agents block. Falls back to empty defaults on failure so a
//! missing section renders as an empty card rather than a page-level error.

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::TenantId;
use systemprompt::models::Config;
use uuid::Uuid;

use crate::repositories::bridge_grp::{find_bridge_user, BridgeUserRow};
use crate::repositories::profile_grp::usage as usage_repo;

use super::{
    AgentItem, AgentsBlock, BridgeProfileBlock, ConversationSummary, ProfileUsage,
    RecentConversation,
};

pub(super) struct UsageSections {
    pub(super) d1: usage_repo::UsageWindow,
    pub(super) d7: usage_repo::UsageWindow,
    pub(super) d30: usage_repo::UsageWindow,
    pub(super) top_models: Vec<usage_repo::ModelShare>,
    pub(super) conversations: usage_repo::ConversationSummary,
    pub(super) bridge_user: Option<BridgeUserRow>,
}

pub(super) async fn fetch_usage_sections(pool: &Arc<PgPool>, user_id: &str) -> UsageSections {
    let pool_for_d1 = Arc::clone(pool);
    let pool_for_d7 = Arc::clone(pool);
    let pool_for_d30 = Arc::clone(pool);
    let pool_for_models = Arc::clone(pool);
    let pool_for_conv = Arc::clone(pool);
    let pool_for_user = Arc::clone(pool);

    let user_id_d1 = user_id.to_string();
    let user_id_d7 = user_id.to_string();
    let user_id_d30 = user_id.to_string();
    let user_id_models = user_id.to_string();
    let user_id_conv = user_id.to_string();
    let user_id_user = user_id.to_string();

    let (d1, d7, d30, top_models, conversations, bridge_user) = tokio::join!(
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
            find_bridge_user(&pool_for_user, &user_id_user)
                .await
                .inspect_err(|e| {
                    tracing::warn!(error = %e, user_id = %user_id_user, "bridge_profile: find_bridge_user failed");
                })
                .ok()
                .flatten()
        }
    );

    UsageSections {
        d1,
        d7,
        d30,
        top_models,
        conversations,
        bridge_user,
    }
}

pub(super) fn build_usage(sections: UsageSections) -> ProfileUsage {
    let conversations = sections.conversations;
    ProfileUsage {
        d1: sections.d1.into(),
        d7: sections.d7.into(),
        d30: sections.d30.into(),
        top_models: sections.top_models.into_iter().map(Into::into).collect(),
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
    }
}

pub(super) fn read_config_strings() -> (Option<String>, Option<String>) {
    Config::get().map_or((None, None), |c| {
        (
            Some(c.jwt_issuer.clone()),
            Some(c.api_external_url.trim_end_matches('/').to_string()),
        )
    })
}

pub(super) fn read_tenant_id() -> Option<String> {
    let bootstrap = ProfileBootstrap::get().ok()?;
    bootstrap
        .cloud
        .as_ref()
        .and_then(|cloud| cloud.tenant_id.as_ref().map(|id| id.as_str().to_string()))
}

pub(super) fn build_bridge_profile_block() -> Option<BridgeProfileBlock> {
    let profile = ProfileBootstrap::get().ok()?;
    let gateway = profile
        .gateway
        .as_ref()
        .and_then(systemprompt::models::profile::GatewayState::resolved)
        .filter(|g| g.enabled)?;

    let base = profile.server.api_external_url.trim_end_matches('/');
    let prefix = gateway.inference_path_prefix.trim_end_matches('/');
    let inference_gateway_base_url = format!("{base}{prefix}");

    let models: Vec<String> = profile
        .providers
        .providers
        .iter()
        .flat_map(|entry| {
            entry.models.iter().flat_map(|m| {
                std::iter::once(m.id.as_str().to_owned())
                    .chain(m.aliases.iter().map(|a| a.as_str().to_owned()))
            })
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

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

pub(super) fn build_agents_block() -> AgentsBlock {
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

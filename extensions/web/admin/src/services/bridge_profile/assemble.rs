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
use systemprompt::identifiers::{TenantId, UserId};
use systemprompt::models::Config;
use uuid::Uuid;

use crate::repositories::bridge::{BridgeIdentityRow, find_bridge_user};
use crate::repositories::users::usage as usage_repo;

use super::{
    BridgeAgentItem, BridgeAgentsBlock, BridgeProfileBlock, BridgeProfileUsage,
    ProfileMarketplaceView,
};

pub(super) struct BridgeUsageSections {
    pub(super) d1: usage_repo::UsageWindow,
    pub(super) d7: usage_repo::UsageWindow,
    pub(super) d30: usage_repo::UsageWindow,
    pub(super) top_models: Vec<usage_repo::ModelShare>,
    pub(super) conversations: usage_repo::ConversationSummary,
    pub(super) bridge_user: Option<BridgeIdentityRow>,
}

pub(super) async fn load_usage_sections(
    pool: &Arc<PgPool>,
    user_id: &UserId,
) -> BridgeUsageSections {
    let pool_for_d1 = Arc::clone(pool);
    let pool_for_d7 = Arc::clone(pool);
    let pool_for_d30 = Arc::clone(pool);
    let pool_for_models = Arc::clone(pool);
    let pool_for_conv = Arc::clone(pool);
    let pool_for_user = Arc::clone(pool);

    let user_id_d1 = user_id.to_owned();
    let user_id_d7 = user_id.to_owned();
    let user_id_d30 = user_id.to_owned();
    let user_id_models = user_id.to_owned();
    let user_id_conv = user_id.to_owned();
    let user_id_user = user_id.to_owned();

    let (d1, d7, d30, top_models, conversations, bridge_user) = tokio::join!(
        async move {
            usage_repo::get_usage_window(&pool_for_d1, &user_id_d1, 1)
                .await
                .unwrap_or_default()
        },
        async move {
            usage_repo::get_usage_window(&pool_for_d7, &user_id_d7, 7)
                .await
                .unwrap_or_default()
        },
        async move {
            usage_repo::get_usage_window(&pool_for_d30, &user_id_d30, 30)
                .await
                .unwrap_or_default()
        },
        async move {
            usage_repo::list_top_models(
                &pool_for_models,
                &user_id_models,
                Some(usage_repo::CONVERSATION_WINDOW_DAYS),
                5,
            )
            .await
            .unwrap_or_default()
        },
        async move {
            usage_repo::get_conversation_summary(&pool_for_conv, &user_id_conv)
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

    BridgeUsageSections {
        d1,
        d7,
        d30,
        top_models,
        conversations,
        bridge_user,
    }
}

pub(super) fn build_usage(sections: BridgeUsageSections) -> BridgeProfileUsage {
    BridgeProfileUsage {
        d1: sections.d1,
        d7: sections.d7,
        d30: sections.d30,
        top_models: sections.top_models,
        conversations: sections.conversations,
    }
}

pub(crate) fn read_config_strings() -> (Option<String>, Option<String>) {
    Config::get().map_or((None, None), |c| {
        (
            Some(c.jwt_issuer.clone()),
            Some(c.api_external_url.trim_end_matches('/').to_owned()),
        )
    })
}

pub(super) fn read_tenant_id() -> Option<TenantId> {
    let bootstrap = ProfileBootstrap::get().ok()?;
    bootstrap
        .cloud
        .as_ref()
        .and_then(|cloud| cloud.tenant_id.clone())
}

pub(super) fn build_bridge_profile_block() -> Option<BridgeProfileBlock> {
    let profile = ProfileBootstrap::get().ok()?;
    let services = systemprompt::loader::ServicesBootstrap::get().ok()?;
    let gateway = services.gateway_config().filter(|g| g.enabled)?;

    let base = profile.server.api_external_url.trim_end_matches('/');
    let prefix = gateway.inference_path_prefix.trim_end_matches('/');
    let inference_gateway_base_url = format!("{base}{prefix}");

    let models: Vec<String> = services
        .providers
        .advertised_model_ids(&[])
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    let organization_uuid = profile
        .cloud
        .as_ref()
        .and_then(|cloud| cloud.tenant_id.as_ref())
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

fn canonicalize_org_uuid(tenant_id: &TenantId) -> String {
    let s = tenant_id.as_str();
    let suffix = s.strip_prefix("local_").unwrap_or(s);
    if let Ok(parsed) = Uuid::parse_str(suffix) {
        return parsed.to_string();
    }
    Uuid::new_v5(&Uuid::NAMESPACE_OID, s.as_bytes()).to_string()
}

pub(super) fn build_agents_block() -> BridgeAgentsBlock {
    let services_path = match ProfileBootstrap::get() {
        Ok(p) => PathBuf::from(&p.paths.services),
        Err(_) => return BridgeAgentsBlock::default(),
    };

    let agents = match crate::repositories::config::agents::list_configured_agents(&services_path) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(error = %e, "list_configured_agents failed for profile pane");
            return BridgeAgentsBlock::default();
        },
    };

    let visible: Vec<_> = agents.into_iter().filter(|a| a.show_in_ui).collect();
    let total = visible.len() as i64;
    let enabled = visible.iter().filter(|a| a.enabled).count() as i64;

    let items = visible
        .into_iter()
        .map(|a| BridgeAgentItem {
            id: a.id.as_str().to_owned(),
            display_name: if a.name.is_empty() {
                a.id.as_str().to_owned()
            } else {
                a.name
            },
            enabled: a.enabled,
            host_running: false,
        })
        .collect();

    BridgeAgentsBlock {
        total,
        enabled,
        items,
    }
}

// Why: the card answers "what do I actually reach", not "what does the manifest
// offer", so it resolves through the same subject matrix the admin pages use
// rather than reading the YAML audience. A user whose group grants a
// marketplace and whose per-user rule denies it must see it absent here.
pub(super) async fn build_marketplaces(
    pool: &PgPool,
    user_id: &UserId,
    roles: Vec<String>,
) -> Vec<ProfileMarketplaceView> {
    let Ok(services_path) = crate::handlers::shared::get_services_path() else {
        return Vec::new();
    };
    let manifests =
        crate::repositories::marketplace::manifests::list_marketplace_configs(&services_path)
            .unwrap_or_default();
    if manifests.is_empty() {
        return Vec::new();
    }
    let rows = manifests
        .iter()
        .map(|m| (m.id.clone(), m.name.clone(), None))
        .collect();
    let sections = vec![("marketplace".to_owned(), "Marketplaces".to_owned(), rows)];

    let Ok(subject) =
        crate::repositories::users::access_control::user_subject(pool, user_id, roles)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "profile: subject attributes failed"))
    else {
        return Vec::new();
    };
    let resolved = crate::repositories::users::access_control::resolve_subject_matrix(
        pool, &subject, sections,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "profile: marketplace matrix failed"))
    .unwrap_or_default();

    let Some(section) = resolved.into_iter().next() else {
        return Vec::new();
    };
    section
        .rows
        .into_iter()
        .filter(|row| row.effective == "allow")
        .filter_map(|row| {
            let manifest = manifests.iter().find(|m| m.id == row.entity_id)?;
            Some(ProfileMarketplaceView {
                id: manifest.id.clone(),
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                plugin_count: manifest.plugins.len(),
                layer: row.source.layer,
            })
        })
        .collect()
}

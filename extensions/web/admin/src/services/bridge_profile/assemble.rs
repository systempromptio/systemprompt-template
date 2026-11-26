//! Per-section assembly for the bridge profile payload.
//!
//! Each function owns one card on the profile pane: the concurrent usage
//! fan-out, the usage view-model, config/identity strings, the bridge gateway
//! block, and the agents block. Falls back to empty defaults on failure so a
//! missing section renders as an empty card rather than a page-level error.

use crate::env::AdminEnv;

use sqlx::PgPool;
use systemprompt::identifiers::{TenantId, UserId};
use uuid::Uuid;

use crate::repositories::bridge::{BridgeIdentityRow, find_bridge_user};
#[cfg(feature = "gateway")]
use crate::repositories::users::usage as usage_repo;

#[cfg(feature = "gateway")]
use super::ProfileUsage;
use super::{AgentItem, AgentsBlock, BridgeProfileBlock, ProfileMarketplaceView};

pub(super) async fn load_bridge_user(pool: &PgPool, user_id: &UserId) -> Option<BridgeIdentityRow> {
    find_bridge_user(pool, user_id)
        .await
        .inspect_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "bridge_profile: find_bridge_user failed");
        })
        .ok()
        .flatten()
}

#[cfg(feature = "gateway")]
pub(super) async fn load_usage(pool: &PgPool, user_id: &UserId) -> ProfileUsage {
    let window = |days: i32| async move {
        usage_repo::get_usage_window(pool, user_id, days)
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, days, "bridge profile: usage window failed"),
            )
            .unwrap_or_default()
    };
    let top_models = async {
        usage_repo::list_top_models(pool, user_id, Some(usage_repo::CONVERSATION_WINDOW_DAYS), 5)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "bridge profile: top skills failed"))
            .unwrap_or_default()
    };
    #[cfg(feature = "agents")]
    let conversations = async {
        usage_repo::get_conversation_summary(pool, user_id)
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, "bridge profile: conversation summary failed"),
            )
            .unwrap_or_default()
    };
    #[cfg(feature = "agents")]
    let (d1, d7, d30, top_models, conversations) =
        tokio::join!(window(1), window(7), window(30), top_models, conversations);
    #[cfg(not(feature = "agents"))]
    let (d1, d7, d30, top_models) = tokio::join!(window(1), window(7), window(30), top_models);
    ProfileUsage {
        d1,
        d7,
        d30,
        top_models,
        #[cfg(feature = "agents")]
        conversations,
    }
}

pub(crate) fn read_config_strings(env: &AdminEnv) -> (Option<String>, Option<String>) {
    (
        Some(env.config().jwt_issuer.clone()),
        Some(
            env.config()
                .api_external_url
                .trim_end_matches('/')
                .to_owned(),
        ),
    )
}

pub(super) fn read_tenant_id(env: &AdminEnv) -> Option<TenantId> {
    env.profile().cloud_tenant_id.clone()
}

pub(super) fn build_bridge_profile_block(env: &AdminEnv) -> Option<BridgeProfileBlock> {
    let services = env.services();
    let gateway = services.gateway_config().filter(|g| g.enabled)?;

    let base = env.config().api_external_url.trim_end_matches('/');
    let prefix = gateway.inference_path_prefix.trim_end_matches('/');
    let inference_gateway_base_url = format!("{base}{prefix}");

    let models: Vec<String> = env
        .services_view()
        .catalog()
        .advertised_models(None)
        .map(|model| model.id.as_str().to_owned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    let organization_uuid = env
        .profile()
        .cloud_tenant_id
        .as_ref()
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

pub(super) fn build_agents_block(env: &AdminEnv) -> AgentsBlock {
    let services_path = env.baked_services_root();

    let agents = match crate::repositories::config::agents::list_configured_agents(&services_path) {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(error = %e, "list_configured_agents failed for profile pane");
            return AgentsBlock::default();
        },
    };

    let visible: Vec<_> = agents.into_iter().filter(|a| a.show_in_ui).collect();
    let total = visible.len() as i64;
    let enabled = visible.iter().filter(|a| a.enabled).count() as i64;

    let items = visible
        .into_iter()
        .map(|a| AgentItem {
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

    AgentsBlock {
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
    env: &AdminEnv,
    pool: &PgPool,
    user_id: &UserId,
    roles: Vec<String>,
) -> crate::error::AdminResult<Vec<ProfileMarketplaceView>> {
    let services_path = crate::handlers::shared::get_services_path(env);
    let manifests =
        crate::repositories::marketplace::manifests::list_marketplace_configs(&services_path)
            .inspect_err(
                |e| tracing::warn!(error = %e, "bridge profile: marketplace configs failed"),
            )
            .unwrap_or_default();
    if manifests.is_empty() {
        return Ok(Vec::new());
    }
    let rows = manifests
        .iter()
        .map(|m| (m.id.as_str().to_owned(), m.name.clone(), None))
        .collect();
    let sections = vec![("marketplace".to_owned(), "Marketplaces".to_owned(), rows)];

    let subject =
        crate::repositories::users::access_control::user_subject(env, user_id, roles).await?;
    let resolved = crate::repositories::users::access_control::resolve_subject_matrix(
        env, pool, &subject, sections,
    )
    .await?;

    let Some(section) = resolved.into_iter().next() else {
        return Ok(Vec::new());
    };
    Ok(section
        .rows
        .into_iter()
        .filter(|row| row.effective == "allow")
        .filter_map(|row| {
            let manifest = manifests.iter().find(|m| m.id.as_str() == row.entity_id)?;
            Some(ProfileMarketplaceView {
                id: manifest.id.clone(),
                name: manifest.name.clone(),
                version: manifest.version.clone(),
                plugin_count: manifest.plugins.len(),
                layer: row.source.layer,
            })
        })
        .collect())
}

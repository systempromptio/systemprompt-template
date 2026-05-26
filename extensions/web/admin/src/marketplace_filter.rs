//! [`MarketplaceFilter`] implementation for the systemprompt template.
//!
//! Resolves a user's `(roles, department)` from `users` joined to
//! `user_profile_ext` and consults `access_control_rules` rows keyed by the entity's own
//! [`EntityKind`] (`Plugin`, `Skill`, `Agent`, `McpServer`) to decide
//! which marketplace items the gateway should sign for that user.
//!
//! Default policy is **explicit allow** — if no rule matches and the
//! entity does not flag `default_included = true`, the item is dropped.
//! This matches the YAML-driven access pattern used elsewhere in the
//! admin extension (see `repositories::governance_grp::acl_yaml_loader`).

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{
    AgentId, HookId, MarketplaceId, McpServerId, PluginId, RouteId, SkillId, UserId,
};
use systemprompt::marketplace::{
    register_marketplace_filter, MarketplaceCandidate, MarketplaceFilter, MarketplaceFilterError,
};
use systemprompt_security::authz::{
    resolve as resolve_access, AccessControlRepository, Decision, EntityKind, EntityRef,
    ResolveInput,
};

fn entity_ref_for(kind: EntityKind, id: &str) -> EntityRef {
    match kind {
        EntityKind::Plugin => EntityRef::Plugin(PluginId::new(id)),
        EntityKind::Skill => EntityRef::Skill(SkillId::new(id)),
        EntityKind::Agent => EntityRef::Agent(AgentId::new(id)),
        EntityKind::McpServer => EntityRef::McpServer(McpServerId::new(id)),
        EntityKind::Marketplace => EntityRef::Marketplace(MarketplaceId::new(id)),
        EntityKind::GatewayRoute => EntityRef::GatewayRoute(RouteId::new(id)),
        EntityKind::Hook => EntityRef::Hook(HookId::new(id)),
    }
}

use crate::repositories::users_grp::users::get_user_roles_department;

#[derive(Debug)]
pub struct TemplateMarketplaceFilter {
    pool: Arc<PgPool>,
    repo: AccessControlRepository,
}

impl TemplateMarketplaceFilter {
    /// # Errors
    /// Returns [`MarketplaceFilterError::Backend`] if the configured database
    /// pool is not Postgres-backed.
    pub fn from_db(db: &DbPool) -> Result<Arc<dyn MarketplaceFilter>, MarketplaceFilterError> {
        let pool = db
            .pool_arc()
            .map_err(|e| MarketplaceFilterError::Backend(e.to_string()))?;
        Ok(Arc::new(Self {
            repo: AccessControlRepository::from_pool(Arc::clone(&pool)),
            pool,
        }))
    }

    async fn user_principal(
        &self,
        user_id: &UserId,
    ) -> Result<(Vec<String>, String), MarketplaceFilterError> {
        match get_user_roles_department(self.pool.as_ref(), user_id.as_str()).await {
            Ok(Some(pair)) => Ok(pair),
            Ok(None) => Err(MarketplaceFilterError::UnknownUser(user_id.to_string())),
            Err(e) => Err(MarketplaceFilterError::Backend(e.to_string())),
        }
    }

    async fn keep_ids(
        &self,
        user_id: &str,
        roles: &[String],
        department: &str,
        kind: EntityKind,
        ids: &[String],
    ) -> Result<std::collections::HashSet<String>, MarketplaceFilterError> {
        if ids.is_empty() {
            return Ok(std::collections::HashSet::new());
        }
        let bulk = self
            .repo
            .list_rules_bulk(kind, ids)
            .await
            .map_err(|e| MarketplaceFilterError::Backend(e.to_string()))?;
        let mut keep = std::collections::HashSet::with_capacity(ids.len());
        for id in ids {
            let entity_rules = bulk.get(id).map_or(&[][..], Vec::as_slice);
            let default_included = self
                .repo
                .get_entity(kind, id)
                .await
                .inspect_err(|e| {
                    tracing::warn!(
                        error = %e, kind = ?kind, id = %id,
                        "marketplace_filter: get_entity lookup failed; treating as default_included=None"
                    );
                })
                .ok()
                .flatten()
                .map(|e| e.default_included);
            let entity = entity_ref_for(kind, id);
            let uid = UserId::new(user_id);
            let decision = resolve_access(ResolveInput {
                entity: &entity,
                rules: entity_rules,
                user_id: &uid,
                user_roles: roles,
                department,
                default_included,
            });
            if matches!(decision, Decision::Allow { .. }) {
                keep.insert(id.clone());
            }
        }
        Ok(keep)
    }
}

#[async_trait::async_trait]
impl MarketplaceFilter for TemplateMarketplaceFilter {
    async fn filter(
        &self,
        user_id: &UserId,
        candidate: MarketplaceCandidate,
    ) -> Result<MarketplaceCandidate, MarketplaceFilterError> {
        let (roles, department) = self.user_principal(user_id).await?;
        let uid = user_id.as_str();

        let plugin_ids: Vec<String> = candidate.plugins.iter().map(|p| p.id.to_string()).collect();
        let skill_ids: Vec<String> = candidate.skills.iter().map(|s| s.id.to_string()).collect();
        let agent_ids: Vec<String> = candidate.agents.iter().map(|a| a.id.to_string()).collect();
        let hook_ids: Vec<String> = candidate.hooks.iter().map(|h| h.id.to_string()).collect();
        let mcp_ids: Vec<String> = candidate
            .managed_mcp_servers
            .iter()
            .map(|m| m.name.to_string())
            .collect();

        let (keep_plugins, keep_skills, keep_agents, keep_hooks, keep_mcp) = tokio::try_join!(
            self.keep_ids(uid, &roles, &department, EntityKind::Plugin, &plugin_ids),
            self.keep_ids(uid, &roles, &department, EntityKind::Skill, &skill_ids),
            self.keep_ids(uid, &roles, &department, EntityKind::Agent, &agent_ids),
            self.keep_ids(uid, &roles, &department, EntityKind::Hook, &hook_ids),
            self.keep_ids(uid, &roles, &department, EntityKind::McpServer, &mcp_ids),
        )?;

        Ok(MarketplaceCandidate {
            plugins: candidate
                .plugins
                .into_iter()
                .filter(|p| keep_plugins.contains(p.id.as_str()))
                .collect(),
            skills: candidate
                .skills
                .into_iter()
                .filter(|s| keep_skills.contains(s.id.as_str()))
                .collect(),
            agents: candidate
                .agents
                .into_iter()
                .filter(|a| keep_agents.contains(a.id.as_str()))
                .collect(),
            hooks: candidate
                .hooks
                .into_iter()
                .filter(|h| keep_hooks.contains(h.id.as_str()))
                .collect(),
            managed_mcp_servers: candidate
                .managed_mcp_servers
                .into_iter()
                .filter(|m| keep_mcp.contains(m.name.as_str()))
                .collect(),
        })
    }
}

register_marketplace_filter!(TemplateMarketplaceFilter::from_db, priority = 100);

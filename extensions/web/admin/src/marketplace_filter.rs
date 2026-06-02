//! [`MarketplaceFilter`] implementation for the systemprompt template.
//!
//! Resolves a user's `(roles, department)` from `users` joined to
//! `user_profile_ext` and consults `access_control_rules` rows keyed by the entity's own
//! [`EntityKind`] (`Plugin`, `Skill`, `Agent`, `McpServer`) to decide
//! which marketplace items the gateway should sign for that user.
//!
//! Default policy is **explicit allow**, but the owning marketplace is passed
//! to the resolver as a parent: a member is kept if it has its own allow rule
//! (or `default_included = true`) **or it inherits the marketplace's grant**.
//! An explicit member-level deny still overrides the inherited allow. This lets
//! a single marketplace grant cover every member skill/agent/mcp without a
//! per-entity rule (see `services/access-control/roles.yaml`). If neither the
//! member nor the marketplace grants access, the item is dropped.

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
    resolve as resolve_access, AccessControlRepository, AccessRule, Decision, EntityKind,
    EntityRef, ResolveInput, ResolveParent,
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

struct CandidateEntityIds {
    plugins: Vec<String>,
    skills: Vec<String>,
    agents: Vec<String>,
    hooks: Vec<String>,
    mcp: Vec<String>,
}

impl CandidateEntityIds {
    fn from_candidate(candidate: &MarketplaceCandidate) -> Self {
        Self {
            plugins: candidate.plugins.iter().map(|p| p.id.to_string()).collect(),
            skills: candidate.skills.iter().map(|s| s.id.to_string()).collect(),
            agents: candidate.agents.iter().map(|a| a.id.to_string()).collect(),
            hooks: candidate.hooks.iter().map(|h| h.id.to_string()).collect(),
            mcp: candidate
                .managed_mcp_servers
                .iter()
                .map(|m| m.name.to_string())
                .collect(),
        }
    }
}

type KeepSet = std::collections::HashSet<String>;

struct KeepSets {
    plugins: KeepSet,
    skills: KeepSet,
    agents: KeepSet,
    hooks: KeepSet,
    mcp: KeepSet,
}

fn apply_keep_sets(candidate: MarketplaceCandidate, keep: &KeepSets) -> MarketplaceCandidate {
    MarketplaceCandidate {
        plugins: candidate
            .plugins
            .into_iter()
            .filter(|p| keep.plugins.contains(p.id.as_str()))
            .collect(),
        skills: candidate
            .skills
            .into_iter()
            .filter(|s| keep.skills.contains(s.id.as_str()))
            .collect(),
        agents: candidate
            .agents
            .into_iter()
            .filter(|a| keep.agents.contains(a.id.as_str()))
            .collect(),
        hooks: candidate
            .hooks
            .into_iter()
            .filter(|h| keep.hooks.contains(h.id.as_str()))
            .collect(),
        managed_mcp_servers: candidate
            .managed_mcp_servers
            .into_iter()
            .filter(|m| keep.mcp.contains(m.name.as_str()))
            .collect(),
        // Carry the owning marketplace context through unchanged; the
        // filter only shrinks entry lists, it must not drop the scope the
        // gateway attached.
        marketplace_id: candidate.marketplace_id,
        access: candidate.access,
    }
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
        parents: &[ResolveParent<'_>],
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
            let _ = department;
            let decision = resolve_access(ResolveInput {
                entity: &entity,
                rules: entity_rules,
                user_id: &uid,
                user_roles: roles,
                default_included,
                parents,
            });
            if matches!(decision, Decision::Allow { .. }) {
                keep.insert(id.clone());
            }
        }
        Ok(keep)
    }

    async fn marketplace_parent(
        &self,
        candidate: &MarketplaceCandidate,
    ) -> Result<Option<(EntityRef, Vec<AccessRule>, Option<bool>)>, MarketplaceFilterError> {
        let Some(mp_id) = candidate.marketplace_id.as_ref() else {
            return Ok(None);
        };
        let id = mp_id.as_str();
        let rules = self
            .repo
            .list_rules_for_entity(EntityKind::Marketplace, id)
            .await
            .map_err(|e| MarketplaceFilterError::Backend(e.to_string()))?;
        let default_included = self
            .repo
            .get_entity(EntityKind::Marketplace, id)
            .await
            .inspect_err(|e| {
                tracing::warn!(
                    error = %e, marketplace_id = %id,
                    "marketplace_filter: marketplace get_entity lookup failed; falling back to candidate access"
                );
            })
            .ok()
            .flatten()
            .map(|e| e.default_included)
            .or_else(|| candidate.access.as_ref().map(|a| a.default_included));
        Ok(Some((
            EntityRef::Marketplace(MarketplaceId::new(id)),
            rules,
            default_included,
        )))
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

        let mp_parent = self.marketplace_parent(&candidate).await?;
        let parents: Vec<ResolveParent<'_>> = mp_parent
            .as_ref()
            .map(|(entity, rules, default_included)| ResolveParent {
                entity,
                rules,
                default_included: *default_included,
            })
            .into_iter()
            .collect();

        let ids = CandidateEntityIds::from_candidate(&candidate);

        let (plugins, skills, agents, hooks, mcp) = tokio::try_join!(
            self.keep_ids(
                uid,
                &roles,
                &department,
                EntityKind::Plugin,
                &ids.plugins,
                &parents
            ),
            self.keep_ids(
                uid,
                &roles,
                &department,
                EntityKind::Skill,
                &ids.skills,
                &parents
            ),
            self.keep_ids(
                uid,
                &roles,
                &department,
                EntityKind::Agent,
                &ids.agents,
                &parents
            ),
            self.keep_ids(
                uid,
                &roles,
                &department,
                EntityKind::Hook,
                &ids.hooks,
                &parents
            ),
            self.keep_ids(
                uid,
                &roles,
                &department,
                EntityKind::McpServer,
                &ids.mcp,
                &parents
            ),
        )?;

        let keep = KeepSets {
            plugins,
            skills,
            agents,
            hooks,
            mcp,
        };
        Ok(apply_keep_sets(candidate, &keep))
    }
}

register_marketplace_filter!(TemplateMarketplaceFilter::from_db, priority = 100);

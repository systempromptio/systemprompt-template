//! [`MarketplaceFilter`] implementation for the systemprompt template.
//!
//! Resolves a user's `(roles, department)` from `users` joined to
//! `user_profile_ext` and consults `access_control_rules` rows keyed by the
//! entity's own [`EntityKind`] (`Plugin`, `Skill`, `Agent`, `McpServer`) to
//! decide which marketplace items the gateway should sign for that user.
//!
//! Default policy is **explicit allow**. The owning marketplace is passed to
//! the resolver as a parent, so one marketplace rule covers every member
//! skill/agent/mcp that declares no rules of its own, without per-entity
//! boilerplate (see `services/access-control/roles.yaml`). A member that
//! declares any rule owns its decision outright. If neither path grants
//! access, the item is dropped.

mod keepsets;

pub(crate) use keepsets::entity_ref_for;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{MarketplaceId, UserId};
use systemprompt::marketplace::{
    MarketplaceCandidate, MarketplaceFilter, MarketplaceFilterError, register_marketplace_filter,
};
use systemprompt_security::authz::{
    AccessControlRepository, AccessRule, Decision, EntityKind, EntityRef, ResolveInput,
    ResolveParent, resolve as resolve_access,
};

use keepsets::{CandidateEntityIds, KeepIdsQuery, KeepSets, apply_keep_sets};

use crate::authz::{dimensions, subject_attributes_for};
use crate::repositories::users::queries::find_user_roles_department;

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
        match find_user_roles_department(self.pool.as_ref(), user_id).await {
            Ok(Some(pair)) => Ok(pair),
            Ok(None) => Err(MarketplaceFilterError::UnknownUser(user_id.to_string())),
            Err(e) => Err(MarketplaceFilterError::Backend(e.to_string())),
        }
    }

    async fn keep_ids(
        &self,
        query: KeepIdsQuery<'_>,
    ) -> Result<std::collections::HashSet<String>, MarketplaceFilterError> {
        let KeepIdsQuery {
            user_id,
            roles,
            kind,
            ids,
            parents,
        } = query;
        if ids.is_empty() {
            return Ok(std::collections::HashSet::new());
        }
        let bulk = self
            .repo
            .list_rules_bulk(kind, ids)
            .await
            .map_err(|e| MarketplaceFilterError::Backend(e.to_string()))?;
        let uid = UserId::new(user_id);
        let attributes = subject_attributes_for(self.pool.as_ref(), &uid).await;
        let dimensions = dimensions(self.pool.as_ref());
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
            let decision = resolve_access(ResolveInput {
                entity: &entity,
                rules: entity_rules,
                user_id: &uid,
                user_roles: roles,
                default_included,
                parents,
                attributes: &attributes,
                dimensions,
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
        let (roles, _department) = self.user_principal(user_id).await?;
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
            self.keep_ids(KeepIdsQuery {
                user_id: uid,
                roles: &roles,
                kind: EntityKind::Plugin,
                ids: &ids.plugins,
                parents: &parents,
            }),
            self.keep_ids(KeepIdsQuery {
                user_id: uid,
                roles: &roles,
                kind: EntityKind::Skill,
                ids: &ids.skills,
                parents: &parents,
            }),
            self.keep_ids(KeepIdsQuery {
                user_id: uid,
                roles: &roles,
                kind: EntityKind::Agent,
                ids: &ids.agents,
                parents: &parents,
            }),
            self.keep_ids(KeepIdsQuery {
                user_id: uid,
                roles: &roles,
                kind: EntityKind::Hook,
                ids: &ids.hooks,
                parents: &parents,
            }),
            self.keep_ids(KeepIdsQuery {
                user_id: uid,
                roles: &roles,
                kind: EntityKind::McpServer,
                ids: &ids.mcp,
                parents: &parents,
            }),
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

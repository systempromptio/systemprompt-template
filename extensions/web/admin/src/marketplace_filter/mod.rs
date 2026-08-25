//! [`MarketplaceFilter`] implementation for the systemprompt template.
//!
//! Resolves a user's `(roles, department)` from `users` joined to
//! `user_profile_ext` and hands the subject to core's
//! [`keep_sets`] resolver, which consults `access_control_rules` per entry
//! kind with the owning marketplace cascaded as a parent: one marketplace
//! rule covers every member that declares no rules of its own, and a member
//! that declares any rule owns its decision outright. Default policy is
//! **explicit allow**: if neither path grants access, the item is dropped
//! (see `services/access-control/roles.yaml`).

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::marketplace::{
    KeepSetsSubject, MarketplaceCandidate, MarketplaceFilter, MarketplaceFilterError, keep_sets,
    register_marketplace_filter,
};
use systemprompt_security::authz::AccessControlRepository;

use crate::authz::{dimensions, subject_attributes_for};
use crate::repositories::users::queries::find_user_roles_department;

#[derive(Debug)]
pub struct TemplateMarketplaceFilter {
    pool: Arc<PgPool>,
    repo: AccessControlRepository,
}

impl TemplateMarketplaceFilter {
    pub fn from_db(db: &DbPool) -> Result<Arc<dyn MarketplaceFilter>, MarketplaceFilterError> {
        let pool = db
            .pool_arc()
            .map_err(|e| MarketplaceFilterError::Backend(e.to_string()))?;
        Ok(Arc::new(Self {
            repo: AccessControlRepository::from_pool(Arc::clone(&pool)),
            pool,
        }))
    }

    async fn user_roles(&self, user_id: &UserId) -> Result<Vec<String>, MarketplaceFilterError> {
        match find_user_roles_department(self.pool.as_ref(), user_id).await {
            Ok(Some((roles, _department))) => Ok(roles),
            Ok(None) => Err(MarketplaceFilterError::UnknownUser(user_id.to_string())),
            Err(e) => Err(MarketplaceFilterError::Backend(e.to_string())),
        }
    }
}

#[async_trait::async_trait]
impl MarketplaceFilter for TemplateMarketplaceFilter {
    async fn filter(
        &self,
        user_id: &UserId,
        mut candidate: MarketplaceCandidate,
    ) -> Result<MarketplaceCandidate, MarketplaceFilterError> {
        let roles = self.user_roles(user_id).await?;
        let attributes = subject_attributes_for(self.pool.as_ref(), user_id).await;
        let keep = keep_sets(
            &self.repo,
            &candidate,
            KeepSetsSubject {
                user_id,
                roles: &roles,
                attributes: &attributes,
                dimensions: dimensions(self.pool.as_ref()),
            },
        )
        .await?;
        candidate.retain_entries(&keep);
        Ok(candidate)
    }
}

register_marketplace_filter!(TemplateMarketplaceFilter::from_db, priority = 100);

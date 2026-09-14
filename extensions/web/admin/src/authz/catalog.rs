//! Current subject and parent policies reused across one catalog request.

use crate::error::{AdminError, AdminResult};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex,
    SubjectAttributes, allowed_ids,
};

pub(crate) struct CatalogAccess {
    repo: AccessControlRepository,
    pool: PgPool,
    user: UserId,
    roles: Vec<String>,
    attributes: SubjectAttributes,
    chains: ParentChainIndex,
}

impl CatalogAccess {
    pub(crate) async fn load(pool: &PgPool, user: &UserId) -> AdminResult<Self> {
        let identity = crate::repositories::users::queries::find_identity_envelope(pool, user)
            .await?
            .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
        if identity.status != "active" {
            return Err(AdminError::Forbidden("Active account required".into()));
        }
        let services =
            systemprompt::loader::ServicesBootstrap::get().map_err(AdminError::internal)?;
        let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
        let attributes = super::subject_attributes_for(pool, user).await?;
        let chains = ParentChainIndex::load(&repo, Arc::new(ChainSources::from_services(services)))
            .await
            .map_err(AdminError::internal)?;
        Ok(Self {
            repo,
            pool: pool.clone(),
            user: user.clone(),
            roles: identity.roles,
            attributes,
            chains,
        })
    }

    pub(crate) async fn allowed(
        &self,
        kind: EntityKind,
        ids: &[String],
    ) -> AdminResult<std::collections::HashSet<String>> {
        allowed_ids(
            &self.repo,
            BulkKeepQuery {
                user_id: &self.user,
                roles: &self.roles,
                kind,
                ids,
                chains: &self.chains,
                attributes: &self.attributes,
                dimensions: super::dimensions(&self.pool),
            },
        )
        .await
        .map_err(AdminError::internal)
    }
}

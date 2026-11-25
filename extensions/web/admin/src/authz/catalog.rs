//! Current subject and parent policies reused across one catalog request.

use crate::env::AdminEnv;
use crate::error::{AdminError, AdminResult};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::governance::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex,
    SubjectAttributes, SubjectDimension, allowed_ids,
};
use systemprompt::identifiers::UserId;

pub(crate) struct CatalogAccess {
    repo: AccessControlRepository,
    user: UserId,
    roles: Vec<String>,
    attributes: SubjectAttributes,
    dimensions: Vec<SubjectDimension>,
    chains: ParentChainIndex,
}

impl CatalogAccess {
    pub(crate) async fn load(env: &AdminEnv, pool: &PgPool, user: &UserId) -> AdminResult<Self> {
        let identity = crate::repositories::users::queries::find_identity_envelope(pool, user)
            .await?
            .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
        if identity.status != "active" {
            return Err(AdminError::Forbidden("Active account required".into()));
        }
        let services = env.services();
        let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
        let attributes = super::subject_attributes_for(env.subjects(), user).await?;
        let dimensions = super::dimensions(env.subjects())?;
        let chains =
            ParentChainIndex::load(&repo, Arc::new(ChainSources::from_services(&services)))
                .await
                .map_err(AdminError::internal)?;
        Ok(Self {
            repo,
            user: user.clone(),
            roles: identity.roles,
            attributes,
            dimensions,
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
                dimensions: &self.dimensions,
            },
        )
        .await
        .map_err(AdminError::internal)
    }
}

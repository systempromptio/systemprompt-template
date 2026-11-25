//! [`MarketplaceFilter`] implementation for the systemprompt template.
//!
//! Resolves a user's `(roles, project)` from `users` joined to
//! `user_profile_ext` and hands the subject to core's
//! [`keep_sets`] resolver, which consults `access_control_rules` per entry
//! kind with the owning marketplace cascaded as a parent: one marketplace
//! rule covers every member that declares no rules of its own, and a member
//! that declares any rule owns its decision outright. Default policy is
//! **explicit allow**: if neither path grants access, the item is dropped
//! (see `services/access-control/rules.yaml`).

use crate::env::AdminEnv;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::extension::{Contributions, FactoryError, Resolved};
use systemprompt::governance::authz::{AccessControlRepository, SubjectSourceCell};
use systemprompt::identifiers::UserId;
use systemprompt::manifest::services::InstallServicesView;
use systemprompt::marketplace::contract::MarketplaceSlot;
use systemprompt::marketplace::{
    KeepSetsSubject, MarketplaceCandidate, MarketplaceFilter, MarketplaceFilterError, keep_sets,
};

use crate::authz::{dimensions, subject_attributes_for};
use crate::repositories::users::queries::find_user_access_profile;

#[derive(Debug)]
pub struct TemplateMarketplaceFilter {
    pool: Arc<PgPool>,
    repo: AccessControlRepository,
    subjects: SubjectSourceCell,
    services: InstallServicesView,
}

pub fn contribute_marketplace_filter(c: &mut Contributions<'_>) {
    c.slot::<MarketplaceSlot>().filter(|r: &Resolved<'_>| {
        let filter: Arc<dyn MarketplaceFilter> = Arc::new(TemplateMarketplaceFilter::new(
            r.get::<DbPool>()?.pool(),
            r.get::<SubjectSourceCell>()?,
            r.get::<InstallServicesView>()?,
        ));
        Ok::<_, FactoryError>(filter)
    });
}

impl TemplateMarketplaceFilter {
    #[must_use]
    pub fn new(
        pool: Arc<PgPool>,
        subjects: SubjectSourceCell,
        services: InstallServicesView,
    ) -> Self {
        Self {
            repo: AccessControlRepository::from_pool(Arc::clone(&pool)),
            pool,
            subjects,
            services,
        }
    }

    pub(crate) fn from_env(pool: Arc<PgPool>, env: &AdminEnv) -> Self {
        Self::new(pool, env.subjects().clone(), env.services_view().clone())
    }

    async fn user_roles(&self, user_id: &UserId) -> Result<Vec<String>, MarketplaceFilterError> {
        match find_user_access_profile(self.pool.as_ref(), user_id).await {
            Ok(Some(profile)) => Ok(profile.roles),
            Ok(None) => Err(MarketplaceFilterError::Backend(Box::new(
                systemprompt::traits::RepositoryError::not_found("user", user_id),
            ))),
            Err(e) => Err(MarketplaceFilterError::Backend(Box::new(e))),
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
        let attributes = subject_attributes_for(&self.subjects, user_id)
            .await
            .map_err(|e| MarketplaceFilterError::Backend(Box::new(e)))?;
        let dimensions =
            dimensions(&self.subjects).map_err(|e| MarketplaceFilterError::Backend(Box::new(e)))?;
        let mut keep = keep_sets(
            &self.repo,
            &candidate,
            KeepSetsSubject {
                user_id,
                roles: &roles,
                attributes: &attributes,
                dimensions: &dimensions,
            },
        )
        .await?;
        // Why: Authorization and connection readiness are independent gates. A
        // grant can never make an otherwise denied marketplace visible, and a
        // server access control admitted is still withheld until the person's
        // connection to it is ready — visibly, so a missing server on a bridge
        // is traceable to the sub-condition that failed.
        let connections = crate::services::connector_accounts::get_connections(
            &self.pool,
            &self.services.config(),
            user_id,
        )
        .await
        .map_err(|e| MarketplaceFilterError::Backend(Box::new(e)))?;
        let diagnostics = &mut candidate.diagnostics;
        keep.mcp_servers.retain(|id| {
            let Some(connection) = connections
                .connections
                .iter()
                .find(|c| c.provider == id.as_str())
            else {
                return true;
            };
            match connection.readiness() {
                Ok(()) => true,
                Err(reason) => {
                    tracing::warn!(
                        user_id = %user_id,
                        mcp_server = %id,
                        reason = reason.as_str(),
                        status = %connection.status,
                        configured = connection.configured,
                        entitled = connection.entitled,
                        verified = connection.verified_at.is_some(),
                        "marketplace filter dropped an authorized MCP server: connection not ready"
                    );
                    diagnostics.push(format!(
                        "mcp server '{id}' was admitted by access control but dropped because the \
                         connection is not ready ({reason}; status={}, configured={}, entitled={}, \
                         verified={})",
                        connection.status,
                        connection.configured,
                        connection.entitled,
                        connection.verified_at.is_some(),
                    ));
                    false
                },
            }
        });
        candidate.retain_entries(&keep);
        Ok(candidate)
    }
}

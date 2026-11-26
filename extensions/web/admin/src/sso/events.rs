//! What a directory sign-in changes beyond the session: the web member's
//! sign-in sink and role resolver, contributed to core's `SsoSlot`.
//!
//! - [`DirectorySignInSink`] replays an AD FS assertion's groups onto the
//!   directory half of group and project membership after a sign-in, and
//!   revokes what a refused person still holds when the directory no longer
//!   vouches for them (no group, or a domain off the allow-list).
//! - [`DirectoryRoleResolver`] joins the directory's roles with the roles an
//!   admin granted by hand, and revokes every credential minted while a
//!   write-tier role was held when the directory takes it away.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::oauth_issuance::sso::RoleResolver;
use systemprompt::traits::sso::{AuthEventSink, SsoError, SsoLoginRefused, SsoLoginSuccess};

use crate::repositories::users::{revocation, roles};
use crate::repositories::{groups, projects};

use super::adfs;

/// Sign-in side effects of an AD FS outcome.
#[derive(Debug)]
pub struct DirectorySignInSink {
    pool: Arc<PgPool>,
}

impl DirectorySignInSink {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthEventSink for DirectorySignInSink {
    async fn on_login_success(&self, event: &SsoLoginSuccess<'_>) {
        if event.provider_id != adfs::PROVIDER_ID {
            return;
        }
        let groups = &event.identity.groups;
        if let Err(e) =
            groups::members::replace_directory_group_memberships(&self.pool, event.user_id, groups)
                .await
        {
            tracing::warn!(error = %e, user_id = %event.user_id, "Failed to persist ADFS group memberships");
        }
        if let Err(e) = projects::members::replace_directory_project_memberships(
            &self.pool,
            event.user_id,
            groups,
        )
        .await
        {
            tracing::warn!(error = %e, user_id = %event.user_id, "Failed to persist ADFS project memberships");
        }
    }

    // Why: a refused sign-in is only half of a deprovision. The person still
    // holds a cookie session and a linked bridge from the last time the
    // directory vouched for them, and neither expires on its own.
    async fn on_login_refused(&self, event: &SsoLoginRefused<'_>) {
        if event.provider_id != adfs::PROVIDER_ID
            || !matches!(event.reason, "no_group" | "forbidden")
        {
            return;
        }
        let Some(email) = event.email else {
            return;
        };
        match revocation::revoke_access_by_email(&self.pool, email).await {
            Ok(Some((user_id, counts))) if !counts.is_empty() => tracing::warn!(
                %user_id,
                reason = event.reason,
                sessions = counts.sessions,
                api_keys = counts.api_keys,
                device_certs = counts.device_certs,
                exchange_codes = counts.exchange_codes,
                "Revoked credentials after ADFS refused the sign-in"
            ),
            Ok(_) => {},
            Err(e) => {
                tracing::error!(error = %e, reason = event.reason, "Failed to revoke credentials after a refused ADFS sign-in");
            },
        }
    }
}

/// The directory half of a user's roles, joined with the manual half.
#[derive(Debug)]
pub struct DirectoryRoleResolver {
    pool: Arc<PgPool>,
}

impl DirectoryRoleResolver {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleResolver for DirectoryRoleResolver {
    async fn resolve(
        &self,
        user_id: &UserId,
        directory_roles: &[String],
    ) -> Result<Vec<String>, SsoError> {
        let projected = roles::project_directory_roles(&self.pool, user_id, directory_roles)
            .await
            .map_err(SsoError::internal)?;
        if projected.lost_manage {
            match revocation::revoke_user_access(&self.pool, user_id).await {
                Ok(counts) => tracing::warn!(
                    %user_id,
                    sessions = counts.sessions,
                    api_keys = counts.api_keys,
                    "Revoked credentials after the directory removed a write-tier role"
                ),
                Err(e) => return Err(SsoError::internal(e)),
            }
        }
        Ok(projected.roles)
    }
}

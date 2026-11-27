//! The resolved caller identity carried through admin request handling.

use serde::Serialize;
use systemprompt::identifiers::{Email, SessionId, UserId};
use systemprompt::models::auth::UserType;

use super::role::{ROLES_CONSOLE, ROLES_MANAGE, ROLES_PLATFORM, Role, has_any, parse_roles};
use super::users::CookieSession;
use crate::repositories::users::queries::UserAccessProfile;

/// The signed-in console principal, built only by the verifying middleware.
///
/// Its fields are crate-private and `UserContext::signed_in` takes a
/// [`CookieSession`] that only core-verified credentials produce.
#[derive(Debug, Clone, Serialize)]
pub struct UserContext {
    pub(crate) user_id: UserId,
    pub(crate) username: String,
    pub(crate) email: Email,
    pub(crate) roles: Vec<String>,
    // Why: the groups and projects the caller belongs to, resolved once per
    // request. Listings narrow to these for a caller who may not see the
    // whole estate, so they are part of identity rather than something each
    // handler re-reads.
    pub(crate) group_ids: Vec<String>,
    pub(crate) project_ids: Vec<String>,
    // Why: `is_admin` is the write tier — `admin` or `platform_admin` — and
    // still guards every privileged mutation. `is_console` is the wider "may
    // see the admin dashboard" test, which `project_manager` also passes.
    pub(crate) is_admin: bool,
    pub(crate) is_console: bool,
    pub(crate) is_platform_admin: bool,
    pub(crate) is_developer: bool,
    pub(crate) email_verified: bool,
    pub(crate) session_id: Option<SessionId>,
}

/// Why a verified credential is not a signed-in console principal.
///
/// A verified token proves only that core minted it. The anonymous session
/// core hands every first-time visitor verifies too, and its account row
/// carries the `anonymous` role, which is outside the console's closed role
/// set. Every console gate reads [`UserContext`], and a `UserContext` exists
/// only for a caller that passes `UserContext::signed_in`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotSignedIn {
    #[error("the credential is an anonymous session")]
    AnonymousSession,
    #[error("the credential's account does not exist")]
    NoAccount,
    #[error("the account holds no console role")]
    NoConsoleRole,
}

impl NotSignedIn {
    pub const fn refuse_token(user_type: UserType) -> Result<(), Self> {
        match user_type {
            UserType::Anon => Err(Self::AnonymousSession),
            UserType::Admin
            | UserType::User
            | UserType::A2a
            | UserType::Mcp
            | UserType::Service
            | UserType::Unknown => Ok(()),
        }
    }
}

impl UserContext {
    pub(crate) fn signed_in(
        session: CookieSession,
        account: Option<UserAccessProfile>,
    ) -> Result<Self, NotSignedIn> {
        let account = account.ok_or(NotSignedIn::NoAccount)?;
        if parse_roles(&account.roles).is_empty()
            || account
                .roles
                .iter()
                .any(|r| r.eq_ignore_ascii_case("anonymous"))
        {
            return Err(NotSignedIn::NoConsoleRole);
        }
        let roles = account.roles;
        Ok(Self {
            user_id: session.user_id,
            username: session.username,
            email: session.email,
            is_admin: roles_grant_manage(&roles),
            is_console: roles_grant_console(&roles),
            is_platform_admin: roles_grant_platform(&roles),
            is_developer: roles_grant_developer(&roles),
            roles,
            group_ids: account.group_ids,
            project_ids: account.project_ids,
            email_verified: false,
            session_id: session.session_id,
        })
    }
}

// Why: whether a role set reaches the admin dashboard. Kept beside
// `UserContext` so the middleware and every test fixture derive the flag from
// one rule instead of each restating it.
#[must_use]
pub fn roles_grant_console(roles: &[String]) -> bool {
    has_any(roles, ROLES_CONSOLE)
}

#[must_use]
pub fn roles_grant_manage(roles: &[String]) -> bool {
    has_any(roles, ROLES_MANAGE)
}

#[must_use]
pub fn roles_grant_platform(roles: &[String]) -> bool {
    has_any(roles, ROLES_PLATFORM)
}

#[must_use]
pub fn roles_grant_developer(roles: &[String]) -> bool {
    has_any(roles, &[Role::Developer])
}

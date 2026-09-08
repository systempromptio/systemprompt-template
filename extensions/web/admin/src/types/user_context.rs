//! The resolved caller identity carried through admin request handling.

use serde::Serialize;
use systemprompt::identifiers::{Email, SessionId, UserId};

use super::role::{ROLES_CONSOLE, ROLES_MANAGE, ROLES_PLATFORM, Role, has_any};

#[derive(Debug, Clone, Serialize)]
pub struct UserContext {
    pub user_id: UserId,
    pub username: String,
    pub email: Email,
    pub roles: Vec<String>,
    pub department: String,
    // Why: the groups and projects the caller belongs to, resolved once per
    // request. Listings narrow to these for a caller who may not see the
    // whole estate, so they are part of identity rather than something each
    // handler re-reads.
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
    // Why: `is_admin` is the write tier — `admin` or `platform_admin` — and
    // still guards every privileged mutation. `is_console` is the wider "may
    // see the admin dashboard" test, which `project_manager` also passes.
    pub is_admin: bool,
    pub is_console: bool,
    pub is_platform_admin: bool,
    pub is_developer: bool,
    pub email_verified: bool,
    pub session_id: Option<SessionId>,
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

//! The flat role set the admin plane authorises on.
//!
//! Roles are a closed set, not free text: the router names them, the role
//! editor grants them, and the directory projects onto them, so all three
//! agree only if one enum owns the spelling. The `ROLES_*` slices are the
//! three tiers the router layers, and [`authorize_role_change`] is the pure
//! rule the role editor applies before it writes.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    PlatformAdmin,
    Admin,
    Developer,
    User,
    ProjectManager,
    KnowledgeWorker,
    SuperAdmin,
}

impl Role {
    pub const ALL: [Self; 7] = [
        Self::PlatformAdmin,
        Self::Admin,
        Self::Developer,
        Self::User,
        Self::ProjectManager,
        Self::KnowledgeWorker,
        Self::SuperAdmin,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlatformAdmin => "platform_admin",
            Self::Admin => "admin",
            Self::Developer => "developer",
            Self::User => "user",
            Self::ProjectManager => "project_manager",
            Self::KnowledgeWorker => "knowledge_worker",
            Self::SuperAdmin => "super_admin",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PlatformAdmin => "Platform admin",
            Self::Admin => "Admin",
            Self::Developer => "Developer",
            Self::User => "User",
            Self::ProjectManager => "Project manager",
            Self::KnowledgeWorker => "Knowledge worker",
            Self::SuperAdmin => "Super Admin",
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|r| r.as_str() == s).ok_or(())
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// Why: the three tiers the admin router layers. `developer` and
// `knowledge_worker` and `super_admin` are entitlement roles — they decide
// which marketplace and MCP servers a person reaches — and sit in none of them.
// Reads open to anyone who may
// see the console; writes to the two admin roles; the platform tier to
// `platform_admin` alone, which is what holds the directory-shaped controls
// (AD mappings, granting `platform_admin` itself).
pub const ROLES_CONSOLE: &[Role] = &[Role::PlatformAdmin, Role::Admin, Role::ProjectManager];
pub const ROLES_MANAGE: &[Role] = &[Role::PlatformAdmin, Role::Admin];
pub const ROLES_PLATFORM: &[Role] = &[Role::PlatformAdmin];

#[must_use]
pub fn has_any(roles: &[String], accepted: &[Role]) -> bool {
    roles
        .iter()
        .filter_map(|r| r.parse::<Role>().ok())
        .any(|r| accepted.contains(&r))
}

// Why: unknown strings are dropped rather than rejected. `users.roles` is a
// free-text array an older release may have written into, and a stale value
// must not make a whole account unreadable.
#[must_use]
pub fn parse_roles(roles: &[String]) -> Vec<Role> {
    roles
        .iter()
        .filter_map(|r| r.parse::<Role>().ok())
        .collect()
}

/// Why a role edit was refused.
///
/// Each arm is a rule the dashboard cannot be allowed to break, and all of
/// them are decidable without touching the database, so the rule is
/// unit-testable rather than integration-testable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoleChangeRefusal {
    PlatformAdminRequired,
    LastPlatformAdmin,
    UnknownRole(String),
    DirectoryRole(String),
}

impl fmt::Display for RoleChangeRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlatformAdminRequired => f.write_str(
                "Only a platform admin may grant or revoke platform_admin or super_admin",
            ),
            Self::LastPlatformAdmin => f.write_str("The last platform admin cannot be demoted"),
            Self::UnknownRole(role) => write!(
                f,
                "Unknown role '{role}'; valid roles are platform_admin, admin, developer, user, \
                 project_manager, knowledge_worker, super_admin"
            ),
            Self::DirectoryRole(role) => write!(
                f,
                "Role {role} comes from the directory and cannot be revoked here"
            ),
        }
    }
}

// Why: The whole rule for a role edit, as a pure function of what the caller
// holds, what the target held, what is being asked for, and how many
// platform admins exist.
//
// `directory_roles` are the roles the target's AD groups project onto them:
// removing one here would be undone at their next sign-in, so it is refused
// rather than silently reverted.
pub fn authorize_role_change(
    caller_roles: &[String],
    before: &[String],
    after: &[String],
    directory_roles: &[String],
    platform_admin_count: i64,
) -> Result<(), RoleChangeRefusal> {
    for role in after {
        if role.parse::<Role>().is_err() {
            return Err(RoleChangeRefusal::UnknownRole(role.clone()));
        }
    }

    let platform = Role::PlatformAdmin.as_str();
    let was_platform_admin = before.iter().any(|r| r == platform);
    let will_be_platform_admin = after.iter().any(|r| r == platform);
    let executive = Role::SuperAdmin.as_str();
    let executive_changed =
        before.iter().any(|r| r == executive) != after.iter().any(|r| r == executive);
    if (was_platform_admin != will_be_platform_admin || executive_changed)
        && !caller_roles.iter().any(|r| r == platform)
    {
        return Err(RoleChangeRefusal::PlatformAdminRequired);
    }
    if was_platform_admin && !will_be_platform_admin && platform_admin_count <= 1 {
        return Err(RoleChangeRefusal::LastPlatformAdmin);
    }

    for role in directory_roles {
        if !after.contains(role) {
            return Err(RoleChangeRefusal::DirectoryRole(role.clone()));
        }
    }

    Ok(())
}

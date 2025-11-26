use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct BaseRole {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub permissions: HashSet<&'static str>,
}

#[derive(Debug, Copy, Clone)]
pub struct BaseRoles;

impl BaseRoles {
    pub const ANONYMOUS: &'static str = "anonymous";
    pub const ADMIN: &'static str = "admin";

    pub fn anonymous() -> BaseRole {
        BaseRole {
            name: Self::ANONYMOUS,
            display_name: "Anonymous",
            description: "Unauthenticated user with minimal permissions",
            permissions: HashSet::from(["users.read"]),
        }
    }

    pub fn admin() -> BaseRole {
        BaseRole {
            name: Self::ADMIN,
            display_name: "Administrator",
            description: "Full system administrator with all permissions",
            permissions: HashSet::new(),
        }
    }

    pub fn all() -> Vec<BaseRole> {
        vec![Self::anonymous(), Self::admin()]
    }

    pub const fn is_admin_permission_wildcard() -> bool {
        true
    }
}

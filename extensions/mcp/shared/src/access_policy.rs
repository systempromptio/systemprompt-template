//! Shared administrative role policy for the application.

#[must_use]
pub fn roles_grant_manage(roles: &[String]) -> bool {
    roles
        .iter()
        .any(|role| matches!(role.as_str(), "admin" | "platform_admin"))
}

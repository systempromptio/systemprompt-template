use anyhow::Result;
use systemprompt_models::auth::Permission;

/// Validates user permissions against required permissions
///
/// Permissions are defined in services.yaml config files:
/// ```yaml
/// agents:
///   admin-agent:
///     oauth:
///       required: true
///       scopes: [admin]
/// ```
///
/// User permissions are stored in JWT claims as Vec<Permission>
///
/// Validation: User must have AT LEAST ONE matching permission (with hierarchy support)
pub struct ScopeValidator;

impl ScopeValidator {
    /// Check if user has required permissions
    ///
    /// # Arguments
    /// * `user_permissions` - Permissions from JWT claims
    /// * `required_permissions` - Permissions from services.yaml oauth.scopes
    ///
    /// # Returns
    /// * `Ok(true)` - User has at least one matching permission (or higher via implies())
    /// * `Ok(false)` - User does not have required permissions
    pub fn validate(user_permissions: &[Permission], required_permissions: &[Permission]) -> Result<bool> {
        if required_permissions.is_empty() {
            return Ok(true);
        }

        let has_required_permission = required_permissions.iter().any(|required_perm| {
            user_permissions.iter().any(|user_perm| {
                user_perm == required_perm || user_perm.implies(required_perm)
            })
        });

        Ok(has_required_permission)
    }

    /// Check if user has required permissions, returning error if not
    pub fn require(user_permissions: &[Permission], required_permissions: &[Permission]) -> Result<()> {
        if !Self::validate(user_permissions, required_permissions)? {
            return Err(anyhow::anyhow!(
                "User does not have required permissions. Required: {:?}, User has: {:?}",
                required_permissions,
                user_permissions
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_with_matching_permission() {
        let user_permissions = vec![Permission::Admin, Permission::User];
        let required_permissions = vec![Permission::Admin];

        let result = ScopeValidator::validate(&user_permissions, &required_permissions).unwrap();
        assert!(result);
    }

    #[test]
    fn test_validate_without_matching_permission() {
        let user_permissions = vec![Permission::User];
        let required_permissions = vec![Permission::Admin];

        let result = ScopeValidator::validate(&user_permissions, &required_permissions).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_validate_empty_required_permissions() {
        let user_permissions = vec![Permission::User];
        let required_permissions = vec![];

        let result = ScopeValidator::validate(&user_permissions, &required_permissions).unwrap();
        assert!(result);
    }

    #[test]
    fn test_validate_multiple_required_permissions() {
        let user_permissions = vec![Permission::User];
        let required_permissions = vec![Permission::Admin, Permission::User];

        let result = ScopeValidator::validate(&user_permissions, &required_permissions).unwrap();
        assert!(result); // Has user
    }

    #[test]
    fn test_validate_with_hierarchy() {
        let user_permissions = vec![Permission::Admin];
        let required_permissions = vec![Permission::User];

        let result = ScopeValidator::validate(&user_permissions, &required_permissions).unwrap();
        assert!(result); // Admin implies User
    }

    #[test]
    fn test_require_with_access() {
        let user_permissions = vec![Permission::Admin];
        let required_permissions = vec![Permission::Admin];

        let result = ScopeValidator::require(&user_permissions, &required_permissions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_require_without_access() {
        let user_permissions = vec![Permission::User];
        let required_permissions = vec![Permission::Admin];

        let result = ScopeValidator::require(&user_permissions, &required_permissions);
        assert!(result.is_err());
    }
}

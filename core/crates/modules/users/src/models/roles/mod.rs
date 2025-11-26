use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: String,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_default: bool,
}

impl Role {
    pub fn get_permissions(&self) -> HashSet<String> {
        serde_json::from_str(&self.permissions).unwrap_or_else(|_| HashSet::new())
    }

    pub fn set_permissions(&mut self, permissions: HashSet<String>) {
        self.permissions = serde_json::to_string(&permissions.into_iter().collect::<Vec<_>>())
            .unwrap_or_else(|_| "[]".to_string());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    pub role: Role,
    pub permissions: HashSet<String>,
}

impl From<Role> for RoleWithPermissions {
    fn from(role: Role) -> Self {
        let permissions = role.get_permissions();
        Self { role, permissions }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRole {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub permissions: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRole {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<HashSet<String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RoleValidationError {
    #[error("Invalid permission: {0}")]
    InvalidPermission(String),

    #[error("Invalid role name: {0}")]
    InvalidRoleName(String),

    #[error("System role cannot be modified")]
    SystemRoleImmutable,

    #[error("Permission not found in system: {0}")]
    PermissionNotFound(String),

    #[error("Role name must be alphanumeric with underscores: {0}")]
    InvalidRoleFormat(String),
}

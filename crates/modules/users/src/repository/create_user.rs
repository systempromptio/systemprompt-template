use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use uuid::Uuid;

use crate::models::users::{CreateUserRequest, UserResponse};

/// Creates a new user in the database.
///
/// # Validation
///
/// * Username must be non-empty and between 3-50 characters
/// * Email must be non-empty and contain @ symbol
/// * Full name must be non-empty if provided
pub async fn create_user(
    db: &dyn DatabaseProvider,
    request: CreateUserRequest,
) -> Result<UserResponse> {
    if request.name.trim().is_empty() {
        return Err(anyhow!("Username cannot be empty"));
    }

    if request.name.len() < 3 || request.name.len() > 50 {
        return Err(anyhow!(
            "Username must be between 3 and 50 characters (got {})",
            request.name.len()
        ));
    }

    if request.email.trim().is_empty() {
        return Err(anyhow!("Email cannot be empty"));
    }

    if !request.email.contains('@') {
        return Err(anyhow!("Invalid email format: must contain '@' symbol"));
    }

    if let Some(ref full_name) = request.full_name {
        if full_name.trim().is_empty() {
            return Err(anyhow!("Full name cannot be empty if provided"));
        }
    }

    let user_uuid = Uuid::new_v4();
    let now = Utc::now();

    let query = DatabaseQueryEnum::CreateUser.get(db);
    db.fetch_one(
        &query,
        &[
            &user_uuid.to_string(),
            &request.name,
            &request.email,
            &request.full_name,
            &request.full_name,
            &request.name,
            &now,
            &now,
        ],
    )
    .await
    .and_then(|row| UserResponse::from_json_row(&row))
    .context(format!("Failed to create user '{}'", request.name))
}

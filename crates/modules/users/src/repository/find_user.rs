use anyhow::{anyhow, Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use crate::models::users::UserResponse;

/// Finds a user by their username.
///
/// # Validation
/// * Username must be non-empty
pub async fn find_by_name(db: &dyn DatabaseProvider, name: &str) -> Result<Option<UserResponse>> {
    if name.trim().is_empty() {
        return Err(anyhow!("Username cannot be empty"));
    }

    let query = DatabaseQueryEnum::GetUserByName.get(db);
    let row = db
        .fetch_optional(&query, &[&name])
        .await
        .context(format!("Failed to find user by name '{name}'"))?;

    row.map(|r| UserResponse::from_json_row(&r)).transpose()
}

/// Finds a user by their email address.
///
/// # Validation
/// * Email must be non-empty
/// * Email must contain @ symbol
pub async fn find_by_email(db: &dyn DatabaseProvider, email: &str) -> Result<Option<UserResponse>> {
    if email.trim().is_empty() {
        return Err(anyhow!("Email cannot be empty"));
    }

    if !email.contains('@') {
        return Err(anyhow!("Invalid email format: must contain '@' symbol"));
    }

    let query = DatabaseQueryEnum::GetUserByEmail.get(db);
    let row = db
        .fetch_optional(&query, &[&email])
        .await
        .context(format!("Failed to find user by email '{email}'"))?;

    row.map(|r| UserResponse::from_json_row(&r)).transpose()
}

pub async fn get_by_id(db: &dyn DatabaseProvider, user_uuid: &str) -> Result<Option<UserResponse>> {
    let query = DatabaseQueryEnum::GetUserById.get(db);
    let row = db
        .fetch_optional(&query, &[&user_uuid])
        .await
        .context("Failed to query user by UUID")?;

    row.map(|r| UserResponse::from_json_row(&r)).transpose()
}

pub async fn find_first_admin(db: &dyn DatabaseProvider) -> Result<Option<UserResponse>> {
    let query = DatabaseQueryEnum::FindFirstAdmin.get(db);
    let row = db
        .fetch_optional(&query, &[])
        .await
        .context("Failed to query first admin user")?;

    row.map(|r| UserResponse::from_json_row(&r)).transpose()
}

pub async fn find_first_user(db: &dyn DatabaseProvider) -> Result<Option<UserResponse>> {
    let query = DatabaseQueryEnum::FindFirstUser.get(db);
    let row = db
        .fetch_optional(&query, &[])
        .await
        .context("Failed to query first non-admin user")?;

    row.map(|r| UserResponse::from_json_row(&r)).transpose()
}

pub async fn find_by_role(db: &dyn DatabaseProvider, role: &str) -> Result<Option<UserResponse>> {
    let query = DatabaseQueryEnum::FindByRole.get(db);
    let row = db
        .fetch_optional(&query, &[&role])
        .await
        .context("Failed to query user by role")?;

    row.map(|r| UserResponse::from_json_row(&r)).transpose()
}

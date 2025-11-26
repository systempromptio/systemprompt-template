use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, FromDbValue};

use crate::models::users::UserResponse;

/// List users with optional search filter
///
/// # Parameters
/// - `db`: Database provider
/// - `filter`: Optional search term (searches name, email, `full_name`)
///
/// # Returns
/// Vector of `UserResponse` objects matching the criteria
pub async fn list_users(
    db: &dyn DatabaseProvider,
    filter: Option<&str>,
) -> Result<Vec<UserResponse>> {
    let query = DatabaseQueryEnum::ListUsers.get(db);
    let rows = db
        .fetch_all(&query, &[&filter])
        .await
        .context("Failed to list users")?;

    rows.into_iter()
        .map(|row| UserResponse::from_json_row(&row))
        .collect()
}

pub async fn get_user_count(db: &dyn DatabaseProvider) -> Result<i64> {
    let query = DatabaseQueryEnum::CountUsers.get(db);
    let value = db
        .fetch_scalar_value(&query, &[])
        .await
        .context("Failed to get user count")?;
    i64::from_db_value(&value).context("Failed to convert count to i64")
}

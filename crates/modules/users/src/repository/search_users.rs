use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use crate::models::users::UserResponse;

pub async fn search_users(
    db: &dyn DatabaseProvider,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<UserResponse>> {
    let q = DatabaseQueryEnum::SearchUsers.get(db);
    let rows = db
        .fetch_all(&q, &[&query, &limit, &offset])
        .await
        .context("Failed to search users")?;

    rows.into_iter()
        .map(|row| UserResponse::from_json_row(&row))
        .collect()
}

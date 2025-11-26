use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

pub async fn delete_user(db: &dyn DatabaseProvider, user_id: &str) -> Result<bool> {
    let query = DatabaseQueryEnum::DeleteUser.get(db);
    let rows_affected = db
        .execute(&query, &[&user_id])
        .await
        .context(format!("Failed to delete user '{user_id}'"))?;

    Ok(rows_affected > 0)
}

use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use super::find_user;

pub async fn assign_roles(db: &dyn DatabaseProvider, name: &str, roles: &[String]) -> Result<bool> {
    let Some(user) = find_user::find_by_name(db, name).await? else {
        return Ok(false);
    };

    let query = DatabaseQueryEnum::AssignRole.get(db);
    let rows_affected = db
        .execute(&query, &[&roles, &user.name])
        .await
        .context("Failed to assign roles")?;

    Ok(rows_affected > 0)
}

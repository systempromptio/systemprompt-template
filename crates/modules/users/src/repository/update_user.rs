use anyhow::Result;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};

use super::find_user;

pub async fn update_user(
    db: &dyn DatabaseProvider,
    name: &str,
    email: Option<&str>,
    full_name: Option<&str>,
    status: Option<&str>,
) -> Result<bool> {
    let Some(user) = find_user::find_by_name(db, name).await? else {
        return Ok(false);
    };

    if !has_fields_to_update(email, full_name, status) {
        return Ok(false);
    }

    let updated = execute_update(db, email, full_name, status, &user.uuid).await?;

    Ok(updated)
}

const fn has_fields_to_update(
    email: Option<&str>,
    full_name: Option<&str>,
    status: Option<&str>,
) -> bool {
    email.is_some() || full_name.is_some() || status.is_some()
}

async fn execute_update(
    db: &dyn DatabaseProvider,
    email: Option<&str>,
    full_name: Option<&str>,
    status: Option<&str>,
    user_uuid: &str,
) -> Result<bool> {
    let query_enum = match (email, full_name, status) {
        (Some(_), Some(_), Some(_)) => DatabaseQueryEnum::UpdateUserAllFields,
        (Some(_), Some(_), None) => DatabaseQueryEnum::UpdateUserEmailFullName,
        (Some(_), None, Some(_)) => DatabaseQueryEnum::UpdateUserEmailStatus,
        (None, Some(_), Some(_)) => DatabaseQueryEnum::UpdateUserFullNameStatus,
        (Some(_), None, None) => DatabaseQueryEnum::UpdateUserEmail,
        (None, Some(_), None) => DatabaseQueryEnum::UpdateUserFullName,
        (None, None, Some(_)) => DatabaseQueryEnum::UpdateUserStatus,
        (None, None, None) => return Ok(false),
    };

    let query = query_enum.get(db);

    let rows_affected = match (email, full_name, status) {
        (Some(e), Some(f), Some(s)) => db.execute(&query, &[&e, &f, &s, &user_uuid]).await?,
        (Some(e), Some(f), None) => db.execute(&query, &[&e, &f, &user_uuid]).await?,
        (Some(e), None, Some(s)) => db.execute(&query, &[&e, &s, &user_uuid]).await?,
        (None, Some(f), Some(s)) => db.execute(&query, &[&f, &s, &user_uuid]).await?,
        (Some(e), None, None) => db.execute(&query, &[&e, &user_uuid]).await?,
        (None, Some(f), None) => db.execute(&query, &[&f, &user_uuid]).await?,
        (None, None, Some(s)) => db.execute(&query, &[&s, &user_uuid]).await?,
        (None, None, None) => return Ok(false),
    };

    Ok(rows_affected > 0)
}

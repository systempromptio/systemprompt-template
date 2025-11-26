use anyhow::Result;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use systemprompt_models::auth::{UserRole, UserStatus};

pub async fn create_anonymous_user(db: &dyn DatabaseProvider, user_id: &str) -> Result<()> {
    let status = UserStatus::Temporary.to_string();
    let roles = vec![UserRole::Anonymous.to_string()];
    let query = DatabaseQueryEnum::CreateAnonymousUser.get(db);

    db.execute(
        &query,
        &[
            &user_id,
            &user_id,
            &format!("{user_id}@anonymous.systemprompt.io"),
            &"Anonymous User".to_string(),
            &"Anonymous".to_string(),
            &status,
            &roles.as_slice(),
        ],
    )
    .await?;

    Ok(())
}

pub async fn delete_anonymous_user(db: &dyn DatabaseProvider, user_id: &str) -> Result<u64> {
    let status = UserStatus::Temporary.to_string();
    let query = DatabaseQueryEnum::DeleteAnonymousUser.get(db);

    let deleted = db.execute(&query, &[&user_id, &status]).await?;

    Ok(deleted)
}

pub async fn is_temporary_anonymous(db: &dyn DatabaseProvider, user_id: &str) -> Result<bool> {
    let status = UserStatus::Temporary.to_string();
    let query = DatabaseQueryEnum::IsTemporaryAnonymous.get(db);

    let row = db.fetch_optional(&query, &[&user_id, &status]).await?;

    Ok(row.is_some())
}

pub async fn cleanup_old_anonymous_users(db: &dyn DatabaseProvider) -> Result<u64> {
    let status = UserStatus::Temporary.to_string();
    let query = DatabaseQueryEnum::CleanupOldAnonymousUsers.get(db);

    let deleted = db.execute(&query, &[&status]).await?;

    Ok(deleted)
}

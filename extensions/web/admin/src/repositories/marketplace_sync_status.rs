use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub async fn mark_user_dirty(pool: &PgPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"
        INSERT INTO marketplace_sync_status (user_id, dirty, last_changed_at)
        VALUES ($1, true, NOW())
        ON CONFLICT (user_id) DO UPDATE SET dirty = true, last_changed_at = NOW()
        ",
        user_id.as_str(),
    )
    .execute(pool)
    .await?;

    let persistent_repo = std::path::PathBuf::from("storage/marketplace-versions")
        .join(user_id.as_str())
        .join("repo.git");
    if persistent_repo.exists() {
        if let Err(e) = std::fs::remove_dir_all(&persistent_repo) {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to invalidate persistent marketplace repo");
        }
    }

    if let Err(e) = super::marketplace_sync::invalidate_git_cache(user_id) {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to invalidate marketplace git cache");
    }

    Ok(())
}

pub async fn get_dirty_users(pool: &PgPool, limit: i64) -> Result<Vec<UserId>, sqlx::Error> {
    let rows = sqlx::query!(
        r"
        SELECT user_id FROM marketplace_sync_status
        WHERE dirty = true
        ORDER BY last_changed_at ASC
        LIMIT $1
        FOR UPDATE SKIP LOCKED
        ",
        limit,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| UserId::new(&r.user_id)).collect())
}

pub async fn mark_user_synced(pool: &PgPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"
        UPDATE marketplace_sync_status
        SET dirty = false, last_synced_at = NOW(), sync_error = NULL
        WHERE user_id = $1
        ",
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_sync_error(
    pool: &PgPool,
    user_id: &UserId,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"
        UPDATE marketplace_sync_status
        SET sync_error = $2, dirty = false
        WHERE user_id = $1
        ",
        user_id.as_str(),
        error,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_sync_status(pool: &PgPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM marketplace_sync_status WHERE user_id = $1",
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

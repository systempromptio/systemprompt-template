use std::sync::Arc;

use sqlx::PgPool;

pub async fn mark_user_dirty(pool: &Arc<PgPool>, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO marketplace_sync_status (user_id, dirty, last_changed_at)
        VALUES ($1, true, NOW())
        ON CONFLICT (user_id) DO UPDATE SET dirty = true, last_changed_at = NOW()
        ",
    )
    .bind(user_id)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}

pub async fn get_dirty_users(pool: &Arc<PgPool>, limit: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String,)>(
        r"
        SELECT user_id FROM marketplace_sync_status
        WHERE dirty = true
        ORDER BY last_changed_at ASC
        LIMIT $1
        FOR UPDATE SKIP LOCKED
        ",
    )
    .bind(limit)
    .fetch_all(pool.as_ref())
    .await?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn mark_user_synced(pool: &Arc<PgPool>, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        UPDATE marketplace_sync_status
        SET dirty = false, last_synced_at = NOW(), sync_error = NULL
        WHERE user_id = $1
        ",
    )
    .bind(user_id)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}

pub async fn mark_sync_error(
    pool: &Arc<PgPool>,
    user_id: &str,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        UPDATE marketplace_sync_status
        SET sync_error = $2
        WHERE user_id = $1
        ",
    )
    .bind(user_id)
    .bind(error)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}


use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub async fn list_selected_org_plugins(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT org_plugin_id FROM user_selected_org_plugins WHERE user_id = $1 ORDER BY selected_at",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.org_plugin_id).collect())
}

pub async fn has_any_selections(pool: &PgPool, user_id: &UserId) -> Result<bool, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM user_selected_org_plugins WHERE user_id = $1) as \"exists!\"",
        user_id.as_str(),
    )
    .fetch_one(pool)
    .await?;
    Ok(row.exists)
}

pub async fn remove_selected_org_plugin(
    pool: &PgPool,
    user_id: &UserId,
    org_plugin_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM user_selected_org_plugins WHERE user_id = $1 AND org_plugin_id = $2",
        user_id.as_str(),
        org_plugin_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_selected_org_plugins(
    pool: &PgPool,
    user_id: &UserId,
    plugin_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM user_selected_org_plugins WHERE user_id = $1",
        user_id.as_str(),
    )
    .execute(&mut *tx)
    .await?;

    for plugin_id in plugin_ids {
        sqlx::query!(
            "INSERT INTO user_selected_org_plugins (user_id, org_plugin_id) VALUES ($1, $2)",
            user_id.as_str(),
            plugin_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

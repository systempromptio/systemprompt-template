use std::sync::Arc;

use sqlx::PgPool;

pub async fn list_selected_org_plugins(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT org_plugin_id FROM user_selected_org_plugins WHERE user_id = $1 ORDER BY selected_at",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

pub async fn has_any_selections(pool: &Arc<PgPool>, user_id: &str) -> Result<bool, sqlx::Error> {
    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS(SELECT 1 FROM user_selected_org_plugins WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(pool.as_ref())
            .await?;
    Ok(row.0)
}

pub async fn set_selected_org_plugins(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_ids: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM user_selected_org_plugins WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    for plugin_id in plugin_ids {
        sqlx::query(
            "INSERT INTO user_selected_org_plugins (user_id, org_plugin_id) VALUES ($1, $2)",
        )
        .bind(user_id)
        .bind(plugin_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

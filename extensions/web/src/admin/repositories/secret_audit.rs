use std::sync::Arc;

use sqlx::PgPool;

pub struct AuditLogRow {
    pub id: String,
    pub var_name: String,
    pub action: String,
    pub actor_id: String,
    pub ip_address: Option<String>,
    pub created_at: String,
}

pub async fn list_audit_log(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
        "SELECT id, var_name, action, actor_id, ip_address, \
         created_at::text FROM secret_audit_log \
         WHERE user_id = $1 AND plugin_id = $2 \
         ORDER BY created_at DESC LIMIT 100",
    )
    .bind(user_id)
    .bind(plugin_id)
    .fetch_all(pool.as_ref())
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, var_name, action, actor_id, ip_address, created_at)| AuditLogRow {
                id,
                var_name,
                action,
                actor_id,
                ip_address,
                created_at,
            },
        )
        .collect())
}

pub async fn insert_audit_entry(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
    action: &str,
) -> Result<(), sqlx::Error> {
    let audit_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
         VALUES ($1, $2, $3, '*', $4, $2)",
    )
    .bind(&audit_id)
    .bind(user_id)
    .bind(plugin_id)
    .bind(action)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}

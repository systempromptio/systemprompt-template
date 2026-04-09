use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug)]
pub struct AuditLogRow {
    pub id: String,
    pub var_name: String,
    pub action: String,
    pub actor_id: String,
    pub ip_address: Option<String>,
    pub created_at: String,
}

pub async fn list_audit_log(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<Vec<AuditLogRow>, sqlx::Error> {
    let rows = sqlx::query_as!(
        AuditLogRow,
        r#"SELECT id, var_name, action, actor_id, ip_address, created_at::text as "created_at!" FROM secret_audit_log WHERE user_id = $1 AND plugin_id = $2 ORDER BY created_at DESC LIMIT 100"#,
        user_id as &UserId,
        plugin_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn insert_audit_entry(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
    action: &str,
) -> Result<(), sqlx::Error> {
    let audit_id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) VALUES ($1, $2, $3, '*', $4, $2)",
        &audit_id,
        user_id as &UserId,
        plugin_id,
        action,
    )
    .execute(pool)
    .await?;
    Ok(())
}

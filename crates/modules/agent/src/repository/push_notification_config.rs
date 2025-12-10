use anyhow::Result;
use chrono::Utc;
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use crate::models::a2a::protocol::PushNotificationConfig;

#[derive(FromRow)]
struct PushNotificationConfigRow {
    url: String,
    endpoint: String,
    token: Option<String>,
    headers: Option<serde_json::Value>,
    authentication: Option<serde_json::Value>,
}

pub struct PushNotificationConfigRepository {
    pool: Arc<PgPool>,
}

impl std::fmt::Debug for PushNotificationConfigRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PushNotificationConfigRepository")
            .field("pool", &"<PgPool>")
            .finish()
    }
}

impl PushNotificationConfigRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn add_config(
        &self,
        task_id: &str,
        config: &PushNotificationConfig,
    ) -> Result<String> {
        let config_id = uuid::Uuid::new_v4().to_string();
        let headers_json = config
            .headers
            .as_ref()
            .map(serde_json::to_value)
            .transpose()?;
        let auth_json = config
            .authentication
            .as_ref()
            .map(serde_json::to_value)
            .transpose()?;
        let now = Utc::now();

        sqlx::query!(
            r#"INSERT INTO task_push_notification_configs
                (id, task_id, url, endpoint, token, headers, authentication, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            config_id,
            task_id,
            config.url,
            config.endpoint,
            config.token,
            headers_json,
            auth_json,
            now,
            now
        )
        .execute(&*self.pool)
        .await?;

        Ok(config_id)
    }

    pub async fn get_config(
        &self,
        task_id: &str,
        config_id: &str,
    ) -> Result<Option<PushNotificationConfig>> {
        let row = sqlx::query_as!(
            PushNotificationConfigRow,
            r#"SELECT
                url as "url!",
                endpoint as "endpoint!",
                token,
                headers,
                authentication
            FROM task_push_notification_configs
            WHERE task_id = $1 AND id = $2"#,
            task_id,
            config_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        row.map(|r| Self::row_to_config(&r)).transpose()
    }

    pub async fn list_configs(&self, task_id: &str) -> Result<Vec<PushNotificationConfig>> {
        let rows: Vec<PushNotificationConfigRow> = sqlx::query_as!(
            PushNotificationConfigRow,
            r#"SELECT
                url as "url!",
                endpoint as "endpoint!",
                token,
                headers,
                authentication
            FROM task_push_notification_configs
            WHERE task_id = $1"#,
            task_id
        )
        .fetch_all(&*self.pool)
        .await?;

        rows.iter()
            .map(|r| Self::row_to_config(r))
            .collect::<Result<Vec<_>>>()
    }

    pub async fn delete_config(&self, task_id: &str, config_id: &str) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM task_push_notification_configs WHERE task_id = $1 AND id = $2",
            task_id,
            config_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_all_for_task(&self, task_id: &str) -> Result<u64> {
        let result = sqlx::query!(
            "DELETE FROM task_push_notification_configs WHERE task_id = $1",
            task_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    fn row_to_config(row: &PushNotificationConfigRow) -> Result<PushNotificationConfig> {
        let headers = row
            .headers
            .as_ref()
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?;
        let authentication = row
            .authentication
            .as_ref()
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()?;

        Ok(PushNotificationConfig {
            url: row.url.clone(),
            endpoint: row.endpoint.clone(),
            token: row.token.clone(),
            headers,
            authentication,
        })
    }
}

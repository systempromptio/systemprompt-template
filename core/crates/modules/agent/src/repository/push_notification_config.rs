use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, JsonRow};

use crate::models::a2a::protocol::PushNotificationConfig;

pub struct PushNotificationConfigRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl std::fmt::Debug for PushNotificationConfigRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PushNotificationConfigRepository")
            .field("db", &"<DatabaseProvider>")
            .finish()
    }
}

impl PushNotificationConfigRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
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
            .map(|h| serde_json::to_string(h))
            .transpose()?;

        let auth_json = config
            .authentication
            .as_ref()
            .map(|a| serde_json::to_string(a))
            .transpose()?;

        let query = DatabaseQueryEnum::InsertPushNotificationConfig.get(self.db.as_ref());
        self.db
            .execute(
                &query,
                &[
                    &config_id,
                    &task_id,
                    &config.url,
                    &config.endpoint,
                    &config.token,
                    &headers_json,
                    &auth_json,
                    &Utc::now(),
                    &Utc::now(),
                ],
            )
            .await?;

        Ok(config_id)
    }

    pub async fn get_config(
        &self,
        task_id: &str,
        config_id: &str,
    ) -> Result<Option<PushNotificationConfig>> {
        let query = DatabaseQueryEnum::GetPushNotificationConfigById.get(self.db.as_ref());
        let row = self
            .db
            .fetch_optional(&query, &[&task_id, &config_id])
            .await?;

        row.map(|r| Self::row_to_config(&r)).transpose()
    }

    pub async fn list_configs(&self, task_id: &str) -> Result<Vec<PushNotificationConfig>> {
        let query = DatabaseQueryEnum::InsertPushNotificationConfig.get(self.db.as_ref());
        let rows = self.db.fetch_all(&query, &[&task_id]).await?;

        rows.iter()
            .map(|r| Self::row_to_config(r))
            .collect::<Result<Vec<_>>>()
    }

    pub async fn delete_config(&self, task_id: &str, config_id: &str) -> Result<bool> {
        let query = DatabaseQueryEnum::DeletePushNotificationConfigById.get(self.db.as_ref());
        let rows_affected = self.db.execute(&query, &[&task_id, &config_id]).await?;

        Ok(rows_affected > 0)
    }

    pub async fn delete_all_for_task(&self, task_id: &str) -> Result<u64> {
        let query = DatabaseQueryEnum::DeletePushNotificationConfig.get(self.db.as_ref());
        self.db.execute(&query, &[&task_id]).await
    }

    fn row_to_config(row: &JsonRow) -> Result<PushNotificationConfig> {
        let url = row
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing url"))?
            .to_string();

        let endpoint = row
            .get("endpoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing endpoint"))?
            .to_string();

        let token = row
            .get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let headers = row
            .get("headers")
            .and_then(|v| v.as_str())
            .map(|s| serde_json::from_str(s))
            .transpose()?;

        let authentication = row
            .get("authentication")
            .and_then(|v| v.as_str())
            .map(|s| serde_json::from_str(s))
            .transpose()?;

        Ok(PushNotificationConfig {
            url,
            endpoint,
            token,
            headers,
            authentication,
        })
    }
}

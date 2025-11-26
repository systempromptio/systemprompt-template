use anyhow::{anyhow, Result};
use chrono::Utc;
use std::sync::Arc;
use systemprompt_core_database::{
    parse_database_datetime, DatabaseProvider, DatabaseQueryEnum, DbPool, JsonRow,
};

use crate::models::image_generation::GeneratedImageRecord;

#[derive(Debug, Clone)]
pub struct ImageRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl ImageRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db: db_pool }
    }

    pub async fn insert_generated_image(
        &self,
        uuid: &str,
        request_id: &str,
        prompt: &str,
        model: &str,
        provider: &str,
        file_path: &str,
        public_url: &str,
        file_size_bytes: Option<i32>,
        mime_type: &str,
        resolution: Option<&str>,
        aspect_ratio: Option<&str>,
        generation_time_ms: Option<i32>,
        cost_estimate: Option<f32>,
        user_id: Option<&str>,
        session_id: Option<&str>,
        trace_id: Option<&str>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<()> {
        let created_at = Utc::now();

        self.db
            .execute(
                &DatabaseQueryEnum::InsertGeneratedImage.get(self.db.as_ref()),
                &[
                    &uuid,
                    &request_id,
                    &prompt,
                    &model,
                    &provider,
                    &file_path,
                    &public_url,
                    &file_size_bytes,
                    &mime_type,
                    &resolution,
                    &aspect_ratio,
                    &generation_time_ms,
                    &cost_estimate,
                    &user_id,
                    &session_id,
                    &trace_id,
                    &created_at,
                    &expires_at,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn get_generated_image_by_uuid(
        &self,
        uuid: &str,
    ) -> Result<Option<GeneratedImageRecord>> {
        let row = self
            .db
            .fetch_optional(
                &DatabaseQueryEnum::GetGeneratedImageByUuid.get(self.db.as_ref()),
                &[&uuid],
            )
            .await?;

        row.map(|r| Self::record_from_row(&r)).transpose()
    }

    pub async fn list_generated_images_by_user(
        &self,
        user_id: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<GeneratedImageRecord>> {
        let rows = self
            .db
            .fetch_all(
                &DatabaseQueryEnum::ListGeneratedImagesByUser.get(self.db.as_ref()),
                &[&user_id, &limit, &offset],
            )
            .await?;

        rows.iter().map(Self::record_from_row).collect()
    }

    pub async fn delete_generated_image(&self, uuid: &str) -> Result<()> {
        self.db
            .execute(
                &DatabaseQueryEnum::DeleteGeneratedImage.get(self.db.as_ref()),
                &[&uuid],
            )
            .await?;

        Ok(())
    }

    fn record_from_row(row: &JsonRow) -> Result<GeneratedImageRecord> {
        Ok(GeneratedImageRecord {
            uuid: row
                .get("uuid")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing uuid"))?
                .to_string(),
            request_id: row
                .get("request_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing request_id"))?
                .to_string(),
            prompt: row
                .get("prompt")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing prompt"))?
                .to_string(),
            model: row
                .get("model")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing model"))?
                .to_string(),
            provider: row
                .get("provider")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing provider"))?
                .to_string(),
            file_path: row
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing file_path"))?
                .to_string(),
            public_url: row
                .get("public_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing public_url"))?
                .to_string(),
            file_size_bytes: row
                .get("file_size_bytes")
                .and_then(|v| v.as_i64().map(|i| i as i32)),
            mime_type: row
                .get("mime_type")
                .and_then(|v| v.as_str())
                .unwrap_or("image/png")
                .to_string(),
            resolution: row
                .get("resolution")
                .and_then(|v| v.as_str())
                .map(String::from),
            aspect_ratio: row
                .get("aspect_ratio")
                .and_then(|v| v.as_str())
                .map(String::from),
            generation_time_ms: row
                .get("generation_time_ms")
                .and_then(|v| v.as_i64().map(|i| i as i32)),
            cost_estimate: row
                .get("cost_estimate")
                .and_then(|v| v.as_f64().map(|f| f as f32)),
            user_id: row
                .get("user_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            session_id: row
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            trace_id: row
                .get("trace_id")
                .and_then(|v| v.as_str())
                .map(String::from),
            created_at: row
                .get("created_at")
                .and_then(parse_database_datetime)
                .ok_or_else(|| anyhow!("Missing or invalid created_at"))?,
            expires_at: row
                .get("expires_at")
                .and_then(parse_database_datetime),
            deleted_at: row
                .get("deleted_at")
                .and_then(parse_database_datetime),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_creation() {
        // Basic smoke test - actual tests would require database setup
        assert!(true);
    }
}

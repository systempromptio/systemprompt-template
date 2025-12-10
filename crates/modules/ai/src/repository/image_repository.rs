use crate::error::RepositoryError;
use crate::models::GeneratedImage;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct ImageRepository {
    pool: Arc<PgPool>,
}

impl ImageRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn insert(
        &self,
        request_id: &str,
        user_id: Option<&str>,
        prompt: &str,
        public_url: &str,
        file_path: &str,
        provider: &str,
        model: &str,
        resolution: Option<&str>,
        generation_time_ms: Option<i32>,
        cost_estimate: Option<rust_decimal::Decimal>,
    ) -> Result<GeneratedImage, RepositoryError> {
        let uuid = Uuid::new_v4().to_string();

        sqlx::query_as!(
            GeneratedImage,
            r#"
            INSERT INTO generated_images (
                uuid, request_id, user_id, prompt, public_url, file_path, provider, model,
                resolution, generation_time_ms, cost_estimate, mime_type, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'image/png', CURRENT_TIMESTAMP)
            RETURNING id, uuid, request_id, prompt, model, provider, file_path, public_url,
                      file_size_bytes, mime_type, resolution, aspect_ratio, generation_time_ms,
                      cost_estimate, user_id, session_id, trace_id, created_at, expires_at, deleted_at
            "#,
            uuid,
            request_id,
            user_id,
            prompt,
            public_url,
            file_path,
            provider,
            model,
            resolution,
            generation_time_ms,
            cost_estimate
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<GeneratedImage>, RepositoryError> {
        sqlx::query_as!(
            GeneratedImage,
            r#"
            SELECT id, uuid, request_id, prompt, model, provider, file_path, public_url,
                   file_size_bytes, mime_type, resolution, aspect_ratio, generation_time_ms,
                   cost_estimate, user_id, session_id, trace_id, created_at, expires_at, deleted_at
            FROM generated_images
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_by_uuid(&self, uuid: &str) -> Result<Option<GeneratedImage>, RepositoryError> {
        sqlx::query_as!(
            GeneratedImage,
            r#"
            SELECT id, uuid, request_id, prompt, model, provider, file_path, public_url,
                   file_size_bytes, mime_type, resolution, aspect_ratio, generation_time_ms,
                   cost_estimate, user_id, session_id, trace_id, created_at, expires_at, deleted_at
            FROM generated_images
            WHERE uuid = $1 AND deleted_at IS NULL
            "#,
            uuid
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn list_by_user(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<GeneratedImage>, RepositoryError> {
        sqlx::query_as!(
            GeneratedImage,
            r#"
            SELECT id, uuid, request_id, prompt, model, provider, file_path, public_url,
                   file_size_bytes, mime_type, resolution, aspect_ratio, generation_time_ms,
                   cost_estimate, user_id, session_id, trace_id, created_at, expires_at, deleted_at
            FROM generated_images
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            user_id,
            limit
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn soft_delete(&self, id: i32) -> Result<(), RepositoryError> {
        sqlx::query!(
            "UPDATE generated_images SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1",
            id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}

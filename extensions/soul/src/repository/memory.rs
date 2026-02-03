use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;

use crate::identifiers::MemoryId;
use crate::models::{CreateMemoryParams, MemoryType, SoulMemory};

#[derive(Debug, Clone)]
pub struct MemoryRepository {
    pool: Arc<PgPool>,
}

impl MemoryRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, params: &CreateMemoryParams) -> Result<SoulMemory, sqlx::Error> {
        let id = MemoryId::generate();
        let now = Utc::now();
        let memory_type = params.memory_type.as_str();
        let category = params.category.as_str();
        let priority = params.priority.unwrap_or(50);
        let confidence = params.confidence.unwrap_or(1.0_f32);

        sqlx::query_as!(
            SoulMemory,
            r#"
            INSERT INTO soul_memories (
                id, memory_type, category, subject, content, context_text,
                priority, confidence, source_task_id, source_context_id,
                tags, metadata, created_at, updated_at, expires_at, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, TRUE)
            RETURNING
                id as "id: MemoryId",
                memory_type,
                category,
                subject,
                content,
                context_text,
                priority,
                confidence as "confidence: f32",
                source_task_id,
                source_context_id,
                tags,
                metadata,
                created_at,
                updated_at,
                last_accessed_at,
                access_count,
                expires_at,
                is_active
            "#,
            id.as_str(),
            memory_type,
            category,
            params.subject,
            params.content,
            params.context_text,
            priority,
            confidence,
            params.source_task_id,
            params.source_context_id,
            params.tags.as_deref(),
            params.metadata,
            now,
            now,
            params.expires_at
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<SoulMemory>, sqlx::Error> {
        sqlx::query_as!(
            SoulMemory,
            r#"
            SELECT
                id,
                memory_type,
                category,
                subject,
                content,
                context_text,
                priority,
                confidence as "confidence: f32",
                source_task_id,
                source_context_id,
                tags,
                metadata,
                created_at,
                updated_at,
                last_accessed_at,
                access_count,
                expires_at,
                is_active
            FROM soul_memories
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn get_context(
        &self,
        memory_types: Option<&[MemoryType]>,
        subject: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SoulMemory>, sqlx::Error> {
        let type_filter: Option<Vec<String>> =
            memory_types.map(|types| types.iter().map(|t| t.as_str().to_string()).collect());

        sqlx::query_as!(
            SoulMemory,
            r#"
            SELECT
                id,
                memory_type,
                category,
                subject,
                content,
                context_text,
                priority,
                confidence as "confidence: f32",
                source_task_id,
                source_context_id,
                tags,
                metadata,
                created_at,
                updated_at,
                last_accessed_at,
                access_count,
                expires_at,
                is_active
            FROM soul_memories
            WHERE is_active = TRUE
              AND context_text IS NOT NULL
              AND (expires_at IS NULL OR expires_at > NOW())
              AND ($1::TEXT[] IS NULL OR memory_type = ANY($1))
              AND ($2::TEXT IS NULL OR subject ILIKE '%' || $2 || '%')
            ORDER BY
                CASE memory_type
                    WHEN 'core' THEN 1
                    WHEN 'long_term' THEN 2
                    WHEN 'short_term' THEN 3
                    WHEN 'working' THEN 4
                END,
                priority DESC
            LIMIT $3
            "#,
            type_filter.as_deref(),
            subject,
            limit
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn search(
        &self,
        query: &str,
        memory_type: Option<&str>,
        category: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SoulMemory>, sqlx::Error> {
        sqlx::query_as!(
            SoulMemory,
            r#"
            SELECT
                id,
                memory_type,
                category,
                subject,
                content,
                context_text,
                priority,
                confidence as "confidence: f32",
                source_task_id,
                source_context_id,
                tags,
                metadata,
                created_at,
                updated_at,
                last_accessed_at,
                access_count,
                expires_at,
                is_active
            FROM soul_memories
            WHERE is_active = TRUE
              AND (expires_at IS NULL OR expires_at > NOW())
              AND (
                  content ILIKE '%' || $1 || '%'
                  OR subject ILIKE '%' || $1 || '%'
                  OR $1 = ANY(tags)
              )
              AND ($2::TEXT IS NULL OR memory_type = $2)
              AND ($3::TEXT IS NULL OR category = $3)
            ORDER BY priority DESC, created_at DESC
            LIMIT $4
            "#,
            query,
            memory_type,
            category,
            limit
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn recall(&self, id: &str) -> Result<Option<SoulMemory>, sqlx::Error> {
        sqlx::query_as!(
            SoulMemory,
            r#"
            UPDATE soul_memories
            SET last_accessed_at = NOW(), access_count = access_count + 1
            WHERE id = $1 AND is_active = TRUE
            RETURNING
                id,
                memory_type,
                category,
                subject,
                content,
                context_text,
                priority,
                confidence as "confidence: f32",
                source_task_id,
                source_context_id,
                tags,
                metadata,
                created_at,
                updated_at,
                last_accessed_at,
                access_count,
                expires_at,
                is_active
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn update_content(
        &self,
        id: &str,
        content: &str,
        context_text: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE soul_memories
            SET content = $2, context_text = $3, updated_at = NOW()
            WHERE id = $1 AND is_active = TRUE
            "#,
            id,
            content,
            context_text
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn forget(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE soul_memories
            SET is_active = FALSE, updated_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn cleanup_expired(&self) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE soul_memories
            SET is_active = FALSE, updated_at = NOW()
            WHERE expires_at IS NOT NULL AND expires_at <= NOW() AND is_active = TRUE
            "#
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn count_active(&self) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!" FROM soul_memories
            WHERE is_active = TRUE AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_by_type(&self, memory_type: &str) -> Result<Vec<SoulMemory>, sqlx::Error> {
        sqlx::query_as!(
            SoulMemory,
            r#"
            SELECT
                id,
                memory_type,
                category,
                subject,
                content,
                context_text,
                priority,
                confidence as "confidence: f32",
                source_task_id,
                source_context_id,
                tags,
                metadata,
                created_at,
                updated_at,
                last_accessed_at,
                access_count,
                expires_at,
                is_active
            FROM soul_memories
            WHERE memory_type = $1 AND is_active = TRUE
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY priority DESC, created_at DESC
            "#,
            memory_type
        )
        .fetch_all(&*self.pool)
        .await
    }
}

use anyhow::{Context, Result};
use chrono::Utc;
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::{ContentId, FileId, SessionId, TraceId, UserId};

use crate::models::{ContentFile, File, FileMetadata, FileRole};

#[derive(Debug, Clone)]
pub struct FileRepository {
    db: DbPool,
}

impl FileRepository {
    pub const fn new(db: DbPool) -> Self {
        Self { db }
    }

    async fn get_pool(&self) -> Result<std::sync::Arc<sqlx::PgPool>> {
        self.db
            .as_ref()
            .get_postgres_pool_arc()
            .context("Failed to get PostgreSQL pool")
    }

    pub async fn insert(
        &self,
        id: &FileId,
        file_path: &str,
        public_url: &str,
        mime_type: &str,
        file_size_bytes: Option<i64>,
        ai_content: bool,
        metadata: serde_json::Value,
        user_id: Option<&UserId>,
        session_id: Option<&SessionId>,
        trace_id: Option<&TraceId>,
    ) -> Result<FileId> {
        let pool = self.get_pool().await?;
        let id_uuid = uuid::Uuid::parse_str(id.as_str())
            .with_context(|| format!("Invalid UUID for file id: {}", id.as_str()))?;
        let now = Utc::now();

        let user_id_str = user_id.map(|u| u.as_str());
        let session_id_str = session_id.map(|s| s.as_str());
        let trace_id_str = trace_id.map(|t| t.as_str());

        sqlx::query_as!(
            File,
            r#"
            INSERT INTO files (id, file_path, public_url, mime_type, file_size_bytes, ai_content, metadata, user_id, session_id, trace_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
            ON CONFLICT (file_path) DO UPDATE SET
                public_url = EXCLUDED.public_url,
                mime_type = EXCLUDED.mime_type,
                file_size_bytes = EXCLUDED.file_size_bytes,
                ai_content = EXCLUDED.ai_content,
                metadata = EXCLUDED.metadata,
                updated_at = EXCLUDED.updated_at
            RETURNING id, file_path, public_url, mime_type, file_size_bytes, ai_content, metadata, user_id, session_id, trace_id, created_at, updated_at, deleted_at
            "#,
            id_uuid,
            file_path,
            public_url,
            mime_type,
            file_size_bytes,
            ai_content,
            metadata,
            user_id_str,
            session_id_str,
            trace_id_str,
            now
        )
        .fetch_one(pool.as_ref())
        .await
        .with_context(|| {
            format!(
                "Failed to insert file (id: {}, path: {}, url: {})",
                id.as_str(),
                file_path,
                public_url
            )
        })?;

        Ok(id.clone())
    }

    pub async fn insert_file(&self, file: &File) -> Result<FileId> {
        let file_id = FileId::new(file.id.to_string());
        let user_id = file.user_id.as_ref().map(|s| UserId::new(s.clone()));
        let session_id = file.session_id.as_ref().map(|s| SessionId::new(s.clone()));
        let trace_id = file.trace_id.as_ref().map(|s| TraceId::new(s.clone()));

        self.insert(
            &file_id,
            &file.file_path,
            &file.public_url,
            &file.mime_type,
            file.file_size_bytes,
            file.ai_content,
            file.metadata.clone(),
            user_id.as_ref(),
            session_id.as_ref(),
            trace_id.as_ref(),
        )
        .await
    }

    pub async fn get_by_id(&self, id: &FileId) -> Result<Option<File>> {
        let pool = self.get_pool().await?;
        let id_uuid = uuid::Uuid::parse_str(id.as_str()).context("Invalid UUID for file id")?;

        sqlx::query_as!(
            File,
            r#"
            SELECT id, file_path, public_url, mime_type, file_size_bytes, ai_content, metadata, user_id, session_id, trace_id, created_at, updated_at, deleted_at
            FROM files
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id_uuid
        )
        .fetch_optional(pool.as_ref())
        .await
        .context(format!("Failed to get file by id: {id}"))
    }

    pub async fn get_by_path(&self, path: &str) -> Result<Option<File>> {
        let pool = self.get_pool().await?;
        sqlx::query_as!(
            File,
            r#"
            SELECT id, file_path, public_url, mime_type, file_size_bytes, ai_content, metadata, user_id, session_id, trace_id, created_at, updated_at, deleted_at
            FROM files
            WHERE file_path = $1 AND deleted_at IS NULL
            "#,
            path
        )
        .fetch_optional(pool.as_ref())
        .await
        .context(format!("Failed to get file by path: {path}"))
    }

    pub async fn list_by_user(
        &self,
        user_id: &UserId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<File>> {
        let pool = self.get_pool().await?;
        let user_id_str = user_id.as_str();
        sqlx::query_as!(
            File,
            r#"
            SELECT id, file_path, public_url, mime_type, file_size_bytes, ai_content, metadata, user_id, session_id, trace_id, created_at, updated_at, deleted_at
            FROM files
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id_str,
            limit,
            offset
        )
        .fetch_all(pool.as_ref())
        .await
        .context(format!("Failed to list files for user: {user_id}"))
    }

    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<File>> {
        let pool = self.get_pool().await?;
        sqlx::query_as!(
            File,
            r#"
            SELECT id, file_path, public_url, mime_type, file_size_bytes, ai_content, metadata, user_id, session_id, trace_id, created_at, updated_at, deleted_at
            FROM files
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool.as_ref())
        .await
        .context("Failed to list all files")
    }

    pub async fn soft_delete(&self, id: &FileId) -> Result<()> {
        let pool = self.get_pool().await?;
        let id_uuid = uuid::Uuid::parse_str(id.as_str()).context("Invalid UUID for file id")?;
        let now = Utc::now();

        sqlx::query!(
            r#"
            UPDATE files
            SET deleted_at = $1, updated_at = $1
            WHERE id = $2
            "#,
            now,
            id_uuid
        )
        .execute(pool.as_ref())
        .await
        .context(format!("Failed to soft delete file: {id}"))?;

        Ok(())
    }

    pub async fn update_metadata(&self, id: &FileId, metadata: &FileMetadata) -> Result<()> {
        let pool = self.get_pool().await?;
        let id_uuid = uuid::Uuid::parse_str(id.as_str()).context("Invalid UUID for file id")?;
        let metadata_json = serde_json::to_value(metadata)?;
        let now = Utc::now();

        sqlx::query!(
            r#"
            UPDATE files
            SET metadata = $1, updated_at = $2
            WHERE id = $3
            "#,
            metadata_json,
            now,
            id_uuid
        )
        .execute(pool.as_ref())
        .await
        .context(format!("Failed to update metadata for file: {id}"))?;

        Ok(())
    }

    pub async fn link_to_content(
        &self,
        content_id: &ContentId,
        file_id: &FileId,
        role: FileRole,
        display_order: i32,
    ) -> Result<ContentFile> {
        let pool = self.get_pool().await?;
        let file_id_uuid =
            uuid::Uuid::parse_str(file_id.as_str()).context("Invalid UUID for file id")?;
        let now = Utc::now();
        let content_id_str = content_id.as_str();

        sqlx::query_as!(
            ContentFile,
            r#"
            INSERT INTO content_files (content_id, file_id, role, display_order, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (content_id, file_id, role) DO UPDATE
            SET display_order = $4
            RETURNING id, content_id, file_id, role, display_order, created_at
            "#,
            content_id_str,
            file_id_uuid,
            role.as_str(),
            display_order,
            now
        )
        .fetch_one(pool.as_ref())
        .await
        .context(format!(
            "Failed to link file {file_id} to content {content_id}"
        ))
    }

    pub async fn unlink_from_content(
        &self,
        content_id: &ContentId,
        file_id: &FileId,
    ) -> Result<()> {
        let pool = self.get_pool().await?;
        let file_id_uuid =
            uuid::Uuid::parse_str(file_id.as_str()).context("Invalid UUID for file id")?;
        let content_id_str = content_id.as_str();

        sqlx::query!(
            r#"
            DELETE FROM content_files
            WHERE content_id = $1 AND file_id = $2
            "#,
            content_id_str,
            file_id_uuid
        )
        .execute(pool.as_ref())
        .await
        .context(format!(
            "Failed to unlink file {file_id} from content {content_id}"
        ))?;

        Ok(())
    }

    pub async fn list_files_by_content(
        &self,
        content_id: &ContentId,
    ) -> Result<Vec<(File, ContentFile)>> {
        let pool = self.get_pool().await?;
        let content_id_str = content_id.as_str();
        let rows = sqlx::query!(
            r#"
            SELECT
                f.id, f.file_path, f.public_url, f.mime_type, f.file_size_bytes, f.ai_content,
                f.metadata, f.user_id, f.session_id, f.trace_id, f.created_at, f.updated_at, f.deleted_at,
                cf.id as cf_id, cf.content_id, cf.file_id as cf_file_id, cf.role, cf.display_order, cf.created_at as cf_created_at
            FROM files f
            INNER JOIN content_files cf ON cf.file_id = f.id
            WHERE cf.content_id = $1 AND f.deleted_at IS NULL
            ORDER BY cf.display_order ASC, cf.created_at ASC
            "#,
            content_id_str
        )
        .fetch_all(pool.as_ref())
        .await
        .context(format!("Failed to list files for content: {content_id}"))?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let file = File {
                    id: row.id,
                    file_path: row.file_path,
                    public_url: row.public_url,
                    mime_type: row.mime_type,
                    file_size_bytes: row.file_size_bytes,
                    ai_content: row.ai_content,
                    metadata: row.metadata,
                    user_id: row.user_id,
                    session_id: row.session_id,
                    trace_id: row.trace_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    deleted_at: row.deleted_at,
                };

                let content_file = ContentFile {
                    id: row.cf_id,
                    content_id: row.content_id,
                    file_id: row.cf_file_id,
                    role: row.role,
                    display_order: row.display_order,
                    created_at: row.cf_created_at,
                };

                (file, content_file)
            })
            .collect())
    }

    pub async fn get_featured_image(&self, content_id: &ContentId) -> Result<Option<File>> {
        let pool = self.get_pool().await?;
        let content_id_str = content_id.as_str();
        sqlx::query_as!(
            File,
            r#"
            SELECT f.id, f.file_path, f.public_url, f.mime_type, f.file_size_bytes, f.ai_content,
                   f.metadata, f.user_id, f.session_id, f.trace_id, f.created_at, f.updated_at, f.deleted_at
            FROM files f
            INNER JOIN content_files cf ON cf.file_id = f.id
            WHERE cf.content_id = $1
              AND cf.role = 'featured'
              AND f.deleted_at IS NULL
            LIMIT 1
            "#,
            content_id_str
        )
        .fetch_optional(pool.as_ref())
        .await
        .context(format!(
            "Failed to get featured image for content: {content_id}"
        ))
    }

    pub async fn set_featured(&self, file_id: &FileId, content_id: &ContentId) -> Result<()> {
        let pool = self.get_pool().await?;
        let file_id_uuid =
            uuid::Uuid::parse_str(file_id.as_str()).context("Invalid UUID for file id")?;
        let content_id_str = content_id.as_str();
        let mut tx = pool.begin().await?;

        sqlx::query!(
            r#"
            UPDATE content_files
            SET role = 'attachment'
            WHERE content_id = $1 AND role = 'featured'
            "#,
            content_id_str
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            UPDATE content_files
            SET role = 'featured'
            WHERE file_id = $1 AND content_id = $2
            "#,
            file_id_uuid,
            content_id_str
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_files::File;

pub struct FilesRepository {
    pool: Arc<PgPool>,
}

impl FilesRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn list_all_files(&self, limit: i64, offset: i64) -> Result<Vec<File>> {
        let rows = sqlx::query_as!(
            File,
            r#"
            SELECT
                id,
                file_path,
                public_url,
                mime_type,
                file_size_bytes,
                ai_content,
                metadata,
                user_id,
                session_id,
                trace_id,
                created_at,
                updated_at,
                deleted_at
            FROM files
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}

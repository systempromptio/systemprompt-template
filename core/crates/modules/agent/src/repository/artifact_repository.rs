use crate::models::a2a::artifact::Artifact;
use crate::models::a2a::message::Part;
use crate::models::ArtifactRow;
use anyhow::{Context, Result};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_models::a2a::ArtifactMetadata;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug, Clone)]
pub struct ArtifactRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for ArtifactRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl ArtifactRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    fn get_pg_pool(&self) -> Result<Arc<PgPool>> {
        self.db_pool
            .as_ref()
            .get_postgres_pool()
            .context("PostgreSQL pool not available")
    }

    pub async fn create_artifact(
        &self,
        task_id: &TaskId,
        context_id: &ContextId,
        artifact: &Artifact,
    ) -> Result<()> {
        if artifact.artifact_id.trim().is_empty() {
            return Err(anyhow::anyhow!("Artifact ID cannot be empty"));
        }

        if artifact.name.as_ref().map_or(true, |n| n.trim().is_empty()) {
            return Err(anyhow::anyhow!("Artifact name cannot be empty"));
        }

        let pool = self.get_pg_pool()?;
        let task_id_str = task_id.as_str();
        let context_id_str = context_id.as_str();
        let artifact_type = &artifact.metadata.artifact_type;
        let source = artifact.metadata.source.as_deref();
        let tool_name = artifact.metadata.tool_name.as_deref();
        let mcp_execution_id = artifact.metadata.mcp_execution_id.as_deref();
        let fingerprint = artifact.metadata.fingerprint.as_deref();
        let skill_id = artifact.metadata.skill_id.as_deref();
        let skill_name = artifact.metadata.skill_name.as_deref();

        let reduced_metadata = serde_json::json!({
            "rendering_hints": artifact.metadata.rendering_hints,
            "mcp_schema": artifact.metadata.mcp_schema,
            "is_internal": artifact.metadata.is_internal,
            "execution_index": artifact.metadata.execution_index,
            "artifact_extensions": artifact.extensions,
        });

        sqlx::query!(
            "INSERT INTO task_artifacts (task_id, context_id, artifact_id, name, description,
             artifact_type, source, tool_name, mcp_execution_id, fingerprint, skill_id,
             skill_name, metadata)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            task_id_str,
            context_id_str,
            artifact.artifact_id,
            artifact.name,
            artifact.description,
            artifact_type,
            source,
            tool_name,
            mcp_execution_id,
            fingerprint,
            skill_id,
            skill_name,
            reduced_metadata
        )
        .execute(pool.as_ref())
        .await
        .context(format!(
            "Failed to insert artifact '{}' (ID: {}) for task '{}' in context '{}'",
            artifact.name.as_deref().unwrap_or("<unnamed>"),
            artifact.artifact_id,
            task_id.as_str(),
            context_id.as_str()
        ))?;

        for (idx, part) in artifact.parts.iter().enumerate() {
            self.insert_part(context_id.as_str(), &artifact.artifact_id, idx as i32, part)
                .await?;
        }

        Ok(())
    }

    async fn insert_part(
        &self,
        context_id: &str,
        artifact_id: &str,
        sequence: i32,
        part: &Part,
    ) -> Result<()> {
        let pool = self.get_pg_pool()?;
        let part_metadata = serde_json::json!({});

        match part {
            Part::Text(text_part) => {
                sqlx::query!(
                    "INSERT INTO artifact_parts (artifact_id, context_id, part_kind,
                     sequence_number, text_content, metadata)
                     VALUES ($1, $2, 'text', $3, $4, $5)",
                    artifact_id,
                    context_id,
                    sequence,
                    text_part.text,
                    part_metadata
                )
                .execute(pool.as_ref())
                .await?;
            },
            Part::File(file_part) => {
                sqlx::query!(
                    "INSERT INTO artifact_parts (artifact_id, context_id, part_kind,
                     sequence_number, file_name, file_mime_type, file_bytes, metadata)
                     VALUES ($1, $2, 'file', $3, $4, $5, $6, $7)",
                    artifact_id,
                    context_id,
                    sequence,
                    file_part.file.name,
                    file_part.file.mime_type,
                    file_part.file.bytes,
                    part_metadata
                )
                .execute(pool.as_ref())
                .await?;
            },
            Part::Data(data_part) => {
                let data_json = serde_json::to_value(&data_part.data)?;
                sqlx::query!(
                    "INSERT INTO artifact_parts (artifact_id, context_id, part_kind,
                     sequence_number, data_content, metadata)
                     VALUES ($1, $2, 'data', $3, $4, $5)",
                    artifact_id,
                    context_id,
                    sequence,
                    data_json,
                    part_metadata
                )
                .execute(pool.as_ref())
                .await?;
            },
        }

        Ok(())
    }

    pub async fn get_artifacts_by_task(&self, task_id: &str) -> Result<Vec<Artifact>> {
        let pool = self.get_pg_pool()?;

        let rows = sqlx::query_as!(
            ArtifactRow,
            r#"SELECT
                artifact_id as "artifact_id!",
                task_id as "task_id!",
                context_id,
                name,
                description,
                artifact_type as "artifact_type!",
                source,
                tool_name,
                mcp_execution_id,
                fingerprint,
                skill_id,
                skill_name,
                metadata,
                created_at as "artifact_created_at!"
            FROM task_artifacts WHERE task_id = $1 ORDER BY created_at ASC"#,
            task_id
        )
        .fetch_all(pool.as_ref())
        .await?;

        self.build_artifacts_from_rows(rows).await
    }

    pub async fn get_artifacts_by_context(&self, context_id: &str) -> Result<Vec<Artifact>> {
        let pool = self.get_pg_pool()?;

        let rows = sqlx::query_as!(
            ArtifactRow,
            r#"SELECT
                artifact_id as "artifact_id!",
                task_id as "task_id!",
                context_id,
                name,
                description,
                artifact_type as "artifact_type!",
                source,
                tool_name,
                mcp_execution_id,
                fingerprint,
                skill_id,
                skill_name,
                metadata,
                created_at as "artifact_created_at!"
            FROM task_artifacts WHERE context_id = $1 ORDER BY created_at ASC"#,
            context_id
        )
        .fetch_all(pool.as_ref())
        .await?;

        self.build_artifacts_from_rows(rows).await
    }

    pub async fn get_artifacts_by_user_id(
        &self,
        user_id: &str,
        limit: Option<i32>,
    ) -> Result<Vec<Artifact>> {
        let pool = self.get_pg_pool()?;
        let lim_i64 = limit.map(i64::from).unwrap_or(1000);

        let rows = sqlx::query_as!(
            ArtifactRow,
            r#"SELECT
                a.artifact_id as "artifact_id!",
                a.task_id as "task_id!",
                a.context_id,
                a.name,
                a.description,
                a.artifact_type as "artifact_type!",
                a.source,
                a.tool_name,
                a.mcp_execution_id,
                a.fingerprint,
                a.skill_id,
                a.skill_name,
                a.metadata,
                a.created_at as "artifact_created_at!"
            FROM task_artifacts a
            JOIN agent_tasks t ON a.task_id = t.task_id
            WHERE t.user_id = $1
            ORDER BY a.created_at DESC LIMIT $2"#,
            user_id,
            lim_i64
        )
        .fetch_all(pool.as_ref())
        .await?;

        self.build_artifacts_from_rows(rows).await
    }

    async fn build_artifacts_from_rows(&self, rows: Vec<ArtifactRow>) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        for row in rows {
            let metadata_value = row.metadata.unwrap_or_else(|| serde_json::json!({}));

            let metadata = ArtifactMetadata {
                artifact_type: row.artifact_type,
                context_id: ContextId::new(row.context_id.clone().unwrap_or_default()),
                created_at: row.artifact_created_at.to_rfc3339(),
                task_id: TaskId::new(row.task_id.clone()),
                rendering_hints: metadata_value.get("rendering_hints").cloned(),
                source: row.source,
                mcp_execution_id: row.mcp_execution_id,
                mcp_schema: metadata_value.get("mcp_schema").cloned(),
                is_internal: metadata_value.get("is_internal").and_then(|v| v.as_bool()),
                fingerprint: row.fingerprint,
                tool_name: row.tool_name,
                execution_index: metadata_value
                    .get("execution_index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize),
                skill_id: row.skill_id,
                skill_name: row.skill_name,
            };

            let extensions = metadata_value
                .get("artifact_extensions")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_else(|| {
                    vec![serde_json::json!(
                        "https://systemprompt.io/extensions/artifact-rendering/v1"
                    )]
                });

            let parts = self
                .get_artifact_parts(&row.artifact_id, row.context_id.as_deref().unwrap_or(""))
                .await?;

            artifacts.push(Artifact {
                artifact_id: row.artifact_id,
                name: row.name,
                description: row.description,
                parts,
                extensions,
                metadata,
            });
        }

        Ok(artifacts)
    }

    async fn get_artifact_parts(&self, artifact_id: &str, context_id: &str) -> Result<Vec<Part>> {
        let pool = self.get_pg_pool()?;

        let rows = sqlx::query!(
            r#"SELECT id, artifact_id, context_id, part_kind, sequence_number, text_content,
                    file_name, file_mime_type, file_uri, file_bytes, data_content, metadata
             FROM artifact_parts WHERE artifact_id = $1 AND context_id = $2 ORDER BY
             sequence_number ASC"#,
            artifact_id,
            context_id
        )
        .fetch_all(pool.as_ref())
        .await?;

        let mut parts = Vec::new();

        for row in rows {
            let part = match row.part_kind.as_str() {
                "text" => {
                    let text = row.text_content.unwrap_or_default();
                    Part::Text(crate::models::a2a::message::TextPart { text })
                },
                "file" => {
                    let bytes = row.file_bytes.or(row.file_uri).unwrap_or_default();

                    let file = crate::models::a2a::message::FileWithBytes {
                        name: row.file_name,
                        mime_type: row.file_mime_type,
                        bytes,
                    };

                    Part::File(crate::models::a2a::message::FilePart { file })
                },
                "data" => {
                    let data_value = row.data_content.ok_or_else(|| {
                        RepositoryError::InvalidData(
                            "Missing data_content for data part".to_string(),
                        )
                    })?;

                    let data = data_value
                        .as_object()
                        .ok_or_else(|| {
                            RepositoryError::InvalidData(
                                "data_content must be a JSON object".to_string(),
                            )
                        })?
                        .clone();

                    Part::Data(crate::models::a2a::message::DataPart { data })
                },
                _ => continue,
            };

            parts.push(part);
        }

        Ok(parts)
    }

    pub async fn get_artifact_by_id(&self, artifact_id: &str) -> Result<Option<Artifact>> {
        let pool = self.get_pg_pool()?;

        let rows = sqlx::query_as!(
            ArtifactRow,
            r#"SELECT
                artifact_id as "artifact_id!",
                task_id as "task_id!",
                context_id,
                name,
                description,
                artifact_type as "artifact_type!",
                source,
                tool_name,
                mcp_execution_id,
                fingerprint,
                skill_id,
                skill_name,
                metadata,
                created_at as "artifact_created_at!"
            FROM task_artifacts WHERE artifact_id = $1"#,
            artifact_id
        )
        .fetch_all(pool.as_ref())
        .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let artifacts = self.build_artifacts_from_rows(rows).await?;
        Ok(artifacts.into_iter().next())
    }
}

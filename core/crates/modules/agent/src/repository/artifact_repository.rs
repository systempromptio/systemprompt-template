use crate::models::a2a::artifact::Artifact;
use crate::models::a2a::message::Part;
use crate::utils::parsing::{optional_string, required_string_artifact};
use anyhow::{Context, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};
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
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    /// Creates a new artifact in the database with its associated parts.
    ///
    /// # Arguments
    ///
    /// * `task_id` - ID of the task this artifact belongs to
    /// * `context_id` - ID of the context this artifact belongs to
    /// * `artifact` - Artifact data to store
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Artifact and all parts successfully created
    /// * `Err` - Validation or database operation failed
    ///
    /// # Validation
    ///
    /// * Artifact ID must be non-empty
    /// * Artifact name must be non-empty
    /// * Task ID must be valid
    /// * Context ID must be valid
    ///
    /// # Note
    ///
    /// This method creates both the artifact record and all its parts in separate operations.
    /// If part insertion fails, the artifact record remains in the database.
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
        let metadata_json = serde_json::to_string(&reduced_metadata)
            .context("Failed to serialize artifact metadata")?;

        let query = DatabaseQueryEnum::InsertArtifact.get(self.db_pool.as_ref());
        self.db_pool
            .as_ref()
            .execute(
                &query,
                &[
                    &task_id.as_str(),
                    &context_id.as_str(),
                    &artifact.artifact_id,
                    &artifact.name,
                    &artifact.description,
                    &artifact_type,
                    &source,
                    &tool_name,
                    &mcp_execution_id,
                    &fingerprint,
                    &skill_id,
                    &skill_name,
                    &metadata_json,
                ],
            )
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
        let part_metadata = serde_json::to_string(&serde_json::json!({}))?;
        let none_str: Option<&str> = None;

        let query = DatabaseQueryEnum::InsertArtifactPart.get(self.db_pool.as_ref());
        match part {
            Part::Text(text_part) => {
                self.db_pool
                    .as_ref()
                    .execute(
                        &query,
                        &[
                            &artifact_id,
                            &context_id,
                            &"text",
                            &sequence,
                            &text_part.text.as_str(),
                            &none_str,
                            &none_str,
                            &none_str,
                            &none_str,
                            &none_str,
                            &part_metadata.as_str(),
                        ],
                    )
                    .await?;
            },
            Part::File(file_part) => {
                self.db_pool
                    .as_ref()
                    .execute(
                        &query,
                        &[
                            &artifact_id,
                            &context_id,
                            &"file",
                            &sequence,
                            &none_str,
                            &file_part.file.name.as_deref(),
                            &file_part.file.mime_type.as_deref(),
                            &none_str,
                            &Some(file_part.file.bytes.as_str()),
                            &none_str,
                            &part_metadata.as_str(),
                        ],
                    )
                    .await?;
            },
            Part::Data(data_part) => {
                let data_json = serde_json::to_string(&data_part.data)?;

                self.db_pool
                    .as_ref()
                    .execute(
                        &query,
                        &[
                            &artifact_id,
                            &context_id,
                            &"data",
                            &sequence,
                            &none_str,
                            &none_str,
                            &none_str,
                            &none_str,
                            &none_str,
                            &data_json.as_str(),
                            &part_metadata.as_str(),
                        ],
                    )
                    .await?;
            },
        }

        Ok(())
    }

    pub async fn get_artifacts_by_task(&self, task_id: &str) -> Result<Vec<Artifact>> {
        let query = DatabaseQueryEnum::GetArtifactsByTask.get(self.db_pool.as_ref());
        let rows = self.db_pool.as_ref().fetch_all(&query, &[&task_id]).await?;

        self.build_artifacts_from_rows(rows).await
    }

    pub async fn get_artifacts_by_context(&self, context_id: &str) -> Result<Vec<Artifact>> {
        let query = DatabaseQueryEnum::GetArtifactsByContext.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&context_id])
            .await?;

        self.build_artifacts_from_rows(rows).await
    }

    pub async fn get_artifacts_by_user_id(
        &self,
        user_id: &str,
        limit: Option<i32>,
    ) -> Result<Vec<Artifact>> {
        let rows = if let Some(lim) = limit {
            let query = DatabaseQueryEnum::GetArtifactsByUserLimited.get(self.db_pool.as_ref());
            self.db_pool
                .as_ref()
                .fetch_all(&query, &[&user_id, &lim])
                .await?
        } else {
            let query = DatabaseQueryEnum::GetArtifactsByUser.get(self.db_pool.as_ref());
            self.db_pool.as_ref().fetch_all(&query, &[&user_id]).await?
        };

        self.build_artifacts_from_rows(rows).await
    }

    async fn build_artifacts_from_rows(
        &self,
        rows: Vec<systemprompt_core_database::JsonRow>,
    ) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        for row in rows {
            let artifact_id = required_string_artifact(&row, "artifact_id")
                .map_err(|e| anyhow::anyhow!("Failed to get artifact_id: {}", e))?;
            let task_id = required_string_artifact(&row, "task_id")
                .map_err(|e| anyhow::anyhow!("Failed to get task_id: {}", e))?;
            let name = optional_string(&row, "name");
            let description = optional_string(&row, "description");

            let artifact_type = required_string_artifact(&row, "artifact_type")
                .map_err(|e| anyhow::anyhow!("Failed to get artifact_type: {}", e))?;
            let context_id = required_string_artifact(&row, "context_id")
                .map_err(|e| anyhow::anyhow!("Failed to get context_id: {}", e))?;
            let created_at = required_string_artifact(&row, "artifact_created_at")
                .map_err(|e| anyhow::anyhow!("Failed to get artifact_created_at: {}", e))?;
            let source = optional_string(&row, "source");
            let tool_name = optional_string(&row, "tool_name");
            let mcp_execution_id = optional_string(&row, "mcp_execution_id");
            let fingerprint = optional_string(&row, "fingerprint");
            let skill_id = optional_string(&row, "skill_id");
            let skill_name = optional_string(&row, "skill_name");

            let metadata_json =
                optional_string(&row, "metadata").unwrap_or_else(|| "{}".to_string());
            let metadata_value: serde_json::Value = serde_json::from_str(&metadata_json)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize metadata JSON: {}", e))?;

            let metadata = ArtifactMetadata {
                artifact_type,
                context_id: ContextId::new(context_id.clone()),
                created_at,
                task_id: TaskId::new(task_id.clone()),
                rendering_hints: metadata_value.get("rendering_hints").cloned(),
                source,
                mcp_execution_id,
                mcp_schema: metadata_value.get("mcp_schema").cloned(),
                is_internal: metadata_value.get("is_internal").and_then(|v| v.as_bool()),
                fingerprint,
                tool_name,
                execution_index: metadata_value
                    .get("execution_index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize),
                render_behavior: metadata_value
                    .get("render_behavior")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "both".to_string()),
                skill_id,
                skill_name,
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

            let parts = self.get_artifact_parts(&artifact_id, &context_id).await?;

            artifacts.push(Artifact {
                artifact_id,
                name,
                description,
                parts,
                extensions,
                metadata,
            });
        }

        Ok(artifacts)
    }

    async fn get_artifact_parts(&self, artifact_id: &str, context_id: &str) -> Result<Vec<Part>> {
        let query = DatabaseQueryEnum::GetArtifactParts.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&artifact_id, &context_id])
            .await?;

        let mut parts = Vec::new();

        for row in rows {
            let part_kind = required_string_artifact(&row, "part_kind")
                .map_err(|e| anyhow::anyhow!("Failed to get part_kind: {}", e))?;

            let part = match part_kind.as_str() {
                "text" => {
                    let text = optional_string(&row, "text_content").unwrap_or_default();
                    Part::Text(crate::models::a2a::message::TextPart { text })
                },
                "file" => {
                    let name = optional_string(&row, "file_name");
                    let mime_type = optional_string(&row, "file_mime_type");
                    let bytes = optional_string(&row, "file_bytes")
                        .or_else(|| optional_string(&row, "file_uri"))
                        .unwrap_or_default();

                    let file = crate::models::a2a::message::FileWithBytes {
                        name,
                        mime_type,
                        bytes,
                    };

                    Part::File(crate::models::a2a::message::FilePart { file })
                },
                "data" => {
                    let data_json = optional_string(&row, "data_content").ok_or_else(|| {
                        RepositoryError::InvalidData(
                            "Missing data_content for data part".to_string(),
                        )
                    })?;

                    let data_value: serde_json::Value =
                        serde_json::from_str(&data_json).map_err(|e| {
                            RepositoryError::InvalidData(format!(
                                "Invalid JSON in data_content: {}",
                                e
                            ))
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
        let query = DatabaseQueryEnum::GetArtifactById.get(self.db_pool.as_ref());
        let rows = self
            .db_pool
            .as_ref()
            .fetch_all(&query, &[&artifact_id])
            .await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let artifacts = self.build_artifacts_from_rows(rows).await?;
        Ok(artifacts.into_iter().next())
    }
}

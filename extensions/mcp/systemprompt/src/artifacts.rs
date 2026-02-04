use crate::tools::SERVER_NAME;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{ContextId, TaskId};
use systemprompt::mcp::services::ui_renderer::UiRendererRegistry;
use systemprompt::mcp::{McpArtifactRecord, McpArtifactRepository};
use systemprompt::models::a2a::artifact_metadata::ArtifactMetadata;
use systemprompt::models::a2a::message::{DataPart, Part};
use systemprompt::models::a2a::Artifact;

pub fn parse_ui_uri(uri: &str) -> Option<String> {
    let prefix = format!("ui://{SERVER_NAME}/");
    if uri.starts_with(&prefix) {
        Some(uri[prefix.len()..].to_string())
    } else {
        None
    }
}

pub async fn fetch_artifact(
    db_pool: &DbPool,
    artifact_id: &str,
) -> anyhow::Result<McpArtifactRecord> {
    let repo = McpArtifactRepository::new(db_pool)?;
    repo.find_by_id_str(artifact_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Artifact not found: {artifact_id}"))
}

fn record_to_artifact(record: &McpArtifactRecord) -> Artifact {
    // Convert data to a Part
    let parts = if let Some(data_obj) = record.data.as_object() {
        vec![Part::Data(DataPart {
            data: data_obj.clone(),
        })]
    } else {
        // Wrap non-object data in a "value" key
        let mut map = serde_json::Map::new();
        map.insert("value".to_string(), record.data.clone());
        vec![Part::Data(DataPart { data: map })]
    };

    // Create metadata with required fields
    let context_id = record
        .context_id
        .as_ref()
        .map(|s| ContextId::new(s.clone()))
        .unwrap_or_else(|| ContextId::new("unknown".to_string()));

    let task_id = TaskId::new(format!("mcp-artifact-{}", record.artifact_id));

    let metadata = ArtifactMetadata::new(record.artifact_type.clone(), context_id, task_id)
        .with_mcp_execution_id(record.mcp_execution_id.clone())
        .with_source(record.server_name.clone());

    Artifact {
        id: record.artifact_id.clone().into(),
        name: record.title.clone(),
        description: None,
        parts,
        extensions: vec![],
        metadata,
    }
}

pub async fn render_artifact_ui(
    db_pool: &DbPool,
    ui_registry: &UiRendererRegistry,
    artifact_id: &str,
) -> anyhow::Result<String> {
    let record = fetch_artifact(db_pool, artifact_id).await?;
    let artifact = record_to_artifact(&record);
    let rendered = ui_registry.render(&artifact).await?;
    Ok(rendered.html)
}

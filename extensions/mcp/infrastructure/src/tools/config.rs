use rmcp::{
    model::{CallToolRequestParam, CallToolResult, Content},
    service::RequestContext,
    ErrorData as McpError, RoleServer,
};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ExecutionMetadata, ToolResponse};

use crate::sync::SyncService;

#[derive(Debug, Deserialize)]
struct ConfigInput {
    #[serde(default = "default_filter")]
    filter: String,
    #[serde(default = "default_format")]
    format: String,
}

fn default_filter() -> String {
    "all".to_string()
}

fn default_format() -> String {
    "summary".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigFilter {
    All,
    Agents,
    Mcp,
    Skills,
    Ai,
    Web,
    Content,
    Env,
    Settings,
}

impl std::str::FromStr for ConfigFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(Self::All),
            "agents" => Ok(Self::Agents),
            "mcp" => Ok(Self::Mcp),
            "skills" => Ok(Self::Skills),
            "ai" => Ok(Self::Ai),
            "web" => Ok(Self::Web),
            "content" => Ok(Self::Content),
            "env" => Ok(Self::Env),
            "settings" => Ok(Self::Settings),
            _ => anyhow::bail!("Invalid config filter: {}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigFormat {
    Json,
    Yaml,
    Summary,
}

impl std::str::FromStr for ConfigFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "yaml" => Ok(Self::Yaml),
            "summary" => Ok(Self::Summary),
            _ => anyhow::bail!("Invalid config format: {}", s),
        }
    }
}

#[must_use]
pub fn config_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "filter": {
                "type": "string",
                "enum": ["all", "agents", "mcp", "skills", "ai", "web", "content", "env", "settings"],
                "default": "all",
                "description": "Filter configuration to specific section"
            },
            "format": {
                "type": "string",
                "enum": ["json", "yaml", "summary"],
                "default": "summary",
                "description": "Output format"
            }
        }
    })
}

#[must_use]
pub fn config_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "artifact_id": { "type": "string" },
            "execution_id": { "type": "string" },
            "data": {
                "type": "object",
                "description": "Configuration data based on filter"
            },
            "metadata": { "type": "object" }
        }
    })
}

pub async fn handle_config(
    sync_service: &Arc<SyncService>,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let input: ConfigInput = serde_json::from_value(serde_json::Value::Object(args))
        .map_err(|e| McpError::invalid_params(format!("Invalid input parameters: {e}"), None))?;

    let filter: ConfigFilter = input.filter.parse().map_err(|e: anyhow::Error| {
        McpError::invalid_params(format!("Invalid filter: {e}"), None)
    })?;

    let format: ConfigFormat = input.format.parse().map_err(|e: anyhow::Error| {
        McpError::invalid_params(format!("Invalid format: {e}"), None)
    })?;

    tracing::info!(filter = %input.filter, format = %input.format, "Getting config");

    let status = sync_service
        .get_status()
        .await
        .map_err(|e| McpError::internal_error(format!("Config check failed: {e}"), None))?;

    let config_data = match filter {
        ConfigFilter::All => {
            json!({
                "tenant_id": status.tenant_id,
                "api_url": status.api_url,
                "services_path": status.services_path,
                "database_configured": status.database_configured,
                "cloud_connected": status.cloud_status.connected,
                "app_version": status.cloud_status.app_version
            })
        }
        ConfigFilter::Env => {
            json!({
                "tenant_id": status.tenant_id,
                "api_url": status.api_url,
                "services_path": status.services_path,
                "database_configured": status.database_configured
            })
        }
        ConfigFilter::Settings => {
            json!({
                "cloud_connected": status.cloud_status.connected,
                "deployment_status": status.cloud_status.deployment_status,
                "app_version": status.cloud_status.app_version
            })
        }
        _ => {
            json!({
                "filter": input.filter,
                "note": "Detailed configuration requires Config::global() integration"
            })
        }
    };

    let (summary_text, result_json) = match format {
        ConfigFormat::Json => {
            let json_str = serde_json::to_string_pretty(&config_data).map_err(|e| {
                McpError::internal_error(format!("JSON serialization failed: {e}"), None)
            })?;
            (json_str, config_data)
        }
        ConfigFormat::Yaml => {
            let yaml_str = serde_yaml::to_string(&config_data).map_err(|e| {
                McpError::internal_error(format!("YAML serialization failed: {e}"), None)
            })?;
            (yaml_str, config_data)
        }
        ConfigFormat::Summary => {
            let summary = format!(
                "Configuration ({}): tenant={}, cloud={}, db={}",
                input.filter,
                status.tenant_id,
                if status.cloud_status.connected {
                    "✓"
                } else {
                    "✗"
                },
                if status.database_configured {
                    "✓"
                } else {
                    "✗"
                }
            );
            (summary, config_data)
        }
    };

    let metadata = ExecutionMetadata::new().tool("config");
    let artifact_id = ArtifactId::new(uuid::Uuid::new_v4().to_string());
    let tool_response = ToolResponse::new(
        artifact_id.clone(),
        mcp_execution_id.clone(),
        result_json,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(summary_text)],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}

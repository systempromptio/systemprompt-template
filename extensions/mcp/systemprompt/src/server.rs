use crate::tools::{self, CliInput, CliOutput, CommandResult, SERVER_NAME};
use anyhow::Result;
use chrono::Utc;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, InitializeRequestParams,
    InitializeResult, ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, RawResourceTemplate, ReadResourceRequestParams,
    ReadResourceResult, ResourceContents, ResourceTemplate, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{ArtifactId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::models::{ExecutionStatus, ToolExecutionRequest, ToolExecutionResult};
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::services::ui_renderer::{
    registry::create_default_registry, UiRendererRegistry, MCP_APP_MIME_TYPE,
};
use systemprompt::models::ProfileBootstrap;

#[derive(Clone)]
pub struct SystempromptServer {
    #[allow(dead_code)]
    db_pool: DbPool,
    service_id: McpServerId,
    ui_registry: Arc<UiRendererRegistry>,
    tool_usage_repo: Arc<ToolUsageRepository>,
}

impl SystempromptServer {
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Result<Self, McpError> {
        let tool_usage_repo = Arc::new(
            ToolUsageRepository::new(&db_pool)
                .map_err(|e| McpError::internal_error(format!("Failed to init ToolUsageRepository: {e}"), None))?,
        );
        Ok(Self {
            db_pool,
            service_id,
            ui_registry: Arc::new(create_default_registry()),
            tool_usage_repo,
        })
    }

    pub fn with_custom_registry(
        db_pool: DbPool,
        service_id: McpServerId,
        registry: UiRendererRegistry,
    ) -> Result<Self, McpError> {
        let tool_usage_repo = Arc::new(
            ToolUsageRepository::new(&db_pool)
                .map_err(|e| McpError::internal_error(format!("Failed to init ToolUsageRepository: {e}"), None))?,
        );
        Ok(Self {
            db_pool,
            service_id,
            ui_registry: Arc::new(registry),
            tool_usage_repo,
        })
    }

    pub fn with_extended_registry<F>(db_pool: DbPool, service_id: McpServerId, extend_fn: F) -> Result<Self, McpError>
    where
        F: FnOnce(&mut UiRendererRegistry),
    {
        let mut registry = create_default_registry();
        extend_fn(&mut registry);
        let tool_usage_repo = Arc::new(
            ToolUsageRepository::new(&db_pool)
                .map_err(|e| McpError::internal_error(format!("Failed to init ToolUsageRepository: {e}"), None))?,
        );
        Ok(Self {
            db_pool,
            service_id,
            ui_registry: Arc::new(registry),
            tool_usage_repo,
        })
    }

    fn get_cli_path() -> Result<PathBuf, McpError> {
        if let Ok(path) = std::env::var("SYSTEMPROMPT_CLI_PATH") {
            return Ok(PathBuf::from(path));
        }

        let profile = ProfileBootstrap::get()
            .map_err(|e| McpError::internal_error(format!("Failed to get profile: {e}"), None))?;

        Ok(PathBuf::from(&profile.paths.bin).join("systemprompt"))
    }

    fn get_workdir() -> PathBuf {
        if let Ok(path) = std::env::var("SYSTEMPROMPT_WORKDIR") {
            return PathBuf::from(path);
        }

        ProfileBootstrap::get()
            .map(|p| PathBuf::from(&p.paths.system))
            .unwrap_or_else(|_| PathBuf::from("."))
    }

    fn execute_cli(command: &str, auth_token: &str) -> Result<CliOutput, McpError> {
        let cli_path = Self::get_cli_path()?;
        let workdir = Self::get_workdir();

        let args = shell_words::split(command).map_err(|e| {
            McpError::invalid_params(format!("Failed to parse command arguments: {e}"), None)
        })?;

        tracing::info!(
            cli_path = %cli_path.display(),
            workdir = %workdir.display(),
            args = ?args,
            "Executing CLI command"
        );

        let output = Command::new(&cli_path)
            .args(&args)
            .env("SYSTEMPROMPT_NON_INTERACTIVE", "1")
            .env("SYSTEMPROMPT_OUTPUT_FORMAT", "json")
            .env("SYSTEMPROMPT_AUTH_TOKEN", auth_token)
            .current_dir(workdir)
            .output()
            .map_err(|e| {
                McpError::internal_error(format!("Failed to execute CLI command: {e}"), None)
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        tracing::info!(
            exit_code = exit_code,
            success = success,
            stdout_len = stdout.len(),
            stderr_len = stderr.len(),
            "CLI command completed"
        );

        Ok(CliOutput {
            stdout,
            stderr,
            exit_code,
            success,
        })
    }
}

impl ServerHandler for SystempromptServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: format!("SystemPrompt ({})", self.service_id),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("SystemPrompt CLI".to_string()),
                website_url: None,
            },
            instructions: Some(
                "MANDATORY: Before ANY task, run 'core playbooks show guide_start' to load the required playbook guide. \
                Agents MUST load and follow playbooks before executing tasks. \
                All operations are playbook-driven. Do not improvise commands.".to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("SystemPrompt MCP server initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tool_list = tools::list_tools();
        Ok(ListToolsResult {
            tools: tool_list,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();
        let started_at = Utc::now();

        let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str())
            .await?
            .expect_authenticated("BUG: systemprompt requires OAuth but auth was not enforced")?;

        let request_context = auth_result.context.clone();

        let execution_request = ToolExecutionRequest {
            tool_name: tool_name.clone(),
            server_name: self.service_id.to_string(),
            input: serde_json::to_value(&request.arguments).unwrap_or_default(),
            started_at,
            context: request_context.clone(),
            request_method: Some("mcp".to_string()),
            request_source: Some("systemprompt".to_string()),
            ai_tool_call_id: None,
        };

        let mcp_execution_id = self
            .tool_usage_repo
            .start_execution(&execution_request)
            .await
            .map_err(|e| {
                tracing::error!(tool = %tool_name, error = %e, "Failed to start execution tracking");
                McpError::internal_error(format!("Failed to start execution tracking: {e}"), None)
            })?;

        let arguments = request.arguments.clone().unwrap_or_default();

        let result = match tool_name.as_str() {
            "systemprompt" => {
                let input: CliInput = serde_json::from_value(serde_json::Value::Object(arguments))
                    .map_err(|e| {
                        McpError::invalid_params(format!("Invalid input parameters: {e}"), None)
                    })?;

                let output = Self::execute_cli(&input.command, auth_result.token())?;

                if !output.success {
                    let text_content = format!(
                        "Command failed (exit code {}):\n{}",
                        output.exit_code, output.stderr
                    );
                    Ok(CallToolResult {
                        content: vec![Content::text(text_content)],
                        is_error: Some(true),
                        meta: None,
                        structured_content: None,
                    })
                } else if let Some(cmd_result) = CommandResult::from_stdout(&output.stdout) {
                    let text_content = if let Some(title) = &cmd_result.title {
                        format!(
                            "{}\n\n{}",
                            title,
                            serde_json::to_string_pretty(&cmd_result.data).unwrap_or_default()
                        )
                    } else {
                        serde_json::to_string_pretty(&cmd_result.data).unwrap_or_default()
                    };

                    let structured_content = serde_json::to_value(&cmd_result).ok();

                    Ok(CallToolResult {
                        content: vec![Content::text(text_content)],
                        is_error: Some(false),
                        meta: None,
                        structured_content,
                    })
                } else {
                    Ok(CallToolResult {
                        content: vec![Content::text(output.stdout.clone())],
                        is_error: Some(false),
                        meta: None,
                        structured_content: None,
                    })
                }
            }
            _ => Err(McpError::invalid_params(
                format!(
                    "Unknown tool: '{}'\n\n\
                    MANDATORY FIRST STEP: Run 'core playbooks show guide_start' before any task.\n\n\
                    Use 'systemprompt' tool with command 'core playbooks show guide_start' to get started.",
                    tool_name
                ),
                None,
            )),
        };

        let completed_at = Utc::now();
        let execution_result = ToolExecutionResult {
            output: result
                .as_ref()
                .ok()
                .and_then(|r| r.structured_content.clone()),
            output_schema: None,
            status: if result.is_ok() {
                ExecutionStatus::Success.as_str().to_string()
            } else {
                ExecutionStatus::Failed.as_str().to_string()
            },
            error_message: result.as_ref().err().map(|e| e.message.to_string()),
            started_at,
            completed_at,
        };

        if let Err(e) = self
            .tool_usage_repo
            .complete_execution(&mcp_execution_id, &execution_result)
            .await
        {
            tracing::error!(
                tool = %tool_name,
                mcp_execution_id = %mcp_execution_id,
                error = %e,
                "Failed to complete execution tracking"
            );
        }

        result
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        let raw_template = RawResourceTemplate {
            uri_template: format!("ui://{}/{{artifact_id}}", SERVER_NAME),
            name: "artifact-ui".to_string(),
            title: Some("Artifact UI".to_string()),
            description: Some("Interactive UI for SystemPrompt artifacts. Use with artifact IDs returned from tool calls.".to_string()),
            mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
            icons: None,
        };
        let template = ResourceTemplate {
            raw: raw_template,
            annotations: None,
        };

        Ok(ListResourceTemplatesResult {
            resource_templates: vec![template],
            next_cursor: None,
            meta: None,
        })
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let uri = &request.uri;

        let artifact_id = Self::parse_ui_uri(uri).ok_or_else(|| {
            McpError::invalid_params(
                format!(
                    "Invalid UI resource URI: {}. Expected format: ui://{}/{{artifact_id}}",
                    uri, SERVER_NAME
                ),
                None,
            )
        })?;

        let artifact = self.fetch_artifact(&artifact_id).await.map_err(|e| {
            McpError::internal_error(format!("Failed to fetch artifact: {e}"), None)
        })?;

        let html = self.ui_registry.render(&artifact).await.map_err(|e| {
            McpError::internal_error(format!("Failed to render artifact UI: {e}"), None)
        })?;

        let contents = ResourceContents::TextResourceContents {
            uri: uri.clone(),
            mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
            text: html.html,
            meta: None,
        };

        Ok(ReadResourceResult {
            contents: vec![contents],
        })
    }
}

impl SystempromptServer {
    fn parse_ui_uri(uri: &str) -> Option<String> {
        let prefix = format!("ui://{}/", SERVER_NAME);
        if uri.starts_with(&prefix) {
            Some(uri[prefix.len()..].to_string())
        } else {
            None
        }
    }

    async fn fetch_artifact(
        &self,
        artifact_id: &str,
    ) -> anyhow::Result<systemprompt::models::a2a::Artifact> {
        use systemprompt::agent::repository::content::ArtifactRepository;

        let repo = ArtifactRepository::new(self.db_pool.clone());
        let id = ArtifactId::new(artifact_id);

        repo.get_artifact_by_id(&id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Artifact not found: {}", artifact_id))
    }
}

use crate::tools::{self, CliInput, CliOutput};
use anyhow::Result;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::path::PathBuf;
use std::process::Command;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::models::ProfileBootstrap;

#[derive(Clone)]
pub struct SystempromptServer {
    #[allow(dead_code)]
    db_pool: DbPool,
    service_id: McpServerId,
}

impl SystempromptServer {
    #[must_use]
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Self {
        Self {
            db_pool,
            service_id,
        }
    }

    fn get_cli_path() -> Result<PathBuf, McpError> {
        if let Ok(path) = std::env::var("SYSTEMPROMPT_CLI_PATH") {
            return Ok(PathBuf::from(path));
        }

        let profile = ProfileBootstrap::get().map_err(|e| {
            McpError::internal_error(format!("Failed to get profile: {e}"), None)
        })?;

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
            capabilities: ServerCapabilities::builder().enable_tools().build(),
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

        let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str())
            .await?
            .expect_authenticated(
                "BUG: systemprompt requires OAuth but auth was not enforced",
            )?;

        let arguments = request.arguments.clone().unwrap_or_default();

        match tool_name.as_str() {
            "systemprompt" => {
                let input: CliInput = serde_json::from_value(serde_json::Value::Object(arguments))
                    .map_err(|e| {
                        McpError::invalid_params(format!("Invalid input parameters: {e}"), None)
                    })?;

                let output = Self::execute_cli(&input.command, auth_result.token())?;

                let structured_content = serde_json::to_value(&output).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize output: {e}"), None)
                })?;

                let text_content = if output.success {
                    output.stdout.clone()
                } else {
                    format!(
                        "Command failed (exit code {}):\n{}",
                        output.exit_code, output.stderr
                    )
                };

                Ok(CallToolResult {
                    content: vec![Content::text(text_content)],
                    is_error: Some(!output.success),
                    meta: None,
                    structured_content: Some(structured_content),
                })
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
        }
    }
}

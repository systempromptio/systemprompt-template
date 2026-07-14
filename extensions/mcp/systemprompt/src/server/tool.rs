//! Tool handler, authentication, and dispatch for the `systemprompt` MCP tool.
//!
//! The server in the parent module owns the rmcp `ServerHandler` surface; this
//! module owns what happens per tool call: RBAC enforcement against the
//! registry, access auditing, and turning CLI output into a [`CliArtifact`].

use crate::cli;
use crate::tools::CliInput;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::service::{RequestContext, RoleServer};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::{McpToolExecutor, McpToolHandler};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

pub(super) struct SystempromptToolHandler {
    pub(super) auth_token: String,
}

impl McpToolHandler for SystempromptToolHandler {
    type Input = CliInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        "systemprompt"
    }

    fn description(&self) -> &'static str {
        "Execute SystemPrompt CLI commands."
    }

    async fn handle(
        &self,
        input: Self::Input,
        _ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let output = cli::execute(&input.command, &self.auth_token).await?;

        if !output.success {
            return Err(McpError::internal_error(
                format!(
                    "Command failed (exit code {}):\n{}",
                    output.exit_code, output.stderr
                ),
                None,
            ));
        }

        let summary = output.stdout.clone();

        let artifact = match serde_json::from_str::<CliArtifact>(&output.stdout) {
            Ok(artifact) => artifact,
            Err(e) => {
                tracing::warn!(error = %e, "CLI stdout is not a CliArtifact, returning as text");
                CliArtifact::text(TextArtifact::new(&output.stdout).with_title("Command Output"))
            },
        };

        Ok((artifact, summary))
    }
}

pub(super) async fn authenticate_tool_request(
    db_pool: &DbPool,
    tool_name: &str,
    service_id: &str,
    ctx: &RequestContext<RoleServer>,
    authz_hook: &SharedAuthzHook,
) -> Result<(SysRequestContext, String), McpError> {
    let server_name = service_id;
    let rbac_result = enforce_rbac_from_registry(ctx, service_id, authz_hook).await;

    match rbac_result {
        Ok(result) => {
            match result
                .expect_authenticated("BUG: systemprompt requires OAuth but auth was not enforced")
            {
                Ok(authenticated) => {
                    record_mcp_access(
                        db_pool,
                        authenticated.context.user_id().as_ref(),
                        server_name,
                        tool_name,
                        "authenticated",
                    )
                    .await;
                    let token = authenticated.token().to_owned();
                    Ok((authenticated.context.clone(), token))
                },
                Err(e) => {
                    record_mcp_access_rejected(db_pool, server_name, tool_name, e.message.as_ref())
                        .await;
                    Err(e)
                },
            }
        },
        Err(e) => {
            record_mcp_access_rejected(db_pool, server_name, tool_name, &format!("{e}")).await;
            Err(e)
        },
    }
}

pub(super) async fn dispatch_tool(
    executor: &McpToolExecutor,
    tool_name: &str,
    request: &CallToolRequestParams,
    request_context: &SysRequestContext,
    auth_token: &str,
) -> Result<CallToolResult, McpError> {
    match tool_name {
        "systemprompt" => {
            let handler = SystempromptToolHandler {
                auth_token: auth_token.to_owned(),
            };
            executor.execute(&handler, request, request_context).await
        },
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown tool: '{tool_name}'\n\nMANDATORY FIRST STEP: Run 'core skills show \
                 systemprompt_cli' before any task.\n\nUse 'systemprompt' tool with command 'core \
                 skills show systemprompt_cli' to get started."
            ),
            None,
        )),
    }
}

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
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::{ArtifactIngest, ClientProfile, McpToolExecutor, McpToolHandler};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysRequestContext;
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::{record_mcp_access, record_mcp_access_rejected};

pub(super) struct SystempromptToolHandler<'a> {
    pub(super) auth_token: String,
    pub(super) cli: &'a cli::CliLocation,
    pub(super) ingest: &'a ArtifactIngest,
    pub(super) server_name: &'a str,
}

impl McpToolHandler for SystempromptToolHandler<'_> {
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
        ctx: &SysRequestContext,
        exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let output = cli::execute(self.cli, &input.command, &self.auth_token).await?;

        if !output.success {
            return Err(McpError::internal_error(
                format!(
                    "Command failed (exit code {}):\n{}",
                    output.exit_code, output.stderr
                ),
                None,
            ));
        }

        let artifact = match serde_json::from_str::<CliArtifact>(&output.stdout) {
            Ok(artifact) => artifact,
            Err(e) => {
                tracing::warn!(error = %e, "CLI stdout is not a CliArtifact, returning as text");
                CliArtifact::text(TextArtifact::new(&output.stdout).with_title("Command Output"))
            },
        };
        // Why: the response builder pairs the summary with the artifact's text
        // body on the wire, so echoing stdout as the summary would print the
        // whole output twice for structured clients — and an unbounded body
        // is what a host spills to a file the model cannot read back. An
        // oversize result is kept whole as its own artifact and the model gets
        // a pointer; trimming is only what happens when that store fails.
        if let Some(bytes) = crate::bounds::oversize_bytes(&artifact) {
            let store = super::overflow::OverflowStore {
                ingest: self.ingest,
                server_name: self.server_name,
                ctx,
                command: &input.command,
                exec_id,
            };
            match store.store(&artifact, bytes).await {
                Ok(stored) => {
                    let summary = format!(
                        "Ran `{}` — {bytes} bytes stored as artifact {}",
                        input.command, stored.artifact_id
                    );
                    return Ok((
                        crate::bounds::overflow_pointer(&input.command, &stored),
                        summary,
                    ));
                },
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        bytes,
                        "oversize CLI result could not be stored as an artifact; trimming"
                    );
                },
            }
        }
        let bounded = crate::bounds::bound_artifact(artifact, &input.command);
        let mut summary = format!("Ran `{}`", input.command);
        match bounded.rows {
            Some((kept, received)) => {
                summary.push_str(&format!(" — kept {kept} of {received} rows"));
            },
            None if bounded.truncated => summary.push_str(" — output truncated"),
            None => {},
        }

        Ok((bounded.artifact, summary))
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
                        authenticated.context.user_id(),
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

#[doc(hidden)]
#[derive(Debug)]
pub struct Dispatch<'a> {
    pub service_id: &'a str,
    pub db_pool: &'a DbPool,
    pub executor: &'a McpToolExecutor,
    pub request: &'a CallToolRequestParams,
    pub request_context: &'a SysRequestContext,
    pub client: &'a ClientProfile,
    pub cli: &'a cli::CliLocation,
    pub ingest: &'a Arc<ArtifactIngest>,
}

async fn run_typed<H: McpToolHandler>(
    ctx: &Dispatch<'_>,
    handler: &H,
) -> Result<CallToolResult, McpError> {
    ctx.executor
        .execute(handler, ctx.request, ctx.request_context, ctx.client)
        .await
}

// Why: the typed tools share one constructor shape; matching them here
// keeps `dispatch_tool` a flat name table.
async fn dispatch_typed(
    ctx: &Dispatch<'_>,
    tool_name: &str,
    token: &str,
) -> Option<Result<CallToolResult, McpError>> {
    let cli = ctx.cli;
    Some(match tool_name {
        "user_activity" => run_typed(ctx, &crate::typed::UserActivityHandler { cli, token }).await,
        "conversation_list" => {
            run_typed(ctx, &crate::typed::ConversationListHandler { cli, token }).await
        },
        "usage_by_user" => run_typed(ctx, &crate::typed::UsageByUserHandler { cli, token }).await,
        "request_log" => run_typed(ctx, &crate::typed::RequestLogHandler { cli, token }).await,
        "conversation_audit" => {
            run_typed(ctx, &crate::typed::ConversationAuditHandler { cli, token }).await
        },
        "users" => run_typed(ctx, &crate::typed::UsersHandler { cli, token }).await,
        _ => return None,
    })
}

// Why: Exposed (behind `#[doc(hidden)]`) so the external test workspace can
// assert the unknown-tool arm without an rmcp `Peer`, which only exists once a
// transport is serving. Not part of the public API.
#[doc(hidden)]
pub async fn dispatch_tool(
    ctx: &Dispatch<'_>,
    tool_name: &str,
    auth_token: &str,
) -> Result<CallToolResult, McpError> {
    if let Some(result) = dispatch_typed(ctx, tool_name, auth_token).await {
        return result;
    }
    match tool_name {
        "admin_report" => {
            run_typed(
                ctx,
                &crate::reports::ReportHandler {
                    cli: ctx.cli,
                    token: auth_token,
                },
            )
            .await
        },
        "systemprompt" => {
            let handler = SystempromptToolHandler {
                auth_token: auth_token.to_owned(),
                cli: ctx.cli,
                ingest: ctx.ingest,
                server_name: ctx.service_id,
            };
            run_typed(ctx, &handler).await
        },
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown tool: '{tool_name}'. This server exposes: systemprompt (CLI \
                 passthrough), admin_report, user_activity, conversation_list, usage_by_user, \
                 request_log, conversation_audit, users.\n\nMANDATORY FIRST STEP: Run 'core skills show systemprompt_cli' before \
                 any task.\n\nUse 'systemprompt' tool with command 'core skills show \
                 systemprompt_cli' to get started."
            ),
            None,
        )),
    }
}

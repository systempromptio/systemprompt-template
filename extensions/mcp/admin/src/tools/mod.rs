use rmcp::{model::{Tool, CallToolRequestParam, CallToolResult, CallToolRequestMethod, ListToolsResult}, service::RequestContext, ErrorData as McpError, RoleServer};
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::McpExecutionId;

pub mod content;
pub mod conversations;
pub mod dashboard;
pub mod jobs;
pub mod logs;
pub mod operations;
pub mod traffic;
pub mod users;

pub use content::{content_input_schema, content_output_schema, handle_content};
pub use conversations::{
    conversations_input_schema, conversations_output_schema, handle_conversations,
};
pub use dashboard::{dashboard_input_schema, dashboard_output_schema, handle_dashboard};
pub use jobs::{handle_jobs, jobs_input_schema, jobs_output_schema};
pub use logs::{handle_logs, logs_input_schema, logs_output_schema};
pub use operations::{handle_operations, operations_input_schema, operations_output_schema};
pub use traffic::{handle_traffic, traffic_input_schema, traffic_output_schema};
pub use users::{handle_users, users_input_schema, users_output_schema};

#[must_use] pub fn register_tools() -> Vec<Tool> {
    vec![
        create_tool("dashboard", "System Dashboard",
            "Comprehensive unified dashboard with real-time activity, conversations (24h/7d/30d with trends), recent conversations table, traffic overview, daily trend charts, and agent/tool usage metrics.",
            dashboard_input_schema(), dashboard_output_schema()),
        create_tool("user", "User Management",
            "Manage users, sessions, roles, and user activity.",
            users_input_schema(), users_output_schema()),
        create_tool("traffic", "Traffic Analytics",
            "Website traffic metrics: sessions, requests, unique visitors, device breakdown, geolocation, and client analysis.",
            traffic_input_schema(), traffic_output_schema()),
        create_tool("content", "Content Analytics",
            "Content performance metrics: top articles, category performance, engagement scores, and content trends.",
            content_input_schema(), content_output_schema()),
        create_tool("conversations", "Conversation Analytics",
            "Conversation metrics and details. Call with context_id to get full message history, or without to get analytics: total conversations, messages, success rates, breakdown by agent and status.",
            conversations_input_schema(), conversations_output_schema()),
        create_tool("logs", "System Logs",
            "System logs and error analysis: recent errors, error trends, and detailed error information.",
            logs_input_schema(), logs_output_schema()),
        create_tool("jobs", "Scheduler Jobs",
            "List all scheduler jobs and execute them manually. Call without parameters to list jobs, or with execute_job to trigger a job.",
            jobs_input_schema(), jobs_output_schema()),
        create_tool("operations", "Administrative Operations",
            "Administrative operations for files and content. Actions: list_files (list all files), delete_file (delete file by UUID), delete_content (delete content by UUID).",
            operations_input_schema(), operations_output_schema()),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    ctx: RequestContext<RoleServer>,
    db_pool: &DbPool,
    app_context: &Arc<AppContext>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    match name {
        "dashboard" => handle_dashboard(db_pool, request, ctx, mcp_execution_id).await,
        "user" => handle_users(db_pool, request, ctx, mcp_execution_id).await,
        "traffic" => handle_traffic(db_pool, request, ctx, mcp_execution_id).await,
        "content" => handle_content(db_pool, request, ctx, mcp_execution_id).await,
        "conversations" => {
            handle_conversations(db_pool, request, ctx, mcp_execution_id).await
        }
        "logs" => handle_logs(db_pool, request, ctx, mcp_execution_id).await,
        "jobs" => {
            handle_jobs(
                db_pool,
                request,
                ctx,
                app_context.clone(),
                mcp_execution_id,
            )
            .await
        }
        "operations" => handle_operations(db_pool, request, ctx, mcp_execution_id).await,
        _ => {
            tracing::warn!(tool = %name, "Unknown tool");
            Err(McpError::method_not_found::<CallToolRequestMethod>())
        }
    }
}

pub fn list_tools() -> Result<ListToolsResult, McpError> {
    Ok(ListToolsResult {
        tools: register_tools(),
        next_cursor: None,
    })
}

fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
) -> Tool {
    let input_obj = input_schema.as_object().cloned().unwrap_or_default();
    let output_obj = output_schema.as_object().cloned().unwrap_or_default();

    Tool {
        name: name.to_string().into(),
        title: Some(title.to_string()),
        description: Some(description.to_string().into()),
        input_schema: Arc::new(input_obj),
        output_schema: Some(Arc::new(output_obj)),
        annotations: None,
        icons: None,
    }
}

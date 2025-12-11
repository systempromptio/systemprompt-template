use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_identifiers::McpExecutionId;

pub mod content;
pub mod conversation_details;
pub mod conversations;
pub mod dashboard;
pub mod delete_content;
pub mod files;
pub mod logs;
pub mod operations;
pub mod traffic;
pub mod users;

pub use content::{content_input_schema, content_output_schema, handle_content};
pub use conversation_details::{
    conversation_details_input_schema, conversation_details_output_schema,
    handle_conversation_details,
};
pub use conversations::{
    conversations_input_schema, conversations_output_schema, handle_conversations,
};
pub use dashboard::{dashboard_input_schema, dashboard_output_schema, handle_dashboard};
pub use delete_content::{
    delete_content_input_schema, delete_content_output_schema, handle_delete_content,
};
pub use files::{files_input_schema, files_output_schema, handle_files};
pub use logs::{handle_logs, logs_input_schema, logs_output_schema};
pub use operations::{handle_operations, operations_input_schema, operations_output_schema};
pub use traffic::{handle_traffic, traffic_input_schema, traffic_output_schema};
pub use users::{handle_users, users_input_schema, users_output_schema};

pub fn register_tools() -> Vec<Tool> {
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
            "Conversation metrics: total conversations, messages, success rates, breakdown by agent and status. Includes subject analysis when evaluation data is available.",
            conversations_input_schema(), conversations_output_schema()),
        create_tool("conversation_details", "Conversation Details",
            "Retrieve full message history for a specific conversation by context ID.",
            conversation_details_input_schema(), conversation_details_output_schema()),
        create_tool("logs", "System Logs",
            "System logs and error analysis: recent errors, error trends, and detailed error information.",
            logs_input_schema(), logs_output_schema()),
        create_tool("delete_content", "Delete Content",
            "Delete content or file by UUID. Cascade deletes all related data.",
            delete_content_input_schema(), delete_content_output_schema()),
        create_tool("files", "File Management",
            "List all files in the system with pagination. Shows file path, type, size, and AI content flag.",
            files_input_schema(), files_output_schema()),
        create_tool("operations", "Operations Management",
            "List all scheduler jobs and execute them manually. Call without parameters to list jobs, or with execute_job to trigger a job.",
            operations_input_schema(), operations_output_schema()),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    ctx: RequestContext<RoleServer>,
    logger: LogService,
    db_pool: &DbPool,
    app_context: &Arc<AppContext>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    match name {
        "dashboard" => handle_dashboard(db_pool, request, ctx, logger, mcp_execution_id).await,
        "user" => handle_users(db_pool, request, ctx, logger, mcp_execution_id).await,
        "traffic" => handle_traffic(db_pool, request, ctx, logger, mcp_execution_id).await,
        "content" => handle_content(db_pool, request, ctx, logger, mcp_execution_id).await,
        "conversations" => {
            handle_conversations(db_pool, request, ctx, logger, mcp_execution_id).await
        }
        "conversation_details" => {
            handle_conversation_details(db_pool, request, ctx, logger, mcp_execution_id).await
        }
        "logs" => handle_logs(db_pool, request, ctx, logger, mcp_execution_id).await,
        "delete_content" => {
            handle_delete_content(db_pool, request, ctx, logger, mcp_execution_id).await
        }
        "files" => handle_files(db_pool, request, ctx, logger, mcp_execution_id).await,
        "operations" => {
            handle_operations(
                db_pool,
                request,
                ctx,
                logger,
                app_context.clone(),
                mcp_execution_id,
            )
            .await
        }
        _ => {
            logger
                .warn("admin_tools", &format!("Unknown tool: {}", name))
                .await
                .ok();
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

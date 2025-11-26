use crate::prompts::AdminPrompts;
use anyhow::Result;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;

pub mod content;
pub mod conversations;
pub mod dashboard;
pub mod delete_content;
pub mod logs;
pub mod traffic;
pub mod users;

pub use content::{content_input_schema, content_output_schema, handle_content};
pub use conversations::{
    conversations_input_schema, conversations_output_schema, handle_conversations,
};
pub use dashboard::{dashboard_input_schema, dashboard_output_schema, handle_dashboard};
pub use delete_content::{
    delete_content_input_schema, delete_content_output_schema, handle_delete_content,
};
pub use logs::{handle_logs, logs_input_schema, logs_output_schema};
pub use traffic::{handle_traffic, traffic_input_schema, traffic_output_schema};
pub use users::{handle_users, users_input_schema, users_output_schema};

#[derive(Debug, Clone)]
pub struct AdminTools {
    db_pool: DbPool,
    _prompts: Arc<AdminPrompts>,
}

impl AdminTools {
    pub fn new(db_pool: DbPool, prompts: Arc<AdminPrompts>) -> Self {
        Self {
            db_pool,
            _prompts: prompts,
        }
    }

    pub async fn list_tools(&self) -> Result<ListToolsResult, McpError> {
        let tools = vec![
            self.create_tool("dashboard", "System Dashboard", "Comprehensive unified dashboard with real-time activity, conversations (24h/7d/30d with trends), recent conversations table, traffic overview, daily trend charts, and agent/tool usage metrics.", dashboard_input_schema(), dashboard_output_schema())?,
            self.create_tool("user", "User Management", "Manage users, sessions, roles, and user activity. Actions: list_users, get_user, update_user_roles, list_sessions, user_activity", users_input_schema(), users_output_schema())?,
            self.create_tool("traffic", "Traffic Analytics", "Website traffic metrics: sessions, requests, unique visitors, device breakdown, geolocation, and client analysis.", traffic_input_schema(), traffic_output_schema())?,
            self.create_tool("content", "Content Analytics", "Content performance metrics: top articles, category performance, engagement scores, and content trends (new, trending, evergreen).", content_input_schema(), content_output_schema())?,
            self.create_tool("conversations", "Conversation Analytics", "Conversation metrics: total conversations, messages, success rates, breakdown by agent and status. Includes subject analysis when evaluation data is available.", conversations_input_schema(), conversations_output_schema())?,
            self.create_tool("logs", "System Logs", "System logs and error analysis: recent errors, error trends, and detailed error information.", logs_input_schema(), logs_output_schema())?,
            self.create_tool("delete_content", "Delete Content", "Delete content by UUID. Cascade deletes all related data (tags, revisions, child content).", delete_content_input_schema(), delete_content_output_schema())?,
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    fn create_tool(
        &self,
        name: &str,
        title: &str,
        description: &str,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
    ) -> Result<Tool, McpError> {
        let input_obj = input_schema
            .as_object()
            .ok_or_else(|| {
                McpError::internal_error("Invalid input schema format".to_string(), None)
            })?
            .clone();

        let output_obj = output_schema
            .as_object()
            .ok_or_else(|| {
                McpError::internal_error("Invalid output schema format".to_string(), None)
            })?
            .clone();

        Ok(Tool {
            name: name.to_string().into(),
            title: Some(title.to_string().into()),
            description: Some(description.to_string().into()),
            input_schema: Arc::new(input_obj),
            output_schema: Some(Arc::new(output_obj)),
            annotations: None,
            icons: None,
        })
    }

    pub async fn call_tool(
        &self,
        name: &str,
        request: CallToolRequestParam,
        ctx: RequestContext<RoleServer>,
        logger: systemprompt_core_logging::LogService,
    ) -> Result<CallToolResult, McpError> {
        match name {
            "dashboard" => handle_dashboard(&self.db_pool, request, ctx, logger).await,
            "user" => handle_users(&self.db_pool, request, ctx, logger).await,
            "traffic" => handle_traffic(&self.db_pool, request, ctx, logger).await,
            "content" => handle_content(&self.db_pool, request, ctx, logger).await,
            "conversations" => handle_conversations(&self.db_pool, request, ctx, logger).await,
            "logs" => handle_logs(&self.db_pool, request, ctx, logger).await,
            "delete_content" => handle_delete_content(&self.db_pool, request, ctx, logger).await,
            _ => Err(McpError::method_not_found::<CallToolRequestMethod>()),
        }
    }
}

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
        create_tool("logs", "System Logs",
            "System logs and error analysis: recent errors, error trends, and detailed error information.",
            logs_input_schema(), logs_output_schema()),
        create_tool("delete_content", "Delete Content",
            "Delete content by UUID. Cascade deletes all related data (tags, revisions, child content).",
            delete_content_input_schema(), delete_content_output_schema()),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    ctx: RequestContext<RoleServer>,
    logger: LogService,
    db_pool: &DbPool,
) -> Result<CallToolResult, McpError> {
    logger
        .info("admin_tools", &format!("Handling tool call: {}", name))
        .await
        .ok();

    match name {
        "dashboard" => handle_dashboard(db_pool, request, ctx, logger).await,
        "user" => handle_users(db_pool, request, ctx, logger).await,
        "traffic" => handle_traffic(db_pool, request, ctx, logger).await,
        "content" => handle_content(db_pool, request, ctx, logger).await,
        "conversations" => handle_conversations(db_pool, request, ctx, logger).await,
        "logs" => handle_logs(db_pool, request, ctx, logger).await,
        "delete_content" => handle_delete_content(db_pool, request, ctx, logger).await,
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
        title: Some(title.to_string().into()),
        description: Some(description.to_string().into()),
        input_schema: Arc::new(input_obj),
        output_schema: Some(Arc::new(output_obj)),
        annotations: None,
        icons: None,
    }
}

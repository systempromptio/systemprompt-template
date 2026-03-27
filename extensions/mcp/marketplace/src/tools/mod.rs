use rmcp::model::Tool;
use std::sync::Arc;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::AiService;
use systemprompt::database::DbPool;
use systemprompt::mcp::{default_tool_visibility, tool_ui_meta, McpToolExecutor, McpToolHandler};

pub mod analyze_skill;
pub mod create_agent;
pub mod create_mcp_server;
pub mod create_plugin;
pub mod create_skill;
pub mod delete_agent;
pub mod delete_mcp_server;
pub mod delete_plugin;
pub mod delete_skill;
pub mod get_plugin;
pub mod get_secrets;
pub mod list_agents;
pub mod list_mcp_servers;
pub mod list_plugins;
pub mod list_skills;
pub mod manage_secrets;
pub mod shared;
pub mod sync_skills;
mod tool_dispatch;
pub mod update_agent;
pub mod update_mcp_server;
pub mod update_plugin;
pub mod update_skill;

pub use tool_dispatch::handle_tool_call;

pub const SERVER_NAME: &str = "skill-manager";

pub struct ToolServices {
    pub db_pool: DbPool,
    pub ai_service: Arc<AiService>,
    pub skill_loader: Arc<SkillService>,
    pub executor: McpToolExecutor,
}

fn build_tool<H: McpToolHandler>(handler: &H) -> Tool {
    let input_schema = handler.input_schema();
    let input_obj = input_schema.as_object().cloned().unwrap_or_else(serde_json::Map::new);
    let output_schema = handler.output_schema();
    let output_obj = output_schema.as_object().cloned().unwrap_or_else(serde_json::Map::new);

    let mut tool = Tool::default();
    tool.name = handler.tool_name().to_string().into();
    tool.description = Some(handler.description().to_string().into());
    tool.input_schema = Arc::new(input_obj);
    tool.output_schema = Some(Arc::new(output_obj));
    tool.meta = Some(rmcp::model::Meta(tool_ui_meta(
        SERVER_NAME,
        &default_tool_visibility(),
    )));
    tool
}

#[must_use]
pub fn list_tools(services: &ToolServices) -> Vec<Tool> {
    vec![
        build_tool(&create_skill::CreateSkillHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&update_skill::UpdateSkillHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&list_skills::ListSkillsHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&analyze_skill::AnalyzeSkillHandler {
            db_pool: services.db_pool.clone(),
            ai_service: services.ai_service.clone(),
            skill_loader: services.skill_loader.clone(),
            progress: None,
        }),
        build_tool(&delete_skill::DeleteSkillHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&create_agent::CreateAgentHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&list_agents::ListAgentsHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&update_agent::UpdateAgentHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&delete_agent::DeleteAgentHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&create_mcp_server::CreateMcpServerHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&list_mcp_servers::ListMcpServersHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&update_mcp_server::UpdateMcpServerHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&delete_mcp_server::DeleteMcpServerHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&create_plugin::CreatePluginHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&get_plugin::GetPluginHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&update_plugin::UpdatePluginHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&delete_plugin::DeletePluginHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&list_plugins::ListPluginsHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&get_secrets::GetSecretsHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&manage_secrets::ManageSecretsHandler {
            db_pool: services.db_pool.clone(),
        }),
        build_tool(&sync_skills::SyncSkillsHandler {
            db_pool: services.db_pool.clone(),
            progress: None,
        }),
    ]
}

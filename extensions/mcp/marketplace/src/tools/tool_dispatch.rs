use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::ErrorData;
use systemprompt::mcp::{McpResponseBuilder, ProgressCallback};
use systemprompt::models::execution::context::RequestContext;

use super::{
    analyze_skill, create_agent, create_mcp_server, create_plugin, create_skill, delete_agent,
    delete_mcp_server, delete_plugin, delete_skill, get_plugin, get_secrets, list_agents,
    list_mcp_servers, list_plugins, list_skills, manage_secrets, sync_skills, update_agent,
    update_mcp_server, update_plugin, update_skill, ToolServices,
};

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParams,
    ctx: RequestContext,
    services: &ToolServices,
    progress: Option<ProgressCallback>,
) -> Result<CallToolResult, ErrorData> {
    handle_tool_call_inner(name, request, ctx, services, progress)
        .await
        .map_err(|e| ErrorData::internal_error(e.to_string(), None))
}

#[allow(clippy::too_many_lines)]
async fn handle_tool_call_inner(
    name: &str,
    request: CallToolRequestParams,
    ctx: RequestContext,
    services: &ToolServices,
    progress: Option<ProgressCallback>,
) -> Result<CallToolResult, systemprompt::mcp::McpError> {
    let executor = &services.executor;

    match name {
        "create_skill" => {
            let handler = create_skill::CreateSkillHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "update_skill" => {
            let handler = update_skill::UpdateSkillHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "list_skills" => {
            let handler = list_skills::ListSkillsHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "analyze_skill" => {
            let handler = analyze_skill::AnalyzeSkillHandler {
                db_pool: services.db_pool.clone(),
                ai_service: services.ai_service.clone(),
                skill_loader: services.skill_loader.clone(),
                progress,
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "delete_skill" => {
            let handler = delete_skill::DeleteSkillHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "create_agent" => {
            let handler = create_agent::CreateAgentHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "list_agents" => {
            let handler = list_agents::ListAgentsHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "update_agent" => {
            let handler = update_agent::UpdateAgentHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "delete_agent" => {
            let handler = delete_agent::DeleteAgentHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "create_mcp_server" => {
            let handler = create_mcp_server::CreateMcpServerHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "list_mcp_servers" => {
            let handler = list_mcp_servers::ListMcpServersHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "update_mcp_server" => {
            let handler = update_mcp_server::UpdateMcpServerHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "delete_mcp_server" => {
            let handler = delete_mcp_server::DeleteMcpServerHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "create_plugin" => {
            let handler = create_plugin::CreatePluginHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "get_plugin" => {
            let handler = get_plugin::GetPluginHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "update_plugin" => {
            let handler = update_plugin::UpdatePluginHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "delete_plugin" => {
            let handler = delete_plugin::DeletePluginHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "list_plugins" => {
            let handler = list_plugins::ListPluginsHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "get_secrets" => {
            let handler = get_secrets::GetSecretsHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "manage_secrets" => {
            let handler = manage_secrets::ManageSecretsHandler {
                db_pool: services.db_pool.clone(),
            };
            executor.execute(&handler, &request, &ctx).await
        }
        "sync_skills" => {
            let handler = sync_skills::SyncSkillsHandler {
                db_pool: services.db_pool.clone(),
                progress,
            };
            executor.execute(&handler, &request, &ctx).await
        }
        _ => Ok(McpResponseBuilder::<()>::build_error(format!(
            "Unknown tool: '{name}'\n\n\
            Available tools: create_skill, update_skill, delete_skill, list_skills, analyze_skill, \
            create_plugin, get_plugin, update_plugin, delete_plugin, list_plugins, \
            get_secrets, manage_secrets, sync_skills, create_agent, list_agents, update_agent, delete_agent, \
            create_mcp_server, list_mcp_servers, update_mcp_server, delete_mcp_server"
        ))),
    }
}

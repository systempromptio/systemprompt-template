use crate::repositories;
use crate::types::UserContext;
use sqlx::PgPool;

use crate::handlers::ssr::types::CheckableEntity;

pub(in crate::handlers::ssr) async fn build_association_lists(
    pool: &PgPool,
    user_ctx: &UserContext,
    plugin_with_assoc: Option<&crate::types::UserPluginWithAssociations>,
) -> (
    Vec<CheckableEntity>,
    Vec<CheckableEntity>,
    Vec<CheckableEntity>,
) {
    let user_skills = repositories::list_user_skills(pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user skills for plugin associations");
            vec![]
        });
    let user_agents = repositories::list_user_agents(pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user agents for plugin associations");
            vec![]
        });
    let user_mcp_servers = repositories::user_mcp_servers::list_user_mcp_servers(
        pool,
        &user_ctx.user_id,
    )
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list user MCP servers for plugin associations");
        vec![]
    });

    let (selected_skills, selected_agents, selected_mcp) = plugin_with_assoc.map_or_else(
        || (Vec::<&str>::new(), Vec::<&str>::new(), Vec::<&str>::new()),
        |p| {
            (
                p.skill_ids.iter().map(AsRef::as_ref).collect(),
                p.agent_ids.iter().map(AsRef::as_ref).collect(),
                p.mcp_server_ids.iter().map(AsRef::as_ref).collect(),
            )
        },
    );

    let skills_list: Vec<CheckableEntity> = user_skills
        .iter()
        .map(|s| CheckableEntity {
            value: s.id.clone(),
            name: s.name.clone(),
            checked: selected_skills.contains(&s.id.as_str()),
        })
        .collect();
    let agents_list: Vec<CheckableEntity> = user_agents
        .iter()
        .map(|a| CheckableEntity {
            value: a.id.clone(),
            name: a.name.clone(),
            checked: selected_agents.contains(&a.id.as_str()),
        })
        .collect();
    let mcp_list: Vec<CheckableEntity> = user_mcp_servers
        .iter()
        .map(|m| CheckableEntity {
            value: m.id.clone(),
            name: m.name.clone(),
            checked: selected_mcp.contains(&m.id.as_str()),
        })
        .collect();

    (skills_list, agents_list, mcp_list)
}

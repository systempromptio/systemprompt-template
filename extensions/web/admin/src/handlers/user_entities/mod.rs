mod agents;
mod batch_delete;
mod delete_account;
mod fork;
mod fork_helpers;
mod fork_lists;
mod hooks;
mod mcp_servers;
mod plugin_onboarding;
mod plugin_selections;
mod plugins;
mod secrets;
mod settings;
mod skills;

pub use agents::{
    create_user_agent_entity_handler, delete_user_agent_entity_handler, list_user_agents_handler,
    update_user_agent_entity_handler,
};
pub use batch_delete::{
    batch_delete_agents_handler, batch_delete_hooks_handler, batch_delete_mcp_servers_handler,
    batch_delete_secrets_handler, batch_delete_skills_handler,
};
pub use delete_account::delete_account_handler;
pub use fork::{fork_org_agent_handler, fork_org_plugin_handler, fork_org_skill_handler};
pub use fork_lists::{
    list_forkable_agents_handler, list_forkable_plugins_handler, list_forkable_skills_handler,
};
pub use hooks::{
    create_user_hook_handler, delete_user_hook_handler, list_user_hooks_handler,
    toggle_user_hook_handler, update_user_hook_handler,
};
pub use mcp_servers::{
    create_user_mcp_server_handler, delete_user_mcp_server_handler, list_user_mcp_servers_handler,
    set_plugin_mcp_servers_handler, update_user_mcp_server_handler,
};
pub use plugin_onboarding::select_and_fork_plugins_handler;
pub use plugin_selections::{
    list_available_plugins_handler, list_selected_plugins_handler, set_selected_plugins_handler,
};
pub use plugins::{
    create_user_plugin_handler, delete_user_plugin_handler, list_user_plugins_handler,
    set_plugin_agents_handler, set_plugin_skills_handler, update_user_plugin_handler,
};
pub use secrets::{
    create_user_secret_handler, delete_skill_secret_handler, delete_user_secret_handler,
    list_skill_secrets_handler, list_user_secrets_handler, update_user_secret_handler,
    upsert_skill_secret_handler,
};
pub use settings::update_user_settings_handler;
pub use skills::{
    create_user_skill_handler, delete_user_skill_handler, list_user_skills_handler,
    update_user_skill_handler,
};

fn read_agent_from_fs(agents_path: &std::path::Path, agent_id: &str) -> (String, String, String) {
    if !agents_path.exists() {
        return (String::new(), String::new(), String::new());
    }

    let md_path = agents_path.join(format!("{agent_id}.md"));
    let system_prompt = if md_path.exists() {
        std::fs::read_to_string(&md_path).unwrap_or_else(|e| {
            tracing::debug!(error = %e, path = %md_path.display(), "Failed to read agent markdown file");
            String::new()
        })
    } else {
        String::new()
    };

    let Ok(entries) = std::fs::read_dir(agents_path) else {
        return (agent_id.to_string(), String::new(), system_prompt);
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        if let Some(agent) = config.get("agents").and_then(|a| a.get(agent_id)) {
            let name = agent
                .get("card")
                .and_then(|c| c.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or(agent_id)
                .to_string();
            let description = agent
                .get("card")
                .and_then(|c| c.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            return (name, description, system_prompt);
        }
    }

    (agent_id.to_string(), String::new(), system_prompt)
}

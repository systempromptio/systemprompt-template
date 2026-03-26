use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::PgPool;

use super::super::handlers;

pub(crate) fn build_auth_write_routes(write_pool: Arc<PgPool>) -> Router {
    let core_routes = build_core_write_routes();
    let user_routes = build_user_write_routes();

    core_routes.merge(user_routes).with_state(write_pool)
}

fn build_core_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/plugins", post(handlers::create_plugin_handler))
        .route("/plugins/import", post(handlers::import_plugin_handler))
        .route(
            "/plugins/{plugin_id}",
            put(handlers::update_plugin_handler).delete(handlers::delete_plugin_handler),
        )
        .route(
            "/plugins/{plugin_id}/skills",
            put(handlers::update_plugin_skills_handler),
        )
        .route(
            "/plugins/{plugin_id}/env",
            put(handlers::update_plugin_env_handler),
        )
        .route("/skills", post(handlers::create_skill_handler))
        .route(
            "/skills/sync-files",
            post(handlers::sync_skill_files_handler),
        )
        .route("/skills/{skill_id}", delete(handlers::delete_skill_handler))
        .route(
            "/skills/{skill_id}/files/{*file_path}",
            put(handlers::update_skill_file_handler),
        )
        .route("/agents", post(handlers::create_agent_handler))
        .route(
            "/agents/{agent_id}",
            put(handlers::update_agent_handler).delete(handlers::delete_agent_handler),
        )
        .route("/user-agents", post(handlers::create_user_agent_handler))
        .route(
            "/user-agents/{agent_id}",
            axum::routing::delete(handlers::delete_user_agent_handler),
        )
        .route(
            "/marketplace-plugins/{plugin_id}/ratings",
            post(handlers::submit_rating_handler),
        )
}

fn build_user_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .merge(user_plugin_routes())
        .merge(user_entity_routes())
        .merge(user_fork_routes())
        .merge(user_secret_routes())
        .merge(user_mcp_routes())
        .merge(user_hook_routes())
        .merge(user_selection_routes())
        .merge(user_settings_routes())
}

fn user_settings_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/settings",
            put(handlers::user_entities::update_user_settings_handler),
        )
        .route(
            "/user/account",
            delete(handlers::user_entities::delete_account_handler),
        )
}

fn user_plugin_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/plugins",
            get(handlers::user_entities::list_user_plugins_handler)
                .post(handlers::user_entities::create_user_plugin_handler),
        )
        .route(
            "/user/plugins/{plugin_id}",
            put(handlers::user_entities::update_user_plugin_handler)
                .delete(handlers::user_entities::delete_user_plugin_handler),
        )
        .route(
            "/user/plugins/{plugin_id}/skills",
            put(handlers::user_entities::set_plugin_skills_handler),
        )
        .route(
            "/user/plugins/{plugin_id}/agents",
            put(handlers::user_entities::set_plugin_agents_handler),
        )
}

fn user_entity_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/skills/batch-delete",
            post(handlers::user_entities::batch_delete_skills_handler),
        )
        .route(
            "/user/agents/batch-delete",
            post(handlers::user_entities::batch_delete_agents_handler),
        )
        .route(
            "/user/skills",
            get(handlers::user_entities::list_user_skills_handler)
                .post(handlers::user_entities::create_user_skill_handler),
        )
        .route(
            "/user/skills/{skill_id}",
            put(handlers::user_entities::update_user_skill_handler)
                .delete(handlers::user_entities::delete_user_skill_handler),
        )
        .route(
            "/user/agents",
            get(handlers::user_entities::list_user_agents_handler)
                .post(handlers::user_entities::create_user_agent_entity_handler),
        )
        .route(
            "/user/agents/{agent_id}",
            put(handlers::user_entities::update_user_agent_entity_handler)
                .delete(handlers::user_entities::delete_user_agent_entity_handler),
        )
}

fn user_fork_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/fork/plugin",
            post(handlers::user_entities::fork_org_plugin_handler),
        )
        .route(
            "/user/fork/skill",
            post(handlers::user_entities::fork_org_skill_handler),
        )
        .route(
            "/user/fork/agent",
            post(handlers::user_entities::fork_org_agent_handler),
        )
        .route(
            "/user/forkable/plugins",
            get(handlers::user_entities::list_forkable_plugins_handler),
        )
        .route(
            "/user/forkable/skills",
            get(handlers::user_entities::list_forkable_skills_handler),
        )
        .route(
            "/user/forkable/agents",
            get(handlers::user_entities::list_forkable_agents_handler),
        )
}

fn user_secret_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/secrets/batch-delete",
            post(handlers::user_entities::batch_delete_secrets_handler),
        )
        .route(
            "/user/secrets",
            get(handlers::user_entities::list_user_secrets_handler)
                .post(handlers::user_entities::create_user_secret_handler),
        )
        .route(
            "/user/secrets/{plugin_id}/{var_name}",
            put(handlers::user_entities::update_user_secret_handler)
                .delete(handlers::user_entities::delete_user_secret_handler),
        )
        .route(
            "/user/skills/{skill_id}/secrets",
            get(handlers::user_entities::list_skill_secrets_handler)
                .put(handlers::user_entities::upsert_skill_secret_handler),
        )
        .route(
            "/user/skills/{skill_id}/secrets/{var_name}",
            axum::routing::delete(handlers::user_entities::delete_skill_secret_handler),
        )
}

fn user_mcp_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/mcp-servers/batch-delete",
            post(handlers::user_entities::batch_delete_mcp_servers_handler),
        )
        .route(
            "/user/mcp-servers",
            get(handlers::user_entities::list_user_mcp_servers_handler)
                .post(handlers::user_entities::create_user_mcp_server_handler),
        )
        .route(
            "/user/mcp-servers/{mcp_server_id}",
            put(handlers::user_entities::update_user_mcp_server_handler)
                .delete(handlers::user_entities::delete_user_mcp_server_handler),
        )
        .route(
            "/user/plugins/{plugin_id}/mcp-servers",
            put(handlers::user_entities::set_plugin_mcp_servers_handler),
        )
}

fn user_hook_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/hooks/batch-delete",
            post(handlers::user_entities::batch_delete_hooks_handler),
        )
        .route(
            "/user/hooks",
            get(handlers::user_entities::list_user_hooks_handler)
                .post(handlers::user_entities::create_user_hook_handler),
        )
        .route(
            "/user/hooks/{hook_id}",
            put(handlers::user_entities::update_user_hook_handler)
                .delete(handlers::user_entities::delete_user_hook_handler),
        )
        .route(
            "/user/hooks/{hook_id}/toggle",
            put(handlers::user_entities::toggle_user_hook_handler),
        )
}

fn user_selection_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/user/available-plugins",
            get(handlers::user_entities::list_available_plugins_handler),
        )
        .route(
            "/user/selected-plugins",
            get(handlers::user_entities::list_selected_plugins_handler)
                .put(handlers::user_entities::set_selected_plugins_handler),
        )
        .route(
            "/user/onboarding/select-and-fork",
            post(handlers::user_entities::select_and_fork_plugins_handler),
        )
}

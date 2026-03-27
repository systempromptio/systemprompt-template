use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use systemprompt::identifiers::UserId;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::{UpdatePluginEnvRequest, UserQuery};

use super::responses::PluginEnvResponse;

pub(crate) async fn list_plugin_env_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<UserQuery>,
) -> Response {
    let default_user_id = UserId::new("admin");
    let user_id = match super::extract_user_from_cookie(&headers) {
        Ok(session) => UserId::new(&session.user_id),
        Err(_) => query
            .user_id
            .as_ref()
            .map_or_else(|| default_user_id.clone(), UserId::new),
    };

    let definitions = load_plugin_variable_defs(&plugin_id).unwrap_or_else(|_| vec![]);

    let stored = match repositories::list_plugin_env_vars(&pool, &user_id, &plugin_id).await {
        Ok(vars) => vars,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list plugin env vars");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };

    let stored_names: std::collections::HashSet<String> = stored
        .iter()
        .filter(|v| !v.var_value.is_empty())
        .map(|v| v.var_name.clone())
        .collect();
    let missing_required: Vec<String> = definitions
        .iter()
        .filter_map(|def| {
            let name = def.get("name")?.as_str()?;
            let required = def
                .get("required")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            if required && !stored_names.contains(name) {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();

    Json(PluginEnvResponse {
        definitions,
        stored,
        valid: missing_required.is_empty(),
        missing_required,
    })
    .into_response()
}

pub(crate) async fn update_plugin_env_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<UserQuery>,
    Json(body): Json<UpdatePluginEnvRequest>,
) -> Response {
    let default_user_id = UserId::new("admin");
    let user_id = match super::extract_user_from_cookie(&headers) {
        Ok(session) => UserId::new(&session.user_id),
        Err(_) => query
            .user_id
            .as_ref()
            .map_or_else(|| default_user_id.clone(), UserId::new),
    };

    for var in &body.variables {
        if let Err(e) = repositories::upsert_plugin_env_var(
            &pool,
            &user_id,
            &plugin_id,
            &var.var_name,
            &var.var_value,
            var.is_secret,
        )
        .await
        {
            tracing::error!(error = %e, "Failed to upsert plugin env var");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    }

    let p = pool.clone();
    let uid = user_id.clone();
    let pid = plugin_id.clone();
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_updated(&uid, ActivityEntity::Plugin, &pid, &pid),
        )
        .await;
    });

    StatusCode::NO_CONTENT.into_response()
}

// JSON: protocol boundary (plugin variable definitions have user-defined schema)
fn load_plugin_variable_defs(
    plugin_id: &str,
) -> Result<Vec<serde_json::Value>, crate::error::MarketplaceError> {
    let services_path = ProfileBootstrap::get()
        .map(|p| std::path::PathBuf::from(&p.paths.services))
        .map_err(|e| {
            crate::error::MarketplaceError::Internal(format!("Failed to load profile: {e}"))
        })?;
    let config_path = services_path
        .join("plugins")
        .join(plugin_id)
        .join("config.yaml");
    if !config_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&config_path)?;
    let val: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let variables = val
        .get("plugin")
        .and_then(|p| p.get("variables"))
        .and_then(|v| v.as_sequence())
        .cloned()
        .unwrap_or_else(Vec::new);

    let defs: Vec<serde_json::Value> = variables
        .into_iter()
        .filter_map(|v| {
            serde_json::to_value(v)
                .map_err(|e| {
                    tracing::warn!(error = %e, "Failed to convert variable definition to JSON");
                })
                .ok()
        })
        .collect();
    Ok(defs)
}

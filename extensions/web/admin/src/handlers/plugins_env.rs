use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;

use systemprompt::identifiers::UserId;

use crate::handlers::shared;
use crate::repositories;
use crate::types::UserQuery;

use super::responses::PluginEnvResponse;

pub async fn list_plugin_env_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<UserQuery>,
) -> Response {
    // Why: cookie absence is the "logged-out" branch, not an error — the
    // handler falls through to `resolve_principal` to honour the explicit
    // `user_id` query parameter.
    let cookie_uid = super::extract_user_from_cookie(&headers)
        .ok()
        .map(|s| s.user_id);
    let Some(user_id) = resolve_principal(
        cookie_uid.as_ref().map(UserId::as_str),
        query.user_id.as_deref(),
    ) else {
        return shared::error_response(StatusCode::UNAUTHORIZED, "missing principal");
    };

    let definitions = load_plugin_variable_defs(&plugin_id).unwrap_or_else(|e| {
        tracing::debug!(error = %e, plugin_id = %plugin_id, "Failed to load plugin variable definitions");
        vec![]
    });

    let stored = match repositories::list_plugin_env_vars(&pool, &user_id, &plugin_id).await {
        Ok(vars) => vars,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list plugin env vars");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
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

/// Resolve the principal for a plugin-env request from already-validated inputs.
///
/// Returns `Some(UserId)` only when an authenticated cookie session or an
/// explicit `user_id` query parameter is present. Never synthesizes.
pub fn resolve_principal(
    cookie_user_id: Option<&str>,
    query_user_id: Option<&str>,
) -> Option<UserId> {
    cookie_user_id.or(query_user_id).map(UserId::new)
}

fn load_plugin_variable_defs(
    plugin_id: &str,
) -> Result<Vec<serde_json::Value>, systemprompt_web_shared::error::MarketplaceError> {
    let services_path = ProfileBootstrap::get()
        .map(|p| std::path::PathBuf::from(&p.paths.services))
        .map_err(|e| {
            systemprompt_web_shared::error::MarketplaceError::Internal(format!(
                "Failed to load profile: {e}"
            ))
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

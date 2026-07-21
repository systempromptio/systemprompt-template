//! HTTP handlers for plugin environment variables.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;

use systemprompt::identifiers::UserId;

use crate::handlers::shared;
use crate::repositories;
use crate::types::UserQuery;

use super::responses::PluginEnvResponse;

pub(crate) async fn list_plugin_env_handler(
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
        query.user_id.as_ref().map(UserId::as_str),
    ) else {
        return shared::error_response(StatusCode::UNAUTHORIZED, "missing principal");
    };

    let definitions = load_plugin_variable_defs(&plugin_id).unwrap_or_else(|e| {
        tracing::debug!(error = %e, plugin_id = %plugin_id, "Failed to load plugin variable definitions");
        vec![]
    });

    let stored = match repositories::marketplace::plugin_env::list_plugin_env_vars(
        &pool, &user_id, &plugin_id,
    )
    .await
    {
        Ok(vars) => vars,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list plugin env vars");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            );
        },
    };

    let stored_names: std::collections::HashSet<String> = stored
        .iter()
        .filter(|v| !v.var_value.is_empty())
        .map(|v| v.var_name.clone())
        .collect();
    let missing_required: Vec<String> = definitions
        .iter()
        .filter(|def| def.required && !stored_names.contains(&def.name))
        .map(|def| def.name.clone())
        .collect();

    Json(PluginEnvResponse {
        definitions,
        stored,
        valid: missing_required.is_empty(),
        missing_required,
    })
    .into_response()
}

/// Resolve the principal for a plugin-env request from already-validated
/// inputs.
///
/// A principal resolves only from an authenticated cookie session or an
/// explicit `user_id` query parameter. Absent both, the request has no
/// principal — one is never synthesized to make the call succeed.
pub fn resolve_principal(
    cookie_user_id: Option<&str>,
    query_user_id: Option<&str>,
) -> Option<UserId> {
    cookie_user_id.or(query_user_id).map(UserId::new)
}

fn load_plugin_variable_defs(
    plugin_id: &str,
) -> Result<Vec<PluginVariableDef>, systemprompt_web_shared::error::MarketplaceError> {
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
    let config: PluginConfigFile = serde_yaml::from_str(&content)?;
    Ok(config.plugin.map(|p| p.variables).unwrap_or_default())
}

/// The `plugin.variables` block of `services/plugins/<id>/config.yaml`.
///
/// Only the one block is modelled; every other key in the file is a concern of
/// whatever loads it, so the outer shapes stay deliberately permissive.
#[derive(Debug, Deserialize)]
struct PluginConfigFile {
    plugin: Option<PluginSection>,
}

#[derive(Debug, Deserialize)]
struct PluginSection {
    #[serde(default)]
    variables: Vec<PluginVariableDef>,
}

/// One environment variable a plugin declares it needs.
///
/// Serialized straight to the plugin-env screen, which reads exactly these
/// fields — so the type is the contract with that screen, not just a parse.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PluginVariableDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// A declared variable is required unless it says otherwise.
    #[serde(default = "required_by_default")]
    pub required: bool,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub example: String,
}

const fn required_by_default() -> bool {
    true
}

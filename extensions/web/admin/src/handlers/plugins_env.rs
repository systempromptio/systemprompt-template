//! HTTP handlers for plugin environment variables.

use crate::env::AdminEnv;
use axum::Extension;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use systemprompt::identifiers::PluginId;

use crate::error::AdminResult;
use crate::repositories;
use crate::types::UserContext;

use super::responses::PluginEnvResponse;

pub(crate) async fn list_plugin_env_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(env): Extension<AdminEnv>,
    Path(plugin_id): Path<PluginId>,
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let user_id = user_ctx.user_id;

    // Why: this must not degrade to an empty list — `definitions` decides
    // `missing_required`, which decides `valid`, which is the flag the UI uses
    // to enable "Generate/Export". A `config.yaml` that fails to parse would
    // otherwise read as "this plugin needs no configuration" and unlock the
    // export of a bundle whose required secrets were never collected — a gate
    // failing open. A genuinely absent config is a different thing and still
    // answers `Ok(vec![])` below, so the no-configuration case is untouched.
    let definitions = load_plugin_variable_defs(&env, &plugin_id)?;

    let stored =
        repositories::marketplace::plugin_env::list_plugin_env_vars(&pool, &user_id, &plugin_id)
            .await?;

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

    Ok(Json(PluginEnvResponse {
        definitions,
        stored,
        valid: missing_required.is_empty(),
        missing_required,
    })
    .into_response())
}

fn load_plugin_variable_defs(
    env: &AdminEnv,
    plugin_id: &PluginId,
) -> Result<Vec<PluginVariableDef>, systemprompt_web_shared::error::MarketplaceError> {
    let services_path = env.baked_services_root();
    let config_path = services_path
        .join("plugins")
        .join(plugin_id.as_str())
        .join("config.yaml");
    if !config_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&config_path)?;
    let config: PluginConfigFile = serde_yaml::from_str(&content)?;
    Ok(config.plugin.map(|p| p.variables).unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct PluginConfigFile {
    plugin: Option<PluginSection>,
}

#[derive(Debug, Deserialize)]
struct PluginSection {
    #[serde(default)]
    variables: Vec<PluginVariableDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PluginVariableDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
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

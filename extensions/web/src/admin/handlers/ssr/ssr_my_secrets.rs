use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn my_secrets_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let env_vars = repositories::list_all_user_env_vars(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user secrets");
            vec![]
        });

    let user_plugins = repositories::user_plugins::list_user_plugins(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let total_count = env_vars.len();
    let secret_count = env_vars.iter().filter(|v| v.is_secret).count();

    let mut plugin_groups: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
        std::collections::BTreeMap::new();
    for v in &env_vars {
        let entry = plugin_groups.entry(v.plugin_id.clone()).or_default();
        entry.push(json!({
            "id": v.id,
            "plugin_id": v.plugin_id,
            "var_name": v.var_name,
            "var_value": v.var_value,
            "is_secret": v.is_secret,
        }));
    }

    let groups_json: Vec<serde_json::Value> = plugin_groups
        .into_iter()
        .map(|(plugin_id, vars)| {
            let count = vars.len();
            json!({
                "plugin_id": plugin_id,
                "variables": vars,
                "count": count,
            })
        })
        .collect();

    let plugins_json: Vec<serde_json::Value> = user_plugins
        .iter()
        .map(|p| json!({"id": p.plugin_id, "name": p.name}))
        .collect();

    let data = json!({
        "page": "my-secrets",
        "title": "My Secrets",
        "groups": groups_json,
        "plugins": plugins_json,
        "stats": {
            "total_count": total_count,
            "secret_count": secret_count,
        },
    });
    super::render_page(&engine, "my-secrets", &data, &user_ctx, &mkt_ctx)
}

use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{MySecretsPageData, NamedEntity, SecretGroupView, SecretVarView, SecretsStats};

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

    let user_plugins = repositories::list_user_plugins(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|_| vec![]);

    let total_count = env_vars.len();
    let secret_count = env_vars.iter().filter(|v| v.is_secret).count();

    let mut plugin_groups: std::collections::BTreeMap<String, Vec<SecretVarView>> =
        std::collections::BTreeMap::new();
    for v in &env_vars {
        let entry = plugin_groups.entry(v.plugin_id.clone()).or_default();
        entry.push(SecretVarView {
            id: v.id.clone(),
            plugin_id: v.plugin_id.clone(),
            var_name: v.var_name.clone(),
            var_value: v.var_value.clone(),
            is_secret: v.is_secret,
        });
    }

    let groups: Vec<SecretGroupView> = plugin_groups
        .into_iter()
        .map(|(plugin_id, vars)| {
            let count = vars.len();
            SecretGroupView {
                plugin_id,
                variables: vars,
                count,
            }
        })
        .collect();

    let plugins: Vec<NamedEntity> = user_plugins
        .iter()
        .map(|p| NamedEntity {
            id: p.plugin_id.clone(),
            name: p.name.clone(),
        })
        .collect();

    let data = MySecretsPageData {
        page: "my-secrets",
        title: "My Secrets",
        groups,
        plugins,
        stats: SecretsStats {
            total_count,
            secret_count,
        },
    };

    let value = serde_json::to_value(&data).unwrap_or_else(|_| serde_json::Value::Null);
    super::render_page(&engine, "my-secrets", &value, &user_ctx, &mkt_ctx)
}

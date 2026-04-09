use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use sqlx::PgPool;

use systemprompt::identifiers::SkillId;

use crate::admin::activity::{self, ActivityEntity, NewActivity};
use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::{CreateSkillRequest, ImportPluginRequest, UserContext};

use super::resources::get_services_path;
use super::responses::ImportUserBundleResponse;

pub async fn import_plugin_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<ImportPluginRequest>,
) -> Response {
    let bundle = match fetch_plugin_bundle(&body.url).await {
        Ok(b) => b,
        Err(r) => return r,
    };

    let import_target = body.import_target.as_deref().unwrap_or("site");

    if import_target == "user" {
        import_bundle_for_user(&pool, &user_ctx, &bundle).await
    } else {
        import_bundle_for_site(&pool, &bundle)
    }
}

async fn fetch_plugin_bundle(url: &str) -> Result<repositories::export::PluginBundle, Response> {
    let resp = reqwest::get(url).await.map_err(|e| {
        shared::error_response(
            StatusCode::BAD_REQUEST,
            &format!("Failed to fetch URL: {e}"),
        )
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        return Err(shared::error_response(
            StatusCode::BAD_REQUEST,
            &format!("Failed to fetch URL: HTTP {status}"),
        ));
    }

    resp.json::<repositories::export::PluginBundle>()
        .await
        .map_err(|e| {
            shared::error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to parse plugin bundle JSON: {e}"),
            )
        })
}

fn import_bundle_for_site(pool: &PgPool, bundle: &repositories::export::PluginBundle) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    match repositories::import_plugin_bundle(&services_path, bundle) {
        Ok(plugin) => {
            let name = plugin.name.clone();
            let pid = plugin.id.clone();
            let pool = pool.clone();
            tokio::spawn(async move {
                activity::record(
                    &pool,
                    NewActivity::entity_imported(
                        "admin",
                        ActivityEntity::Plugin,
                        &pid,
                        &name,
                        &format!("Imported plugin '{name}'"),
                    ),
                )
                .await;
            });
            (StatusCode::CREATED, Json(plugin)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to import plugin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

async fn import_bundle_for_user(
    pool: &PgPool,
    user_ctx: &UserContext,
    bundle: &repositories::export::PluginBundle,
) -> Response {
    let skills: Vec<_> = bundle
        .files
        .iter()
        .filter(|f| {
            f.path.starts_with("skills/")
                && std::path::Path::new(&f.path)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        })
        .collect();

    if skills.is_empty() {
        return shared::error_response(StatusCode::BAD_REQUEST, "No skills found in bundle");
    }

    let mut imported_count = 0u32;
    for skill_file in &skills {
        let skill_id = skill_file
            .path
            .strip_prefix("skills/")
            .unwrap_or(&skill_file.path)
            .strip_suffix(".md")
            .unwrap_or(&skill_file.path)
            .to_string();
        let req = CreateSkillRequest {
            skill_id: SkillId::new(skill_id.clone()),
            name: skill_id.clone(),
            description: String::new(),
            content: skill_file.content.clone(),
            tags: vec![],
            base_skill_id: None,
        };
        match repositories::user_skills::create_user_skill(pool, &user_ctx.user_id, &req).await {
            Ok(_) => imported_count += 1,
            Err(e) => {
                tracing::warn!(error = %e, skill_id = %skill_id, "Failed to import user skill");
            }
        }
    }

    let user_id = user_ctx.user_id.clone();
    let bundle_id = bundle.id.clone();
    let pool = pool.clone();
    tokio::spawn(async move {
        activity::record(
            &pool,
            NewActivity::user_skill_imported(&user_id, &bundle_id, imported_count),
        )
        .await;
    });

    (
        StatusCode::CREATED,
        Json(ImportUserBundleResponse {
            message: format!("Imported {imported_count} skills for current user"),
            imported_count,
        }),
    )
        .into_response()
}

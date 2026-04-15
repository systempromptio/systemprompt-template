use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub async fn infra_config_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(resp) => return *resp,
    };

    let config_dir = services_path.join("config");
    let files = list_config_files(&config_dir);
    let has_files = !files.is_empty();

    let data = json!({
        "page": "infra-config",
        "title": "Infrastructure — Configuration",
        "cli_command": "systemprompt admin config show",
        "config_dir": config_dir.display().to_string(),
        "files": files,
        "has_files": has_files,
    });
    super::render_page(&engine, "infra-config", &data, &user_ctx, &mkt_ctx)
}

fn list_config_files(dir: &std::path::Path) -> Vec<serde_json::Value> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let valid = match ext {
            "yaml" | "yml" => std::fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_yaml::from_str::<serde_yaml::Value>(&c).ok())
                .is_some(),
            "json" => std::fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
                .is_some(),
            _ => true,
        };
        files.push(json!({
            "name": name,
            "path": path.display().to_string(),
            "extension": ext,
            "size_bytes": size,
            "valid": valid,
        }));
    }
    files.sort_by(|a, b| {
        a.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    });
    files
}

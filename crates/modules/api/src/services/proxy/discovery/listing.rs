use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use systemprompt_core_system::AppContext;
use systemprompt_models::repository::ServiceRepository;

pub async fn list_mcp_services(
    State(ctx): State<AppContext>,
) -> Result<impl IntoResponse, StatusCode> {
    let service_repo = ServiceRepository::new(ctx.db_pool().clone());
    let services = match service_repo.get_all_running_services().await {
        Ok(services) => services,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mcp_services: Vec<serde_json::Value> = services
        .into_iter()
        .filter(|service| service.module_name == "mcp")
        .map(|service| {
            json!({
                "name": service.name,
                "url": format!("{}/server/{}/mcp", ctx.config().api_server_url, service.name),
                "status": service.status,
                "port": service.port,
            })
        })
        .collect();

    Ok(Json(json!({ "services": mcp_services })))
}

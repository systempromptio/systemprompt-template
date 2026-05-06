use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories;
use crate::repositories::analytics_grp::{list_recent_gateway_requests, RecentGatewayRequestRow};
use crate::repositories::find_matching_route_index;
use crate::templates::AdminTemplateEngine;
use crate::types::{GatewayRouteView, IdQuery, MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const RECENT_TRAFFIC_LIMIT: i64 = 50;

pub async fn gateway_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let profile_path = match super::super::shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let config = match repositories::get_gateway_config(&profile_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load gateway config");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!("<h1>Error</h1><p>{e}</p>")),
            )
                .into_response();
        }
    };

    let recent = list_recent_gateway_requests(&pool, RECENT_TRAFFIC_LIMIT)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch recent gateway traffic");
            Vec::new()
        });

    let recent_json: Vec<serde_json::Value> = recent
        .iter()
        .map(|row| recent_row_to_json(row, &config.routes))
        .collect();

    let routes_json: Vec<serde_json::Value> = config
        .routes
        .iter()
        .enumerate()
        .map(|(idx, r)| {
            json!({
                "index": idx,
                "id": r.id,
                "model_pattern": r.model_pattern,
                "provider": r.provider,
                "endpoint": r.endpoint,
                "endpoint_short": shorten_url(&r.endpoint),
                "api_key_secret": r.api_key_secret,
                "upstream_model": r.upstream_model,
                "extra_headers_count": r.extra_headers.len(),
            })
        })
        .collect();

    let data = json!({
        "page": "gateway",
        "title": "Gateway",
        "config": {
            "enabled": config.enabled,
            "auth_scheme": config.auth_scheme,
            "inference_path_prefix": config.inference_path_prefix,
            "catalog_path": config.catalog_path,
            "profile_path": config.profile_path,
        },
        "routes": routes_json,
        "route_count": config.routes.len(),
        "recent_traffic": recent_json,
        "recent_limit": RECENT_TRAFFIC_LIMIT,
    });

    super::render_page(&engine, "gateway", &data, &user_ctx, &mkt_ctx)
}

pub async fn gateway_route_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(params): Query<IdQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let edit_index = params.id().and_then(|s| s.parse::<usize>().ok());
    let is_edit = edit_index.is_some();

    let route = if let Some(idx) = edit_index {
        let profile_path = match super::super::shared::get_profile_path() {
            Ok(p) => p,
            Err(r) => return *r,
        };
        repositories::get_gateway_config(&profile_path)
            .ok()
            .and_then(|c| c.routes.into_iter().nth(idx))
    } else {
        None
    };

    let route_json = route.as_ref().map(|r| {
        json!({
            "model_pattern": r.model_pattern,
            "provider": r.provider,
            "endpoint": r.endpoint,
            "api_key_secret": r.api_key_secret,
            "upstream_model": r.upstream_model,
            "extra_headers": r.extra_headers,
        })
    });

    let data = json!({
        "page": "gateway-route-edit",
        "title": if is_edit { "Edit Gateway Route" } else { "Add Gateway Route" },
        "is_edit": is_edit,
        "edit_index": edit_index,
        "route": route_json,
    });
    super::render_page(&engine, "gateway-route-edit", &data, &user_ctx, &mkt_ctx)
}

fn recent_row_to_json(
    row: &RecentGatewayRequestRow,
    routes: &[GatewayRouteView],
) -> serde_json::Value {
    let matched = find_matching_route_index(routes, &row.model);
    let total_tokens = row.input_tokens.unwrap_or(0) + row.output_tokens.unwrap_or(0);
    let cost_usd = row.cost_microdollars as f64 / 1_000_000.0;
    let rejected_at_boundary =
        row.model == "unknown" && row.provider == "unknown" && row.trace_id.is_none();
    json!({
        "id": row.id,
        "created_at": row.created_at.to_rfc3339(),
        "provider": row.provider,
        "model": row.model,
        "status": row.status,
        "input_tokens": row.input_tokens,
        "output_tokens": row.output_tokens,
        "total_tokens": total_tokens,
        "cost_usd": format!("{cost_usd:.4}"),
        "latency_ms": row.latency_ms,
        "trace_id": row.trace_id,
        "error_message": row.error_message,
        "matched_route_index": matched,
        "matched_pattern": matched.and_then(|i| routes.get(i).map(|r| r.model_pattern.clone())),
        "rejected_at_boundary": rejected_at_boundary,
    })
}

fn shorten_url(url: &str) -> String {
    let trimmed = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    if trimmed.len() <= 40 {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..37])
    }
}

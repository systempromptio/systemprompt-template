#![allow(unused_qualifications)]

use axum::{
    extract::{Extension, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use validator::Validate;

use crate::repository::OAuthRepository;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::api::{ApiError, CollectionResponse, PaginationInfo};
use systemprompt_models::api::PaginationParams;

#[derive(Debug, Deserialize, Validate)]
pub struct ListClientsQuery {
    #[serde(flatten)]
    pub pagination: PaginationParams,

    #[validate(length(min = 1, max = 50))]
    pub status: Option<String>,
}

pub async fn list_clients(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Query(query): Query<ListClientsQuery>,
) -> impl IntoResponse {
    let repository = OAuthRepository::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    // Validate query parameters
    if let Err(e) = query.validate() {
        logger
            .log(
                LogLevel::Info,
                "oauth_api",
                "OAuth clients list rejected - validation failed",
                Some(serde_json::json!({
                    "reason": "Validation error",
                    "requested_by": req_ctx.auth.user_id.as_str()
                })),
            )
            .await
            .ok();
        return ApiError::validation_error(format!("Invalid query parameters: {e}"), vec![])
            .into_response();
    }

    let page = query.pagination.page;
    let per_page = query.pagination.per_page;
    let offset = query.pagination.offset();
    let limit = query.pagination.limit();

    let clients_result = repository.list_clients_paginated(limit, offset).await;
    let count_result = repository.count_clients().await;

    match (clients_result, count_result) {
        (Ok(clients), Ok(total)) => {
            logger
                .log(
                    LogLevel::Info,
                    "oauth_api",
                    "OAuth clients listed",
                    Some(serde_json::json!({
                        "count": clients.len(),
                        "total": total,
                        "page": page,
                        "per_page": per_page,
                        "requested_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            let pagination = PaginationInfo::new(total, page, per_page);
            let response = CollectionResponse::paginated(
                clients
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<crate::models::clients::api::OAuthClientResponse>>(),
                pagination,
            );
            response.into_response()
        },
        (Err(e), _) | (_, Err(e)) => {
            logger
                .log(
                    LogLevel::Error,
                    "oauth_api",
                    "OAuth clients list failed",
                    Some(serde_json::json!({
                        "reason": format!("Database error: {}", e),
                        "requested_by": req_ctx.auth.user_id.as_str()
                    })),
                )
                .await
                .ok();
            ApiError::internal_error(format!("Failed to list clients: {e}")).into_response()
        },
    }
}

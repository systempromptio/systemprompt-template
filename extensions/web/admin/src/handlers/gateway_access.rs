//! Subject helpers — distinct roles and user search — used by the inline
//! access panel and matrix view. Per-route access mutation lives in
//! [`crate::handlers::entity_access`] (parameterized on `entity_type`).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;

use crate::handlers::shared;
use crate::repositories::{fetch_distinct_roles, list_users};

pub(crate) async fn list_distinct_roles_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match fetch_distinct_roles(&pool).await {
        Ok(roles) => Json(serde_json::json!({ "roles": roles })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch distinct roles");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error")
        },
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct UserSearchQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

pub(crate) async fn search_users_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UserSearchQuery>,
) -> Response {
    let users = match list_users(&pool).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list users for search");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal error");
        },
    };
    let q = query.q.unwrap_or_default().to_lowercase();
    let limit = query.limit.unwrap_or(10).min(50);
    let filtered: Vec<serde_json::Value> = users
        .into_iter()
        .filter(|u| {
            if q.is_empty() {
                return true;
            }
            let id_match = u.user_id.as_ref().to_lowercase().contains(&q);
            let name_match = u
                .display_name
                .as_deref()
                .is_some_and(|n| n.to_lowercase().contains(&q));
            let email_match = u
                .email
                .as_ref()
                .is_some_and(|e| e.as_ref().to_lowercase().contains(&q));
            id_match || name_match || email_match
        })
        .take(limit)
        .map(|u| {
            serde_json::json!({
                "id": u.user_id.as_ref(),
                "display_name": u.display_name,
                "email": u.email.as_ref().map(systemprompt::identifiers::Email::as_ref),
            })
        })
        .collect();
    Json(serde_json::json!({ "users": filtered })).into_response()
}

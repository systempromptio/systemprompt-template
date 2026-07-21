//! Subject helpers — distinct roles and user search — used by the inline
//! access panel and matrix view. Per-route access mutation lives in
//! [`crate::handlers::entity_access`] (parameterized on `entity_type`).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::users::queries::{list_distinct_roles, list_users};

/// JSON body returned by [`list_distinct_roles_handler`].
#[derive(Debug, Serialize)]
pub(crate) struct DistinctRolesResponse {
    pub roles: Vec<String>,
}

/// One entry in [`UserSearchResponse::users`].
#[derive(Debug, Serialize)]
pub(crate) struct UserSearchEntry {
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

/// JSON body returned by [`search_users_handler`].
#[derive(Debug, Serialize)]
pub(crate) struct UserSearchResponse {
    pub users: Vec<UserSearchEntry>,
}

pub(crate) async fn list_distinct_roles_handler(
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    let roles = list_distinct_roles(&pool).await?;
    Ok(Json(DistinctRolesResponse { roles }).into_response())
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
) -> AdminResult<Response> {
    let users = list_users(&pool).await?;
    let q = query.q.unwrap_or_default().to_lowercase();
    let limit = query.limit.unwrap_or(10).min(50);
    let users: Vec<UserSearchEntry> = users
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
        .map(|u| UserSearchEntry {
            id: u.user_id.as_ref().to_owned(),
            display_name: u.display_name,
            email: u.email.as_ref().map(|e| e.as_ref().to_owned()),
        })
        .collect();
    Ok(Json(UserSearchResponse { users }).into_response())
}

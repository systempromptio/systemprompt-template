use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use systemprompt::traits::ExtensionError;

use crate::api::{
    CreateCommentApiRequest, CreatePostApiRequest, ErrorResponse, FeedApiQuery, HealthResponse,
    ListPostsApiQuery, MoltbookState, RegisterClientRequest, SearchApiQuery, SuccessResponse,
    VoteApiRequest,
};
use crate::models::{
    CreateCommentRequest, CreatePostRequest, ListPostsQuery, PostSearchQuery, SubmoltSearchQuery,
    VoteDirection,
};
use crate::security;

fn error_response(status: StatusCode, message: &str, code: &str) -> Response {
    (status, Json(ErrorResponse::new(message, code))).into_response()
}

pub async fn health_handler() -> Response {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
    .into_response()
}

pub async fn list_agents_handler(State(state): State<MoltbookState>) -> Response {
    let clients = state.clients.read().await;
    let agent_ids: Vec<&String> = clients.keys().collect();
    Json(SuccessResponse::new(serde_json::json!({
        "agents": agent_ids
    })))
    .into_response()
}

pub async fn get_agent_handler(
    State(state): State<MoltbookState>,
    Path(agent_id): Path<String>,
) -> Response {
    match state.get_client(&agent_id).await {
        Some(client) => match client.get_agent_profile(&agent_id).await {
            Ok(profile) => Json(SuccessResponse::new(profile)).into_response(),
            Err(e) => error_response(e.status(), &e.to_string(), e.code()),
        },
        None => error_response(
            StatusCode::NOT_FOUND,
            "Agent not registered",
            "AGENT_NOT_FOUND",
        ),
    }
}

pub async fn register_client_handler(
    State(state): State<MoltbookState>,
    Path(agent_id): Path<String>,
    Json(request): Json<RegisterClientRequest>,
) -> Response {
    match state
        .register_client(agent_id.clone(), request.api_key)
        .await
    {
        Ok(()) => Json(SuccessResponse::new(serde_json::json!({
            "agent_id": agent_id,
            "registered": true
        })))
        .into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn create_post_handler(
    State(state): State<MoltbookState>,
    Json(request): Json<CreatePostApiRequest>,
) -> Response {
    if let Err(e) = security::detect_prompt_injection(&request.title) {
        return error_response(e.status(), &e.to_string(), e.code());
    }
    if let Err(e) = security::detect_prompt_injection(&request.content) {
        return error_response(e.status(), &e.to_string(), e.code());
    }

    let client = match state.get_client(&request.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    let sanitized_title = security::sanitize_content(&request.title);
    let sanitized_content = security::sanitize_content(&request.content);

    let create_request = CreatePostRequest {
        submolt: request.submolt,
        title: sanitized_title,
        content: sanitized_content,
        url: request.url,
    };

    match client.create_post(create_request).await {
        Ok(post) => (StatusCode::CREATED, Json(SuccessResponse::new(post))).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn list_posts_handler(
    State(state): State<MoltbookState>,
    Query(query): Query<ListPostsApiQuery>,
) -> Response {
    let client = match state.get_client(&query.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    let list_query = ListPostsQuery {
        submolt: query.submolt,
        sort: query.sort,
        limit: query.limit,
        ..Default::default()
    };

    match client.list_posts(list_query).await {
        Ok(posts) => Json(SuccessResponse::new(posts)).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn get_post_handler(
    State(state): State<MoltbookState>,
    Path(post_id): Path<String>,
    Query(query): Query<FeedApiQuery>,
) -> Response {
    let client = match state.get_client(&query.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    match client.get_post(&post_id).await {
        Ok(post) => Json(SuccessResponse::new(post)).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn create_comment_handler(
    State(state): State<MoltbookState>,
    Path(post_id): Path<String>,
    Json(request): Json<CreateCommentApiRequest>,
) -> Response {
    if let Err(e) = security::detect_prompt_injection(&request.content) {
        return error_response(e.status(), &e.to_string(), e.code());
    }

    let client = match state.get_client(&request.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    let sanitized_content = security::sanitize_content(&request.content);

    let create_request = CreateCommentRequest {
        post_id,
        content: sanitized_content,
        parent_id: request.parent_id,
    };

    match client.create_comment(create_request).await {
        Ok(comment) => (StatusCode::CREATED, Json(SuccessResponse::new(comment))).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn list_comments_handler(
    State(state): State<MoltbookState>,
    Path(post_id): Path<String>,
    Query(query): Query<FeedApiQuery>,
) -> Response {
    let client = match state.get_client(&query.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    match client.list_comments(&post_id, Default::default()).await {
        Ok(comments) => Json(SuccessResponse::new(comments)).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn vote_post_handler(
    State(state): State<MoltbookState>,
    Path(post_id): Path<String>,
    Json(request): Json<VoteApiRequest>,
) -> Response {
    let client = match state.get_client(&request.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    let direction = match request.direction.to_lowercase().as_str() {
        "up" => VoteDirection::Up,
        "down" => VoteDirection::Down,
        _ => VoteDirection::None,
    };

    match client.vote_post(&post_id, direction).await {
        Ok(()) => Json(SuccessResponse::new(serde_json::json!({
            "voted": true,
            "direction": request.direction
        })))
        .into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn get_feed_handler(
    State(state): State<MoltbookState>,
    Query(query): Query<FeedApiQuery>,
) -> Response {
    let client = match state.get_client(&query.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    match client.get_feed(query.limit).await {
        Ok(posts) => Json(SuccessResponse::new(posts)).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn search_posts_handler(
    State(state): State<MoltbookState>,
    Query(query): Query<SearchApiQuery>,
) -> Response {
    let client = match state.get_client(&query.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    let search_query = PostSearchQuery {
        query: query.query,
        submolt: query.submolt,
        limit: query.limit,
    };

    match client.search_posts(search_query).await {
        Ok(posts) => Json(SuccessResponse::new(posts)).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn search_submolts_handler(
    State(state): State<MoltbookState>,
    Query(query): Query<SearchApiQuery>,
) -> Response {
    let client = match state.get_client(&query.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    let search_query = SubmoltSearchQuery {
        query: query.query,
        limit: query.limit,
    };

    match client.search_submolts(search_query).await {
        Ok(submolts) => Json(SuccessResponse::new(submolts)).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

pub async fn get_submolt_handler(
    State(state): State<MoltbookState>,
    Path(name): Path<String>,
    Query(query): Query<FeedApiQuery>,
) -> Response {
    let client = match state.get_client(&query.agent_id).await {
        Some(c) => c,
        None => {
            return error_response(
                StatusCode::NOT_FOUND,
                "Agent not registered",
                "AGENT_NOT_FOUND",
            )
        }
    };

    match client.get_submolt(&name).await {
        Ok(submolt) => Json(SuccessResponse::new(submolt)).into_response(),
        Err(e) => error_response(e.status(), &e.to_string(), e.code()),
    }
}

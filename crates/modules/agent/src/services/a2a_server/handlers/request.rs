use axum::{
    extract::{Json, Request, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_system::RequestContext;

use super::state::AgentHandlerState;
use crate::models::a2a::Task;
use crate::services::a2a_server::auth::validate_oauth_for_request;
use crate::services::a2a_server::builders::task::build_canceled_task;
use crate::services::a2a_server::errors::JsonRpcErrorBuilder;
use crate::services::a2a_server::processing::message::MessageProcessor;
use crate::services::a2a_server::streaming::create_sse_stream;

pub async fn handle_agent_request(
    State(state): State<Arc<AgentHandlerState>>,
    request: Request,
) -> impl IntoResponse {
    let log = state.log.clone();
    let start_time = std::time::Instant::now();

    log.info("a2a_request", "🚨 Agent request handler invoked")
        .await
        .ok();

    let context = request
        .extensions()
        .get::<RequestContext>()
        .cloned()
        .expect("RequestContext must be present after middleware - this is a bug if missing");

    let (parts, body) = request.into_parts();
    let headers = parts.headers.clone();

    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32700, "message": "Failed to read request body"},
                    "id": null
                })),
            )
                .into_response();
        },
    };

    let payload: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32700, "message": "Invalid JSON"},
                    "id": null
                })),
            )
                .into_response();
        },
    };

    let jsonrpc_request =
        match serde_json::from_value::<crate::models::a2a::A2aJsonRpcRequest>(payload) {
            Ok(req) => req,
            Err(e) => {
                let error_response = JsonRpcErrorBuilder::invalid_request()
                    .with_data(json!(
                        "Request must be valid JSON-RPC 2.0 with jsonrpc, method, params, and id"
                    ))
                    .log_error(format!("Invalid JSON-RPC request: {}", e))
                    .build(
                        &crate::models::a2a::jsonrpc::NumberOrString::Number(0),
                        &log,
                    )
                    .await;
                return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
            },
        };

    let request_id = jsonrpc_request.id.clone();
    log.info(
        "a2a_request",
        &format!("Processing A2A JSON-RPC method: {}", jsonrpc_request.method),
    )
    .await
    .ok();

    let requires_oauth = should_require_oauth(&jsonrpc_request, &state).await;

    if requires_oauth {
        log.info("a2a_request", "Request requires OAuth2 authentication")
            .await
            .ok();

        let required_scopes = {
            let config = state.config.read().await;
            config.oauth.scopes.clone()
        };

        if let Err((status, error_response)) =
            validate_oauth_for_request(&headers, &request_id, &required_scopes, &log).await
        {
            return (status, Json(error_response)).into_response();
        }
    }

    let is_streaming = jsonrpc_request.method == "message/stream";

    let a2a_request = match jsonrpc_request.parse_request() {
        Ok(request) => request,
        Err(e) => {
            let error_str = e.to_string();

            if error_str.contains("missing field `contextId`") {
                let helpful_message = json!({
                    "error": "contextId is required",
                    "message": "JWT token and contextId are required to use this API.",
                    "instructions": {
                        "step1": {
                            "description": "Obtain a JWT token (no registration required)",
                            "endpoint": "POST /api/v1/core/oauth/session",
                            "example": "curl -X POST http://localhost:8080/api/v1/core/oauth/session"
                        },
                        "step2": {
                            "description": "Create a context using your JWT token",
                            "endpoint": "POST /api/v1/core/contexts",
                            "headers": {
                                "Authorization": "Bearer YOUR_JWT_TOKEN",
                                "Content-Type": "application/json"
                            },
                            "body": {
                                "name": "My Context",
                                "description": "Optional description"
                            },
                            "example": "curl -X POST http://localhost:8080/api/v1/core/contexts -H 'Authorization: Bearer YOUR_JWT' -H 'Content-Type: application/json' -d '{\"name\":\"My Context\"}'"
                        },
                        "step3": {
                            "description": "Include contextId in your message/stream request",
                            "example": {
                                "jsonrpc": "2.0",
                                "method": "message/stream",
                                "params": {
                                    "message": {
                                        "kind": "message",
                                        "role": "user",
                                        "contextId": "YOUR_CONTEXT_ID",
                                        "parts": [{"kind": "text", "text": "hello"}],
                                        "messageId": "unique-message-id"
                                    }
                                },
                                "id": 1
                            }
                        }
                    }
                });

                let error_response = JsonRpcErrorBuilder::invalid_params()
                    .with_data(helpful_message)
                    .log_error(format!(
                        "Missing required contextId in message/stream request"
                    ))
                    .build(&request_id, &log)
                    .await;
                return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
            } else {
                let error_response = JsonRpcErrorBuilder::method_not_found()
                    .with_data(json!(format!(
                        "Unsupported method: {}",
                        jsonrpc_request.method
                    )))
                    .log_error(format!(
                        "Invalid A2A request method '{}': {}",
                        jsonrpc_request.method, e
                    ))
                    .build(&request_id, &log)
                    .await;
                return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
            }
        },
    };

    let mut enriched_context = context.clone();
    match &a2a_request {
        A2aRequestParams::SendMessage(ref params)
        | A2aRequestParams::SendStreamingMessage(ref params) => {
            if params.message.context_id.as_str().is_empty() {
                let error_response = JsonRpcErrorBuilder::invalid_params()
                    .with_data(json!({
                        "error": "contextId cannot be empty",
                        "message": "contextId must be a valid non-empty string. Please create a context first using POST /api/v1/core/contexts"
                    }))
                    .log_error("Rejected request with empty contextId".to_string())
                    .build(&request_id, &log)
                    .await;
                return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
            }
            enriched_context = enriched_context.with_context_id(params.message.context_id.clone());
        },
        _ => {},
    }

    if is_streaming {
        log.info(
            "a2a_request",
            "Processing message/stream request with SSE response",
        )
        .await
        .ok();

        log.info("a2a_request", "DEBUG: After first log, before match")
            .await
            .ok();

        let request_id_value = Some(match request_id {
            crate::models::a2a::jsonrpc::NumberOrString::String(ref s) => {
                serde_json::Value::String(s.to_string())
            },
            crate::models::a2a::jsonrpc::NumberOrString::Number(n) => {
                serde_json::Value::Number(serde_json::Number::from(n))
            },
        });

        log.info(
            "a2a_request",
            &format!(
                "🔷 Calling handle_streaming_request with request_id: {:?}",
                request_id_value
            ),
        )
        .await
        .ok();

        let stream = handle_streaming_request(
            a2a_request,
            state.clone(),
            request_id_value,
            enriched_context,
        )
        .await;

        log.info(
            "a2a_request",
            "🔶 handle_streaming_request returned, stream created",
        )
        .await
        .ok();

        let latency_ms = start_time.elapsed().as_millis();
        log.info(
            "a2a_request",
            &format!(
                "SSE stream initialized in {}ms for message/stream",
                latency_ms
            ),
        )
        .await
        .ok();

        return Sse::new(stream)
            .keep_alive(KeepAlive::default())
            .into_response();
    }

    use crate::models::a2a::A2aRequestParams;

    let push_notification_response = match &a2a_request {
        A2aRequestParams::SetTaskPushNotificationConfig(params) => {
            use crate::services::a2a_server::handlers::push_notification_config::handle_set_push_notification_config;

            log.info(
                "a2a_request",
                "Handling tasks/pushNotificationConfig/set request",
            )
            .await
            .ok();

            Some(
                handle_set_push_notification_config(State(state.clone()), params.clone(), &log)
                    .await,
            )
        },
        A2aRequestParams::GetTaskPushNotificationConfig(params) => {
            use crate::services::a2a_server::handlers::push_notification_config::handle_get_push_notification_config;

            log.info(
                "a2a_request",
                "Handling tasks/pushNotificationConfig/get request",
            )
            .await
            .ok();

            Some(
                handle_get_push_notification_config(State(state.clone()), params.clone(), &log)
                    .await,
            )
        },
        A2aRequestParams::ListTaskPushNotificationConfig(_params) => {
            log.info(
                "a2a_request",
                "Handling tasks/pushNotificationConfig/list request",
            )
            .await
            .ok();

            None
        },
        A2aRequestParams::DeleteTaskPushNotificationConfig(params) => {
            use crate::services::a2a_server::handlers::push_notification_config::handle_delete_push_notification_config;

            log.info(
                "a2a_request",
                "Handling tasks/pushNotificationConfig/delete request",
            )
            .await
            .ok();

            Some(
                handle_delete_push_notification_config(State(state.clone()), params.clone(), &log)
                    .await,
            )
        },
        _ => None,
    };

    if let Some(result) = push_notification_response {
        let (status, json_response) = match result {
            Ok((status, json)) => (status, json),
            Err((status, json)) => (status, json),
        };

        let mut response_value = json_response.0;
        if let Some(obj) = response_value.as_object_mut() {
            obj.insert(
                "id".to_string(),
                match &request_id {
                    crate::models::a2a::jsonrpc::NumberOrString::String(s) => {
                        serde_json::Value::String(s.clone())
                    },
                    crate::models::a2a::jsonrpc::NumberOrString::Number(n) => {
                        serde_json::Value::Number(serde_json::Number::from(*n))
                    },
                },
            );
        }

        let latency_ms = start_time.elapsed().as_millis();
        log.info(
            "a2a_request",
            &format!(
                "Push notification config request processed in {}ms",
                latency_ms
            ),
        )
        .await
        .ok();

        return (status, Json(response_value)).into_response();
    }

    let response_result =
        handle_non_streaming_request(a2a_request, &state, &enriched_context).await;

    let json_rpc_response = match response_result {
        Ok(task) => match serde_json::to_value(task) {
            Ok(task_value) => json!({
                "jsonrpc": "2.0",
                "result": task_value,
                "id": request_id
            }),
            Err(e) => {
                JsonRpcErrorBuilder::internal_error()
                    .with_data(json!("Task serialization failed"))
                    .log_error(format!("Failed to serialize task response: {}", e))
                    .build(&request_id, &log)
                    .await
            },
        },
        Err(e) => {
            JsonRpcErrorBuilder::internal_error()
                .with_data(json!(format!("Request handling failed: {}", e)))
                .log_error(format!("A2A request handling failed: {}", e))
                .build(&request_id, &log)
                .await
        },
    };

    let latency_ms = start_time.elapsed().as_millis();
    let latency_ms = i64::try_from(latency_ms).unwrap_or(i64::MAX);
    log.info(
        "a2a_request",
        &format!(
            "A2A request processed in {}ms (OAuth: {}, Method: {})",
            latency_ms, requires_oauth, jsonrpc_request.method
        ),
    )
    .await
    .ok();

    (StatusCode::OK, Json(json_rpc_response)).into_response()
}

async fn validate_message_context(
    message: &crate::models::a2a::Message,
    user_id: Option<&str>,
    db_pool: &systemprompt_core_database::DbPool,
) -> Result<(), String> {
    let context_id = &message.context_id;

    let user_id =
        user_id.ok_or_else(|| "User authentication required for message processing".to_string())?;

    // Fail immediately if user_id is missing or invalid (security check)
    if user_id == "missing-user-id" || user_id.is_empty() {
        return Err(
            "Authentication required: x-user-id header must be set by API proxy after JWT validation"
                .to_string(),
        );
    }

    // Note: Anonymous users (with regular UUIDs) and authenticated users are both allowed
    // Context ownership validation will check if they own the context

    let task_repo = Arc::new(crate::repository::TaskRepository::new(db_pool.clone()));
    let artifact_repo = Arc::new(crate::repository::ArtifactRepository::new(db_pool.clone()));
    let context_repo =
        crate::repository::ContextRepository::new(db_pool.clone(), task_repo, artifact_repo);
    context_repo
        .validate_context_ownership(context_id.as_str(), user_id)
        .await
        .map_err(|e| format!("Context validation failed: {}", e))?;

    Ok(())
}

async fn handle_non_streaming_request(
    request: crate::models::a2a::A2aRequestParams,
    state: &AgentHandlerState,
    context: &RequestContext,
) -> Result<Task, Box<dyn std::error::Error + Send + Sync>> {
    use crate::models::a2a::*;
    let log = state.log.clone();

    let config = state.config.read().await;
    let agent_name = config.name.clone();
    drop(config);

    match request {
        A2aRequestParams::SendMessage(params) => {
            log.info("a2a_request", "Handling message/send request")
                .await
                .ok();

            validate_message_context(
                &params.message,
                Some(context.user_id().as_str()),
                &state.db_pool,
            )
            .await?;

            let message_processor =
                MessageProcessor::new(state.db_pool.clone(), state.ai_service.clone(), log.clone());

            message_processor
                .handle_message(params.message, &agent_name, context)
                .await
                .map_err(|e| e.into())
        },
        A2aRequestParams::SendStreamingMessage(params) => {
            log.info(
                "a2a_request",
                "Handling message/stream request (fallback to non-streaming)",
            )
            .await
            .ok();

            validate_message_context(
                &params.message,
                Some(context.user_id().as_str()),
                &state.db_pool,
            )
            .await?;

            let message_processor =
                MessageProcessor::new(state.db_pool.clone(), state.ai_service.clone(), log.clone());

            message_processor
                .handle_message(params.message, &agent_name, context)
                .await
                .map_err(|e| e.into())
        },
        A2aRequestParams::GetTask(params) => {
            log.info(
                "a2a_request",
                &format!("Handling tasks/get request for task: {}", params.id),
            )
            .await
            .ok();

            use crate::repository::TaskRepository;
            let task_repo = TaskRepository::new(state.db_pool.clone());

            match task_repo.get_task(&params.id).await {
                Ok(Some(task)) => Ok(task),
                Ok(None) => Err(format!("Task not found: {}", params.id).into()),
                Err(e) => Err(format!("Failed to retrieve task: {}", e).into()),
            }
        },
        A2aRequestParams::CancelTask(params) => {
            log.info(
                "a2a_request",
                &format!("Handling tasks/cancel request for task: {}", params.id),
            )
            .await
            .ok();

            use crate::repository::TaskRepository;
            let task_repo = TaskRepository::new(state.db_pool.clone());

            match task_repo.get_task(&params.id).await {
                Ok(Some(task)) => Ok(build_canceled_task(params.id.into(), task.context_id)),
                Ok(None) => Err(format!("Task not found: {}", params.id).into()),
                Err(e) => Err(format!("Failed to look up task: {}", e).into()),
            }
        },
        A2aRequestParams::SetTaskPushNotificationConfig(_)
        | A2aRequestParams::GetTaskPushNotificationConfig(_)
        | A2aRequestParams::ListTaskPushNotificationConfig(_)
        | A2aRequestParams::DeleteTaskPushNotificationConfig(_) => {
            Err("Push notification config requests should be handled before this point".into())
        },
        _ => {
            log.warn(
                "a2a_request",
                &format!("Unsupported A2A request type: {:?}", request),
            )
            .await
            .ok();
            Err("Unsupported request type".into())
        },
    }
}

async fn handle_streaming_request(
    request: crate::models::a2a::A2aRequestParams,
    state: Arc<AgentHandlerState>,
    request_id: Option<serde_json::Value>,
    context: RequestContext,
) -> impl futures::stream::Stream<Item = Result<Event, std::convert::Infallible>> + Send {
    use crate::models::a2a::*;
    use futures::StreamExt;
    use tokio_stream::wrappers::UnboundedReceiverStream;

    let log = state.log.clone();

    log.info(
        "a2a_request",
        &format!(
            "🟦 handle_streaming_request called with request type: {:?}",
            match &request {
                A2aRequestParams::SendStreamingMessage(_) => "SendStreamingMessage",
                A2aRequestParams::SendMessage(_) => "SendMessage",
                A2aRequestParams::GetTask(_) => "GetTask",
                A2aRequestParams::CancelTask(_) => "CancelTask",
                _ => "Other",
            }
        ),
    )
    .await
    .ok();

    let config = state.config.read().await;
    let agent_name = config.name.clone();
    drop(config);

    let stream = match request {
        A2aRequestParams::SendStreamingMessage(params) => {
            log.info(
                "a2a_request",
                "✅ Matched SendStreamingMessage - calling create_sse_stream",
            )
            .await
            .ok();

            if let Err(err) = validate_message_context(
                &params.message,
                Some(context.user_id().as_str()),
                &state.db_pool,
            )
            .await
            {
                log.error(
                    "a2a_request",
                    &format!("Context validation failed for streaming request: {}", err),
                )
                .await
                .ok();

                let error_event = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32602,
                        "message": "Invalid params",
                        "data": err
                    },
                    "id": request_id
                });

                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                tx.send(Event::default().data(error_event.to_string())).ok();
                return UnboundedReceiverStream::new(rx).map(Ok);
            }

            // Extract pushNotificationConfig from params (A2A spec-compliant)
            let callback_config = params
                .configuration
                .as_ref()
                .and_then(|c| c.push_notification_config.clone());

            create_sse_stream(
                params.message,
                agent_name,
                state,
                request_id,
                context,
                callback_config,
            )
            .await
            .map(Ok)
        },
        _ => {
            log.warn(
                "a2a_request",
                "❌ Did NOT match SendStreamingMessage - returning error stream",
            )
            .await
            .ok();
            let error_event = json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": "Method not found",
                    "data": "Only message/stream requests are supported for streaming"
                },
                "id": request_id
            });

            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            tx.send(Event::default().data(error_event.to_string())).ok();
            UnboundedReceiverStream::new(rx).map(Ok)
        },
    };

    stream
}

async fn should_require_oauth(
    _request: &crate::models::a2a::A2aJsonRpcRequest,
    state: &AgentHandlerState,
) -> bool {
    let config = state.config.read().await;
    config.oauth.required
}

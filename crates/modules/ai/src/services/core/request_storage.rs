use crate::models::ai::{AiRequest, AiResponse};
use crate::models::{AiRequestRecord, AiRequestRecordBuilder};
use crate::repository::AIRequestRepository;
use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, McpExecutionId, SessionId, TaskId, TraceId, UserId};

#[derive(Debug)]
pub struct RequestStorage {
    ai_request_repo: AIRequestRepository,
    session_repo: AnalyticsSessionRepository,
}

impl RequestStorage {
    pub const fn new(
        ai_request_repo: AIRequestRepository,
        session_repo: AnalyticsSessionRepository,
    ) -> Self {
        Self {
            ai_request_repo,
            session_repo,
        }
    }

    pub fn store(
        &self,
        request: &AiRequest,
        response: &AiResponse,
        context: &RequestContext,
        status: &str,
        error_message: Option<&str>,
        cost_cents: i32,
    ) {
        let record = build_record(response, context, status, error_message, cost_cents);
        let messages = extract_messages(request, response, status);
        let tool_calls = extract_tool_calls(response);
        self.spawn_storage_task(record, messages, tool_calls);
    }

    fn spawn_storage_task(
        &self,
        record: AiRequestRecord,
        messages: Vec<MessageData>,
        tool_calls: Vec<ToolCallData>,
    ) {
        let repo = self.ai_request_repo.clone();
        let session_repo = self.session_repo.clone();
        let user_id = record.user_id.clone();
        let session_id = record.session_id.clone();
        let tokens = record.tokens.tokens_used;
        let cost = record.cost_cents;
        let app_request_id = record.request_id.clone();

        tokio::spawn(async move {
            let db_id = match store_request_async(&repo, &record).await {
                Some(id) => id,
                None => {
                    tracing::error!(
                        "PERSISTENCE_FAILURE: AI request {} not stored - aborting message/tool \
                         storage",
                        app_request_id
                    );
                    return;
                },
            };

            store_messages_async(&repo, &db_id, &app_request_id, messages).await;
            store_tool_calls_async(&repo, &db_id, &app_request_id, tool_calls).await;
            update_session_usage_async(&session_repo, &user_id, session_id.as_ref(), tokens, cost)
                .await;
        });
    }
}

fn build_record(
    response: &AiResponse,
    context: &RequestContext,
    status: &str,
    error_message: Option<&str>,
    cost_cents: i32,
) -> AiRequestRecord {
    let user_id = UserId::new(context.user_id().as_str());

    let mut builder = AiRequestRecordBuilder::new(response.request_id.to_string(), user_id)
        .provider(&response.provider)
        .model(&response.model)
        .tokens(
            response.input_tokens.map(|t| t as i32),
            response.output_tokens.map(|t| t as i32),
        )
        .cache(
            response.cache_hit,
            response.cache_read_tokens.map(|t| t as i32),
            response.cache_creation_tokens.map(|t| t as i32),
        )
        .streaming(response.is_streaming)
        .cost(cost_cents)
        .latency(response.latency_ms as i32);

    if !context.session_id().as_str().is_empty() {
        builder = builder.session_id(SessionId::new(context.session_id().as_str()));
    }

    if let Some(task_id) = context.task_id() {
        builder = builder.task_id(TaskId::new(task_id.as_str()));
    }

    if !context.context_id().as_str().is_empty() {
        builder = builder.context_id(ContextId::new(context.context_id().as_str()));
    }

    if !context.trace_id().as_str().is_empty() {
        builder = builder.trace_id(TraceId::new(context.trace_id().as_str()));
    }

    if let Some(mcp_execution_id) = context.mcp_execution_id() {
        builder = builder.mcp_execution_id(McpExecutionId::new(mcp_execution_id.as_str()));
    }

    builder = match status {
        "completed" => builder.completed(),
        "failed" | "error" => {
            let msg = error_message.unwrap_or("Unknown error");
            builder.failed(msg)
        },
        _ => builder,
    };

    builder.build().unwrap_or_else(|_| {
        tracing::error!(
            "PERSISTENCE_FAILURE: Failed to build AiRequestRecord for request_id={}",
            response.request_id
        );
        AiRequestRecordBuilder::new(response.request_id.to_string(), UserId::new("unknown"))
            .provider("unknown")
            .model("unknown")
            .build()
            .expect("Fallback record should always build")
    })
}

struct MessageData {
    role: String,
    content: String,
    sequence: i32,
}

struct ToolCallData {
    ai_tool_call_id: String,
    tool_name: String,
    tool_input: String,
    sequence: i32,
}

fn extract_messages(request: &AiRequest, response: &AiResponse, status: &str) -> Vec<MessageData> {
    let mut messages = Vec::new();
    let mut sequence = 0;

    for msg in &request.messages {
        let role = match msg.role {
            crate::models::ai::MessageRole::System => "system",
            crate::models::ai::MessageRole::User => "user",
            crate::models::ai::MessageRole::Assistant => "assistant",
        };

        messages.push(MessageData {
            role: role.to_string(),
            content: msg.content.clone(),
            sequence,
        });
        sequence += 1;
    }

    if status == "completed" && !response.content.is_empty() {
        messages.push(MessageData {
            role: "assistant".to_string(),
            content: response.content.clone(),
            sequence,
        });
    }

    messages
}

fn extract_tool_calls(response: &AiResponse) -> Vec<ToolCallData> {
    response
        .tool_calls
        .iter()
        .enumerate()
        .map(|(i, tc)| ToolCallData {
            ai_tool_call_id: tc.ai_tool_call_id.as_str().to_string(),
            tool_name: tc.name.clone(),
            tool_input: serde_json::to_string(&tc.arguments).unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to serialize tool call arguments for {}: {}",
                    tc.name,
                    e
                );
                "{}".to_string()
            }),
            sequence: i as i32,
        })
        .collect()
}

async fn store_request_async(
    repo: &AIRequestRepository,
    record: &AiRequestRecord,
) -> Option<String> {
    match repo.store(record).await {
        Ok(db_id) => {
            tracing::debug!(
                "Stored AI request: db_id={}, app_request_id={}",
                db_id,
                record.request_id
            );
            Some(db_id)
        },
        Err(error) => {
            tracing::error!(
                "PERSISTENCE_FAILURE: Failed to store AI request app_request_id={}: {}",
                record.request_id,
                error
            );
            None
        },
    }
}

async fn store_messages_async(
    repo: &AIRequestRepository,
    db_id: &str,
    app_request_id: &str,
    messages: Vec<MessageData>,
) {
    let message_count = messages.len();
    let mut success_count = 0;
    let mut error_count = 0;

    for msg in messages {
        match repo
            .insert_message(db_id, &msg.role, &msg.content, msg.sequence)
            .await
        {
            Ok(_) => success_count += 1,
            Err(error) => {
                error_count += 1;
                tracing::error!(
                    "PERSISTENCE_FAILURE: Failed to store {} message seq={} for db_id={} \
                     (app_request_id={}): {}",
                    msg.role,
                    msg.sequence,
                    db_id,
                    app_request_id,
                    error
                );
            },
        }
    }

    if error_count > 0 {
        tracing::error!(
            "PERSISTENCE_FAILURE: Message storage incomplete for db_id={}: {}/{} messages stored",
            db_id,
            success_count,
            message_count
        );
    } else if message_count > 0 {
        tracing::debug!(
            "Stored {} messages for AI request db_id={}",
            message_count,
            db_id
        );
    }
}

async fn store_tool_calls_async(
    repo: &AIRequestRepository,
    db_id: &str,
    app_request_id: &str,
    tool_calls: Vec<ToolCallData>,
) {
    if tool_calls.is_empty() {
        return;
    }

    let tool_count = tool_calls.len();
    let mut success_count = 0;
    let mut error_count = 0;

    for tc in tool_calls {
        match repo
            .insert_tool_call(
                db_id,
                &tc.ai_tool_call_id,
                &tc.tool_name,
                &tc.tool_input,
                tc.sequence,
            )
            .await
        {
            Ok(_) => success_count += 1,
            Err(error) => {
                error_count += 1;
                tracing::error!(
                    "PERSISTENCE_FAILURE: Failed to store tool call {} seq={} for db_id={} \
                     (app_request_id={}): {}",
                    tc.tool_name,
                    tc.sequence,
                    db_id,
                    app_request_id,
                    error
                );
            },
        }
    }

    if error_count > 0 {
        tracing::error!(
            "PERSISTENCE_FAILURE: Tool call storage incomplete for db_id={}: {}/{} tool calls \
             stored",
            db_id,
            success_count,
            tool_count
        );
    } else {
        tracing::debug!(
            "Stored {} tool calls for AI request db_id={}",
            tool_count,
            db_id
        );
    }
}

async fn update_session_usage_async(
    session_repo: &AnalyticsSessionRepository,
    user_id: &UserId,
    session_id: Option<&SessionId>,
    tokens: Option<i32>,
    cost_cents: i32,
) {
    if user_id.as_str() == "system" {
        return;
    }

    let Some(session_id) = session_id else {
        return;
    };

    ensure_session_exists(session_repo, session_id, user_id).await;

    let tokens = tokens.unwrap_or(0);
    if let Err(error) = session_repo
        .increment_ai_usage(session_id.as_str(), tokens, cost_cents)
        .await
    {
        tracing::error!(
            "PERSISTENCE_FAILURE: Failed to increment AI usage for session {}: {}",
            session_id.as_str(),
            error
        );
    }
}

async fn ensure_session_exists(
    session_repo: &AnalyticsSessionRepository,
    session_id: &SessionId,
    user_id: &UserId,
) {
    let exists = match session_repo.session_exists(session_id.as_str()).await {
        Ok(exists) => exists,
        Err(error) => {
            tracing::error!(
                "PERSISTENCE_FAILURE: Failed to check session {}: {}",
                session_id.as_str(),
                error
            );
            return;
        },
    };

    if exists {
        return;
    }

    let jwt_expiration = systemprompt_core_system::Config::global().jwt_access_token_expiration;
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(jwt_expiration);

    if let Err(error) = session_repo
        .create_session(
            session_id.as_str(),
            Some(user_id.as_str()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            false,
            expires_at,
        )
        .await
    {
        tracing::error!(
            "PERSISTENCE_FAILURE: Failed to create session {}: {}",
            session_id.as_str(),
            error
        );
    }
}

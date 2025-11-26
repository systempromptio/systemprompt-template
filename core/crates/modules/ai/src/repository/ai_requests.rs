use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool, JsonRow};
use systemprompt_identifiers::{ContextId, SessionId, TaskId, TraceId, UserId};
use systemprompt_models::ai::{AiMessage, MessageRole, SamplingMetadata};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AiRequestRepository {
    db: Arc<dyn DatabaseProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequestMessage {
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequestToolCall {
    pub tool_name: String,
    pub tool_input: String,
    pub mcp_execution_id: Option<String>,
    pub ai_tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SamplingParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<i32>,
    pub stop_sequences: Option<String>,
}

impl AiRequestRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db: db_pool }
    }

    pub async fn store_ai_request(
        &self,
        request_id: Uuid,
        user_id: &UserId,
        session_id: &SessionId,
        task_id: Option<&TaskId>,
        context_id: Option<&ContextId>,
        trace_id: Option<&TraceId>,
        provider: &str,
        model: &str,
        messages: &[AiRequestMessage],
        sampling_params: &SamplingParams,
        tool_calls: Option<&[AiRequestToolCall]>,
        tokens_used: Option<i32>,
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
        cache_hit: bool,
        cache_read_tokens: Option<i32>,
        cache_creation_tokens: Option<i32>,
        is_streaming: bool,
        cost_cents: i32,
        latency_ms: i32,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<()> {
        let id = request_id.to_string();
        let request_id_str = request_id.to_string();

        self.db
            .execute(
                &DatabaseQueryEnum::InsertAiRequest.get(self.db.as_ref()),
                &[
                    &id,
                    &request_id_str,
                    &user_id.as_str(),
                    &session_id.as_str(),
                    &task_id.map(TaskId::as_str),
                    &context_id.map(ContextId::as_str),
                    &trace_id.map(TraceId::as_str),
                    &provider,
                    &model,
                    &sampling_params.temperature,
                    &sampling_params.top_p,
                    &sampling_params.max_tokens,
                    &sampling_params.stop_sequences,
                    &tokens_used,
                    &input_tokens,
                    &output_tokens,
                    &cache_hit,
                    &cache_read_tokens,
                    &cache_creation_tokens,
                    &is_streaming,
                    &cost_cents,
                    &latency_ms,
                    &status,
                    &error_message,
                ],
            )
            .await
            .context("Failed to insert AI request")?;

        for (idx, message) in messages.iter().enumerate() {
            self.db
                .execute(
                    &DatabaseQueryEnum::InsertRequestMessage.get(self.db.as_ref()),
                    &[
                        &request_id_str,
                        &message.role,
                        &message.content,
                        &message.name,
                        &message.tool_call_id,
                        &(idx as i32),
                    ],
                )
                .await
                .context(format!("Failed to insert request message at index {idx}"))?;
        }

        if let Some(calls) = tool_calls {
            for (idx, call) in calls.iter().enumerate() {
                self.db
                    .execute(
                        &DatabaseQueryEnum::InsertToolCall.get(self.db.as_ref()),
                        &[
                            &request_id_str,
                            &call.tool_name,
                            &call.tool_input,
                            &call.mcp_execution_id,
                            &call.ai_tool_call_id,
                            &(idx as i32),
                        ],
                    )
                    .await
                    .context(format!(
                        "Failed to insert tool call '{}' at index {}",
                        call.tool_name, idx
                    ))?;
            }
        }

        Ok(())
    }

    pub async fn store_image_request(
        &self,
        request_id: &str,
        user_id: &str,
        session_id: Option<&str>,
        trace_id: Option<&str>,
        provider: &str,
        model: &str,
        cost_cents: Option<i32>,
        latency_ms: Option<i32>,
        status: &str,
        image_count: Option<i32>,
    ) -> Result<()> {
        self.db
            .execute(
                &DatabaseQueryEnum::InsertAiImageRequest.get(self.db.as_ref()),
                &[
                    &request_id,
                    &request_id,
                    &user_id,
                    &session_id,
                    &trace_id,
                    &provider,
                    &model,
                    &cost_cents,
                    &latency_ms,
                    &status,
                    &image_count,
                ],
            )
            .await
            .context("Failed to insert AI image request")?;

        Ok(())
    }

    pub async fn update_ai_request_completion(
        &self,
        request_id: Uuid,
        tokens_used: Option<i32>,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<()> {
        let request_id_str = request_id.to_string();

        self.db
            .execute(
                &DatabaseQueryEnum::UpdateAiRequestStatus.get(self.db.as_ref()),
                &[&tokens_used, &status, &error_message, &request_id_str],
            )
            .await
            .context("Failed to update AI request status")?;

        Ok(())
    }

    pub async fn add_response_message(&self, request_id: Uuid, content: &str) -> Result<()> {
        let request_id_str = request_id.to_string();

        let query = DatabaseQueryEnum::GetAiMessageMaxSequence.get(self.db.as_ref());
        let row = self
            .db
            .fetch_one(&query, &[&request_id_str])
            .await
            .context("Failed to fetch max message sequence")?;
        let max_seq = row
            .get("max_seq")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow::anyhow!("Missing max_seq in query result"))?;
        let next_seq = (max_seq + 1) as i32;

        let insert_query = DatabaseQueryEnum::InsertResponseMessage.get(self.db.as_ref());
        self.db
            .execute(&insert_query, &[&request_id_str, &content, &next_seq])
            .await
            .context("Failed to insert response message")?;

        Ok(())
    }

    pub async fn get_messages(&self, request_id: Uuid) -> Result<Vec<AiRequestMessage>> {
        let request_id_str = request_id.to_string();
        let query = DatabaseQueryEnum::GetRequestMessages.get(self.db.as_ref());
        let rows = self
            .db
            .fetch_all(&query, &[&request_id_str])
            .await
            .context("Failed to fetch request messages")?;

        rows.into_iter()
            .map(|row| {
                Ok(AiRequestMessage {
                    role: row
                        .get("role")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("Missing role"))?
                        .to_string(),
                    content: row
                        .get("content")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("Missing content"))?
                        .to_string(),
                    name: row.get("name").and_then(|v| v.as_str()).map(String::from),
                    tool_call_id: row
                        .get("tool_call_id")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                })
            })
            .collect()
    }

    pub async fn get_tool_calls(&self, request_id: Uuid) -> Result<Vec<AiRequestToolCall>> {
        let request_id_str = request_id.to_string();
        let query = DatabaseQueryEnum::GetToolCalls.get(self.db.as_ref());
        let rows = self
            .db
            .fetch_all(&query, &[&request_id_str])
            .await
            .context("Failed to fetch tool calls")?;

        rows.into_iter()
            .map(|row| {
                Ok(AiRequestToolCall {
                    tool_name: row
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("Missing tool_name"))?
                        .to_string(),
                    tool_input: row
                        .get("tool_input")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow!("Missing tool_input"))?
                        .to_string(),
                    mcp_execution_id: row
                        .get("mcp_execution_id")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    ai_tool_call_id: row
                        .get("ai_tool_call_id")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                })
            })
            .collect()
    }

    pub async fn get_user_ai_usage(
        &self,
        user_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Vec<AiUsageSummary>> {
        let query = match (start_date, end_date) {
            (Some(_), Some(_)) => DatabaseQueryEnum::GetUserAiUsageWithDateRange,
            (Some(_), None) => DatabaseQueryEnum::GetUserAiUsageSinceDate,
            (None, Some(_)) => DatabaseQueryEnum::GetUserAiUsageUntilDate,
            (None, None) => DatabaseQueryEnum::GetUserAiUsageAll,
        }
        .get(self.db.as_ref());

        let rows = match (start_date, end_date) {
            (Some(start), Some(end)) => self
                .db
                .fetch_all(&query, &[&user_id, &start, &end])
                .await
                .context("Failed to fetch user AI usage with date range")?,
            (Some(start), None) => self
                .db
                .fetch_all(&query, &[&user_id, &start])
                .await
                .context("Failed to fetch user AI usage since date")?,
            (None, Some(end)) => self
                .db
                .fetch_all(&query, &[&user_id, &end])
                .await
                .context("Failed to fetch user AI usage until date")?,
            (None, None) => self
                .db
                .fetch_all(&query, &[&user_id])
                .await
                .context("Failed to fetch all user AI usage")?,
        };

        rows.into_iter()
            .map(|row| AiUsageSummary::from_json_row(&row))
            .collect()
    }

    pub async fn get_session_ai_usage(&self, session_id: &str) -> Result<SessionAiSummary> {
        let row = self
            .db
            .fetch_one(
                &DatabaseQueryEnum::ListAiRequestsBySession.get(self.db.as_ref()),
                &[&session_id],
            )
            .await
            .context("Failed to fetch session AI usage")?;

        SessionAiSummary::from_json_row(&row)
    }

    pub async fn get_cost_summary_by_user(&self, days: i32) -> Result<Vec<UserCostSummary>> {
        let rows = self
            .db
            .fetch_all(
                &DatabaseQueryEnum::ListAiRequestsBySession.get(self.db.as_ref()),
                &[&days],
            )
            .await
            .context("Failed to fetch cost summary by user")?;

        rows.into_iter()
            .map(|row| UserCostSummary::from_json_row(&row))
            .collect()
    }

    pub async fn get_provider_usage(
        &self,
        days: i32,
        user_id: Option<&str>,
    ) -> Result<Vec<ProviderUsage>> {
        let query = match user_id {
            Some(_) => DatabaseQueryEnum::GetProviderUsageByUser,
            None => DatabaseQueryEnum::GetProviderUsageAll,
        }
        .get(self.db.as_ref());

        let rows = match user_id {
            Some(uid) => self
                .db
                .fetch_all(&query, &[&days, &uid])
                .await
                .context("Failed to fetch provider usage by user")?,
            None => self
                .db
                .fetch_all(&query, &[&days])
                .await
                .context("Failed to fetch provider usage for all users")?,
        };

        rows.into_iter()
            .map(|row| ProviderUsage::from_json_row(&row))
            .collect()
    }

    pub async fn link_ai_tool_call_by_provider_id(
        &self,
        ai_tool_call_id: &str,
        mcp_execution_id: &str,
    ) -> Result<bool> {
        let query = r"
            UPDATE ai_request_tool_calls
            SET mcp_execution_id = $1,
                updated_at = CURRENT_TIMESTAMP
            WHERE ai_tool_call_id = $2
              AND mcp_execution_id IS NULL
            RETURNING id
        ";

        let rows = self
            .db
            .fetch_all(&query, &[&mcp_execution_id, &ai_tool_call_id])
            .await
            .context("Failed to link AI tool call by provider ID")?;

        Ok(!rows.is_empty())
    }

    pub async fn link_tool_calls_to_recent_executions(
        &self,
        ai_tool_call_ids: &[String],
    ) -> Result<usize> {
        if ai_tool_call_ids.is_empty() {
            return Ok(0);
        }

        let mut total_linked = 0;
        for ai_call_id in ai_tool_call_ids {
            let query = r"
                UPDATE ai_request_tool_calls artc
                SET mcp_execution_id = (
                    SELECT mcp_execution_id
                    FROM mcp_tool_executions mte
                    WHERE mte.ai_tool_call_id = $1
                    AND mte.status = 'success'
                    ORDER BY mte.completed_at DESC
                    LIMIT 1
                ),
                updated_at = CURRENT_TIMESTAMP
                WHERE artc.ai_tool_call_id = $1
                AND artc.mcp_execution_id IS NULL
                RETURNING id
            ";

            let rows = self
                .db
                .fetch_all(&query, &[&ai_call_id])
                .await
                .ok()
                .unwrap_or_default();
            total_linked += rows.len();
        }

        Ok(total_linked)
    }
}

#[derive(Debug)]
pub struct AiUsageSummary {
    pub provider: String,
    pub model: String,
    pub request_count: i32,
    pub total_tokens: Option<i32>,
    pub total_cost_cents: Option<i32>,
    pub avg_latency_ms: Option<f64>,
    pub usage_date: String,
}

impl AiUsageSummary {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            provider: row
                .get("provider")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing provider"))?
                .to_string(),
            model: row
                .get("model")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing model"))?
                .to_string(),
            request_count: row
                .get("request_count")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing request_count"))? as i32,
            total_tokens: row
                .get("total_tokens")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            total_cost_cents: row
                .get("total_cost_cents")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            avg_latency_ms: row.get("avg_latency_ms").and_then(serde_json::Value::as_f64),
            usage_date: row
                .get("usage_date")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing usage_date"))?
                .to_string(),
        })
    }
}

#[derive(Debug)]
pub struct SessionAiSummary {
    pub request_count: i32,
    pub total_tokens: Option<i32>,
    pub total_cost_cents: Option<i32>,
    pub avg_latency_ms: Option<f64>,
    pub first_request: Option<String>,
    pub last_request: Option<String>,
}

impl SessionAiSummary {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            request_count: row
                .get("request_count")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing request_count"))? as i32,
            total_tokens: row
                .get("total_tokens")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            total_cost_cents: row
                .get("total_cost_cents")
                .and_then(serde_json::Value::as_i64)
                .map(|v| v as i32),
            avg_latency_ms: row.get("avg_latency_ms").and_then(serde_json::Value::as_f64),
            first_request: row
                .get("first_request")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            last_request: row
                .get("last_request")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
        })
    }
}

#[derive(Debug)]
pub struct UserCostSummary {
    pub user_id: String,
    pub request_count: i32,
    pub total_tokens: Option<i32>,
    pub total_cost_cents: Option<i32>,
    pub avg_latency_ms: Option<f64>,
}

#[derive(Debug)]
pub struct ProviderUsage {
    pub provider: String,
    pub model: String,
    pub request_count: i32,
    pub total_tokens: Option<i32>,
    pub total_cost_cents: Option<i32>,
    pub avg_latency_ms: Option<f64>,
}

impl UserCostSummary {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            user_id: row
                .get("user_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing user_id"))?
                .to_string(),
            request_count: row
                .get("request_count")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing request_count"))? as i32,
            total_tokens: row
                .get("total_tokens")
                .and_then(serde_json::Value::as_i64)
                .map(|i| i as i32),
            total_cost_cents: row
                .get("total_cost_cents")
                .and_then(serde_json::Value::as_i64)
                .map(|i| i as i32),
            avg_latency_ms: row.get("avg_latency_ms").and_then(serde_json::Value::as_f64),
        })
    }
}

impl ProviderUsage {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        Ok(Self {
            provider: row
                .get("provider")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing provider"))?
                .to_string(),
            model: row
                .get("model")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing model"))?
                .to_string(),
            request_count: row
                .get("request_count")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing request_count"))? as i32,
            total_tokens: row
                .get("total_tokens")
                .and_then(serde_json::Value::as_i64)
                .map(|i| i as i32),
            total_cost_cents: row
                .get("total_cost_cents")
                .and_then(serde_json::Value::as_i64)
                .map(|i| i as i32),
            avg_latency_ms: row.get("avg_latency_ms").and_then(serde_json::Value::as_f64),
        })
    }
}

impl From<&AiMessage> for AiRequestMessage {
    fn from(msg: &AiMessage) -> Self {
        Self {
            role: match msg.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
            },
            content: msg.content.clone(),
            name: None,
            tool_call_id: None,
        }
    }
}

impl From<&SamplingMetadata> for SamplingParams {
    fn from(metadata: &SamplingMetadata) -> Self {
        Self {
            temperature: metadata.temperature.map(f64::from),
            top_p: metadata.top_p.map(f64::from),
            max_tokens: None,
            stop_sequences: metadata
                .stop_sequences
                .as_ref()
                .and_then(|seq| serde_json::to_string(seq).ok()),
        }
    }
}

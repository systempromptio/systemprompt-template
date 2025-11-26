use anyhow::{anyhow, Result};
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool, JsonRow};
use systemprompt_identifiers::{SessionId, UserId};
use systemprompt_models::Config;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug, Clone)]
pub struct AnalyticsSessionRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for AnalyticsSessionRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl AnalyticsSessionRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn create_authenticated_session(
        &self,
        session_id: &SessionId,
        user_id: &UserId,
        fingerprint_hash: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        device_type: Option<&str>,
        browser: Option<&str>,
        os: Option<&str>,
        country: Option<&str>,
        region: Option<&str>,
        city: Option<&str>,
        preferred_locale: Option<&str>,
        referrer_source: Option<&str>,
        referrer_url: Option<&str>,
        landing_page: Option<&str>,
        entry_url: Option<&str>,
        utm_source: Option<&str>,
        utm_medium: Option<&str>,
        utm_campaign: Option<&str>,
        is_bot: bool,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        self.create_session(
            session_id.as_str(),
            Some(user_id.as_str()),
            fingerprint_hash,
            ip_address,
            user_agent,
            device_type,
            browser,
            os,
            country,
            region,
            city,
            preferred_locale,
            referrer_source,
            referrer_url,
            landing_page,
            entry_url,
            utm_source,
            utm_medium,
            utm_campaign,
            is_bot,
            expires_at,
        )
        .await
    }

    pub async fn create_session(
        &self,
        session_id: &str,
        user_id: Option<&str>,
        fingerprint_hash: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        device_type: Option<&str>,
        browser: Option<&str>,
        os: Option<&str>,
        country: Option<&str>,
        region: Option<&str>,
        city: Option<&str>,
        preferred_locale: Option<&str>,
        referrer_source: Option<&str>,
        referrer_url: Option<&str>,
        landing_page: Option<&str>,
        entry_url: Option<&str>,
        utm_source: Option<&str>,
        utm_medium: Option<&str>,
        utm_campaign: Option<&str>,
        is_bot: bool,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let query = DatabaseQueryEnum::CreateAnalyticsSession.get(self.db_pool.as_ref());

        self.db_pool
            .execute(
                &query,
                &[
                    &session_id,
                    &user_id,
                    &fingerprint_hash,
                    &ip_address,
                    &user_agent,
                    &device_type,
                    &browser,
                    &os,
                    &country,
                    &region,
                    &city,
                    &preferred_locale,
                    &referrer_source,
                    &referrer_url,
                    &landing_page,
                    &entry_url,
                    &utm_source,
                    &utm_medium,
                    &utm_campaign,
                    &is_bot,
                    &expires_at,
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn session_exists(&self, session_id: &str) -> Result<bool> {
        let query = DatabaseQueryEnum::SessionExists.get(self.db_pool.as_ref());

        let result = self.db_pool.fetch_optional(&query, &[&session_id]).await?;

        Ok(result.is_some())
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>> {
        let query = DatabaseQueryEnum::GetAnalyticsSession.get(self.db_pool.as_ref());

        let result = self.db_pool.fetch_optional(&query, &[&session_id]).await?;

        result.map(|r| SessionRecord::from_json_row(&r)).transpose()
    }

    pub async fn update_session_activity(
        &self,
        session_id: &str,
        endpoint: &str,
        response_time_ms: u64,
        is_success: bool,
    ) -> Result<()> {
        let query = DatabaseQueryEnum::UpdateSessionActivity.get(self.db_pool.as_ref());

        let error_increment = i32::from(!is_success);

        self.db_pool
            .execute(
                &query,
                &[
                    &(response_time_ms as i64),
                    &(response_time_ms as i64),
                    &error_increment,
                    &error_increment,
                    &session_id,
                ],
            )
            .await?;

        let query = DatabaseQueryEnum::UpdateSessionEndpoints.get(self.db_pool.as_ref());

        self.db_pool
            .execute(&query, &[&endpoint, &endpoint, &endpoint, &session_id])
            .await?;

        Ok(())
    }

    pub async fn record_endpoint_request(
        &self,
        session_id: &str,
        endpoint_path: &str,
        http_method: &str,
        response_status: u16,
        response_time_ms: u64,
    ) -> Result<()> {
        let query = DatabaseQueryEnum::RecordEndpointRequest.get(self.db_pool.as_ref());

        self.db_pool
            .execute(
                &query,
                &[
                    &session_id,
                    &endpoint_path,
                    &http_method,
                    &i32::from(response_status),
                    &(response_time_ms as i32),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn increment_ai_usage(
        &self,
        session_id: &str,
        tokens_used: i32,
        cost_cents: i32,
    ) -> Result<()> {
        let query = DatabaseQueryEnum::IncrementSessionAiUsage.get(self.db_pool.as_ref());

        self.db_pool
            .execute(&query, &[&tokens_used, &cost_cents, &session_id])
            .await?;

        Ok(())
    }

    pub async fn increment_task_activity(
        &self,
        session_id: &str,
        task_count: i32,
        message_count: i32,
    ) -> Result<()> {
        let query = DatabaseQueryEnum::IncrementSessionTaskActivity.get(self.db_pool.as_ref());

        self.db_pool
            .execute(&query, &[&task_count, &message_count, &session_id])
            .await?;

        Ok(())
    }

    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let query = DatabaseQueryEnum::EndAnalyticsSession.get(self.db_pool.as_ref());

        self.db_pool.execute(&query, &[&session_id]).await?;

        Ok(())
    }

    pub async fn get_active_sessions(&self, limit: Option<i32>) -> Result<Vec<SessionRecord>> {
        let base_query = DatabaseQueryEnum::GetAnalyticsActiveSessions.get(self.db_pool.as_ref());
        let mut query = base_query.to_string();

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let rows = self.db_pool.fetch_all(&query, &[]).await?;

        rows.iter()
            .map(SessionRecord::from_json_row)
            .collect()
    }

    pub async fn cleanup_inactive_sessions(&self, hours_threshold: i32) -> Result<u64> {
        let query = DatabaseQueryEnum::CleanupInactiveAnalyticsSessions.get(self.db_pool.as_ref());

        let result = self.db_pool.execute(&query, &[&hours_threshold]).await?;

        Ok(result)
    }

    pub async fn cleanup_expired_anonymous_sessions(&self) -> Result<u64> {
        let mut tx = self.db_pool.begin_transaction().await?;

        let query = DatabaseQueryEnum::CleanupExpiredAnonymousSessions.get(self.db_pool.as_ref());
        let expired_sessions = tx.fetch_all(&query, &[]).await?;

        let session_ids: Vec<String> = expired_sessions
            .iter()
            .filter_map(|r| r.get("session_id")?.as_str().map(ToString::to_string))
            .collect();

        if session_ids.is_empty() {
            return Ok(0);
        }

        let mut deleted = 0u64;

        for session_id in session_ids {
            let query = DatabaseQueryEnum::DeleteContextBySession.get(self.db_pool.as_ref());
            tx.execute(&query, &[&session_id]).await?;

            let query = DatabaseQueryEnum::DeleteSessionById.get(self.db_pool.as_ref());
            deleted += tx.execute(&query, &[&session_id]).await?;
        }

        tx.commit().await?;

        Ok(deleted)
    }

    pub async fn migrate_session_to_registered_user(
        &self,
        session_id: &str,
        old_user_id: &str,
        new_user_id: &str,
    ) -> Result<SessionMigrationResult> {
        tracing::info!(
            "Starting session migration: session_id={}, old_user_id={}, new_user_id={}",
            session_id,
            old_user_id,
            new_user_id
        );

        let mut tx = self.db_pool.begin_transaction().await?;

        let migrate_query = DatabaseQueryEnum::MigrateSessionToUser.get(self.db_pool.as_ref());
        tx.execute(&migrate_query, &[&new_user_id, &session_id])
            .await?;

        tracing::debug!("Session record updated: session_id={}", session_id);

        let query = DatabaseQueryEnum::UpdateSessionContexts.get(self.db_pool.as_ref());
        let contexts = tx.execute(&query, &[&new_user_id, &old_user_id]).await?;
        tracing::debug!(
            "Migrated {} contexts from {} to {}",
            contexts,
            old_user_id,
            new_user_id
        );

        let query = DatabaseQueryEnum::UpdateSessionUserAgentTasks.get(self.db_pool.as_ref());
        let tasks = tx.execute(&query, &[&new_user_id, &old_user_id]).await?;
        tracing::debug!(
            "Migrated {} agent tasks from {} to {}",
            tasks,
            old_user_id,
            new_user_id
        );

        let query = DatabaseQueryEnum::UpdateSessionUserTaskMessages.get(self.db_pool.as_ref());
        let messages = tx.execute(&query, &[&new_user_id, &old_user_id]).await?;
        tracing::debug!(
            "Migrated {} task messages from {} to {}",
            messages,
            old_user_id,
            new_user_id
        );

        let query = DatabaseQueryEnum::UpdateSessionUserAiRequests.get(self.db_pool.as_ref());
        let ai_requests = tx.execute(&query, &[&new_user_id, &old_user_id]).await?;
        tracing::debug!(
            "Migrated {} AI requests from {} to {}",
            ai_requests,
            old_user_id,
            new_user_id
        );

        let query = DatabaseQueryEnum::UpdateSessionUserLogs.get(self.db_pool.as_ref());
        let logs = tx.execute(&query, &[&new_user_id, &old_user_id]).await?;
        tracing::debug!(
            "Migrated {} logs from {} to {}",
            logs,
            old_user_id,
            new_user_id
        );

        let query = DatabaseQueryEnum::UpdateSessionUserToolExecutions.get(self.db_pool.as_ref());
        let tool_executions = tx.execute(&query, &[&new_user_id, &old_user_id]).await?;
        tracing::debug!(
            "Migrated {} tool executions from {} to {}",
            tool_executions,
            old_user_id,
            new_user_id
        );

        let query = DatabaseQueryEnum::DeleteTemporarySessionUser.get(self.db_pool.as_ref());
        let deleted = tx.execute(&query, &[&old_user_id]).await?;
        tracing::debug!(
            "Deleted {} temporary user records for {}",
            deleted,
            old_user_id
        );

        tx.commit().await?;

        let result = SessionMigrationResult {
            contexts,
            tasks,
            messages,
            ai_requests,
            logs,
            tool_executions,
        };

        tracing::info!(
            "Session migration completed: session_id={}, contexts={}, tasks={}, messages={}, ai_requests={}, logs={}, tool_executions={}",
            session_id,
            result.contexts,
            result.tasks,
            result.messages,
            result.ai_requests,
            result.logs,
            result.tool_executions
        );

        Ok(result)
    }

    pub async fn find_recent_session_by_fingerprint(
        &self,
        fingerprint: &str,
        max_age_seconds: i64,
    ) -> Result<Option<SessionRecord>> {
        let query = DatabaseQueryEnum::FindSessionByFingerprint.get(self.db_pool.as_ref());

        let result = self
            .db_pool
            .fetch_optional(&query, &[&fingerprint, &max_age_seconds])
            .await?;

        result.map(|r| SessionRecord::from_json_row(&r)).transpose()
    }

    pub async fn get_or_create_public_session(
        &self,
        session_id: &str,
        fingerprint_hash: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        device_type: Option<&str>,
        browser: Option<&str>,
        os: Option<&str>,
        country: Option<&str>,
        region: Option<&str>,
        city: Option<&str>,
        preferred_locale: Option<&str>,
        referrer_source: Option<&str>,
        referrer_url: Option<&str>,
        landing_page: Option<&str>,
        entry_url: Option<&str>,
        utm_source: Option<&str>,
        utm_medium: Option<&str>,
        utm_campaign: Option<&str>,
        is_bot: bool,
    ) -> Result<String> {
        let jwt_expiration_seconds = Config::global().jwt_access_token_expiration;

        if let Some(fingerprint) = fingerprint_hash {
            if let Some(existing_session) = self
                .find_recent_session_by_fingerprint(fingerprint, jwt_expiration_seconds)
                .await?
            {
                return Ok(existing_session.session_id);
            }
        }

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(jwt_expiration_seconds);
        self.create_session(
            session_id,
            None,
            fingerprint_hash,
            ip_address,
            user_agent,
            device_type,
            browser,
            os,
            country,
            region,
            city,
            preferred_locale,
            referrer_source,
            referrer_url,
            landing_page,
            entry_url,
            utm_source,
            utm_medium,
            utm_campaign,
            is_bot,
            expires_at,
        )
        .await?;

        Ok(session_id.to_string())
    }

    pub async fn mark_as_scanner(&self, session_id: &str) -> Result<()> {
        let query = DatabaseQueryEnum::MarkSessionAsScanner.get(self.db_pool.as_ref());
        self.db_pool.execute(&query, &[&session_id]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SessionMigrationResult {
    pub contexts: u64,
    pub tasks: u64,
    pub messages: u64,
    pub ai_requests: u64,
    pub logs: u64,
    pub tool_executions: u64,
}

impl SessionMigrationResult {
    pub const fn total_records_migrated(&self) -> u64 {
        self.contexts
            + self.tasks
            + self.messages
            + self.ai_requests
            + self.logs
            + self.tool_executions
    }
}

#[derive(Debug)]
pub struct SessionRecord {
    pub session_id: String,
    pub user_id: Option<String>,
    pub started_at: String,
    pub last_activity_at: String,
    pub ended_at: Option<String>,
    pub request_count: i32,
    pub task_count: i32,
    pub message_count: i32,
    pub error_count: i32,
    pub ai_request_count: i32,
    pub total_tokens_used: i32,
    pub total_ai_cost_cents: i32,
    pub avg_response_time_ms: Option<f64>,
    pub success_rate: Option<f64>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub endpoints_accessed: String,
}

impl SessionRecord {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let session_id = row
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing session_id"))?
            .to_string();

        let user_id = row
            .get("user_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let started_at = row
            .get("started_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing started_at"))?
            .to_string();

        let last_activity_at = row
            .get("last_activity_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing last_activity_at"))?
            .to_string();

        let ended_at = row
            .get("ended_at")
            .and_then(|v| v.as_str())
            .map(String::from);

        let request_count = row
            .get("request_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing request_count"))? as i32;

        let task_count = row
            .get("task_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing task_count"))? as i32;

        let message_count = row
            .get("message_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing message_count"))? as i32;

        let error_count = row
            .get("error_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing error_count"))? as i32;

        let ai_request_count =
            row.get("ai_request_count")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing ai_request_count"))? as i32;

        let total_tokens_used =
            row.get("total_tokens_used")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing total_tokens_used"))? as i32;

        let total_ai_cost_cents =
            row.get("total_ai_cost_cents")
                .and_then(serde_json::Value::as_i64)
                .ok_or_else(|| anyhow!("Missing total_ai_cost_cents"))? as i32;

        let avg_response_time_ms = row.get("avg_response_time_ms").and_then(serde_json::Value::as_f64);

        let success_rate = row.get("success_rate").and_then(serde_json::Value::as_f64);

        let device_type = row
            .get("device_type")
            .and_then(|v| v.as_str())
            .map(String::from);

        let browser = row
            .get("browser")
            .and_then(|v| v.as_str())
            .map(String::from);

        let os = row.get("os").and_then(|v| v.as_str()).map(String::from);

        let country = row
            .get("country")
            .and_then(|v| v.as_str())
            .map(String::from);

        let endpoints_accessed = row
            .get("endpoints_accessed")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing endpoints_accessed"))?
            .to_string();

        Ok(Self {
            session_id,
            user_id,
            started_at,
            last_activity_at,
            ended_at,
            request_count,
            task_count,
            message_count,
            error_count,
            ai_request_count,
            total_tokens_used,
            total_ai_cost_cents,
            avg_response_time_ms,
            success_rate,
            device_type,
            browser,
            os,
            country,
            endpoints_accessed,
        })
    }
}

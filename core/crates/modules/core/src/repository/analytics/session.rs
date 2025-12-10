use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use systemprompt_core_database::DbPool;

use crate::models::analytics::AnalyticsSession;

#[derive(Clone, Debug)]
pub struct SessionRepository {
    pool: DbPool,
}

impl SessionRepository {
    pub const fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Option<&str>,
        fingerprint_hash: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        referrer_url: Option<&str>,
    ) -> Result<AnalyticsSession> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        let session_id = uuid::Uuid::new_v4().to_string();
        sqlx::query_as!(
            AnalyticsSession,
            r#"
            INSERT INTO user_sessions (
                session_id, user_id, fingerprint_hash, ip_address, user_agent, referrer_url,
                started_at, last_activity_at, request_count, task_count,
                ai_request_count, message_count, is_bot, is_scanner
            )
            VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 0, 0, 0, 0, false, false)
            RETURNING session_id, user_id, fingerprint_hash, ip_address, user_agent, device_type,
                      browser, os, country, city, referrer_url, utm_source, utm_medium,
                      utm_campaign, is_bot, is_scanner, started_at, last_activity_at,
                      ended_at, request_count, task_count, ai_request_count, message_count
            "#,
            session_id,
            user_id,
            fingerprint_hash,
            ip_address,
            user_agent,
            referrer_url
        )
        .fetch_one(pool.as_ref())
        .await
        .map_err(Into::into)
    }

    pub async fn get(&self, session_id: &str) -> Result<Option<AnalyticsSession>> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query_as!(
            AnalyticsSession,
            r#"
            SELECT session_id, user_id, fingerprint_hash, ip_address, user_agent, device_type,
                   browser, os, country, city, referrer_url, utm_source, utm_medium,
                   utm_campaign, is_bot, is_scanner, started_at, last_activity_at,
                   ended_at, request_count, task_count, ai_request_count, message_count
            FROM user_sessions
            WHERE session_id = $1
            "#,
            session_id
        )
        .fetch_optional(pool.as_ref())
        .await
        .map_err(Into::into)
    }

    pub async fn find_by_fingerprint(
        &self,
        fingerprint_hash: &str,
        user_id: &str,
    ) -> Result<Option<AnalyticsSession>> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query_as!(
            AnalyticsSession,
            r#"
            SELECT session_id, user_id, fingerprint_hash, ip_address, user_agent, device_type,
                   browser, os, country, city, referrer_url, utm_source, utm_medium,
                   utm_campaign, is_bot, is_scanner, started_at, last_activity_at,
                   ended_at, request_count, task_count, ai_request_count, message_count
            FROM user_sessions
            WHERE fingerprint_hash = $1 AND user_id = $2 AND ended_at IS NULL
            ORDER BY last_activity_at DESC
            LIMIT 1
            "#,
            fingerprint_hash,
            user_id
        )
        .fetch_optional(pool.as_ref())
        .await
        .map_err(Into::into)
    }

    pub async fn get_active_sessions(&self, user_id: &str) -> Result<Vec<AnalyticsSession>> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query_as!(
            AnalyticsSession,
            r#"
            SELECT session_id, user_id, fingerprint_hash, ip_address, user_agent, device_type,
                   browser, os, country, city, referrer_url, utm_source, utm_medium,
                   utm_campaign, is_bot, is_scanner, started_at, last_activity_at,
                   ended_at, request_count, task_count, ai_request_count, message_count
            FROM user_sessions
            WHERE user_id = $1 AND ended_at IS NULL
            ORDER BY last_activity_at DESC
            "#,
            user_id
        )
        .fetch_all(pool.as_ref())
        .await
        .map_err(Into::into)
    }

    pub async fn update_activity(&self, session_id: &str) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            "UPDATE user_sessions SET last_activity_at = CURRENT_TIMESTAMP WHERE session_id = $1",
            session_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn increment_request_count(&self, session_id: &str) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            "UPDATE user_sessions SET request_count = request_count + 1, last_activity_at = \
             CURRENT_TIMESTAMP WHERE session_id = $1",
            session_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn increment_task_count(&self, session_id: &str) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            "UPDATE user_sessions SET task_count = task_count + 1, last_activity_at = \
             CURRENT_TIMESTAMP WHERE session_id = $1",
            session_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn increment_ai_request_count(&self, session_id: &str) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            "UPDATE user_sessions SET ai_request_count = ai_request_count + 1, last_activity_at = \
             CURRENT_TIMESTAMP WHERE session_id = $1",
            session_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn increment_message_count(&self, session_id: &str) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            "UPDATE user_sessions SET message_count = message_count + 1, last_activity_at = \
             CURRENT_TIMESTAMP WHERE session_id = $1",
            session_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            "UPDATE user_sessions SET ended_at = CURRENT_TIMESTAMP WHERE session_id = $1",
            session_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn mark_as_scanner(&self, session_id: &str) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            "UPDATE user_sessions SET is_scanner = true WHERE session_id = $1",
            session_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn cleanup_inactive(&self, inactive_hours: i32) -> Result<u64> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        let cutoff = Utc::now() - Duration::hours(inactive_hours as i64);
        let result = sqlx::query!(
            r#"
            UPDATE user_sessions
            SET ended_at = CURRENT_TIMESTAMP
            WHERE ended_at IS NULL AND last_activity_at < $1
            "#,
            cutoff
        )
        .execute(pool.as_ref())
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn migrate_session(&self, old_user_id: &str, new_user_id: &str) -> Result<u64> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        let result = sqlx::query!(
            "UPDATE user_sessions SET user_id = $1 WHERE user_id = $2",
            new_user_id,
            old_user_id
        )
        .execute(pool.as_ref())
        .await?;
        Ok(result.rows_affected())
    }

    #[allow(clippy::too_many_arguments)]
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
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        sqlx::query!(
            r#"
            INSERT INTO user_sessions (
                session_id, user_id, fingerprint_hash, ip_address, user_agent,
                device_type, browser, os, country, region, city, preferred_locale,
                referrer_source, referrer_url, landing_page, entry_url,
                utm_source, utm_medium, utm_campaign, is_bot, expires_at,
                started_at, last_activity_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
            session_id,
            user_id,
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
            expires_at
        )
        .execute(pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn find_recent_session_by_fingerprint(
        &self,
        fingerprint_hash: &str,
        max_age_seconds: i64,
    ) -> Result<Option<SessionRecord>> {
        let pool = self
            .pool
            .pool_arc()
            .context("Failed to get database pool")?;
        let cutoff = Utc::now() - Duration::seconds(max_age_seconds);
        sqlx::query_as!(
            SessionRecord,
            r#"
            SELECT session_id, user_id, expires_at
            FROM user_sessions
            WHERE fingerprint_hash = $1
              AND last_activity_at > $2
              AND ended_at IS NULL
            ORDER BY last_activity_at DESC
            LIMIT 1
            "#,
            fingerprint_hash,
            cutoff
        )
        .fetch_optional(pool.as_ref())
        .await
        .map_err(Into::into)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<AnalyticsSession>> {
        self.get(session_id).await
    }

    pub async fn migrate_session_to_registered_user(
        &self,
        _session_id: &str,
        old_user_id: &str,
        new_user_id: &str,
    ) -> Result<SessionMigrationResult> {
        let sessions_migrated = self.migrate_session(old_user_id, new_user_id).await?;
        Ok(SessionMigrationResult { sessions_migrated })
    }

    pub async fn session_exists(&self, session_id: &str) -> Result<bool> {
        let pool = self.pool.pool_arc().context("Failed to get pool")?;
        let result = sqlx::query_scalar!(
            r#"SELECT 1 as "exists" FROM user_sessions WHERE session_id = $1 LIMIT 1"#,
            session_id
        )
        .fetch_optional(&*pool)
        .await?;
        Ok(result.is_some())
    }

    pub async fn increment_ai_usage(
        &self,
        session_id: &str,
        tokens: i32,
        cost_cents: i32,
    ) -> Result<()> {
        let pool = self.pool.pool_arc().context("Failed to get pool")?;
        let cost_cents_i64 = i64::from(cost_cents);
        sqlx::query!(
            r#"
            UPDATE user_sessions
            SET ai_request_count = COALESCE(ai_request_count, 0) + 1,
                total_tokens_used = COALESCE(total_tokens_used, 0) + $1,
                total_ai_cost_cents = COALESCE(total_ai_cost_cents, 0) + $2,
                last_activity_at = CURRENT_TIMESTAMP
            WHERE session_id = $3
            "#,
            tokens,
            cost_cents_i64,
            session_id
        )
        .execute(&*pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionRecord {
    pub session_id: String,
    pub user_id: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionMigrationResult {
    pub sessions_migrated: u64,
}

impl SessionMigrationResult {
    pub const fn total_records_migrated(&self) -> u64 {
        self.sessions_migrated
    }
}

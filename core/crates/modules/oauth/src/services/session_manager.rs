use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sha2::{Sha256, Digest};
use uuid::Uuid;

use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_core_system::services::SessionAnalytics;
use systemprompt_core_users::repository::UserRepository;
use systemprompt_identifiers::{SessionId, UserId, ClientId};

use crate::services::generation::tokens::generate_anonymous_jwt;

/// DEPRECATED: Use `SessionCreationService` instead.
/// This struct is kept for backward compatibility but is no longer actively used.
/// It duplicates logic from `SessionCreationService` and should be removed in a future version.
#[deprecated(
    since = "0.1.0",
    note = "Use SessionCreationService instead - this has duplicate logic and will be removed"
)]
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub is_new: bool,
    pub jwt_token: String,
}

#[derive(Debug, Clone, Copy)]
pub enum SessionType {
    Anonymous,
    Authenticated,
}

/// DEPRECATED: Use `SessionCreationService` instead.
/// This struct duplicates functionality and should not be used for new code.
#[deprecated(
    since = "0.1.0",
    note = "Use SessionCreationService instead - this has duplicate logic and will be removed"
)]
#[derive(Debug, Clone)]
pub struct SessionManager {
    session_repo: AnalyticsSessionRepository,
    user_repo: UserRepository,
}

#[allow(deprecated)]
impl SessionManager {
    pub fn new(
        session_repo: AnalyticsSessionRepository,
        user_repo: UserRepository,
        _geoip_reader: Option<()>,  // Removed - was never used
    ) -> Self {
        Self {
            session_repo,
            user_repo,
        }
    }

    pub async fn get_or_create_anonymous_session(
        &self,
        analytics: SessionAnalytics,
        client_id: &ClientId,
        jwt_secret: &str,
    ) -> Result<SessionInfo> {
        let is_bot = analytics.is_bot() || analytics.is_bot_ip();
        let fingerprint = self.compute_fingerprint(&analytics);

        let max_age_seconds = 7 * 24 * 60 * 60;
        let fingerprint_lookup = tokio::time::timeout(
            tokio::time::Duration::from_millis(50),
            self.session_repo
                .find_recent_session_by_fingerprint(&fingerprint, max_age_seconds)
        ).await;

        if let Ok(Ok(Some(existing_session))) = fingerprint_lookup {
            if let Some(user_id_str) = &existing_session.user_id {
                let user_id = UserId::new(user_id_str.clone());
                let session_id = SessionId::new(existing_session.session_id().clone());

                let token = generate_anonymous_jwt(
                    user_id_str,
                    &existing_session.session_id,
                    client_id,
                    jwt_secret,
                )?;

                return Ok(SessionInfo {
                    session_id,
                    user_id,
                    is_new: false,
                    jwt_token: token,
                });
            }
        }

        self.create_new_anonymous_session(analytics, is_bot, fingerprint, client_id, jwt_secret).await
    }

    async fn create_new_anonymous_session(
        &self,
        analytics: SessionAnalytics,
        is_bot: bool,
        fingerprint: String,
        client_id: &ClientId,
        jwt_secret: &str,
    ) -> Result<SessionInfo> {
        let session_id = SessionId::new(format!("sess_{}", Uuid::new_v4()));
        let user_id = UserId::new(Uuid::new_v4());

        self.user_repo
            .create_anonymous_user(user_id.as_str())
            .await?;

        let jwt_expiration_seconds = systemprompt_core_system::Config::global().jwt_access_token_expiration;
        let expires_at = Utc::now() + Duration::seconds(jwt_expiration_seconds);

        self.session_repo
            .create_session(
                session_id.as_str(),
                None,
                Some(&fingerprint),
                analytics.ip_address.as_deref(),
                analytics.user_agent.as_deref(),
                analytics.device_type.as_deref(),
                analytics.browser.as_deref(),
                analytics.os.as_deref(),
                analytics.country.as_deref(),
                analytics.region.as_deref(),
                analytics.city.as_deref(),
                analytics.preferred_locale.as_deref(),
                analytics.referrer_source.as_deref(),
                analytics.referrer_url.as_deref(),
                analytics.landing_page.as_deref(),
                analytics.entry_url.as_deref(),
                analytics.utm_source.as_deref(),
                analytics.utm_medium.as_deref(),
                analytics.utm_campaign.as_deref(),
                is_bot,
                expires_at,
            )
            .await?;

        let token = generate_anonymous_jwt(
            user_id.as_str(),
            session_id.as_str(),
            client_id,
            jwt_secret,
        )?;

        Ok(SessionInfo {
            session_id,
            user_id,
            is_new: true,
            jwt_token: token,
        })
    }

    pub async fn migrate_session(
        &self,
        session_id: &SessionId,
        anonymous_user_id: &UserId,
        authenticated_user_id: &UserId,
    ) -> Result<()> {
        self.session_repo
            .migrate_session_to_registered_user(
                session_id.as_str(),
                anonymous_user_id.as_str(),
                authenticated_user_id.as_str(),
            )
            .await?;

        Ok(())
    }

    pub async fn update_session_activity(
        &self,
        session_id: &SessionId,
        endpoint: &str,
        response_time_ms: u64,
        is_success: bool,
    ) -> Result<()> {
        self.session_repo
            .update_session_activity(
                session_id.as_str(),
                endpoint,
                response_time_ms,
                is_success,
            )
            .await?;

        Ok(())
    }

    pub async fn record_endpoint_request(
        &self,
        session_id: &SessionId,
        path: &str,
        method: &str,
        status_code: u16,
        response_time_ms: u64,
    ) -> Result<()> {
        self.session_repo
            .record_endpoint_request(
                session_id.as_str(),
                path,
                method,
                status_code,
                response_time_ms,
            )
            .await?;

        Ok(())
    }

    pub async fn invalidate_session(
        &self,
        session_id: &SessionId,
    ) -> Result<()> {
        self.session_repo
            .end_session(session_id.as_str())
            .await?;

        Ok(())
    }

    pub async fn get_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<SessionData>> {
        let row = self.session_repo
            .get_session(session_id.as_str())
            .await?;

        Ok(row.map(|r| {
            let started_at = DateTime::parse_from_rfc3339(&r.started_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let last_activity = r.last_activity_at
                .parse::<DateTime<Utc>>()
                .ok();

            let is_active = r.ended_at.is_none();

            SessionData {
                session_id: SessionId::new(r.session_id),
                user_id: r.user_id.map(|u| UserId::new(u)),
                created_at: started_at,
                last_activity,
                is_active,
                request_count: r.request_count as u64,
            }
        }))
    }

    fn compute_fingerprint(&self, analytics: &SessionAnalytics) -> String {
        analytics
            .fingerprint_hash
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                let data = format!(
                    "{}{}",
                    analytics.user_agent.as_deref().unwrap_or("unknown"),
                    analytics.preferred_locale.as_deref().unwrap_or("")
                );
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            })
    }
}

#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub last_activity: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub request_count: u64,
}

impl SessionData {
    pub fn is_anonymous(&self) -> bool {
        self.user_id.is_none()
    }

    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
}

use anyhow::Result;
use axum::http::{HeaderMap, Uri};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use systemprompt_core_system::services::{AnalyticsService, SessionAnalytics};
use systemprompt_core_users::repository::UserRepository;
use systemprompt_identifiers::{ClientId, SessionId, UserId};

use crate::services::generation::tokens::generate_anonymous_jwt;

#[derive(Debug, Clone)]
pub struct AnonymousSessionInfo {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub is_new: bool,
    pub jwt_token: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedSessionInfo {
    pub session_id: SessionId,
}

#[derive(Clone, Debug)]
pub struct SessionCreationService {
    analytics_service: Arc<AnalyticsService>,
    user_repo: UserRepository,
    fingerprint_locks: Arc<RwLock<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl SessionCreationService {
    pub fn new(analytics_service: Arc<AnalyticsService>, user_repo: UserRepository) -> Self {
        Self {
            analytics_service,
            user_repo,
            fingerprint_locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_anonymous_session(
        &self,
        headers: &HeaderMap,
        uri: Option<&Uri>,
        client_id: &ClientId,
        jwt_secret: &str,
    ) -> Result<AnonymousSessionInfo> {
        let analytics = self.analytics_service.extract_analytics(headers, uri);
        let is_bot = AnalyticsService::is_bot(&analytics);
        let fingerprint = AnalyticsService::compute_fingerprint(&analytics);

        self.create_session_internal(analytics, is_bot, fingerprint, client_id, jwt_secret)
            .await
    }

    pub async fn create_authenticated_session(
        &self,
        user_id: &UserId,
        headers: &HeaderMap,
    ) -> Result<SessionId> {
        let session_id = SessionId::new(format!("sess_{}", Uuid::new_v4()));
        let analytics = self.analytics_service.extract_analytics(headers, None);
        let is_bot = AnalyticsService::is_bot(&analytics);

        let global_config = systemprompt_models::Config::global();
        let expires_at = chrono::Utc::now()
            + chrono::Duration::seconds(global_config.jwt_access_token_expiration);

        self.analytics_service
            .create_analytics_session(
                &session_id,
                Some(user_id.as_str()),
                &analytics,
                is_bot,
                expires_at,
            )
            .await?;

        Ok(session_id)
    }

    async fn create_session_internal(
        &self,
        analytics: SessionAnalytics,
        is_bot: bool,
        fingerprint: String,
        client_id: &ClientId,
        jwt_secret: &str,
    ) -> Result<AnonymousSessionInfo> {
        let max_age_seconds = 7 * 24 * 60 * 60;

        let lock = {
            let mut locks = self.fingerprint_locks.write().await;
            locks
                .entry(fingerprint.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };

        let _guard = lock.lock().await;

        let fingerprint_lookup = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            self.analytics_service
                .find_recent_session_by_fingerprint(&fingerprint, max_age_seconds),
        )
        .await;

        if let Ok(Ok(Some(existing_session))) = fingerprint_lookup {
            if let Some(user_id_str) = &existing_session.user_id {
                let user_id = UserId::new(user_id_str.clone());
                let session_id = SessionId::new(existing_session.session_id.clone());

                let token = generate_anonymous_jwt(
                    user_id_str,
                    &existing_session.session_id,
                    client_id,
                    jwt_secret,
                )?;

                return Ok(AnonymousSessionInfo {
                    session_id,
                    user_id,
                    is_new: false,
                    jwt_token: token,
                });
            }
        }

        let session_id = SessionId::new(format!("sess_{}", Uuid::new_v4()));

        let anonymous_user = self.user_repo.create_anonymous_user().await?;
        let user_id = anonymous_user.id;

        let jwt_expiration_seconds =
            systemprompt_models::Config::global().jwt_access_token_expiration;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(jwt_expiration_seconds);

        self.analytics_service
            .create_analytics_session(
                &session_id,
                Some(user_id.as_str()),
                &analytics,
                is_bot,
                expires_at,
            )
            .await?;

        let token =
            generate_anonymous_jwt(user_id.as_str(), session_id.as_str(), client_id, jwt_secret)?;

        Ok(AnonymousSessionInfo {
            session_id,
            user_id,
            is_new: true,
            jwt_token: token,
        })
    }
}

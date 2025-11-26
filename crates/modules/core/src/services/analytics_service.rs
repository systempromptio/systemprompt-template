use anyhow::Result;
use axum::extract::Request;
use axum::http::{HeaderMap, Uri};
use chrono::{DateTime, Utc};
use std::sync::Arc;

use systemprompt_core_database::DbPool;
use systemprompt_identifiers::SessionId;
use systemprompt_models::ContentConfig;

use crate::models::context::GeoIpReader;
use crate::repository::AnalyticsSessionRepository;
use crate::services::SessionAnalytics;

#[derive(Clone, Debug)]
pub struct AnalyticsService {
    geoip_reader: Option<GeoIpReader>,
    content_config: Option<Arc<ContentConfig>>,
    session_repo: AnalyticsSessionRepository,
}

impl AnalyticsService {
    pub const fn new(
        db_pool: DbPool,
        geoip_reader: Option<GeoIpReader>,
        content_config: Option<Arc<ContentConfig>>,
    ) -> Self {
        Self {
            geoip_reader,
            content_config,
            session_repo: AnalyticsSessionRepository::new(db_pool),
        }
    }

    pub fn extract_analytics(&self, headers: &HeaderMap, uri: Option<&Uri>) -> SessionAnalytics {
        SessionAnalytics::from_headers_and_uri(
            headers,
            uri,
            self.geoip_reader.as_ref(),
            self.content_config.as_ref().map(AsRef::as_ref),
        )
    }

    pub fn extract_from_request(&self, request: &Request) -> SessionAnalytics {
        SessionAnalytics::from_request(
            request,
            self.geoip_reader.as_ref(),
            self.content_config.as_ref().map(AsRef::as_ref),
        )
    }

    pub fn is_bot(analytics: &SessionAnalytics) -> bool {
        analytics.is_bot() || analytics.is_bot_ip()
    }

    pub fn compute_fingerprint(analytics: &SessionAnalytics) -> String {
        analytics.fingerprint_hash.as_deref().map_or_else(
            || {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let data = format!(
                    "{}{}",
                    analytics.user_agent.as_deref().unwrap_or("unknown"),
                    analytics.preferred_locale.as_deref().unwrap_or("")
                );

                let mut hasher = DefaultHasher::new();
                data.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            },
            ToString::to_string,
        )
    }

    pub async fn create_analytics_session(
        &self,
        session_id: &SessionId,
        user_id: Option<&str>,
        analytics: &SessionAnalytics,
        is_bot: bool,
        expires_at: DateTime<Utc>,
    ) -> Result<()> {
        let fingerprint = Self::compute_fingerprint(analytics);

        self.session_repo
            .create_session(
                session_id.as_str(),
                user_id,
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

        Ok(())
    }

    pub async fn find_recent_session_by_fingerprint(
        &self,
        fingerprint: &str,
        max_age_seconds: i64,
    ) -> Result<Option<crate::repository::SessionRecord>> {
        self.session_repo
            .find_recent_session_by_fingerprint(fingerprint, max_age_seconds)
            .await
    }

    pub const fn session_repo(&self) -> &AnalyticsSessionRepository {
        &self.session_repo
    }
}

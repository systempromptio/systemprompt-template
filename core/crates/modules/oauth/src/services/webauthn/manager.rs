use anyhow::Result;
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::WebAuthnService;
use crate::repository::OAuthRepository;
use systemprompt_core_logging::LogService;
use systemprompt_core_users::repository::UserRepository;

static WEBAUTHN_SERVICE: OnceCell<RwLock<Option<Arc<WebAuthnService>>>> = OnceCell::new();

#[derive(Debug, Copy, Clone)]
pub struct WebAuthnManager;

impl WebAuthnManager {
    pub async fn get_or_create_service(
        oauth_repo: OAuthRepository,
        user_repo: UserRepository,
        log_service: LogService,
    ) -> Result<Arc<WebAuthnService>> {
        let service_holder = WEBAUTHN_SERVICE.get_or_init(|| RwLock::new(None));

        let read_guard = service_holder.read().await;
        if let Some(service) = read_guard.as_ref() {
            return Ok(service.clone());
        }
        drop(read_guard);

        let mut write_guard = service_holder.write().await;
        if let Some(service) = write_guard.as_ref() {
            return Ok(service.clone());
        }

        let service = Arc::new(WebAuthnService::new(oauth_repo, user_repo, log_service)?);
        *write_guard = Some(service.clone());

        Self::start_cleanup_task(service.clone()).await;

        Ok(service)
    }

    async fn start_cleanup_task(service: Arc<WebAuthnService>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                if let Err(e) = service.cleanup_expired_states().await {
                    tracing::error!("WebAuthn state cleanup error: {}", e);
                }
            }
        });
    }
}

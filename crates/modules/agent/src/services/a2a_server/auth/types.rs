use crate::services::shared::{AgentSessionUser, Result};
use std::sync::Arc;
use systemprompt_core_database::Database;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::auth::{AuthMode, AuthValidationService};
pub use systemprompt_models::AgentOAuthConfig;

pub type AgentAuthenticatedUser = AgentSessionUser;

#[derive(Debug, Clone)]
pub struct AgentOAuthState {
    pub config: AgentOAuthConfig,
    pub auth_service: Arc<AuthValidationService>,
    pub db: Arc<Database>,
    pub log: LogService,
}

impl AgentOAuthState {
    pub async fn new(
        db: Arc<Database>,
        config: AgentOAuthConfig,
        log: LogService,
        jwt_secret: String,
    ) -> Result<Self> {
        Ok(Self {
            config,
            auth_service: Arc::new(AuthValidationService::new(jwt_secret)),
            db,
            log,
        })
    }

    pub fn auth_mode(&self) -> AuthMode {
        if self.config.required {
            AuthMode::Required
        } else {
            AuthMode::Optional
        }
    }
}

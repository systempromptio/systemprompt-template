use std::sync::Arc;
use systemprompt_core_ai::AiService;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use tokio::sync::RwLock;

use crate::services::a2a_server::auth::AgentOAuthState;
use crate::services::a2a_server::config::AgentConfig;

#[derive(Clone)]
pub struct AgentHandlerState {
    pub db_pool: DbPool,
    pub config: Arc<RwLock<AgentConfig>>,
    pub oauth_state: Arc<AgentOAuthState>,
    pub app_context: Arc<AppContext>,
    pub ai_service: Arc<AiService>,
    pub log: LogService,
}

impl std::fmt::Debug for AgentHandlerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentHandlerState")
            .field("db_pool", &"<DbPool>")
            .field("config", &"<Arc<RwLock<AgentConfig>>>")
            .field("oauth_state", &"<Arc<AgentOAuthState>>")
            .field("app_context", &"<Arc<AppContext>>")
            .field("ai_service", &"<Arc<AiService>>")
            .field("log", &"<LogService>")
            .finish()
    }
}

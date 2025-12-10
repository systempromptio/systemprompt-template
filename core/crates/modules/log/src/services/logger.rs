use crate::models::{LogEntry, LogLevel};
use crate::repository::LoggingRepository;
use crate::services::BufferedLogService;
use anyhow::Result;
use chrono::Utc;
use systemprompt_core_database::DbPool;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogOutput {
    Console,
    Database,
    Both,
    None,
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct LogContext {
    pub user_id: systemprompt_identifiers::UserId,
    pub session_id: systemprompt_identifiers::SessionId,
    pub task_id: Option<systemprompt_identifiers::TaskId>,
    pub trace_id: systemprompt_identifiers::TraceId,
    pub context_id: Option<systemprompt_identifiers::ContextId>,
    pub client_id: Option<systemprompt_identifiers::ClientId>,
    pub is_startup: bool,
}

impl Default for LogContext {
    fn default() -> Self {
        Self::system()
    }
}

impl LogContext {
    pub fn new() -> Self {
        Self::system()
    }

    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = systemprompt_identifiers::UserId::new(user_id.into());
        self
    }

    #[must_use]
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = systemprompt_identifiers::SessionId::new(session_id.into());
        self
    }

    #[must_use]
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(systemprompt_identifiers::TaskId::new(task_id.into()));
        self
    }

    #[must_use]
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = systemprompt_identifiers::TraceId::new(trace_id.into());
        self
    }

    #[must_use]
    pub fn with_context_id(mut self, context_id: impl Into<String>) -> Self {
        self.context_id = Some(systemprompt_identifiers::ContextId::new(context_id.into()));
        self
    }

    #[must_use]
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(systemprompt_identifiers::ClientId::new(client_id.into()));
        self
    }

    #[must_use]
    pub const fn with_startup(mut self, is_startup: bool) -> Self {
        self.is_startup = is_startup;
        self
    }

    pub fn system() -> Self {
        Self {
            user_id: systemprompt_identifiers::UserId::system(),
            session_id: systemprompt_identifiers::SessionId::system(),
            task_id: None,
            trace_id: systemprompt_identifiers::TraceId::generate(),
            context_id: None,
            client_id: Some(systemprompt_identifiers::ClientId::system("logger")),
            is_startup: false,
        }
    }

    pub fn startup() -> Self {
        Self {
            user_id: systemprompt_identifiers::UserId::system(),
            session_id: systemprompt_identifiers::SessionId::new("startup".to_string()),
            task_id: None,
            trace_id: systemprompt_identifiers::TraceId::generate(),
            context_id: None,
            client_id: Some(systemprompt_identifiers::ClientId::system("startup")),
            is_startup: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LogService {
    db_pool: DbPool,
    context: LogContext,
    buffered_logger: Option<BufferedLogService>,
}

impl LogService {
    pub const fn new(db_pool: DbPool, context: LogContext) -> Self {
        Self {
            db_pool,
            context,
            buffered_logger: None,
        }
    }

    pub fn with_buffering(db_pool: DbPool, context: LogContext) -> Self {
        let buffered_logger = BufferedLogService::new(db_pool.clone());
        Self {
            db_pool,
            context,
            buffered_logger: Some(buffered_logger),
        }
    }

    pub fn system(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            context: LogContext::system(),
            buffered_logger: None,
        }
    }

    pub fn system_with_buffering(db_pool: DbPool) -> Self {
        let buffered_logger = BufferedLogService::new(db_pool.clone());
        Self {
            db_pool,
            context: LogContext::system(),
            buffered_logger: Some(buffered_logger),
        }
    }

    pub fn startup(db_pool: DbPool) -> Self {
        Self {
            db_pool,
            context: LogContext::startup(),
            buffered_logger: None,
        }
    }

    #[must_use]
    pub fn with_task_id(mut self, task_id: impl Into<String>) -> Self {
        self.context.task_id = Some(systemprompt_identifiers::TaskId::new(task_id.into()));
        self
    }

    #[must_use]
    pub fn with_context_id(mut self, context_id: impl Into<String>) -> Self {
        self.context.context_id = Some(systemprompt_identifiers::ContextId::new(context_id.into()));
        self
    }

    fn should_log_to_db(level: LogLevel, is_startup: bool) -> bool {
        if is_startup {
            return matches!(level, LogLevel::Error | LogLevel::Warn);
        }

        let rust_log = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "warn".to_string())
            .to_lowercase();

        if rust_log.contains("trace") || rust_log.contains("debug") {
            true
        } else if rust_log.contains("info") {
            matches!(level, LogLevel::Error | LogLevel::Warn | LogLevel::Info)
        } else if rust_log.contains("warn") {
            matches!(level, LogLevel::Error | LogLevel::Warn)
        } else if rust_log.contains("error") {
            matches!(level, LogLevel::Error)
        } else {
            matches!(level, LogLevel::Error | LogLevel::Warn)
        }
    }

    pub async fn log(
        &self,
        level: LogLevel,
        module: impl Into<String>,
        message: impl Into<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let module = module.into();
        let message = message.into();

        self.write_to_console(level, &module, &message);

        let mut entry = LogEntry::new(level, &module, &message)
            .with_user_id(self.context.user_id.clone())
            .with_session_id(self.context.session_id.clone())
            .with_trace_id(self.context.trace_id.clone());

        if let Some(meta) = metadata {
            entry = entry.with_metadata(meta);
        }

        if let Some(tid) = &self.context.task_id {
            entry = entry.with_task_id(tid.clone());
        }

        if let Some(cid) = &self.context.context_id {
            entry = entry.with_context_id(cid.clone());
        }

        if let Some(client_id) = &self.context.client_id {
            entry = entry.with_client_id(client_id.clone());
        }

        let should_write_to_db = Self::should_log_to_db(level, self.context.is_startup);

        if should_write_to_db {
            if let Some(buffered) = &self.buffered_logger {
                buffered.log(entry).await.map_err(|e| anyhow::anyhow!(e))?;
            } else {
                let repo = LoggingRepository::new(self.db_pool.clone())
                    .with_terminal(false)
                    .with_database(true);

                repo.log(entry).await.map_err(|e| anyhow::anyhow!(e))?;
            }
        }

        Ok(())
    }

    pub async fn debug(&self, module: impl Into<String>, message: impl Into<String>) -> Result<()> {
        self.log(LogLevel::Debug, module, message, None).await
    }

    pub async fn info(&self, module: impl Into<String>, message: impl Into<String>) -> Result<()> {
        self.log(LogLevel::Info, module, message, None).await
    }

    pub async fn warn(&self, module: impl Into<String>, message: impl Into<String>) -> Result<()> {
        self.log(LogLevel::Warn, module, message, None).await
    }

    pub async fn error(&self, module: impl Into<String>, message: impl Into<String>) -> Result<()> {
        self.log(LogLevel::Error, module, message, None).await
    }

    #[allow(clippy::unused_self)]
    fn write_to_console(&self, level: LogLevel, module: &str, message: &str) {
        let timestamp = Utc::now().format("%H:%M:%S");
        let level_str = match level {
            LogLevel::Error => "\x1b[31mERROR\x1b[0m",
            LogLevel::Warn => "\x1b[33mWARN \x1b[0m",
            LogLevel::Info => "\x1b[32mINFO \x1b[0m",
            LogLevel::Debug => "\x1b[36mDEBUG\x1b[0m",
            LogLevel::Trace => "\x1b[90mTRACE\x1b[0m",
        };
        println!("{timestamp} [{level_str}] {module}: {message}");
    }
}

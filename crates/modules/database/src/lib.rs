pub mod macros;
pub mod models;
pub mod repository;
pub mod services;

// Re-export main types
pub use models::{
    parse_database_datetime, ArtifactId, ClientId, ColumnInfo, ContentId, ContextId, DatabaseInfo,
    DatabaseQuery, DatabaseTransaction, DbValue, ExecutionStepId, FileId, FromDatabaseRow,
    FromDbValue, JsonRow, LogId, QueryResult, QueryRow, QuerySelector, SessionId, SkillId,
    TableInfo, TaskId, ToDbValue, TokenId, TraceId, UserId,
};

pub use services::{
    Database, DatabaseCliDisplay, DatabaseExt, DatabaseProvider, DatabaseProviderExt, DbPool,
    PostgresProvider, SqlExecutor,
};

pub use repository::DatabaseInfoRepository;

// Re-export sqlx for convenience
pub use sqlx::types::Json;
pub use sqlx::{PgPool, Pool, Postgres, Transaction};

// Implementation of DatabaseHandle trait from systemprompt-traits
use systemprompt_traits::DatabaseHandle;

impl DatabaseHandle for Database {
    fn is_connected(&self) -> bool {
        // Try to test connection synchronously
        // This is not ideal but works for the trait requirement
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

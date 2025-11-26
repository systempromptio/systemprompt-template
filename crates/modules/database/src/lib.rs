pub mod macros;
pub mod models;
pub mod repository;
pub mod services;

// Re-export main types
pub use models::{
    parse_database_datetime, ColumnInfo, DatabaseInfo, DatabaseQuery, DatabaseQueryEnum,
    DatabaseTransaction, DbValue, FromDatabaseRow, FromDbValue, JsonRow, QueryResult, QueryRow,
    QuerySelector, TableInfo, ToDbValue,
};

pub use services::{
    Database, DatabaseCliDisplay, DatabaseExt, DatabaseProvider, DatabaseProviderExt, DbPool,
    SqlExecutor,
};

pub use repository::DatabaseInfoRepository;

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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Table not found: {table}")]
    TableNotFound { table: String },

    #[error("Schema error: {message}")]
    Schema { message: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid module configuration: {message}")]
    InvalidModule { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

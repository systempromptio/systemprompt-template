use thiserror::Error;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type DashboardResult<T> = Result<T, DashboardError>;

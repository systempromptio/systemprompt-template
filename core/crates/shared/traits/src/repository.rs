use async_trait::async_trait;

#[async_trait]
pub trait Repository: Send + Sync {
    type Pool;
    type Error: std::error::Error + Send + Sync + 'static;

    fn pool(&self) -> &Self::Pool;
}

#[async_trait]
pub trait CrudRepository<T>: Repository {
    type Id;

    async fn create(&self, entity: T) -> Result<T, Self::Error>;
    async fn get(&self, id: Self::Id) -> Result<Option<T>, Self::Error>;
    async fn update(&self, entity: T) -> Result<T, Self::Error>;
    async fn delete(&self, id: Self::Id) -> Result<(), Self::Error>;
    async fn list(&self) -> Result<Vec<T>, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("database error: {0}")]
    Database(String),

    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("invalid data: {0}")]
    InvalidData(String),

    #[error("constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("generic error: {0}")]
    GenericError(#[from] anyhow::Error),
}

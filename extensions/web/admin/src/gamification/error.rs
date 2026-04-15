#[derive(Debug, thiserror::Error)]
pub enum GamificationError {
    #[error("gamification database error: {0}")]
    Database(#[from] sqlx::Error),
}

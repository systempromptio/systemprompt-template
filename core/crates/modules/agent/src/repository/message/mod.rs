mod parts;
mod persistence;
mod queries;

use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_traits::RepositoryError;

use crate::models::a2a::Message;

pub use parts::get_message_parts;
pub use persistence::{persist_message_sqlx, persist_message_with_tx};
pub use queries::{
    get_messages_by_context, get_messages_by_task, get_next_sequence_number,
    get_next_sequence_number_in_tx, get_next_sequence_number_sqlx,
};

#[derive(Debug, Clone)]
pub struct MessageRepository {
    pool: Arc<PgPool>,
}

impl MessageRepository {
    pub fn new(db_pool: DbPool) -> Result<Self, RepositoryError> {
        let pool = db_pool.as_ref().get_postgres_pool().ok_or_else(|| {
            RepositoryError::Database("PostgreSQL pool not available".to_string())
        })?;
        Ok(Self { pool })
    }

    pub async fn get_messages_by_task(
        &self,
        task_id: &str,
    ) -> Result<Vec<Message>, RepositoryError> {
        get_messages_by_task(&self.pool, task_id).await
    }

    pub async fn get_messages_by_context(
        &self,
        context_id: &str,
    ) -> Result<Vec<Message>, RepositoryError> {
        get_messages_by_context(&self.pool, context_id).await
    }

    pub async fn get_next_sequence_number(&self, task_id: &str) -> Result<i32, RepositoryError> {
        get_next_sequence_number(&self.pool, task_id).await
    }

    pub async fn persist_message_sqlx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        message: &Message,
        task_id: &str,
        context_id: &str,
        sequence_number: i32,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<(), RepositoryError> {
        persist_message_sqlx(
            tx,
            message,
            task_id,
            context_id,
            sequence_number,
            user_id,
            session_id,
            trace_id,
        )
        .await
    }

    pub async fn persist_message_with_tx(
        &self,
        tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
        message: &Message,
        task_id: &str,
        context_id: &str,
        sequence_number: i32,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<(), RepositoryError> {
        persist_message_with_tx(
            tx,
            message,
            task_id,
            context_id,
            sequence_number,
            user_id,
            session_id,
            trace_id,
        )
        .await
    }
}

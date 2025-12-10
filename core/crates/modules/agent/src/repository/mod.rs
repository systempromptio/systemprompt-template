//! Data access layer for Agent entities using repository pattern.
//!
//! This module provides a comprehensive repository pattern implementation for
//! Agent (Agent-to-Agent) communication entities following the A2A protocol
//! specification exactly.
//!
//! ## Architecture
//!
//! The module follows a clean repository pattern with:
//! - Centralized error handling through `RepositoryError`
//! - Database connection pooling with optimized `SQLite` configuration
//! - A2A protocol-compliant data structures
//! - Type-safe data access with proper serialization support
//!
//! ## Usage
//!
//! ```rust
//! use crate::repository::AgentRepositories;
//!
//! let repos = AgentRepositories::new("sqlite:///path/to/db.sqlite").await?;
//! let agent_card = repos.agent_cards.get("agent-uuid").await?;
//! ```

use systemprompt_core_database::DbPool;

// Core repositories
pub mod agent_service_repository;
pub mod artifact_repository;
pub mod context_repository;
pub mod execution_step_repository;
pub mod message;
pub mod push_notification_config;
pub mod skill_repository;
pub mod task;
pub mod task_constructor;
pub mod task_repository;

// Re-export shared types
pub use systemprompt_traits::RepositoryError;

/// Common interface for all repository implementations.
///
/// This trait provides a standardized way to access the database provider
/// across all repository types, ensuring consistent database access patterns.
pub trait Repository {
    /// Returns a reference to the database provider.
    ///
    /// This method provides access to the underlying `DatabaseProvider`
    /// for executing database operations.
    fn pool(&self) -> &DbPool;
}

// Repository exports
pub use agent_service_repository::{
    AgentServerIdPidRow, AgentServerIdRow, AgentServiceRepository, AgentServiceRow,
};
pub use artifact_repository::ArtifactRepository;
pub use context_repository::ContextRepository;
pub use execution_step_repository::ExecutionStepRepository;
pub use message::MessageRepository;
pub use push_notification_config::PushNotificationConfigRepository;
pub use skill_repository::SkillRepository;
pub use task::TaskContextInfo;
pub use task_constructor::TaskConstructor;
pub use task_repository::TaskRepository;

/// A2A Repositories - Centralized access to all repositories
#[derive(Debug)]
pub struct A2ARepositories {
    db_pool: DbPool,
    pub agent_services: AgentServiceRepository,
    pub tasks: TaskRepository,
    pub execution_steps: ExecutionStepRepository,
    pub push_notification_configs: PushNotificationConfigRepository,
}

impl A2ARepositories {
    /// Creates a new A2A repositories instance with PostgreSQL connection pool.
    ///
    /// # Errors
    ///
    /// Returns `RepositoryError::DatabaseError` if:
    /// - The database URL is invalid or malformed
    /// - Connection to the database fails
    /// - Connection pool creation fails
    pub async fn new(database_url: &str) -> Result<Self, RepositoryError> {
        use std::sync::Arc;
        use systemprompt_core_database::Database;

        let db_pool = Database::new_postgres(database_url)
            .await
            .map_err(|e| RepositoryError::GenericError(e))?;
        let db_pool = Arc::new(db_pool);

        let agent_services = AgentServiceRepository::new(db_pool.clone());
        let tasks = TaskRepository::new(db_pool.clone());
        let execution_steps = ExecutionStepRepository::new(db_pool.clone());
        let push_notification_configs = PushNotificationConfigRepository::new(db_pool.clone());

        Ok(Self {
            db_pool,
            agent_services,
            tasks,
            execution_steps,
            push_notification_configs,
        })
    }

    #[must_use]
    pub const fn pool(&self) -> &DbPool {
        &self.db_pool
    }

    /// Creates a new A2A repositories instance using an existing database
    /// provider.
    pub fn from_pool(db_pool: DbPool) -> Self {
        let agent_services = AgentServiceRepository::new(db_pool.clone());
        let tasks = TaskRepository::new(db_pool.clone());
        let execution_steps = ExecutionStepRepository::new(db_pool.clone());
        let push_notification_configs = PushNotificationConfigRepository::new(db_pool.clone());

        Self {
            db_pool,
            agent_services,
            tasks,
            execution_steps,
            push_notification_configs,
        }
    }
}

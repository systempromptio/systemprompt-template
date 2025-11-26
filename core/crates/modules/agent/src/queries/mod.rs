//! A2A query organization following repository pattern.
//!
//! Per the repository pattern guidelines, this module only contains legacy query modules
//! that are being phased out. All SQL queries should now be:
//!
//! 1. Stored in separate .sql files under the appropriate domain directory
//! 2. Loaded via include_str!() directly in repository implementations  
//! 3. Never abstracted through QueryDefinition or similar structures
//!
//! Query organization:
//! - core/agents/*.sql - Agent lifecycle, discovery, and health queries
//! - core/messages/*.sql - Message routing, storage, and delivery queries  
//! - core/sessions/*.sql - Multi-agent communication session queries
//! - core/protocols/*.sql - Protocol negotiation and compatibility queries
//! - fixtures/*.sql - Test data and legacy queries
//!
//! Example proper usage in repository:
//! ```rust
//! impl AgentRepository {
//!     const FIND_BY_ID: &'static str = include_str!("../queries/core/agents/get_agent_by_id.sql");
//!     
//!     pub async fn find_by_id(&self, id: &str) -> Result<Option<Agent>> {
//!         sqlx::query_as(Self::FIND_BY_ID)
//!             .bind(id)
//!             .fetch_optional(&self.pool)
//!             .await
//!             .map_err(Into::into)
//!     }
//! }
//! ```

// Re-export shared query types for legacy modules
// pub use systemprompt_core_system::QueryDefinition;

// Local query macro for legacy modules (DISABLED - no longer needed)
// macro_rules! query {
//     ($name:expr, $file:expr, $desc:expr) => {
//         QueryDefinition {
//             name: $name,
//             sql: include_str!($file),
//             description: $desc,
//         }
//     };
// }

// pub(crate) use query;

// Legacy query modules (to be phased out) - All SQL moved to individual files
// pub mod a2a_agents; // Moved to .sql files
// pub mod a2a_tasks; // Moved to .sql files
// pub mod agents; // Moved to .sql files
// pub mod logging; // Moved to .sql files
// pub mod storage; // Moved to .sql files
// pub mod tasks; // Moved to .sql files

// #[cfg(test)]
// pub mod tests; // No tests.rs file exists

#[cfg(test)]
mod inline_tests {
    // Legacy QueryDefinition tests removed since the pattern is no longer used
    // All SQL queries are now stored in individual .sql files and loaded via include_str!()
}

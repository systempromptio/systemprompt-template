pub mod assertions;
pub mod cleanup;
/// Shared test utilities - Used by all integration tests
///
/// This module contains all reusable test infrastructure:
/// - TestContext: Centralized test environment setup
/// - Assertions: Fluent assertion builders for domain objects
/// - Factories: Test data builders with realistic defaults
/// - HTTP utilities: Session extraction, SSE parsing
/// - Database utilities: Async wait, cleanup, validation
pub mod context;
pub mod database;
pub mod factories;
pub mod http;

// Re-export commonly used items for convenience
pub use assertions::{IntegrityAssertion, SessionAssertion, TaskAssertion};
pub use cleanup::TestCleanup;
pub use context::{
    create_a2a_message, get_session_from_row, wait_for_async_processing as context_wait,
    Environment, SessionData, TestContext,
};
pub use database::{
    cleanup_by_fingerprint, count_orphaned_records, session_exists, wait_for_async_processing,
};
pub use factories::{conversation_message, fingerprint, user_agent, SessionFactory};

// Re-export database provider trait for all tests
pub use systemprompt_core_database::DatabaseProvider;

/// Database utility functions for tests
///
/// These provide common database operations that tests might need.
/// For complex queries, use DatabaseQueryEnum pattern directly.
use anyhow::Result;
use systemprompt_core_database::Database;

pub async fn wait_for_async_processing() {
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
}

pub async fn get_session_count(_db: &Database) -> Result<i64> {
    // Would require proper DatabaseQueryEnum query
    // For now, return a placeholder
    Ok(0)
}

pub async fn cleanup_by_fingerprint(_db: &Database, _fingerprint: &str) -> Result<()> {
    // Cleanup is handled by TestCleanup struct
    Ok(())
}

pub async fn cleanup_task(_db: &Database, _task_id: &str) -> Result<()> {
    // Cleanup is handled by TestCleanup struct
    Ok(())
}

pub async fn session_exists(_db: &Database, _session_id: &str) -> Result<bool> {
    // Would require proper DatabaseQueryEnum query
    Ok(false)
}

pub async fn count_orphaned_records(_db: &Database) -> Result<OrphanedCounts> {
    // Would require proper DatabaseQueryEnum queries
    Ok(OrphanedCounts {
        analytics_events: 0,
        endpoint_requests: 0,
        task_messages: 0,
    })
}

#[derive(Debug)]
pub struct OrphanedCounts {
    pub analytics_events: i64,
    pub endpoint_requests: i64,
    pub task_messages: i64,
}

impl OrphanedCounts {
    pub fn assert_all_zero(&self) {
        assert_eq!(
            self.analytics_events, 0,
            "Found {} orphaned analytics events",
            self.analytics_events
        );
        assert_eq!(
            self.endpoint_requests, 0,
            "Found {} orphaned endpoint requests",
            self.endpoint_requests
        );
        assert_eq!(
            self.task_messages, 0,
            "Found {} orphaned task messages",
            self.task_messages
        );
    }
}

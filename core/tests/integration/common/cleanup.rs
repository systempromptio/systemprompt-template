/// Test cleanup utilities
///
/// Tracks test data that needs to be cleaned up after tests complete.
/// Uses the existing DatabaseQueryEnum pattern for actual cleanup operations.
use anyhow::Result;
use systemprompt_core_database::{Database, DatabaseProvider, DatabaseQueryEnum};

pub struct TestCleanup {
    db: std::sync::Arc<Database>,
    fingerprints: Vec<String>,
    task_ids: Vec<String>,
    session_ids: Vec<String>,
}

impl TestCleanup {
    pub fn new(db: std::sync::Arc<Database>) -> Self {
        Self {
            db,
            fingerprints: Vec::new(),
            task_ids: Vec::new(),
            session_ids: Vec::new(),
        }
    }

    pub fn track_fingerprint(&mut self, fingerprint: String) {
        self.fingerprints.push(fingerprint);
    }

    pub fn track_task(&mut self, task_id: String) {
        self.task_ids.push(task_id);
    }

    pub fn track_task_id(&mut self, task_id: String) {
        self.task_ids.push(task_id);
    }

    pub fn track_session(&mut self, session_id: String) {
        self.session_ids.push(session_id);
    }

    pub async fn cleanup_all(&self) -> Result<()> {
        for fingerprint in &self.fingerprints {
            self.cleanup_by_fingerprint(fingerprint).await.ok();
        }

        for task_id in &self.task_ids {
            self.cleanup_task(task_id).await.ok();
        }

        for session_id in &self.session_ids {
            self.cleanup_session(session_id).await.ok();
        }

        Ok(())
    }

    async fn cleanup_by_fingerprint(&self, fingerprint: &str) -> Result<()> {
        let find_query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(self.db.as_ref());
        let rows = self.db.fetch_all(&find_query, &[&fingerprint]).await?;

        for row in rows {
            if let Some(session_id) = row.get("session_id").and_then(|v| v.as_str()) {
                let delete_query = DatabaseQueryEnum::DeleteSessionById.get(self.db.as_ref());
                self.db.execute(&delete_query, &[&session_id]).await?;
            }
        }

        Ok(())
    }

    async fn cleanup_task(&self, _task_id: &str) -> Result<()> {
        // Task cleanup using DatabaseQueryEnum pattern would go here
        // For now, tasks are cleaned up via cascade deletes
        Ok(())
    }

    async fn cleanup_session(&self, session_id: &str) -> Result<()> {
        let delete_query = DatabaseQueryEnum::DeleteSessionById.get(self.db.as_ref());
        self.db.execute(&delete_query, &[&session_id]).await?;
        Ok(())
    }
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        // Cleanup would run on drop, but we rely on explicit cleanup_all() calls
        // to avoid issues with async in Drop trait
    }
}

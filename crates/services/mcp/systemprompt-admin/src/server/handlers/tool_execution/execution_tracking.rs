use chrono::Utc;

pub struct ExecutionTracker {
    started_at: chrono::DateTime<chrono::Utc>,
}

impl ExecutionTracker {
    pub fn new() -> Self {
        Self {
            started_at: Utc::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().num_milliseconds() as u64
    }
}

impl Default for ExecutionTracker {
    fn default() -> Self {
        Self::new()
    }
}

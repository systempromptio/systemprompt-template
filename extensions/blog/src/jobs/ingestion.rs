//! Content ingestion background job.

/// Scheduled job that ingests markdown content from configured directories.
#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIngestionJob;

impl ContentIngestionJob {
    /// Get the job name.
    pub const fn name() -> &'static str {
        "blog_content_ingestion"
    }

    /// Get the job description.
    pub const fn description() -> &'static str {
        "Ingests markdown content from configured directories into the database"
    }

    /// Get the cron schedule.
    pub const fn schedule() -> &'static str {
        "0 0 * * * *" // Every hour
    }
}

# SystemPrompt Scheduler Module

Infrastructure module for scheduling background jobs, cron tasks, and agentic workflows using tokio-cron-scheduler.

## Overview

The scheduler module provides a foundation for running scheduled background jobs like:
- **Conversation Analysis** - Analyze conversations using LLM and populate `conversation_subjects` table
- **Email Reports** - Send daily/weekly email summaries
- **Database Analytics** - Generate analytics and cache results
- **Cleanup Tasks** - Remove old data, expire sessions, etc.

## Architecture

```
SchedulerService (main entry point)
    ├── Configuration (YAML/env)
    ├── SchedulerRepository (database operations)
    └── Jobs (individual scheduled tasks)
        ├── conversation_analysis.rs
        └── [your custom jobs...]
```

## Database Schema

The module creates the `scheduled_jobs` table to track job execution:

```sql
CREATE TABLE scheduled_jobs (
    id TEXT PRIMARY KEY,
    job_name TEXT NOT NULL UNIQUE,
    schedule TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    last_run DATETIME,
    next_run DATETIME,
    last_status TEXT,
    last_error TEXT,
    run_count INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

## Configuration

### YAML Config (config.yml)

```yaml
scheduler:
  enabled: true
  jobs:
    - name: "conversation_analysis"
      enabled: true
      schedule: "0 0 2 * * *"  # Daily at 2 AM

    - name: "email_reports"
      enabled: false
      schedule: "0 0 9 * * *"  # Daily at 9 AM
```

### Cron Schedule Format

```
┌───────────── second (0 - 59)
│ ┌───────────── minute (0 - 59)
│ │ ┌───────────── hour (0 - 23)
│ │ │ ┌───────────── day of month (1 - 31)
│ │ │ │ ┌───────────── month (1 - 12)
│ │ │ │ │ ┌───────────── day of week (0 - 6) (Sunday = 0)
│ │ │ │ │ │
│ │ │ │ │ │
* * * * * *
```

**Common Examples:**
- `0 0 2 * * *` - Daily at 2 AM
- `0 0 * * * *` - Every hour
- `0 */15 * * * *` - Every 15 minutes
- `0 0 0 * * 1` - Every Monday at midnight
- `0 0 9 1 * *` - First day of every month at 9 AM

## Usage

### Starting the Scheduler

```rust
use systemprompt_core_scheduler::{SchedulerConfig, SchedulerService};

let config = SchedulerConfig {
    enabled: true,
    jobs: vec![
        JobConfig {
            name: "conversation_analysis".to_string(),
            enabled: true,
            schedule: "0 0 2 * * *".to_string(),
        },
    ],
};

let scheduler = SchedulerService::new(config, db_pool.clone());
scheduler.start().await?;
```

### Creating a New Job

**Step 1**: Create job file in `src/services/jobs/your_job.rs`

```rust
use anyhow::Result;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;

pub async fn your_job_function(db_pool: DbPool, logger: LogService) -> Result<()> {
    logger.info("scheduler", "Starting your job").await.ok();

    // Your job logic here

    logger.info("scheduler", "Job completed").await.ok();
    Ok(())
}
```

**Step 2**: Register in `src/services/jobs/mod.rs`

```rust
pub mod conversation_analysis;
pub mod your_job;  // Add this

pub use conversation_analysis::analyze_conversations;
pub use your_job::your_job_function;  // Add this
```

**Step 3**: Add to job executor in `src/services/scheduler.rs`

```rust
async fn execute_job(job_name: &str, db_pool: DbPool, logger: LogService) -> Result<()> {
    match job_name {
        "conversation_analysis" => jobs::analyze_conversations(db_pool, logger).await,
        "your_job" => jobs::your_job_function(db_pool, logger).await,  // Add this
        _ => {
            tracing::warn!("Unknown job: {job_name}");
            Err(SchedulerError::job_not_found(job_name).into())
        }
    }
}
```

**Step 4**: Add to config

```yaml
scheduler:
  enabled: true
  jobs:
    - name: "your_job"
      enabled: true
      schedule: "0 0 3 * * *"  # Daily at 3 AM
```

## Conversation Analysis Job

The included conversation analysis job:

1. **Fetches unanalyzed tasks** - Tasks in `completed` status without entries in `conversation_subjects`
2. **Extracts keywords** - Simple keyword extraction (replace with LLM later)
3. **Classifies topics** - Rule-based topic classification (replace with LLM later)
4. **Stores results** - Saves to `conversation_subjects` table

### Enhancing with LLM

Replace the placeholder `analyze_with_llm()` function with actual Claude API calls:

```rust
use systemprompt_core_ai::services::AiService;

async fn analyze_with_llm(messages: &[String]) -> Result<(Vec<String>, String, f64)> {
    let combined_text = messages.join("\n");

    let prompt = format!(
        "Analyze this conversation and extract:\n\
         1. Keywords (comma-separated)\n\
         2. Primary topic\n\
         3. Confidence score (0.0-1.0)\n\n\
         Conversation:\n{combined_text}"
    );

    // Call Claude API
    let response = ai_service.generate(prompt).await?;

    // Parse response
    // Return (keywords, topic, confidence)
}
```

## Monitoring Jobs

### Check Job Status

```sql
SELECT job_name, enabled, schedule, last_run, last_status, run_count
FROM scheduled_jobs
ORDER BY last_run DESC;
```

### View Recent Logs

```sql
SELECT timestamp, level, module, message
FROM logs
WHERE module = 'scheduler'
ORDER BY timestamp DESC
LIMIT 100;
```

### Disable a Job

```sql
UPDATE scheduled_jobs
SET enabled = 0
WHERE job_name = 'conversation_analysis';
```

## Error Handling

Jobs automatically track success/failure in `scheduled_jobs` table:

- **Success**: `last_status = 'success'`, `last_error = NULL`
- **Failure**: `last_status = 'failed'`, `last_error = error message`

Jobs are retried on next schedule (no automatic retry on failure).

## Performance Considerations

- **Long-running jobs**: Use `LIMIT` in queries to process in batches
- **Database load**: Schedule jobs during low-traffic hours (2-4 AM)
- **LLM rate limits**: Add rate limiting to LLM calls (e.g., max 10 requests/second)

## Dependencies

- `tokio-cron-scheduler` - Cron scheduling
- `systemprompt-core-database` - Database operations
- `systemprompt-core-logging` - Logging
- `systemprompt-core-ai` - AI/LLM services (for conversation analysis)

## Next Steps

1. **Enhance conversation analysis** - Replace placeholder with actual Claude API
2. **Add more jobs** - Email reports, analytics, cleanup
3. **Add job dependencies** - Run job B after job A completes
4. **Add retry logic** - Automatic retry with exponential backoff
5. **Add alerting** - Send notifications on job failures

## Related Documentation

- [tokio-cron-scheduler docs](https://docs.rs/tokio-cron-scheduler/)
- [Cron syntax guide](https://crontab.guru/)
- [SystemPrompt Core CLAUDE.md](../../CLAUDE.md)
